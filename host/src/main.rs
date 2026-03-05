use methods::{
    HASH_PROVER_ELF, HASH_PROVER_ID, SHA256_ONLY_ELF, SHA256_ONLY_ID, SHA3_ONLY_ELF,
    SHA3_ONLY_ID,
};
use risc0_zkvm::{default_prover, ExecutorEnv};
use std::io::Cursor;
use std::time::Instant;

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{} KiB", bytes / 1024)
    } else {
        format!("{} MiB", bytes / (1024 * 1024))
    }
}

fn read_u64_le(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

#[derive(Clone, Copy)]
enum Mode {
    Both,
    Sha256Only,
    Sha3Only,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Mode::Both => "SHA-256 (accelerated) + BLAKE3 (software)",
            Mode::Sha256Only => "SHA-256 only (accelerated)",
            Mode::Sha3Only => "SHA3-256 only (keccak precompile)",
        }
    }

    fn elf(self) -> &'static [u8] {
        match self {
            Mode::Both => HASH_PROVER_ELF,
            Mode::Sha256Only => SHA256_ONLY_ELF,
            Mode::Sha3Only => SHA3_ONLY_ELF,
        }
    }

    fn image_id(self) -> [u32; 8] {
        match self {
            Mode::Both => HASH_PROVER_ID,
            Mode::Sha256Only => SHA256_ONLY_ID,
            Mode::Sha3Only => SHA3_ONLY_ID,
        }
    }
}

fn benchmark(mode: Mode, size: usize) {
    let file_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    println!(
        "--- {} | {} ({} bytes) ---",
        mode.label(),
        format_size(size),
        size
    );

    // Compute expected hashes outside zkVM
    let expected_sha256: [u8; 32] = {
        use sha2::{Digest, Sha256};
        Sha256::digest(&file_data).into()
    };
    let expected_blake3: [u8; 32] = *blake3::hash(&file_data).as_bytes();
    let expected_sha3: [u8; 32] = {
        use sha3::{Digest, Sha3_256};
        Sha3_256::digest(&file_data).into()
    };

    // Send data as raw bytes via POSIX stdin (no serde overhead)
    let env = ExecutorEnv::builder()
        .stdin(Cursor::new(file_data.clone()))
        .build()
        .unwrap();

    let prover = default_prover();

    let prove_start = Instant::now();
    let prove_info = prover.prove(env, mode.elf()).unwrap();
    let prove_elapsed = prove_start.elapsed();

    let receipt = prove_info.receipt;
    let j = &receipt.journal.bytes;

    match mode {
        Mode::Both => {
            // Journal: sha256[32] + blake3[32] + c0[8] + c1[8] + c2[8] + c3[8]
            let sha256_hash: [u8; 32] = j[0..32].try_into().unwrap();
            let blake3_hash: [u8; 32] = j[32..64].try_into().unwrap();

            println!("  SHA-256 (journal):   {}", hex::encode(sha256_hash));
            println!("  SHA-256 (native):    {}", hex::encode(expected_sha256));
            println!(
                "  SHA-256 match:       {}",
                if sha256_hash == expected_sha256 { "OK" } else { "MISMATCH!" }
            );
            println!("  BLAKE3  (journal):   {}", hex::encode(blake3_hash));
            println!("  BLAKE3  (native):    {}", hex::encode(expected_blake3));
            println!(
                "  BLAKE3  match:       {}",
                if blake3_hash == expected_blake3 { "OK" } else { "MISMATCH!" }
            );
            assert_eq!(sha256_hash, expected_sha256);
            assert_eq!(blake3_hash, expected_blake3);

            let c0 = read_u64_le(j, 64);
            let c1 = read_u64_le(j, 72);
            let c2 = read_u64_le(j, 80);
            let c3 = read_u64_le(j, 88);
            println!("  Guest cycle breakdown:");
            println!("    startup+read:      {} cycles", c1 - c0);
            println!("    sha256:            {} cycles ({:.1} cycles/byte)", c2 - c1, (c2 - c1) as f64 / size as f64);
            println!("    blake3:            {} cycles ({:.1} cycles/byte)", c3 - c2, (c3 - c2) as f64 / size as f64);
        }
        Mode::Sha256Only => {
            let sha256_hash: [u8; 32] = j[0..32].try_into().unwrap();

            println!("  SHA-256 (journal):   {}", hex::encode(sha256_hash));
            println!("  SHA-256 (native):    {}", hex::encode(expected_sha256));
            println!(
                "  SHA-256 match:       {}",
                if sha256_hash == expected_sha256 { "OK" } else { "MISMATCH!" }
            );
            assert_eq!(sha256_hash, expected_sha256);

            let c0 = read_u64_le(j, 32);
            let c1 = read_u64_le(j, 40);
            let c2 = read_u64_le(j, 48);
            println!("  Guest cycle breakdown:");
            println!("    startup+read:      {} cycles", c1 - c0);
            println!("    sha256:            {} cycles ({:.1} cycles/byte)", c2 - c1, (c2 - c1) as f64 / size as f64);
        }
        Mode::Sha3Only => {
            let sha3_hash: [u8; 32] = j[0..32].try_into().unwrap();

            println!("  SHA3-256 (journal):  {}", hex::encode(sha3_hash));
            println!("  SHA3-256 (native):   {}", hex::encode(expected_sha3));
            println!(
                "  SHA3-256 match:      {}",
                if sha3_hash == expected_sha3 { "OK" } else { "MISMATCH!" }
            );
            assert_eq!(sha3_hash, expected_sha3);

            let c0 = read_u64_le(j, 32);
            let c1 = read_u64_le(j, 40);
            let c2 = read_u64_le(j, 48);
            println!("  Guest cycle breakdown:");
            println!("    startup+read:      {} cycles", c1 - c0);
            println!("    sha3:              {} cycles ({:.1} cycles/byte)", c2 - c1, (c2 - c1) as f64 / size as f64);
        }
    }

    let verify_start = Instant::now();
    receipt.verify(mode.image_id()).unwrap();
    let verify_elapsed = verify_start.elapsed();

    println!("  Proving time:        {:?}", prove_elapsed);
    println!("  Verification time:   {:?}", verify_elapsed);
    println!("  Total segments:      {}", prove_info.stats.segments);
    println!("  Total cycles:        {}", prove_info.stats.total_cycles);
    println!("  User cycles:         {}", prove_info.stats.user_cycles);
    println!();
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (mode, size_args) = if args.first().map(|s| s.as_str()) == Some("--sha256") {
        (Mode::Sha256Only, &args[1..])
    } else if args.first().map(|s| s.as_str()) == Some("--sha3") {
        (Mode::Sha3Only, &args[1..])
    } else if args.first().map(|s| s.as_str()) == Some("--both") {
        (Mode::Both, &args[1..])
    } else {
        (Mode::Both, args.as_slice())
    };

    let sizes: Vec<usize> = if size_args.is_empty() {
        vec![10 * 1024]
    } else {
        size_args
            .iter()
            .map(|s| {
                let s = s.to_lowercase();
                if let Some(n) = s.strip_suffix("k") {
                    n.parse::<usize>().unwrap() * 1024
                } else if let Some(n) = s.strip_suffix("m") {
                    n.parse::<usize>().unwrap() * 1024 * 1024
                } else {
                    s.parse::<usize>().unwrap()
                }
            })
            .collect()
    };

    println!("waddup ZK Proof-of-Concept Benchmark");
    println!("=====================================");
    println!("Prover: CPU (local)");
    println!("Mode:   {}", mode.label());
    println!();

    for size in sizes {
        benchmark(mode, size);
    }
}
