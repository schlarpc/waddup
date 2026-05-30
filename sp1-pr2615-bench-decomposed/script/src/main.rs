use sp1_sdk::{
    blocking::{ProveRequest, Prover, ProverClient},
    include_elf, Elf, HashableKey, ProvingKey, SP1Proof, SP1ProofWithPublicValues,
    SP1Stdin, SP1VerifyingKey,
};
use std::time::{Duration, Instant};

const CHUNK_ELF: Elf = include_elf!("waddup-chunk-prover");
const ROOT_ELF: Elf = include_elf!("waddup-root-prover");

const PROOF_CHUNK_SIZE: usize = 65536; // 64 KiB per chunk_prover invocation.
const BLAKE3_CHUNK_LEN: usize = 1024;

fn sha256_state_at_boundaries(file: &[u8], num_chunks: usize) -> Vec<[u32; 8]> {
    use sha2::{
        compress256,
        digest::generic_array::{typenum::U64, GenericArray},
    };
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
        0x1f83d9ab, 0x5be0cd19,
    ];
    let mut boundaries = Vec::with_capacity(num_chunks + 1);
    boundaries.push(state);
    for ci in 0..num_chunks {
        let start = ci * PROOF_CHUNK_SIZE;
        let end = ((ci + 1) * PROOF_CHUNK_SIZE).min(file.len());
        let chunk = &file[start..end];
        debug_assert!(chunk.len() % 64 == 0, "chunk must be block-aligned");
        let blocks_per_chunk = chunk.len() / 64;
        let mut blocks: Vec<GenericArray<u8, U64>> = Vec::with_capacity(blocks_per_chunk);
        for bi in 0..blocks_per_chunk {
            let mut block = [0u8; 64];
            block.copy_from_slice(&chunk[bi * 64..(bi + 1) * 64]);
            blocks.push(block.into());
        }
        compress256(&mut state, &blocks);
        boundaries.push(state);
    }
    boundaries
}

