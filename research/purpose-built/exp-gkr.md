# exp-gkr: GKR / sum-check prover (Polyhedra Expander) on Keccak-256 and SHA-256 — CPU throughput on Ryzen 7 5800X

Date: 2026-09-28. Agent: exp-gkr. All work under
`/tmp/claude-1000/-home-schlarpc-waddup/4dfd21de-19a0-4f53-b05a-3d664b78352f/scratchpad/exp-gkr/` (raw logs in `runs/`).

## TL;DR

- Prover family: **GKR (Vanilla / Libra-style sum-check over a layered circuit)** from Polyhedra **Expander**
  (commit `096581e`), circuits from **ExpanderCompilerCollection** (ECC, commit `b5a9470`). Field: **GF(2) circuit values with
  GF(2^128) challenges** (8-lane `GF2x8` SIMD packing), Fiat-Shamir = SHA-256, PCS = **Orion** (linear-code PCS, 128-bit target) or
  **Raw** (no commitment; the input layer is shipped in the clear).
- **Keccak-f[1600] (Keccak-256 of a 64-byte message), single-threaded prover: 588-645 Keccak-f/s per core** (low load).
  = **37-40 KiB/s of 64-byte messages per core** (78-86 KiB/s counted at the 136-byte SHA3-256 rate).
  **16 independent single-threaded provers: 5.1-5.7 k Keccak-f/s = 320-360 KiB/s (64 B/hash), 680-760 KiB/s (136 B rate).**
  A single Expander Vanilla-GKR proof is single-threaded; parallelism is only via independent proofs (threads) or MPI ranks.
- **SHA-256 (one compression per 64-byte block) is 10-20x more expensive than Keccak in this binary-field GKR**: the mod-2^32 adders
  produce a 1229-layer circuit with ~2 M XOR + 65 k AND gates per compression. Measured **~34 compressions/s per core = 2.2 KiB/s**
  (standard layered GKR + Orion, verified) and **117-134 compressions/s = 7-8 KiB/s** with Expander's cross-layer (Virgo++) prototype prover
  (prove only; that prototype has no PCS and no verifier). See timed table below.
- Versus the SP1 baseline (1.6 KiB/s SHA-256+BLAKE3, monolithic): Keccak via GKR is **~25x per core / ~200-230x on 16 threads**
  (64 B/hash accounting) — but that is Keccak, not SHA-256. SHA-256 via GKR-over-GF(2) is only ~1.4x per core / ~20x on
  16 threads (extrapolated), i.e. **well short of the 100x target for SHA-256**.
- The practical bottleneck was **not the prover but the ECC circuit compiler**: ~0.8-1.5 s and ~90-100 MB RAM per Keccak instance,
  ~2 s and ~450 MB per SHA-256 compression, so the largest single proof I could build in 90 min covered 1024 Keccak-f (64 KiB of
  messages, 136 KiB at rate). The prover itself scales linearly (per-hash cost constant from 512 to 1024 hashes).

## What was built (exact commits, commands)

| Repo | Commit | Notes |
|---|---|---|
| PolyhedraZK/Expander | `096581e1957c05be034857e24bb6ea8e8875e093` (main, "GPU commit/open threshold 2^20 -> 65536 ...", 2026-07-20) | rust-toolchain `nightly-2025-05-17`; hard dependency on OpenMPI via `rsmpi` |
| PolyhedraZK/ExpanderCompilerCollection (ECC) | `b5a94702d0fc3304f117d0cdba307517b8b3f6b8` (main, 2026-07-20) | its `[patch]` section already points all Expander crates at `../Expander/*`, so ECC was built against the checkout above |
| rsmpi | `61796831954b679cbe267c1b704ddbcb7fef3715` (pinned by Expander) | bindgen -> needs libclang |

Toolchain: rustup nightly-2025-05-17 (rustc 1.89.0-nightly 16d2276fa) from the `#sp1` nix shell, nested in
`nix shell nixpkgs#openmpi nixpkgs#openmpi.dev nixpkgs#pkg-config nixpkgs#time`, `LIBCLANG_PATH` from `llvmPackages.libclang.lib`.
`RUSTFLAGS="-C target-cpu=native"` everywhere (AVX2 GF2/M31 kernels; no AVX-512 on this CPU). Wrapper: `run.sh`.

