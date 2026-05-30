#![no_main]
sp1_zkvm::entrypoint!(main);

mod blake3_precompile;
mod sha256_precompile;

pub fn main() {
    let file_data: Vec<u8> = sp1_zkvm::io::read_vec();

    println!("cycle-tracker-report-start:sha256");
    let sha256_hash = sha256_precompile::sha256(&file_data);
    println!("cycle-tracker-report-end:sha256");

    println!("cycle-tracker-report-start:blake3");
    let blake3_hash = blake3_precompile::blake3(&file_data);
    println!("cycle-tracker-report-end:blake3");

    sp1_zkvm::io::commit_slice(&sha256_hash);
    sp1_zkvm::io::commit_slice(&blake3_hash);
}