fn main() {
    sp1_sdk::utils::setup_logger();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let size: usize = if args.is_empty() {
        1024 * 1024
    } else {
        let s = args[0].to_lowercase();
        if let Some(n) = s.strip_suffix("k") {
            n.parse::<usize>().unwrap() * 1024
        } else if let Some(n) = s.strip_suffix("m") {
            n.parse::<usize>().unwrap() * 1024 * 1024
        } else {
            s.parse::<usize>().unwrap()
        }
    };
    assert!(
        size % PROOF_CHUNK_SIZE == 0,
        "size {} must be a multiple of {} for this POC",
        size,
        PROOF_CHUNK_SIZE
    );

    let num_chunks = size / PROOF_CHUNK_SIZE;

    // File contents (deterministic pseudo-random matching the existing benches).
    let file: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

    println!("waddup decomposed bench");
    println!("  file size:        {} bytes ({} MiB)", size, size / 1024 / 1024);
    println!("  proof chunk size: {} bytes", PROOF_CHUNK_SIZE);
    println!("  num chunk proofs: {}", num_chunks);
    println!(
        "  blake3 leaves:    {} ({} per chunk)",
        num_chunks * PROOF_CHUNK_SIZE / BLAKE3_CHUNK_LEN,
        PROOF_CHUNK_SIZE / BLAKE3_CHUNK_LEN,
    );

    // Native expected hashes (so we can validate the receipt).
    let expected_sha256: [u8; 32] = {
        use sha2::{Digest, Sha256};
        Sha256::digest(&file).into()
    };
    let expected_blake3: [u8; 32] = *blake3::hash(&file).as_bytes();
    println!("  expected SHA-256: {}", hex::encode(expected_sha256));
    println!("  expected BLAKE3:  {}", hex::encode(expected_blake3));
    println!();

    // One ProverClient for the whole run — this is where session-level cache
    // (lookup tables, basefold prep, gnark FFI init) gets amortized across
    // all the prove() calls below.
    let setup_total = Instant::now();
    let client = ProverClient::from_env();
    let client_init = setup_total.elapsed();

    let pk_chunk_start = Instant::now();
    let pk_chunk = client.setup(CHUNK_ELF.clone()).expect("setup chunk_prover");
    let pk_chunk_elapsed = pk_chunk_start.elapsed();

    let pk_root_start = Instant::now();
    let pk_root = client.setup(ROOT_ELF.clone()).expect("setup root_prover");
    let pk_root_elapsed = pk_root_start.elapsed();

    let vk_chunk: SP1VerifyingKey = pk_chunk.verifying_key().clone();
    let vk_chunk_hash: [u32; 8] = vk_chunk.hash_u32();

    println!("Setup phase:");
    println!("  ProverClient init: {:?}", client_init);
    println!("  setup chunk pk:    {:?}", pk_chunk_elapsed);
    println!("  setup root pk:     {:?}", pk_root_elapsed);
    println!();

    // Precompute SHA-256 boundary states natively.
    let boundaries = sha256_state_at_boundaries(&file, num_chunks);
    assert_eq!(boundaries.len(), num_chunks + 1);

    // Prove each chunk. Use .compressed() so the root can verify_sp1_proof them.
    let mut chunk_proofs: Vec<SP1ProofWithPublicValues> = Vec::with_capacity(num_chunks);
    let mut per_chunk_times: Vec<Duration> = Vec::with_capacity(num_chunks);
    println!("Chunk proving phase ({} chunks):", num_chunks);
    for ci in 0..num_chunks {
        let chunk_bytes: Vec<u8> =
            file[ci * PROOF_CHUNK_SIZE..(ci + 1) * PROOF_CHUNK_SIZE].to_vec();
        let mut stdin = SP1Stdin::new();
        stdin.write(&(ci as u32));
        stdin.write(&boundaries[ci]);
        stdin.write_vec(chunk_bytes);

        let t = Instant::now();
        let proof = client
            .prove(&pk_chunk, stdin)
            .compressed()
            .run()
            .expect("chunk prove failed");
        let elapsed = t.elapsed();
        per_chunk_times.push(elapsed);
        println!("  chunk {:3}/{:3}: {:?}", ci + 1, num_chunks, elapsed);
        chunk_proofs.push(proof);
    }
    let chunk_phase_total: Duration = per_chunk_times.iter().sum();
    let chunk_phase_avg = chunk_phase_total / num_chunks as u32;
    println!("  total: {:?}, avg per chunk: {:?}", chunk_phase_total, chunk_phase_avg);
    println!();

    // Build root stdin.
    let mut root_stdin = SP1Stdin::new();
    root_stdin.write(&vk_chunk_hash);
    root_stdin.write(&(file.len() as u64));
    root_stdin.write(&(num_chunks as u32));
    for proof in &chunk_proofs {
        root_stdin.write_vec(proof.public_values.to_vec());
    }
    // Add each compressed chunk proof for SP1 to witness inside verify_sp1_proof.
    for proof in &chunk_proofs {
        let SP1Proof::Compressed(inner) = proof.proof.clone() else {
            panic!("expected compressed proof");
        };
        root_stdin.write_proof(*inner, vk_chunk.vk.clone());
    }

    println!("Root proving phase:");
    let t = Instant::now();
    // Root must be compressed because verify_sp1_proof is wired into the
    // recursion verifier; core-mode root proofs can't witness the sub-proofs.
    let root_proof = client
        .prove(&pk_root, root_stdin)
        .compressed()
        .run()
        .expect("root prove failed");
    let root_elapsed = t.elapsed();
    println!("  root prove: {:?}", root_elapsed);

    let pv = root_proof.public_values.as_slice();
    let h_content: [u8; 32] = pv[0..32].try_into().unwrap();
    let h_swarm: [u8; 32] = pv[32..64].try_into().unwrap();
    println!();
    println!("Results:");
    println!("  H_content (journal): {}", hex::encode(h_content));
    println!("  H_content (native):  {}", hex::encode(expected_sha256));
    println!(
        "  H_content match:     {}",
        if h_content == expected_sha256 { "OK" } else { "MISMATCH!" }
    );
    println!("  H_swarm   (journal): {}", hex::encode(h_swarm));
    println!("  H_swarm   (native):  {}", hex::encode(expected_blake3));
    println!(
        "  H_swarm   match:     {}",
        if h_swarm == expected_blake3 { "OK" } else { "MISMATCH!" }
    );

    let t = Instant::now();
    client
        .verify(&root_proof, pk_root.verifying_key(), None)
        .expect("root verify failed");
    let verify_elapsed = t.elapsed();
    println!("  root verify: {:?}", verify_elapsed);

    println!();
    println!("Wall-clock summary:");
    let total = client_init
        + pk_chunk_elapsed
        + pk_root_elapsed
        + chunk_phase_total
        + root_elapsed
        + verify_elapsed;
    println!("  setup:        {:?}", client_init + pk_chunk_elapsed + pk_root_elapsed);
    println!("  chunk total:  {:?}", chunk_phase_total);
    println!("  chunk avg:    {:?}", chunk_phase_avg);
    println!("  root prove:   {:?}", root_elapsed);
    println!("  root verify:  {:?}", verify_elapsed);
    println!("  grand total:  {:?}", total);

    assert_eq!(h_content, expected_sha256);
    assert_eq!(h_swarm, expected_blake3);
}
