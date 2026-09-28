# exp-binius: Binius64 prover throughput on SHA-256 / BLAKE3 / Keccak (Ryzen 7 5800X)

## Setup

- Repo: https://github.com/IrreducibleOSS/binius64, default branch, shallow clone.
  Commit: `441fbf51ff0bcb0bcd28f3f1b73f4954029e8577` (2026-09-27, "Bump taiki-e/install-action ... (#2484)").
- Toolchain: `rust-toolchain.toml` pins **stable Rust 1.98.1** (not nightly). Installed with
  `rustup toolchain install 1.98.1 --profile minimal` inside `nix develop /home/schlarpc/waddup#sp1`.
- Build: `RUSTFLAGS="-C target-cpu=native" cargo +1.98.1 build --release -p binius-examples --bin binius-examples`
  (workspace release profile already has `lto = "thin"`, `debug = true`). Clean build: 1 min 36 s.
- One local patch to my clone (NOT the project repo): 12 lines in `crates/examples/src/main.rs` registering the
  already-existing `circuits::independent_hashes::{IndependentSha256Compressions, IndependentBlake3Compressions,
  IndependentKeccakPermutations}` circuits as CLI subcommands `sha256_compressions`, `blake3_compressions`,
  `keccak_permutations` (they were only reachable from the criterion benches). No circuit/prover code was touched.
- Benchmarks run with `flock .../bench.lock`, `\time -v` for peak RSS, `RUST_LOG=info`. Rayon default = 16 threads
  unless noted. Raw logs: `runs/*.log` in this directory.
- **Load caveat**: during the first pass the box was under very heavy load from other agents' builds (1-min load
  average 27–52 on a 16-thread machine; recorded per run below). Runs marked as repeats were re-done at load 2–9 and
  are the numbers used for the derived throughput. Even those may be slightly pessimistic vs. an idle box.

## CPU feature path (no GFNI, no AVX-512 on this CPU)

`rustc +1.98.1 --print cfg -C target-cpu=native` enables `avx2`, `pclmulqdq`, `vpclmulqdq`; no `avx512f`, no `gfni`.

- The prover's field type is `OptimalPackedB128` (`crates/field/src/arch/arch_optimal.rs`):
  `avx512f` → `PackedGhash4x128b`; **`avx2` → `PackedGhash2x128b` (what we get)**; `sse2` → `PackedGhash1x128b`.
  The GHASH GF(2^128) multiply in `packed_ghash_256.rs` is gated on `vpclmulqdq` for the 256-bit CLMUL path,
  which this CPU has. So Binius64's hot field arithmetic runs the intended 256-bit CLMUL SIMD path here; only the
  512-bit lane width is missing.
- GFNI (`crates/field/src/arch/x86_64/gfni/`) is only used by the AES-tower packed fields (`packed_aes_*.rs`),
  which the Binius64 prover in this repo does **not** use (it is GHASH-based). So the lack of GFNI costs nothing.
- README does not warn about GFNI/AVX-512; it only says to set `-C target-cpu=native`. DEVELOPMENT.md notes CI
  runs a pinned CLMUL/AVX2 set and only *check-compiles* the AVX-512/GFNI paths.

## Security parameters used by the examples

- `crates/verifier/src/verify.rs`: `pub const SECURITY_BITS: usize = 96;` (not configurable from the CLI).
- `--log-inv-rate` default **1** (rate 1/2 Reed–Solomon code); minimum allowed 1.
- `n_test_queries = ceil(SECURITY_BITS / -log2(0.5*(1+rate)))` (`crates/iop/src/fri/common.rs:506`)
  = **232 FRI queries** at log_inv_rate=1 (142 at log_inv_rate=2). This is the unique-decoding-radius
  per-query error `(1+ρ)/2`, i.e. a *proven* (not conjectured) bound, but only 96-bit target.
- Merkle hash suite: SHA-256 (default; `--hash-suite blake3` available). Non-ZK mode (default); `--zk` exists.
- No "insecure/test parameters" flags exist in the examples; these are the library's normal parameters.
  96 bits is lower than the 100–128 bits most zkVMs target.

## Circuit shape / constraint counts (from `binius-examples <circuit> stat`)

Binius64 constraints are over 64-bit words: AND constraints (bitwise), ZERO constraints (linear XOR relations),
IMUL (64-bit integer multiply; unused by hashes). "Committed trace" = witness words committed via the PCS.

