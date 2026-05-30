//! waddup root prover: composes N variable-size chunk proofs into the binding
//! statement `SHA256(F) = H_content AND BLAKE3(F) = H_swarm`.
//!
//! Reads from stdin:
//!   - `chunk_vkey: [u32; 8]`
//!   - `file_byte_length: u64`
//!   - `total_leaves: u64`
//!   - `num_chunks: u32`
//!   - For each chunk: `public_values_bytes: Vec<u8>` (variable size)
//!
//! Each chunk PV layout (offsets):
//!   0   chunk_index   (4 bytes)
//!   4   num_leaves    (4 bytes)
//!   8   sha_state_in  (32 bytes)
//!   40  sha_state_out (32 bytes)
//!   72  leaf_cvs[num_leaves] (num_leaves * 32 bytes)
//!
//! Chain check enforced sequentially. BLAKE3 tree merge of all `total_leaves`
//! CVs with ROOT on the last merge. SHA-256 finalized with `file_byte_length`.
#![no_main]

sp1_zkvm::entrypoint!(main);

use sp1_lib::{
    syscall_blake3_compress_inner, syscall_sha256_compress, syscall_sha256_extend,
};

const BLAKE3_BLOCK_LEN: usize = 64;

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

    let nfull = bytes.len() / 64;
    for i in 0..nfull {
        let mut block = [0u8; 64];
        block.copy_from_slice(&bytes[i * 64..(i + 1) * 64]);
        sha256_update_block(&mut state, &block);
    }

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
    let total_leaves: u64 = sp1_zkvm::io::read();
    let num_chunks: u32 = sp1_zkvm::io::read();

    let mut sha_state_running: [u32; 8] = SHA256_IV;
    let mut stack: [[u32; 8]; 64] = [[0u32; 8]; 64];
    let mut stack_len: usize = 0;
    let mut leaf_counter: u64 = 0;

    for i in 0..num_chunks {
        let pv_bytes_vec: Vec<u8> = sp1_zkvm::io::read_vec();
        let pv = &pv_bytes_vec[..];

        println!("cycle-tracker-report-start:verify_chunk");
        let pv_digest = sha256_of(pv);
        sp1_zkvm::lib::verify::verify_sp1_proof(&chunk_vkey, &pv_digest);
        println!("cycle-tracker-report-end:verify_chunk");

        // Parse PV.
        let chunk_index = u32::from_le_bytes([pv[0], pv[1], pv[2], pv[3]]);
        assert_eq!(chunk_index, i, "chunk_index mismatch");
        let num_leaves = u32::from_le_bytes([pv[4], pv[5], pv[6], pv[7]]) as usize;
        assert_eq!(pv.len(), 72 + num_leaves * 32, "pv length");

        let mut sha_state_in = [0u32; 8];
        for k in 0..8 {
            sha_state_in[k] = u32::from_le_bytes([
                pv[8 + k * 4],
                pv[8 + k * 4 + 1],
                pv[8 + k * 4 + 2],
                pv[8 + k * 4 + 3],
            ]);
        }
        assert_eq!(
            sha_state_in, sha_state_running,
            "sha chain break at chunk {}",
            i
        );

        let mut sha_state_out = [0u32; 8];
        for k in 0..8 {
            sha_state_out[k] = u32::from_le_bytes([
                pv[40 + k * 4],
                pv[40 + k * 4 + 1],
                pv[40 + k * 4 + 2],
                pv[40 + k * 4 + 3],
            ]);
        }
        sha_state_running = sha_state_out;

        // Absorb num_leaves leaves.
        println!("cycle-tracker-report-start:absorb_leaves");
        let leaf_base = 72;
        for li in 0..num_leaves {
            let mut leaf = [0u32; 8];
            for k in 0..8 {
                leaf[k] = u32::from_le_bytes([
                    pv[leaf_base + li * 32 + k * 4],
                    pv[leaf_base + li * 32 + k * 4 + 1],
                    pv[leaf_base + li * 32 + k * 4 + 2],
                    pv[leaf_base + li * 32 + k * 4 + 3],
                ]);
            }
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

    // BLAKE3: special-case single leaf (root output IS the leaf — never made a
    // parent merge so ROOT wasn't applied).
    let h_swarm = if total_leaves == 1 {
        // The "leaf CV" we computed didn't carry ROOT because chunk_prover
        // doesn't apply it. For a single-leaf file the leaf's compress must be
        // redone with ROOT to get the correct hash. For our test sizes ≥ 1
        // KiB this is just the 1 KiB case — re-hash the chunk from the PV's
        // chunk_data... but we don't have chunk_data in PV. So: at single-leaf
        // sizes we panic. The driver pads or uses multi-leaf shape instead.
        panic!("single-leaf root not supported by this guest; use ≥ 2 leaves");
    } else {
        let mut current = stack[stack_len - 1];
        stack_len -= 1;
        while stack_len > 0 {
            let left = stack[stack_len - 1];
            stack_len -= 1;
            let is_root = stack_len == 0;
            current = parent_cv(&left, &current, is_root);
        }
        let mut out = [0u8; 32];
        for (i, w) in current.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
        }
        out
    };

    // SHA-256 finalize.
    let mut final_state: [u64; 8] = [0; 8];
    for i in 0..8 {
        final_state[i] = sha_state_running[i] as u64;
    }
    let bit_len = file_byte_length.wrapping_mul(8);
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
