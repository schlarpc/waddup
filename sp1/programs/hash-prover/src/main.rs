#![no_main]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let file_data: Vec<u8> = sp1_zkvm::io::read_vec();

    println!("cycle-tracker-report-start:sha256");
    let sha256_hash = sp1_precompiles_lib::sha256(&file_data);
    println!("cycle-tracker-report-end:sha256");

    println!("cycle-tracker-report-start:blake3");
    let blake3_hash: [u8; 32] = *blake3::hash(&file_data).as_bytes();
    println!("cycle-tracker-report-end:blake3");

    sp1_zkvm::io::commit_slice(&sha256_hash);
    sp1_zkvm::io::commit_slice(&blake3_hash);
}