| circuit (per primitive) | AND | ZERO | committed words | note |
|---|---|---|---|---|
| SHA-256 compression, chained `sha256 --message-len` | ~364 / 64-B block | ~124 | ~462 | 4 KiB: 23,660 AND for 64 blocks; 1 MiB: 5,964,140 AND for 16,384 blocks |
| SHA-256 compression, independent (`sha256_compressions`) | 364 | 124 | 496 | n=1024: 372,736 AND, 126,976 ZERO, 507,904 words |
| BLAKE3 compression, independent (`blake3_compressions`, single-lane gadget) | 192 | 129 | 340 | n=1024: 196,608 AND, 132,096 ZERO, 348,160 words |
| BLAKE3 single-chunk hash (`blake3 --message-len 1024`, 16 compressions) | 168 | 87 | 254 | 2,688 AND, 1,389 ZERO, 4,069 words for 1 KiB |
| Keccak-f[1600] permutation (`keccak_permutations`) | 600 | 25 | 625 | n=1024: 614,400 AND, 25,600 ZERO, 640,000 words |

Notes:
- The `blake3_compress` example subcommand (2x-lane-packed `blake3_compress_2x`) is **broken as a benchmark**: it
  never asserts its output, so the frontend prunes everything and `stat` reports **0 AND constraints / 0
  evaluation instructions** (the shipped snapshot `snapshots/blake3_compress.snap` shows the same). I did not use it.
  The 2x-packed gadget (two 32-bit compressions per 64-bit word) is what the real `blake3_fixed` uses, which is why
  the single-chunk hash costs 168 AND/compression vs 192 for the single-lane independent gadget.
- The `blake3` hash example is limited to a single 1024-byte chunk (`ensure!(len_bytes <= CHUNK_BYTES)`); there is
  no multi-chunk BLAKE3 tree gadget wired into the examples yet (the `blake3_compress_2x_seq`, chunk and tree
  gadgets exist in `crates/circuits/src/blake3/`). So large-message BLAKE3 throughput is measured via independent
  compressions, which is the right unit anyway (BLAKE3 tree = 1 compression per 64 B + ~1/16 parent compressions).
- SHA-256 per block: 364 AND. For comparison SHA-256 has 64 rounds × (Ch, Maj, 3 32-bit adds...) — Binius64 packs
  the 32-bit ops into 64-bit words with free XOR/shift, which is where the low count comes from.

## Timed results

All "Prove" times are the prover's own `Prove [...]` tracing span (the `Prover::prove` call only). "Verify" is the
outer `Verify` span of the example (includes proof deserialization + public-input evaluation; the inner
cryptographic verify is 20–55 ms). "Total wall" is the whole process incl. circuit building, setup, witness gen.
Peak RSS is `\time -v` for the whole process. Threads: Rayon default 16 unless noted.

### A. Full pipeline (`binius-examples <circuit> prove ...`), chained SHA-256 hash of a fixed-length message

| input | blocks | AND constr. | witness words | build circuit | setup | **Prove** | Verify | proof | peak RSS | total wall | load@start |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 256 KiB (`sha256 prove --message-len 262144`) | 4,096 | 1,491,308 | 1,892,835 | 9.3 s | 0.99 s | **0.656 s** | 0.283 s | 310 KiB (317,920 B) | 2.18 GB | 12.7 s | 28.7 |
| 1 MiB (`--message-len 1048576`) | 16,384 | 5,964,140 | 7,569,891 | 38.9 s | 4.48 s | **1.19 s** | 0.366 s | 390 KiB (399,936 B) | 8.58 GB | 51.6 s | 30.5 |
| 2 MiB (`--message-len 2097152`) | 32,768 | 11,927,916 | 15,139,299 | 43.7 s | 5.18 s | **1.52 s** | 0.405 s | 427 KiB (437,984 B) | 17.1 GB | 56.9 s | 4.1 |
| 4 MiB (`--message-len 4194304`) | 65,536 | (~23.9M) | (~30.3M) | **killed** after 15.7 min still in "Building circuit", 21 GB RSS, load 44–52 | | | | | >21 GB | — | 51.5 |

Exact proof sizes (`proof_size_bytes`): 256 KiB → 317,920 B; 1 MiB → 399,936 B; 2 MiB → 437,984 B; BLAKE3 16384 → 399,856 B; Keccak 4096 → 356,704 B; BLAKE3 1 KiB → 61,728 B.

### B. Prover only (`sha256 load-prove` on the saved constraint system / witness / key collection)

Separates the prover from the single-threaded frontend. RSS here is dominated by the loaded constraint system
(1.4 GB on disk at 1 MiB) and key collection (0.9 GB), not by the prover's working set.

