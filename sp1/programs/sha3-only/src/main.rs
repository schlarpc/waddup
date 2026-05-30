#![no_main]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let file_data: Vec<u8> = sp1_zkvm::io::read_vec();

    println!("cycle-tracker-report-start:sha3");
    let sha3_hash = sp1_precompiles_lib::sha3_256(&file_data);
    println!("cycle-tracker-report-end:sha3");

    sp1_zkvm::io::commit_slice(&sha3_hash);
}