Build fixes that were required (both repos' HEADs contain un-gated CUDA code):

1. `poly_commit/src/orion/*.rs`, `tree/src/tree.rs` and ECC `zkcuda/*` declare `extern "C" { fn gpu_* }` without any `#[cfg]`, so a
   CPU-only link fails (`gpu_tree_get_ptrs`, `gpu_pcs_open_with_device_data`, `gpu_tree_free`, `gpu_eval_at_challenge_m31ext3`).
   I compiled `gpu_stub.c` (16 aborting stubs, one per `fn gpu_*` found in either repo) and linked it through
   `Expander/bin/build.rs` (`cargo:rustc-link-arg-bins`), `ECC/expander_compiler/build.rs` and `ECC/circuit-std-rs/build.rs`
   (`cargo:rustc-link-arg-tests`). The GPU paths are guarded at runtime by `GPU_TREE_REGISTRY`, which is only populated by the CUDA
   commit path, so no stub is ever reached (none aborted in any run).
2. ECC `Cargo.toml` enables `poly_commit` feature `cuda_pcs`; removed with `sed 's/, features = \["cuda_pcs"\]//'`.
3. Expander's testing SRS helper (`poly_commit/src/utils.rs:19`) caches the SRS at `/tmp/kzg_srs_<n_vars>.bin` keyed only by input
   size, so a Raw-PCS run leaves a 0-byte file that a later Orion run at the same size fails to deserialize
   (`Failed to deserialize SRS for OrionPCSForExpanderGKR PCS`). All my Orion runs `rm -f /tmp/kzg_srs_*.bin` first; the first Orion
   sweep (`runs/expander_*_Orion_*.log`) died from this and was redone (`runs/expander2_*.log`).
4. Benchmark circuit/witness files were fetched directly with `wget` from Expander's R2 bucket (`keccak-ci/serialization-v6/*`).

Build commands:

```
cd Expander && ../run.sh cargo build --release --bin gkr --bin expander-exec --bin gkr-mpi
cd ECC     && ../run.sh cargo test --release -p expander_compiler --test mod --no-run          # keccak bench (tests/circuit/bench_keccak.rs)
cd ECC     && ../run.sh cargo test --release -p circuit-std-rs --test bench_sha256 --no-run    # sha256 bench (tests/bench_sha256.rs)
```

Benchmark drivers (each run records `uptime` first; all timed runs held `flock .../scratchpad/bench.lock`):
`bench_expander.sh` / `bench_expander2.sh` (bundled Expander `gkr` throughput bench), `bench_ecc.sh` (my sized Keccak test),
`bench_sha.sh` (my sized SHA-256 tests). Peak RSS from GNU `time -v`/`-f %M` plus `/proc/self/status` VmHWM/VmRSS printed per phase.

## Circuits

### Keccak (my ECC test `ECC/expander_compiler/tests/circuit/bench_keccak.rs`)

N Keccak-256(64-byte message) instances per SIMD lane, built from ECC's own `keccak_gf2_vec.rs` gadget with
`api.memorized_simple_call` (sub-circuit reuse), compiled with ECC `compile()` (plain layered circuit), 8 witnesses (one per
`GF2x8` lane) solved with `solve_witnesses`, exported with `export_to_expander_flatten()`, proven with
`executor::prove::<GF2ExtConfigSha2{Raw,Orion}>` and checked with `executor::verify`. Each instance is exactly one Keccak-f
permutation (message padded to one 136-byte rate block). Output digests are `PublicVariable`s.

| N / lane | total Keccak-f (x8 lanes) | layers | log2(input vars) | mul (AND) gates / lane | add (XOR) gates / lane | const gates |
|---|---|---|---|---|---|---|
| 8   | 64   | 49 | 12 | 296,448   | 3,736,576  | 3,920  |
| 64  | 512  | 49 | 15 | 2,371,584 | 29,892,608 | 31,360 |
| 128 | 1024 | 49 | 16 | 4,743,168 | 59,785,216 | 62,720 |

= ~37 k AND + ~467 k XOR gates per Keccak-f per lane, 49 layers (2 per round + I/O).

The bundled Expander benchmark (`bin/src/main.rs`, `-c keccak`) uses pre-compiled circuits from Polyhedra's bucket
(`data/circuit_gf2.txt` etc., serialization v6). The bench's own accounting is 1 Keccak per circuit copy x 8 lanes = 8 Keccak-f
per proof for GF2 (2 copies x 16 lanes = 32 for M31x16, 2 x 8 = 16 for Goldilocksx8); its 49,416-byte Raw proof and its
per-thread rate matching my own N=128 circuit (645 vs 588 /s) are consistent with that accounting.

### SHA-256 (my ECC test `ECC/circuit-std-rs/tests/bench_sha256.rs`)

N single compressions (`SHA256GF2::sha256_compress` from `circuit-std-rs`, i.e. one 64-byte block with the IV, no padding block)
per lane, two pipelines: (a) `compile()` + `executor::prove/verify` exactly like Keccak; (b) `compile_cross_layer()` + the
`crosslayer_prototype::prove_gkr` (Virgo++) prover, which is the path ECC's own SHA-256 tests use — it is a prover-only prototype
(no PCS, no verifier, no public inputs).

| N / lane | total compressions | pipeline | layers | log2(input) | mul / lane | add / lane |
|---|---|---|---|---|---|---|
| 8 | 64 | plain layered | 1229 | 13 | 522,592 | 15,820,040 |
| 8 | 64 | cross-layer  | 1229 | 14 (max in/out vars) | n/a | n/a |

= ~65 k AND + ~2.0 M XOR gates per compression per lane and **1229 layers** (the 32-bit ripple-carry adders serialise into depth).
That is ~4x the XOR count and 25x the depth of a Keccak-f, and depth is what GKR pays for (one sum-check per layer).

## Results

Machine: Ryzen 7 5800X (8c/16t), 64 GiB. Other agents were building/benchmarking concurrently; the 1-min load average at the
start of each run is given. Runs at load > ~20 are marked and are lower bounds (their CPU share was reduced).

### A. Bundled Expander throughput bench (`gkr -f <field> -p <pcs> -c keccak -t <threads> -r 2`)

Each thread runs its own independent prover in a loop on the same small circuit; the bench reports aggregate Keccak-f/s
(second 5-second window). Proof size is per proof (8 Keccak-f for GF2, 32 for M31, 16 for Goldilocks). Note this bench
re-proves a fixed tiny circuit, so it measures steady-state per-hash prover cost with no compile/witness/IO cost.

| field / challenge | PCS | threads | load@start | Keccak-f/s | KiB/s (64 B/hash) | KiB/s (136 B rate) | proof bytes | peak RSS |
|---|---|---|---|---|---|---|---|---|
| GF2 / GF(2^128) | Raw   | 1  | 9.3  | **645**   | 40.3  | 85.7  | 49,416    | 80 MiB   |
| GF2 / GF(2^128) | Raw   | 16 | 8.4  | **5,719** | 357   | 760   | 49,416    | 442 MiB  |
| GF2 / GF(2^128) | Orion | 1  | 2.3  | **603**   | 37.7  | 80.1  | 761,944   | 84 MiB   |
| GF2 / GF(2^128) | Orion | 16 | 2.0  | **5,147** | 322   | 684   | 761,944   | 473 MiB  |
| M31x16 / M31^3  | Orion | 1  | 6.7  | 256       | 16.0  | 34.0  | 2,001,456 | 238 MiB  |
| M31x16 / M31^3  | Orion | 16 | 6.1  | 1,606     | 100   | 213   | 2,001,456 | 1.6 GiB  |
| GF2 (load>40)   | Raw   | 1  | 45 * | 234       | 14.6  | 31.1  | 49,416    | 80 MiB   |
| GF2 (load>40)   | Raw   | 8  | 48 * | 1,805     | 113   | 240   | 49,416    | 248 MiB  |
| GF2 (load>40)   | Raw   | 16 | 47 * | 3,716     | 232   | 494   | 49,416    | 442 MiB  |
| M31x16 (load>40)| Raw   | 1 / 8 / 16 | 41-45 * | 147 / 595 / 566 | 9.2 / 37 / 35 | | 224,864 | 224 MiB / 824 MiB / 1.5 GiB |
| Goldilocksx8 (load>40) | Raw | 1 / 8 / 16 | 47-49 * | 42 / 284 / 355 | 2.6 / 18 / 22 | | 268,296 | 223 MiB / 850 MiB / 1.5 GiB |

`*` = measured while other agents' builds saturated all 16 threads; the clean re-runs (first block) are 2.7x faster.
Thread scaling on the clean GF2 runs is 8.9x for 16 threads on 8 physical cores (SMT), i.e. near-linear per physical core.
The GF2 config is the right one for Keccak (bitwise circuit); M31/Goldilocks need bit-decomposition constraints and are
2.5-15x slower here. BN254 (`-f fr`) was not run (far slower still, and irrelevant for this use).

### B. Sized Keccak proofs (my ECC test; single proof, single thread, prove + verify)

Per-phase times from the test's own `Instant` timers; RSS from `/proc/self/status` after each phase and GNU time max RSS.

| N/lane | Keccak-f | PCS | load@start | compile (s) | witness (s) | export+eval (s) | **prove (s)** | verify (s) | proof bytes | Keccak-f/s | **KiB/s (64 B)** | KiB/s (136 B) | RSS after compile | RSS after prove | peak RSS (process) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 64  | 512  | Raw   | 27.4 * | 73.1 | 0.005 | ~3.6 | **0.923** | 0.173 | 118,800   | 555 | 34.7 | 73.7 | 1.9 GiB | ~3.3 GiB | 6.0 GiB |
| 64  | 512  | Orion | 14.1   | 49.2 | 0.005 | ~3.6 | **0.902** | 0.182 | 1,823,264 | 568 | 35.5 | 75.4 | 2.0 GiB | ~3.3 GiB | 5.9 GiB |
| 128 | 1024 | Orion | 8.5    | 99.0 | 0.003 | 3.9  | **1.742** | 0.304 | 1,951,120 | 588 | 36.7 | 78.1 | 4.2 GiB | 6.4 GiB | 12.0 GiB |
| 8 (untimed, no lock) | 64 | Orion | ~40 * | 24.1 | 0.000 | 0.37 | 0.163 | 0.041 | 1,182,864 | 393 | 24.5 | 52.2 | 0.35 GiB | 0.44 GiB | 0.92 GiB |

Fixed vs per-hash cost (Orion, 512 -> 1024 Keccak-f): prove = 0.062 s + 1.64 ms x hashes (fixed part ~3.5 % at 1024 hashes);
verify = 0.06 s + 0.24 ms x hashes. The verifier is **linear in circuit size** (Expander's verifier walks every gate to evaluate the
add/mul wiring multilinears; there is no preprocessing/structured-circuit shortcut in this path), so it is only ~6x faster than the
prover here.

Proof size: Raw = 118.8 KB for 512 hashes — it contains the whole input layer (2^15 GF2x8 words = 32 KiB) plus ~49 layers of
sum-check messages; it grows linearly with input. Orion = 1.8-2.0 MB and nearly size-independent at these sizes (dominated by the
proximity/query openings of the linear-code PCS; `PCS_SOUNDNESS_BITS = 128` -> many Merkle query openings).

Memory attribution (N=128): the process peak (12 GiB) is the ECC compiler. After compile RSS drops to 4.2 GiB (compiler output +
witness solver); exporting/evaluating the *flattened* Expander circuit (65 M gates x ~32 B) adds ~2.3 GiB; proving adds only a few
MB on top (the GKR prover keeps one layer's evaluations + sum-check scratch). So the prover's own working set is ~2.3 MB per
Keccak-f **because ECC's `export_to_expander_flatten` expands every sub-circuit call**; Expander's native circuit format supports
repeated segments (the bundled bench's 8-hash GF2 circuit is a 2.7 MB file and proves in 80 MiB RSS).

### C. Sized SHA-256 (my ECC test; single proof, single thread)

Timed under the lock (`runs/sha256_*.log`), all at load 2-6:

| N/lane | compressions | pipeline | load | compile (s) | **prove (s)** | verify (s) | proof bytes | compressions/s | **KiB/s (64 B)** | RSS after compile | peak RSS (process) |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 8  | 64  | plain layered + Orion, verified   | 5.9 | 18.1 | **1.854** | 0.259 | 3,458,416 | 34.5 | 2.2 | 3.1 GiB  | 3.5 GiB  |
| 32 | 256 | plain layered + Orion, verified   | 3.8 | 81.3 | **7.562** | 0.986 | 3,902,544 | 33.9 | 2.1 | 12.0 GiB | 14.0 GiB |
| 32 | 256 | plain layered + Raw, verified     | 2.1 | 81.1 | **7.463** | 0.965 | 2,198,080 | 34.3 | 2.1 | 12.1 GiB | 14.0 GiB |
| 8  | 64  | cross-layer prototype, prove only | 4.6 | 11.2 | **0.548** | n/a   | n/a (no PCS) | 117 | 7.3 | 1.1 GiB | 1.5 GiB |
| 32 | 256 | cross-layer prototype, prove only | 2.1 | 50.6 | **1.915** | n/a   | n/a (no PCS) | 134 | 8.4 | 4.0 GiB | 6.1 GiB |

Per-compression prover cost is flat (29 ms plain, 7.1 ms cross-layer) from 64 to 256 compressions; fixed overhead is small
(plain: ~0.0 s; cross-layer: 0.09 s). Verify is again linear in circuit size (3.8 ms per compression). The Raw proof for 256
compressions is 2.2 MB (the input layer is 2^15 x 8 lanes of bits plus 1229 layers x sum-check messages: with 1229 layers the
sum-check transcript itself is ~1.9 MB, which is why even Raw is large here). Compile RAM: ~0.45 GB per compression instance
(14 GiB peak at 256), so 2^12 compressions per proof would need ~60+ GiB just to compile with ECC's flattening pipeline.

## Security parameters / soundness caveats visible in the code

- GKR scheme `Vanilla` (`gkr/src/prover/gkr_vanilla.rs`), degree-2 gates (add/mul + const/uni), two-phase sum-check per layer.
  Challenges are sampled from **GF(2^128)** for the GF2 config (`ChallengeField = GF2_128`, `FIELD_SIZE = 128`), so the
  Schwartz-Zippel loss per sum-check round is ~2^-128 x degree; total GKR soundness loss over 49 x ~2 x 16 rounds is ~2^-115.
  For M31x16 the challenge field is M31^3 (~93 bits) — noticeably weaker; for Goldilocksx8 it is Goldilocks^2 (~128 bits).
- Fiat-Shamir transcript: `BytesHashTranscript<SHA256hasher>` (`FiatShamirHashType::SHA256`) — standard; MiMC/Poseidon variants exist
  for recursion.
- PCS `Raw` (`poly_commit/src/raw.rs`): **not a commitment**. The prover sends the entire input multilinear (the witness/input layer)
  and the verifier evaluates it. Sound, but zero-knowledge-free and non-succinct in the input; only meaningful when the verifier
  already holds the input (which for "prove the hash of my file" defeats the purpose) or as a building block under recursion.
- PCS `Orion` (`poly_commit/src/orion/`): Brakedown/Orion-style linear-code PCS with `PCS_SOUNDNESS_BITS = 128`, code parameters
  `ORION_CODE_PARAMETER_INSTANCE { alpha_g0: 0.33, degree_g0: 6, alpha_g1: 0.337, degree_g1: 6, hamming_weight: 0.055 }`; the
  number of proximity repetitions and column queries is derived from the 128-bit target and the field size at run time
  (`query_complexity`, `proximity_repetitions` in `poly_commit/src/traits.rs`). The expander-graph code's relative distance is
  heuristic (random graphs, no explicit distance proof), as in Orion/Brakedown. The SRS is generated with a test RNG by
  `expander_pcs_init_testing_only` (transparent setup, so this is fine) but the on-disk cache is keyed by size only (bug above).
- Public inputs: in my Keccak circuit the digests are `PublicVariable`s and are checked by the verifier
  (`circuit.public_input` passed to `verify`). In the cross-layer prototype public inputs are unsupported
  (`assert_eq!(simd_public_input.len(), 0)`), so that path proves only "I know a satisfying witness" with no binding to a
  public digest, and it has no verifier at all — its numbers are prover-cost-only.
- Circuit correctness: ECC's Keccak gadget is the project's own test circuit and passed the layered-circuit `run()` check against
  `tiny_keccak` for all 8 lanes at every size; SHA-256 likewise against the reference compressor in `sha256_debug_utils.rs`.
  I did not audit the gadgets for under-constraint; note that over GF(2) the AND/XOR gates *are* the bit operations, so there are
  no range/booleanity side constraints to forget (unlike the M31/Goldilocks formulations).
- Everything here uses `expander_pcs_init_testing_only`, the `Vanilla` scheme and test-grade CLI/test harnesses. The prover
  core is the production Expander code path, but the harnesses (SRS cache, `--test-threads=1` cargo tests, `MPIConfig::prover_new(None,None)`
  single-process mode) are example/test grade.

## Assessment

What worked: Expander HEAD builds on this box (after stubbing 16 un-gated CUDA externs, dropping the `cuda_pcs` feature, and
routing around the SRS-cache bug), MPI only as a link-time dependency (`mpirun` was never needed). Keccak-f over GF(2) proves at
~600 Keccak-f/s per core (~37 KiB/s of 64-byte messages, ~80 KiB/s at the 136-byte rate) with proofs verifying, and 16 parallel
provers reach ~5.5 k Keccak-f/s (~350 KiB/s at 64 B/hash, ~740 KiB/s at rate). Per-hash prover cost is flat from 8 to 1024 hashes
per proof, so the throughput extrapolates.

What did not: (1) SHA-256 in this binary-field GKR is expensive (1229 layers): ~34 compressions/s/core = 2.2 KiB/s, or 117-134/s
(7-8 KiB/s) with the prototype cross-layer prover that has no PCS/verifier. That is ~1.4x the SP1 SHA-256 baseline per core and
~20x on 16 cores — nowhere near 100x. A GKR SHA-256 would need an arithmetisation with native 32-bit adds (M31/Goldilocks with
lookups) or the cross-layer prover finished; neither exists in a runnable form here. (2) The ECC compiler, not the prover, capped the
proof size: ~1 s and ~100 MB per Keccak instance, ~2 s and ~450 MB per SHA-256 compression, so a 2^16-hash proof would need
~6 GiB... ~60 GiB just to compile. Real deployments pre-compile once and reuse the (segmented) circuit; the bundled bench does exactly
that and shows the prover's own RSS is tiny (80 MiB for a running GF2 prover). (3) Load from concurrent agents made the first sweep 2.7x
slow; the clean re-run is the one to trust.

Could a GKR prover stream a 1 GiB file with bounded memory? Yes, structurally: GKR commits only to the input layer and keeps one
layer of evaluations at a time (prover RSS ~80 MiB for the 8-hash circuit, ~2.3 GiB for a *flattened* 1024-hash circuit, of which
almost all is the circuit description, not witness). The natural design is a fixed circuit of K Keccak-f per proof (K x 8 lanes),
streamed over the file in chunks: 1 GiB = 16.8 M 64-byte blocks (or 7.9 M 136-byte rate blocks); at 5.5 k Keccak-f/s that is
~51 min (64 B accounting) / ~24 min (136 B accounting) on this CPU with ~0.5-2 GiB RSS per prover. The catch is chaining: hashing a
file is sequential (each block's state feeds the next), so the K-hash-per-proof chunks must pass the 1600-bit state between proofs as
public I/O, and you end up with ~16 k proofs of 0.76-2 MB each (Orion) unless they are aggregated with Expander's recursion crate
(BN254/KZG, not measured). Expander's MPI mode (`gkr-mpi`) instead proves world_size x K hashes in one aggregated proof across
processes — I did not run it (time), but the bench code multiplies throughput by `world_size`, i.e. it is the same per-core cost with
one proof.

Bottom line for the project: as a third family, GKR-over-GF(2) is the fastest per-core Keccak prover measured here (~600 Keccak-f/s,
~37-80 KiB/s per core depending on accounting) with small prover memory, but SHA-256 is the wrong hash for it (2-7 KiB/s/core), and the
tooling (HEAD does not link CPU-only, SRS cache bug, flattening compiler, prototype cross-layer prover) is research-grade.