| input | threads | load@start | Setup | **Prove** | Verify | peak RSS | wall (incl. 12 s of file loading at 1 MiB) |
|---|---|---|---|---|---|---|---|
| 256 KiB | 16 | 41.8 | 0.49 s | 1.27 s | 0.133 s | 1.44 GB | 4.9 s |
| 256 KiB | 16 | 9.1 | 0.27 s | **0.202 s** | 0.068 s | 1.44 GB | 3.2 s |
| 1 MiB | 16 | 40.1 | 3.65 s | 2.38 s | 0.616 s | 5.73 GB | 21.9 s |
| 1 MiB | 16 | 2.2 | 1.06 s | **0.774 s** | 0.208 s | 5.73 GB | 8.2 s |
| 1 MiB | 8 (RAYON_NUM_THREADS=8) | 3.9 | 1.02 s | **0.755 s** | 0.201 s | 5.73 GB | 7.8 s |
| 1 MiB | 8 | 31.7 | 2.12 s | 1.34 s | 0.340 s | 5.73 GB | 17.4 s |
| 1 MiB | 1 (RAYON_NUM_THREADS=1) | 7.5 | 1.05 s | **3.28 s** | 0.709 s | 5.72 GB | 15.3 s |

Prove-phase breakdown at 1 MiB (16T, clean run): Commit witness ~0.09 s (7.4%), BitAnd sumcheck ~0.48 s (40%),
Shift reduction ~0.49 s (41%), PCS opening (FRI/BaseFold) ~0.02 s (2%). The prover is sumcheck-bound, not
commitment-bound. 8 threads = 16 threads (SMT adds nothing); 1 thread is only 4.2x slower than 8, i.e. parallel
efficiency ~53% on 8 cores.

### C. Other primitives (full pipeline, independent primitive instances, random inputs)

| circuit | n | input bytes | AND | witness words | build | setup | **Prove** | Verify | proof | peak RSS | load |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `blake3_compressions -n 4096` | 4,096 | 256 KiB | 786,432 | 1,392,640 | 17.4 s | 2.08 s | 1.22 s | 0.233 s | 310 KiB | 1.53 GB | 52.2 (noisy) |
| `blake3_compressions -n 16384` | 16,384 | 1 MiB | 3,145,728 | 5,570,560 | 37.3 s | 6.54 s | 1.15 s | 0.327 s | 390 KiB | 5.81 GB | 49.4 (noisy) |
| `blake3_compressions -n 16384` (repeat) | 16,384 | 1 MiB | 3,145,728 | 5,570,560 | 13.1 s | 2.09 s | **0.585 s** | 0.141 s | 390 KiB | 5.73 GB | 3.8 |
| `sha256_compressions -n 4096` | 4,096 | 256 KiB | 1,490,944 | 2,031,616 | 13.3 s | 1.69 s | 0.458 s | 0.130 s | 310 KiB | 2.28 GB | 43.3 |
| `sha256_compressions -n 16384` | 16,384 | 1 MiB | 5,963,776 | 8,126,464 | 47.0 s | 5.92 s | 1.84 s | 0.432 s | 390 KiB | 8.98 GB | 42.5 (noisy) |
| `sha256_compressions -n 16384` (repeat) | 16,384 | 1 MiB | 5,963,776 | 8,126,464 | 23.9 s | 2.75 s | **0.763 s** | 0.202 s | 390 KiB | 8.98 GB | 9.1 |
| `keccak_permutations -n 4096` | 4,096 | 544 KiB (136 B rate) | 2,457,600 | 2,560,000 | 12.8 s | 2.34 s | 0.470 s | 0.146 s | 348 KiB | 5.18 GB | 9.8 |
| `keccak_permutations -n 4096` (repeat) | 4,096 | 544 KiB | 2,457,600 | 2,560,000 | 12.7 s | 2.35 s | **0.462 s** | 0.145 s | 348 KiB | 5.21 GB | 9.3 |
| `blake3 prove --message-len 1024` (real single-chunk hash) | 16 | 1 KiB | 2,688 | 4,069 | 8.7 ms | 6.8 ms | **2.8 ms** | 0.85 ms | 60 KiB | 16 MB | 2.8 |

## Derived throughput (prover wall time only, clean-load runs, 16 threads)

