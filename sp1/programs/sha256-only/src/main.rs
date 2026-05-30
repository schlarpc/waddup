#![no_main]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let file_data: Vec<u8> = sp1_zkvm::io::read_vec();

    println!("cycle-tracker-report-start:sha256");
    let sha256_hash = sp1_precompiles_lib::sha256(&file_data);
    println!("cycle-tracker-report-end:sha256");

    sp1_zkvm::io::commit_slice(&sha256_hash);
}
