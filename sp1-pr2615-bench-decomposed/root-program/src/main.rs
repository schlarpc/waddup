//! waddup root prover: composes N chunk proofs into the binding statement
//! `SHA256(F) = H_content AND BLAKE3(F) = H_swarm`.
//!
//! Reads from stdin in this order:
//!   - `chunk_vkey: [u32; 8]`
//!   - `file_byte_length: u64`
//!   - `num_chunks: u32`
//!   - For each chunk i in 0..num_chunks: `public_values_bytes` (fixed length)
//!
//! For each chunk PV:
//!   - parse (chunk_index, sha_state_in, sha_state_out, leaf_cvs[64])
//!   - assert chunk_index == i
//!   - verify_sp1_proof(chunk_vkey, sha256(public_values_bytes))
//!
//! Chain check: sha_state_out[i] == sha_state_in[i+1]; sha_state_in[0] == IV.
//! SHA-256 final padding using file_byte_length. BLAKE3 tree merge of all
//! N * 64 leaves with ROOT on the last merge.
//!
//! Commits `(H_content, H_swarm)`.
#![no_main]

sp1_zkvm::entrypoint!(main);

use sp1_lib::{
    syscall_blake3_compress_inner, syscall_sha256_compress, syscall_sha256_extend,
};

const BLAKE3_BLOCK_LEN: usize = 64;
const LEAVES_PER_CHUNK: usize = 64;
const CHUNK_PV_BYTES: usize = 4 + 32 + 32 + LEAVES_PER_CHUNK * 32; // 4 + 32 + 32 + 2048 = 2116

const SHA256_IV: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
    0x1f83d9ab, 0x5be0cd19,
];
const BLAKE3_IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C,
    0x1F83D9AB, 0x5BE0CD19,
];
const PARENT: u32 = 1 << 2;
const ROOT: u32 = 1 << 3;