| workload | bytes | Prove | **bytes of hash input / s** |
|---|---|---|---|
| SHA-256 chained, 256 KiB (prover-only) | 262,144 | 0.202 s | **1.30 MB/s (1.24 MiB/s)** |
| SHA-256 chained, 1 MiB (prover-only) | 1,048,576 | 0.774 s | **1.35 MB/s (1.29 MiB/s)** |
| SHA-256 chained, 2 MiB (full pipeline, load 4) | 2,097,152 | 1.52 s | **1.38 MB/s (1.32 MiB/s)** |
| SHA-256 independent compressions, 1 MiB | 1,048,576 | 0.763 s | 1.37 MB/s |
| BLAKE3 compressions (single-lane gadget), 1 MiB | 1,048,576 | 0.585 s | **1.79 MB/s (1.71 MiB/s)** |
| Keccak-f[1600], 4096 perms (Keccak-256 rate 136 B) | 557,056 | 0.462 s | 1.21 MB/s |
| **SHA-256 + BLAKE3 over the same 1 MiB (sum: 0.774 + 0.585 s)** | 1,048,576 | 1.36 s | **0.77 MB/s (0.74 MiB/s ≈ 755 KiB/s)** |

Rate per constraint: ~7.7M AND constraints/s (SHA-256, 16T, 1 MiB) ≈ 9.8M witness words/s. Single thread:
1.82M AND/s → SHA-256 at ~0.32 MB/s on one core.

Fixed vs per-row: 256 KiB → 0.202 s, 1 MiB → 0.774 s, 2 MiB → 1.52 s: linear within noise, fixed overhead of the
prove call ≈ 10–20 ms. The *real* fixed/superlinear costs live outside `prove`: single-threaded frontend circuit
construction (9 s / 39 s / 44 s / >15 min at 256 KiB / 1 MiB / 2 MiB / 4 MiB-under-load) and `Setup`
(key-collection build, 1–5 s). Those are per-circuit-shape costs and would be paid once (or cached via
`save`/`load-prove`) in a production deployment; they are not part of "prover throughput" but they are what
makes the shipped example unusable above ~2 MiB on this box.

Comparison with baselines (same box, from RESEARCH.md): SP1 monolithic w/ precompiles ~1.6 KiB/s; SP1 decomposed
1 MiB in 2300 s (0.45 KiB/s); RISC Zero 1 MiB in 3591 s (0.29 KiB/s).
- SHA-256 alone: 1 MiB in 0.77 s vs 2300 s (SP1) → **~3000x** faster prover time, ~800x vs the 1.6 KiB/s figure.
- SHA-256 + BLAKE3 together (the waddup statement): ~755 KiB/s → **~470x** the 1.6 KiB/s baseline; this is
  4.7x above the "100x" target (160 KiB/s) and ~0.46x of the "1000x" target (1.6 MiB/s). With the 2x-packed
  BLAKE3 gadget (168 vs 192 AND/compression) and no other change, the combined figure would be ~800 KiB/s.
  Reaching 1.6 MiB/s for both hashes would need ~2x more from the prover (e.g. AVX-512/GFNI hardware, or a
  faster machine — this is a 2020 Zen 3 desktop).

## Memory model and scaling to 1 GiB

- **The whole non-public witness is committed as one oracle** (`crates/prover/src/prove.rs:103-115`:
  `pack_witness(...)` then `channel.send_oracle(witness_packed)`), one Merkle tree over the rate-1/2 RS codeword.
  There is no streaming/chunking inside the prover. The witness itself is small: 7.57M words × 8 B = 60 MB at 1 MiB
  of SHA-256 input (packed as GF(2^128) elements: 121 MB; codeword at rate 1/2: 242 MB).
- Measured peak RSS: full pipeline 2.2 GB / 8.6 GB / 17.1 GB at 256 KiB / 1 MiB / 2 MiB (≈ 8.3 GB per MiB,
  linear); prover-only (`load-prove`) 1.44 GB / 5.7 GB at 256 KiB / 1 MiB. Serialized constraint system is
  350 MB / 1.40 GB, key collection 224 MB / 896 MB. So memory is dominated by the **flat, fully-unrolled constraint
  system** (each of the ~6M AND constraints stored explicitly with its operand lists, ≈ 235 B/constraint on disk)
  and the frontend gate graph, not by the polynomial commitment.
- Extrapolation to 1 GiB (16.8M SHA-256 blocks): 6.1G AND constraints (+3.2G for BLAKE3), ~7.7G witness words
  (62 GB witness alone), ~1.4 TB serialized constraint system, ~8.5 TB frontend RSS at the measured slope. **A single
  monolithic Binius64 proof of 1 GiB is impossible on this (or any) box with the flat constraint system.**
  Prover *time* would be fine: ~1024 × 0.77 s ≈ 13 min for SHA-256, ≈ 23 min for SHA-256 + BLAKE3, if memory were
  not the problem.
