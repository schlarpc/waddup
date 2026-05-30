//! SHA-256-of-file using sp1-lib's syscall_sha256_extend + syscall_sha256_compress.
//! Identical to the wrapper used in the main project; duplicated here so the
//! port-specific guest is self-contained.

use sp1_lib::{syscall_sha256_compress, syscall_sha256_extend};

pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut state: [u64; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
        0x1f83d9ab, 0x5be0cd19,
    ];

    let bit_len = (input.len() as u64).wrapping_mul(8);
    let padded_len = {
        let mut n = input.len() + 1 + 8;
        n = (n + 63) & !63;
        n
    };
    let mut buf: Vec<u8> = Vec::with_capacity(padded_len);
    buf.extend_from_slice(input);
    buf.push(0x80);
    while buf.len() < padded_len - 8 {
        buf.push(0);
    }
    buf.extend_from_slice(&bit_len.to_be_bytes());

    let mut w: [u64; 64] = [0; 64];
    for block in buf.chunks_exact(64) {
        for j in 0..16 {
            w[j] = u32::from_be_bytes([
                block[j * 4],
                block[j * 4 + 1],
                block[j * 4 + 2],
                block[j * 4 + 3],
            ]) as u64;
        }
        unsafe {
            syscall_sha256_extend(&mut w as *mut [u64; 64]);
            syscall_sha256_compress(&mut w as *mut [u64; 64], &mut state as *mut [u64; 8]);
        }
    }

    let mut out = [0u8; 32];
    for i in 0..8 {
        out[i * 4..i * 4 + 4].copy_from_slice(&(state[i] as u32).to_be_bytes());
    }
    out
}