#[inline]
fn sha256_update_block(state: &mut [u64; 8], block: &[u8; 64]) {
    let mut w: [u64; 64] = [0; 64];
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
fn parent_cv(left: &[u32; 8], right: &[u32; 8], is_root: bool) -> [u32; 8] {
    let mut msg = [0u32; 16];
    msg[0..8].copy_from_slice(left);
    msg[8..16].copy_from_slice(right);

    let mut state = [0u32; 16];
    state[0..8].copy_from_slice(&BLAKE3_IV);
    state[8..12].copy_from_slice(&BLAKE3_IV[0..4]);
    state[12] = 0;
    state[13] = 0;
    state[14] = BLAKE3_BLOCK_LEN as u32;
    let mut flags = PARENT;
    if is_root {
        flags |= ROOT;
    }
    state[15] = flags;

    blake3_compress(&mut state, &msg, &BLAKE3_IV);
    let mut cv = [0u32; 8];
    cv.copy_from_slice(&state[0..8]);
    cv
}

/// SHA-256 of a small in-memory buffer (used to digest each chunk's public values).
/// Uses the SP1 precompile; fixed pad+length tail.
fn sha256_of(bytes: &[u8]) -> [u8; 32] {
    let mut state: [u64; 8] = [
        SHA256_IV[0] as u64,
        SHA256_IV[1] as u64,
        SHA256_IV[2] as u64,
        SHA256_IV[3] as u64,
        SHA256_IV[4] as u64,
        SHA256_IV[5] as u64,
        SHA256_IV[6] as u64,
        SHA256_IV[7] as u64,
    ];

    // Full blocks.
    let nfull = bytes.len() / 64;
    for i in 0..nfull {
        let mut block = [0u8; 64];
        block.copy_from_slice(&bytes[i * 64..(i + 1) * 64]);
        sha256_update_block(&mut state, &block);
    }

    // Padding.
    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let tail = &bytes[nfull * 64..];
    let mut pad = [0u8; 128];
    pad[..tail.len()].copy_from_slice(tail);
    pad[tail.len()] = 0x80;
    let pad_blocks = if tail.len() + 1 + 8 <= 64 { 1 } else { 2 };
    let total_pad_len = pad_blocks * 64;
    pad[total_pad_len - 8..total_pad_len].copy_from_slice(&bit_len.to_be_bytes());
    for i in 0..pad_blocks {
        let mut block = [0u8; 64];
        block.copy_from_slice(&pad[i * 64..(i + 1) * 64]);
        sha256_update_block(&mut state, &block);
    }

    let mut out = [0u8; 32];
    for i in 0..8 {
        out[i * 4..i * 4 + 4].copy_from_slice(&(state[i] as u32).to_be_bytes());
    }
    out
}

pub fn main() {
    let chunk_vkey: [u32; 8] = sp1_zkvm::io::read();
    let file_byte_length: u64 = sp1_zkvm::io::read();
    let num_chunks: u32 = sp1_zkvm::io::read();

    // Running SHA state across chunks. Starts at IV.
    let mut sha_state_running: [u32; 8] = SHA256_IV;
    // Running BLAKE3 CV stack across all leaves (cross-chunk).
    let mut stack: [[u32; 8]; 64] = [[0u32; 8]; 64];
    let mut stack_len: usize = 0;
    // Total leaves we've absorbed (used for is-root decision).
    let total_leaves: u64 = (num_chunks as u64) * (LEAVES_PER_CHUNK as u64);

    let mut leaf_counter: u64 = 0;

    for i in 0..num_chunks {
        let pv_bytes_vec: Vec<u8> = sp1_zkvm::io::read_vec();
        assert_eq!(pv_bytes_vec.len(), CHUNK_PV_BYTES);
        let pv = &pv_bytes_vec[..];

        // Verify the sub-proof.
        println!("cycle-tracker-report-start:verify_chunk");
        let pv_digest = sha256_of(pv);
        sp1_zkvm::lib::verify::verify_sp1_proof(&chunk_vkey, &pv_digest);
        println!("cycle-tracker-report-end:verify_chunk");

        // Parse.
        let chunk_index = u32::from_le_bytes([pv[0], pv[1], pv[2], pv[3]]);
        assert_eq!(chunk_index, i, "chunk_index mismatch");

        let mut sha_state_in = [0u32; 8];
        for k in 0..8 {
            sha_state_in[k] = u32::from_le_bytes([
                pv[4 + k * 4],
                pv[4 + k * 4 + 1],
                pv[4 + k * 4 + 2],
                pv[4 + k * 4 + 3],
            ]);
        }
        // Chain check.
        assert_eq!(
            sha_state_in, sha_state_running,
            "sha chain break at chunk {}",
            i
        );

        let mut sha_state_out = [0u32; 8];
        for k in 0..8 {
            sha_state_out[k] = u32::from_le_bytes([
                pv[36 + k * 4],
                pv[36 + k * 4 + 1],
                pv[36 + k * 4 + 2],
                pv[36 + k * 4 + 3],
            ]);
        }
        sha_state_running = sha_state_out;

        // Absorb 64 leaves into the BLAKE3 cv stack.
        println!("cycle-tracker-report-start:absorb_leaves");
        let leaf_base = 68;
        for li in 0..LEAVES_PER_CHUNK {
            let mut leaf = [0u32; 8];
            for k in 0..8 {
                leaf[k] = u32::from_le_bytes([
                    pv[leaf_base + li * 32 + k * 4],
                    pv[leaf_base + li * 32 + k * 4 + 1],
                    pv[leaf_base + li * 32 + k * 4 + 2],
                    pv[leaf_base + li * 32 + k * 4 + 3],
                ]);
            }
            // Merge into the running stack.
            let mut new_cv = leaf;
            let mut total = leaf_counter + 1;
            let is_last_leaf = (leaf_counter + 1) == total_leaves;
            while (total & 1) == 0 && stack_len > 0 {
                let left = stack[stack_len - 1];
                stack_len -= 1;
                let next_total = total >> 1;
                let is_root =
                    is_last_leaf && stack_len == 0 && (next_total & 1) == 1;
                new_cv = parent_cv(&left, &new_cv, is_root);
                total = next_total;
            }
            stack[stack_len] = new_cv;
            stack_len += 1;
            leaf_counter += 1;
        }
        println!("cycle-tracker-report-end:absorb_leaves");
    }

    // Final BLAKE3 reduction.
    let mut current = stack[stack_len - 1];
    stack_len -= 1;
    while stack_len > 0 {
        let left = stack[stack_len - 1];
        stack_len -= 1;
        let is_root = stack_len == 0;
        current = parent_cv(&left, &current, is_root);
    }
    let mut h_swarm = [0u8; 32];
    for (i, w) in current.iter().enumerate() {
        h_swarm[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
    }

    // SHA-256 finalize: take running state, pad with file length, compute final.
    let mut final_state: [u64; 8] = [0; 8];
    for i in 0..8 {
        final_state[i] = sha_state_running[i] as u64;
    }
    let bit_len = file_byte_length.wrapping_mul(8);
    // We've processed exactly `file_byte_length` bytes. The padding goes in a
    // fresh block (or two): 0x80, zeros, 64-bit big-endian length.
    let body_remainder = (file_byte_length % 64) as usize;
    let mut pad = [0u8; 128];
    pad[body_remainder] = 0x80;
    let pad_blocks = if body_remainder + 1 + 8 <= 64 { 1 } else { 2 };
    let total_pad_len = pad_blocks * 64;
    pad[total_pad_len - 8..total_pad_len].copy_from_slice(&bit_len.to_be_bytes());
    for i in 0..pad_blocks {
        let mut block = [0u8; 64];
        block.copy_from_slice(&pad[i * 64..(i + 1) * 64]);
        sha256_update_block(&mut final_state, &block);
    }
    let mut h_content = [0u8; 32];
    for i in 0..8 {
        h_content[i * 4..i * 4 + 4]
            .copy_from_slice(&(final_state[i] as u32).to_be_bytes());
    }

    sp1_zkvm::io::commit_slice(&h_content);
    sp1_zkvm::io::commit_slice(&h_swarm);
}