- Chunking is therefore required. Two routes visible in the repo:
  1. Chunk the input (e.g. 1 MiB per proof, chaining the SHA-256 state and BLAKE3 chaining values through public
     inputs): 1024 proofs × ~400 KiB = ~400 MB of proofs, 8.6 GB RSS per chunk, ~23 min total prover time on this
     box; a `crates/recursion` crate exists for aggregating proofs but I did not test it.
  2. The **M4 data-parallel prover** (`crates/m4-prover`, "chips": `register_chip(Sha256Compress2x)` /
     `Blake3Compress2x`, `builder.build_m4()`, `ValueTable`): the circuit for one compression is stored once and the
     witness is a table of N rows, which is exactly the shape of "hash 1 GiB". This is the design intended for large
     uniform workloads, but **none of the CLI examples use it** (only unit tests in `crates/circuits/src/sha256/mod.rs`
     do), so I could not benchmark it in the time box. Its per-row memory would be the value table only
     (~500 words × 8 B = 4 KB per SHA-256 block → ~66 GB at 1 GiB, still needing chunking at ~100–200 MiB per proof).

## Honest assessment

What worked:
- Binius64 built cleanly on stable Rust 1.98.1 with `-C target-cpu=native` in 96 s; the AVX2 + VPCLMULQDQ path is
  the intended optimized path on this CPU (GFNI/AVX-512 are not used by the GHASH-based Binius64 prover at all),
  so the numbers are not handicapped by the missing features except for the 256- vs 512-bit lane width.
- SHA-256 prover throughput ≈ **1.3 MB/s**, BLAKE3 compressions ≈ **1.8 MB/s**, both hashes together ≈
  **0.75 MB/s**, 96-bit security, rate 1/2, 232 FRI queries, proofs of 310–430 KiB, verify 70–400 ms.
  That is ~470x the SP1-with-precompiles baseline for the combined statement, ~3000x vs SP1 decomposed / RISC Zero
  on prover time per MiB.

What did not work / caveats:
- The 4 MiB point could not be measured: the example's frontend (circuit construction) is single-threaded and
  memory-hungry (8.3 GB per MiB of SHA-256 input) and was still building after 15 min under load 44–52 at 21 GB
  RSS; I killed it (10-minute rule, memory pressure for other agents). 2 MiB completed in 57 s total.
- Timings were taken on a heavily loaded box (load 27–52 during the first pass); repeats at load 2–9 were 1.5–3x
  faster, and those are the numbers I quote. A truly idle box may be a bit faster still.
- The measured code is **example-grade**: fixed-length hash circuits from `crates/examples`, random or
  deterministic inputs, non-ZK mode, default `--hash-suite sha256` for the Merkle tree. The circuit library
  (`crates/circuits`) itself looks production-oriented (reference-tested gadgets, snapshot-tested constraint
  counts), but no one has audited this for me and I did not.
- The `blake3_compress` example subcommand proves an empty circuit (no output assertion → 0 constraints); its
  numbers would be meaningless, which is why I used `IndependentBlake3Compressions` (exposed via a 12-line CLI patch
  in my clone) and the single-chunk `blake3` hash. The BLAKE3 figure is for independent compressions of random
  blocks with the *single-lane* gadget (192 AND), not a real multi-chunk BLAKE3 tree; a real tree costs ~+6%
  parent compressions but can use the 2x-packed gadget (168 AND), so ±10% of what is reported.
- Security: `SECURITY_BITS = 96` is hard-coded in `crates/verifier/src/verify.rs`; the query count uses the
  proven unique-decoding bound (232 queries at rate 1/2), so there is no conjectural gap, but 96 bits is below
  the 100–128 bits zkVMs usually target. No "insecure"/"test-only" parameters are involved. Public inputs
  (message words + digest) are `inout` values that the verifier receives in the clear (2 MB of public input at
  1 MiB — the verifier's "Verify Public Input" phase is the largest verifier cost); for the waddup use case the
  file would presumably be a private witness with only the digests public, which makes the verifier cheaper and the
  prover essentially unchanged.
- Peak RSS is for the whole process; the prover's own working set is far smaller than the 5.7 GB prover-only
  figure (which includes the loaded 1.4 GB constraint system and 0.9 GB key collection).

## Files

- Clone + patch: `/tmp/claude-1000/-home-schlarpc-waddup/4dfd21de-19a0-4f53-b05a-3d664b78352f/scratchpad/exp-binius/binius64` (`git diff` shows the 12-line `crates/examples/src/main.rs` patch).
- Binary: `.../exp-binius/target/release/binius-examples`; harness `.../exp-binius/bench.sh`; raw logs `.../exp-binius/runs/*.log`;
  saved constraint systems `.../exp-binius/saved/` (2.9 GB, safe to delete).
