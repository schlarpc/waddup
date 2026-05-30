use sp1_sdk::{
    blocking::{ProveRequest, Prover, ProverClient},
    include_elf, Elf, ProvingKey, SP1Stdin,
};
use std::time::Instant;

const HASH_PROVER_ELF: Elf = include_elf!("sp1-hash-prover");
const SHA256_ONLY_ELF: Elf = include_elf!("sp1-sha256-only");
const SHA3_ONLY_ELF: Elf = include_elf!("sp1-sha3-only");

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{} KiB", bytes / 1024)
    } else {
        format!("{} MiB", bytes / (1024 * 1024))
    }
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
            Mode::Both => "SHA-256 (precompile) + BLAKE3 (software)",
            Mode::Sha256Only => "SHA-256 only (precompile)",
            Mode::Sha3Only => "SHA3-256 only (keccak precompile)",
        }
    }

    fn elf(self) -> Elf {
        match self {
            Mode::Both => HASH_PROVER_ELF,
            Mode::Sha256Only => SHA256_ONLY_ELF,
            Mode::Sha3Only => SHA3_ONLY_ELF,
        }
    }
}

fn benchmark<P: Prover>(client: &P, mode: Mode, size: usize) {
    let file_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    println!(
        "--- {} | {} ({} bytes) ---",
        mode.label(),
        format_size(size),
        size
    );

    // Compute expected hashes natively
    let expected_sha256: [u8; 32] = {
        use sha2::{Digest, Sha256};
        Sha256::digest(&file_data).into()
    };
    let expected_blake3: [u8; 32] = *blake3::hash(&file_data).as_bytes();
    let expected_sha3: [u8; 32] = {
        use tiny_keccak::{Hasher, Sha3};
        let mut h = Sha3::v256();
        h.update(&file_data);
        let mut out = [0u8; 32];
        h.finalize(&mut out);
        out
    };

    let mut stdin = SP1Stdin::new();
    stdin.write_vec(file_data.clone());

    // Execute first (no proving) to get cycle counts cheaply.
    let elf = mode.elf();
    let exec_start = Instant::now();
    let (public_values, report) = client.execute(elf.clone(), stdin.clone()).run().unwrap();
    let exec_elapsed = exec_start.elapsed();

    let pv = public_values.as_slice();
    match mode {
        Mode::Both => {
            let sha256_hash: [u8; 32] = pv[0..32].try_into().unwrap();
            let blake3_hash: [u8; 32] = pv[32..64].try_into().unwrap();
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
        }
        Mode::Sha256Only => {
            let sha256_hash: [u8; 32] = pv[0..32].try_into().unwrap();
            println!("  SHA-256 (journal):   {}", hex::encode(sha256_hash));
            println!("  SHA-256 (native):    {}", hex::encode(expected_sha256));
            println!(
                "  SHA-256 match:       {}",
                if sha256_hash == expected_sha256 { "OK" } else { "MISMATCH!" }
            );
            assert_eq!(sha256_hash, expected_sha256);
        }
        Mode::Sha3Only => {
            let sha3_hash: [u8; 32] = pv[0..32].try_into().unwrap();
            println!("  SHA3-256 (journal):  {}", hex::encode(sha3_hash));
            println!("  SHA3-256 (native):   {}", hex::encode(expected_sha3));
            println!(
                "  SHA3-256 match:      {}",
                if sha3_hash == expected_sha3 { "OK" } else { "MISMATCH!" }
            );
            assert_eq!(sha3_hash, expected_sha3);
        }
    }

    let total_cycles = report.total_instruction_count();
    println!("  Total user cycles:   {}", total_cycles);
    if !report.cycle_tracker.is_empty() {
        let mut entries: Vec<(&String, &u64)> = report.cycle_tracker.iter().collect();
        entries.sort_by_key(|(k, _)| (*k).clone());
        println!("  Cycle tracker regions:");
        for (region, cycles) in entries {
            println!(
                "    {:<18} {} cycles ({:.2} cycles/byte)",
                format!("{}:", region),
                cycles,
                *cycles as f64 / size as f64
            );
        }
    }
    let mut sc: Vec<(String, u64)> = report
        .syscall_counts
        .iter()
        .filter(|(_, v)| **v > 0)
        .map(|(k, v)| (format!("{:?}", k), *v))
        .collect();
    sc.sort_by_key(|(k, _)| k.clone());
    if !sc.is_empty() {
        println!("  Syscall counts (non-zero):");
        for (k, v) in &sc {
            println!("    {:<26} {}", k, v);
        }
    }
    println!("  Execute time:        {:?}", exec_elapsed);

    // Now do the actual proof
    let setup_start = Instant::now();
    let pk = client.setup(elf).expect("failed to setup elf");
    let setup_elapsed = setup_start.elapsed();

    let prove_start = Instant::now();
    let proof = client
        .prove(&pk, stdin)
        .run()
        .expect("failed to generate proof");
    let prove_elapsed = prove_start.elapsed();

    let verify_start = Instant::now();
    client
        .verify(&proof, pk.verifying_key(), None)
        .expect("failed to verify proof");
    let verify_elapsed = verify_start.elapsed();

    println!("  Setup time:          {:?}", setup_elapsed);
    println!("  Proving time:        {:?}", prove_elapsed);
    println!("  Verification time:   {:?}", verify_elapsed);
    println!();
}

fn main() {
    sp1_sdk::utils::setup_logger();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mode, size_args) = match args.first().map(|s| s.as_str()) {
        Some("--sha256") => (Mode::Sha256Only, &args[1..]),
        Some("--sha3") => (Mode::Sha3Only, &args[1..]),
        Some("--both") => (Mode::Both, &args[1..]),
        _ => (Mode::Both, args.as_slice()),
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

    let client = ProverClient::from_env();

    println!("waddup SP1 ZK Proof-of-Concept Benchmark");
    println!("=========================================");
    println!("Prover: {}", std::env::var("SP1_PROVER").unwrap_or_else(|_| "cpu".to_string()));
    println!("Mode:   {}", mode.label());
    println!();

    for size in sizes {
        benchmark(&client, mode, size);
    }
}
