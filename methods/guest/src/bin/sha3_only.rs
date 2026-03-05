#![no_main]

use risc0_zkvm::guest::env;
use std::io::Read;
use tiny_keccak::{Hasher, Sha3};

risc0_zkvm::guest::entry!(main);

fn main() {
    let c0 = env::cycle_count();

    let mut file_data = Vec::new();
    env::stdin().read_to_end(&mut file_data).unwrap();
    let c1 = env::cycle_count();

    let mut hasher = Sha3::v256();
    hasher.update(&file_data);
    let mut sha3_hash = [0u8; 32];
    hasher.finalize(&mut sha3_hash);
    let c2 = env::cycle_count();

    env::commit_slice(&sha3_hash);
    env::commit_slice(&c0.to_le_bytes());
    env::commit_slice(&c1.to_le_bytes());
    env::commit_slice(&c2.to_le_bytes());
}
