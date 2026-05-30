use sp1_sdk::{
    blocking::{ProveRequest, Prover, ProverClient},
    include_elf, Elf, ProvingKey, SP1Stdin,
};
use std::time::Instant;

const ELF: Elf = include_elf!("waddup-bench-program");

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{} KiB", bytes / 1024)
    } else {
        format!("{} MiB", bytes / (1024 * 1024))
    }
}

fn benchmark<P: Prover>(client: &P, size: usize) {
    let file_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    println!(
        "--- SHA-256 (precompile) + BLAKE3 (PR #2615 precompile) | {} ({} bytes) ---",
        format_size(size),
        size
    );

    let expected_sha256: [u8; 32] = {
        use sha2::{Digest, Sha256};
        Sha256::digest(&file_data).into()
    };
    let expected_blake3: [u8; 32] = *blake3::hash(&file_data).as_bytes();

    let mut stdin = SP1Stdin::new();
    stdin.write_vec(file_data.clone());

    let exec_start = Instant::now();
    let (public_values, report) = client.execute(ELF.clone(), stdin.clone()).run().unwrap();
    let exec_elapsed = exec_start.elapsed();

    let pv = public_values.as_slice();
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

    println!("  Total user cycles:   {}", report.total_instruction_count());
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

    let setup_start = Instant::now();
    let pk = client.setup(ELF.clone()).expect("failed to setup elf");
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
    let sizes: Vec<usize> = if args.is_empty() {
        vec![10 * 1024]
    } else {
        args.iter()
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

    println!("waddup SP1 BLAKE3-precompile bench (PR #2615 ported to v6.2.2)");
    println!("===============================================================");
    println!();

    for size in sizes {
        benchmark(&client, size);
    }
}
