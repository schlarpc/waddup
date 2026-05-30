//! Minimal SHA-256 and Keccak-f[1600] wrappers that dispatch to SP1's precompile
//! syscalls.
//!
//! Why this exists: the official `sp1-patches/RustCrypto-hashes` and
//! `sp1-patches/tiny-keccak` branches gate their `succinct` cfg on
//! `target_arch = "riscv32"` and (for sha2) declare the syscalls with the old
//! u32-pointer ABI. SP1 v6.x targets `riscv64im-succinct-zkvm-elf` and exposes
//! `*mut [u64; N]` syscall signatures. Result: the patched crates silently fall
//! back to soft impls when built for SP1 v6.x guests. We bypass them by calling
//! `sp1_lib::syscall_*` directly with the correct ABI.
#![no_std]

extern crate alloc;
use alloc::vec::Vec;

/// SHA-256 of an arbitrary-length byte slice using SP1's `SHA_EXTEND` +
/// `SHA_COMPRESS` precompiles. Output: 32-byte big-endian digest.
pub fn sha256(input: &[u8]) -> [u8; 32] {
    // SHA-256 IV (FIPS 180-4 §5.3.3).
    let mut state: [u64; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
        0x1f83d9ab, 0x5be0cd19,
    ];

    // Standard padding: append 0x80, then zeros, then 64-bit big-endian length
    // in bits. Total length is a multiple of 64 bytes.
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
            // Big-endian load of 4 bytes into u32, zero-extended to u64.
            w[j] = u32::from_be_bytes([
                block[j * 4],
                block[j * 4 + 1],
                block[j * 4 + 2],
                block[j * 4 + 3],
            ]) as u64;
        }
        unsafe {
            sp1_lib::syscall_sha256_extend(&mut w as *mut [u64; 64]);
            sp1_lib::syscall_sha256_compress(
                &mut w as *mut [u64; 64],
                &mut state as *mut [u64; 8],
            );
        }
    }

    let mut out = [0u8; 32];
    for i in 0..8 {
        out[i * 4..i * 4 + 4].copy_from_slice(&(state[i] as u32).to_be_bytes());
    }
    out
}

/// SHA3-256 (NIST FIPS 202 padding) using SP1's `KECCAK_PERMUTE` precompile.
/// Output: 32 bytes. Matches `tiny_keccak::Sha3::v256().update(input).finalize(out)`.
pub fn sha3_256(input: &[u8]) -> [u8; 32] {
    // SHA3-256: rate = 136 bytes (17 lanes of 8B), capacity = 64 bytes (8 lanes).
    const RATE_BYTES: usize = 136;
    let mut state: [u64; 25] = [0; 25];

    // Absorb full 136-byte blocks.
    let mut iter = input.chunks_exact(RATE_BYTES);
    for block in iter.by_ref() {
        for (lane, chunk) in block.chunks_exact(8).enumerate() {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(chunk);
            state[lane] ^= u64::from_le_bytes(bytes);
        }
        unsafe {
            sp1_lib::syscall_keccak_permute(&mut state as *mut [u64; 25]);
        }
    }

    // Final block with NIST SHA3 padding: 0x06 then trailing 0x80 at byte
    // RATE-1. (Mid-block 0x06 and end-byte 0x80 may coincide for length 135.)
    let tail = iter.remainder();
    let mut pad = [0u8; RATE_BYTES];
    pad[..tail.len()].copy_from_slice(tail);
    pad[tail.len()] |= 0x06;
    pad[RATE_BYTES - 1] |= 0x80;
    for (lane, chunk) in pad.chunks_exact(8).enumerate() {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(chunk);
        state[lane] ^= u64::from_le_bytes(bytes);
    }
    unsafe {
        sp1_lib::syscall_keccak_permute(&mut state as *mut [u64; 25]);
    }

    // Squeeze first 32 bytes (4 lanes).
    let mut out = [0u8; 32];
    for lane in 0..4 {
        out[lane * 8..lane * 8 + 8].copy_from_slice(&state[lane].to_le_bytes());
    }
    out
}
