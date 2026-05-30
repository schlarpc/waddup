//! BLAKE3-of-file using `syscall_blake3_compress_inner` (PR #2615 ported to v6.2.2).
//!
//! Wraps the precompile (which does only the 7-round permutation) with the
//! BLAKE3 chunking, tree-mode CV stack, and the `compress` final XOR
//! (`state[i] ^= state[i+8]`, `state[i+8] ^= cv_in[i]`).

use sp1_lib::syscall_blake3_compress_inner;

const IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C,
    0x1F83D9AB, 0x5BE0CD19,
];
const CHUNK_START: u32 = 1 << 0;
const CHUNK_END: u32 = 1 << 1;
const PARENT: u32 = 1 << 2;
const ROOT: u32 = 1 << 3;
const CHUNK_LEN: usize = 1024;
const BLOCK_LEN: usize = 64;

fn compress(state: &mut [u32; 16], msg: &[u32; 16], cv_in: &[u32; 8]) {
    let mut state_u64: [u64; 16] = core::array::from_fn(|i| state[i] as u64);
    let msg_u64: [u64; 16] = core::array::from_fn(|i| msg[i] as u64);
    unsafe {
        syscall_blake3_compress_inner(
            state_u64.as_mut_ptr() as *mut _,
            msg_u64.as_ptr() as *const _,
        );
    }
    for (i, s) in state_u64.iter().enumerate() {
        state[i] = *s as u32;
    }
    for i in 0..8 {
        state[i] ^= state[i + 8];
        state[i + 8] ^= cv_in[i];
    }
}

fn hash_chunk(chunk: &[u8], chunk_counter: u64, is_root_single_chunk: bool) -> [u32; 8] {
    let mut cv = IV;
    let num_blocks = if chunk.is_empty() {
        1
    } else {
        (chunk.len() + BLOCK_LEN - 1) / BLOCK_LEN
    };
    for j in 0..num_blocks {
        let start = j * BLOCK_LEN;
        let end = core::cmp::min(start + BLOCK_LEN, chunk.len());
        let block = &chunk[start..end];

        let mut block_padded = [0u8; BLOCK_LEN];
        block_padded[..block.len()].copy_from_slice(block);
        let mut msg = [0u32; 16];
        for k in 0..16 {
            msg[k] = u32::from_le_bytes([
                block_padded[k * 4],
                block_padded[k * 4 + 1],
                block_padded[k * 4 + 2],
                block_padded[k * 4 + 3],
            ]);
        }

        let mut flags = 0u32;
        if j == 0 {
            flags |= CHUNK_START;
        }
        if j == num_blocks - 1 {
            flags |= CHUNK_END;
            if is_root_single_chunk {
                flags |= ROOT;
            }
        }

        let mut state = [0u32; 16];
        state[0..8].copy_from_slice(&cv);
        state[8..12].copy_from_slice(&IV[0..4]);
        state[12] = (chunk_counter & 0xFFFF_FFFF) as u32;
        state[13] = (chunk_counter >> 32) as u32;
        state[14] = block.len() as u32;
        state[15] = flags;

        let cv_in = cv;
        compress(&mut state, &msg, &cv_in);
        cv.copy_from_slice(&state[0..8]);
    }
    cv
}

fn parent_cv(left: &[u32; 8], right: &[u32; 8], is_root: bool) -> [u32; 8] {
    let mut msg = [0u32; 16];
    msg[0..8].copy_from_slice(left);
    msg[8..16].copy_from_slice(right);

    let mut state = [0u32; 16];
    state[0..8].copy_from_slice(&IV);
    state[8..12].copy_from_slice(&IV[0..4]);
    state[12] = 0;
    state[13] = 0;
    state[14] = BLOCK_LEN as u32;
    let mut flags = PARENT;
    if is_root {
        flags |= ROOT;
    }
    state[15] = flags;

    compress(&mut state, &msg, &IV);
    let mut cv = [0u32; 8];
    cv.copy_from_slice(&state[0..8]);
    cv
}

pub fn blake3(input: &[u8]) -> [u8; 32] {
    if input.len() <= CHUNK_LEN {
        let cv = hash_chunk(input, 0, true);
        return cv_to_bytes(&cv);
    }

    let mut stack: [[u32; 8]; 64] = [[0u32; 8]; 64];
    let mut stack_len: usize = 0;

    for (i, chunk) in input.chunks(CHUNK_LEN).enumerate() {
        let cv = hash_chunk(chunk, i as u64, false);
        let mut new_cv = cv;
        let mut total = i as u64 + 1;
        while (total & 1) == 0 && stack_len > 0 {
            let left = stack[stack_len - 1];
            stack_len -= 1;
            new_cv = parent_cv(&left, &new_cv, false);
            total >>= 1;
        }
        stack[stack_len] = new_cv;
        stack_len += 1;
    }

    let mut current = stack[stack_len - 1];
    stack_len -= 1;
    while stack_len > 0 {
        let left = stack[stack_len - 1];
        stack_len -= 1;
        let is_root = stack_len == 0;
        current = parent_cv(&left, &current, is_root);
    }
    cv_to_bytes(&current)
}

fn cv_to_bytes(cv: &[u32; 8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, w) in cv.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
    }
    out
}
