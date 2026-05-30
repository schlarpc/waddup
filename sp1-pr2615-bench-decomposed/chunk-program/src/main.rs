//! waddup chunk prover: processes one 64 KiB chunk.
//!
//! Reads `(chunk_index: u32, sha_state_in: [u32; 8], chunk: [u8; 65536])`.
//! Produces `(chunk_index, sha_state_in, sha_state_out, leaf_cvs: [[u32; 8]; 64])`.
//!
//! - SHA-256 state is updated block-by-block via SP1 precompiles. No padding here.
//! - BLAKE3 leaves are computed independently for each of 64 sub-chunks; no tree
//!   merging here (root handles that).
//!
//! No heap allocations. Fixed-size stack buffers only. Panic = abort.
#![no_main]

sp1_zkvm::entrypoint!(main);

use sp1_lib::{
    syscall_blake3_compress_inner, syscall_sha256_compress, syscall_sha256_extend,
};

const CHUNK_SIZE: usize = 65536;
const BLAKE3_CHUNK_LEN: usize = 1024;
const BLAKE3_BLOCK_LEN: usize = 64;
const LEAVES_PER_CHUNK: usize = CHUNK_SIZE / BLAKE3_CHUNK_LEN; // 64
const SHA_BLOCK_LEN: usize = 64;

const SHA256_IV: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
    0x1f83d9ab, 0x5be0cd19,
];
const BLAKE3_IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C,
    0x1F83D9AB, 0x5BE0CD19,
];
const CHUNK_START: u32 = 1 << 0;
const CHUNK_END: u32 = 1 << 1;

#[inline]
fn sha256_update_block(state: &mut [u64; 8], block: &[u8; SHA_BLOCK_LEN]) {
    let mut w: [u64; 64] = [0; 64];
    // First 16 words are big-endian u32 from the block.
    for j in 0..16 {
        w[j] = u32::from_be_bytes([
            block[j * 4],
            block[j * 4 + 1],
            block[j * 4 + 2],
            block[j * 4 + 3],
        ]) as u64;
    }
    unsafe {
        syscall_sha256_extend(&mut w);
        syscall_sha256_compress(&mut w, state);
    }
}

/// BLAKE3 compress with the SP1 precompile + the final XOR.
#[inline]
fn blake3_compress(state: &mut [u32; 16], msg: &[u32; 16], cv_in: &[u32; 8]) {
    let mut state_u64: [u64; 16] = [0; 16];
    let mut msg_u64: [u64; 16] = [0; 16];
    for i in 0..16 {
        state_u64[i] = state[i] as u64;
        msg_u64[i] = msg[i] as u64;
    }
    unsafe {
        syscall_blake3_compress_inner(
            state_u64.as_mut_ptr() as *mut _,
            msg_u64.as_ptr() as *const _,
        );
    }
    for i in 0..16 {
        state[i] = state_u64[i] as u32;
    }
    for i in 0..8 {
        state[i] ^= state[i + 8];
        state[i + 8] ^= cv_in[i];
    }
}

#[inline]
fn blake3_leaf_cv(chunk: &[u8; BLAKE3_CHUNK_LEN], counter: u64) -> [u32; 8] {
    let mut cv = BLAKE3_IV;
    for block_idx in 0..(BLAKE3_CHUNK_LEN / BLAKE3_BLOCK_LEN) {
        let block = &chunk[block_idx * BLAKE3_BLOCK_LEN..(block_idx + 1) * BLAKE3_BLOCK_LEN];
        let mut msg = [0u32; 16];
        for k in 0..16 {
            msg[k] = u32::from_le_bytes([
                block[k * 4],
                block[k * 4 + 1],
                block[k * 4 + 2],
                block[k * 4 + 3],
            ]);
        }

        let mut flags = 0u32;
        if block_idx == 0 {
            flags |= CHUNK_START;
        }
        if block_idx == (BLAKE3_CHUNK_LEN / BLAKE3_BLOCK_LEN) - 1 {
            flags |= CHUNK_END;
        }

        let mut state = [0u32; 16];
        state[0..8].copy_from_slice(&cv);
        state[8..12].copy_from_slice(&BLAKE3_IV[0..4]);
        state[12] = (counter & 0xFFFF_FFFF) as u32;
        state[13] = (counter >> 32) as u32;
        state[14] = BLAKE3_BLOCK_LEN as u32;
        state[15] = flags;

        let cv_in = cv;
        blake3_compress(&mut state, &msg, &cv_in);
        cv.copy_from_slice(&state[0..8]);
    }
    cv
}

pub fn main() {
    let chunk_index: u32 = sp1_zkvm::io::read();
    let sha_state_in: [u32; 8] = sp1_zkvm::io::read();
    // 64 KiB chunk read into a fixed-size buffer.
    let chunk_vec = sp1_zkvm::io::read_vec();

    // Copy into stack array. (read_vec gives us a Vec; we move into a stack buf.)
    let mut chunk = [0u8; CHUNK_SIZE];
    chunk.copy_from_slice(&chunk_vec[..CHUNK_SIZE]);
    drop(chunk_vec);

    let chunk_start_counter: u64 = (chunk_index as u64) * (LEAVES_PER_CHUNK as u64);

    let mut sha_state: [u64; 8] = [0; 8];
    for i in 0..8 {
        sha_state[i] = sha_state_in[i] as u64;
    }

    println!("cycle-tracker-report-start:sha256_chunk");
    for block_idx in 0..(CHUNK_SIZE / SHA_BLOCK_LEN) {
        let mut block = [0u8; SHA_BLOCK_LEN];
        block.copy_from_slice(
            &chunk[block_idx * SHA_BLOCK_LEN..(block_idx + 1) * SHA_BLOCK_LEN],
        );
        sha256_update_block(&mut sha_state, &block);
    }
    println!("cycle-tracker-report-end:sha256_chunk");

    let mut sha_state_out = [0u32; 8];
    for i in 0..8 {
        sha_state_out[i] = sha_state[i] as u32;
    }

    println!("cycle-tracker-report-start:blake3_leaves");
    let mut leaf_cvs = [[0u32; 8]; LEAVES_PER_CHUNK];
    for i in 0..LEAVES_PER_CHUNK {
        let mut leaf_chunk = [0u8; BLAKE3_CHUNK_LEN];
        leaf_chunk.copy_from_slice(
            &chunk[i * BLAKE3_CHUNK_LEN..(i + 1) * BLAKE3_CHUNK_LEN],
        );
        leaf_cvs[i] = blake3_leaf_cv(&leaf_chunk, chunk_start_counter + i as u64);
    }
    println!("cycle-tracker-report-end:blake3_leaves");

    // Commit public values in a fixed order. All fixed-size so root can parse
    // by offset.
    sp1_zkvm::io::commit_slice(&chunk_index.to_le_bytes());
    for w in &sha_state_in {
        sp1_zkvm::io::commit_slice(&w.to_le_bytes());
    }
    for w in &sha_state_out {
        sp1_zkvm::io::commit_slice(&w.to_le_bytes());
    }
    for cv in &leaf_cvs {
        for w in cv {
            sp1_zkvm::io::commit_slice(&w.to_le_bytes());
        }
    }
}
