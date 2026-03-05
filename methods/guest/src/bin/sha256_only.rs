#![no_main]

use risc0_zkvm::guest::env;
use sha2::{Digest, Sha256};
use std::io::Read;

risc0_zkvm::guest::entry!(main);

fn main() {
    let c0 = env::cycle_count();

    let mut file_data = Vec::new();
    env::stdin().read_to_end(&mut file_data).unwrap();
    let c1 = env::cycle_count();

    let sha256_hash: [u8; 32] = Sha256::digest(&file_data).into();
    let c2 = env::cycle_count();

    env::commit_slice(&sha256_hash);
    env::commit_slice(&c0.to_le_bytes());
    env::commit_slice(&c1.to_le_bytes());
    env::commit_slice(&c2.to_le_bytes());
}
