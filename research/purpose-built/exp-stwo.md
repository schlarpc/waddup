# exp-stwo: Stwo (Circle STARK / M31) BLAKE2s example AIR prover throughput

Date: 2026-09-28. Machine: Ryzen 7 5800X (8c/16t, Zen 3, AVX2, no AVX-512), 64 GiB, NixOS.

## What was built

- Repo: https://github.com/starkware-libs/stwo, default branch, shallow clone.
  Commit: `826591c6c371376810ca8213b5812d5daf6d5092` (2026-08-13, "Update readme to point to proving repo. (#1457)").
  Note: the README says development moved to `starkware-libs/proving` at end of July 2026; this is the last state of
  the `stwo` repo (workspace version 2.2.0).
- Toolchain: `rust-toolchain.toml` pins `nightly-2026-01-15` (portable_simd). Installed via rustup in the `#sp1` nix shell.
- Build (parallel = rayon feature on; `-C target-cpu=native`):
  ```
  CARGO_TARGET_DIR=<scratch>/exp-stwo/target RUSTFLAGS='-C target-cpu=native' \
    cargo +nightly-2026-01-15 build --release --features parallel -p stwo-examples --example blake_bench
  ```
- Backend: `SimdBackend`. Stwo's SIMD backend is `std::simd::u32x16` (16 lanes of M31). The AVX-512-specific
  M31 multiply paths are behind `cfg(target_feature = "avx512f")` and were NOT compiled on this CPU; the portable
  u32x16 code lowers to pairs of AVX2 ops. StarkWare's published numbers are AVX-512 machines, so expect them to be
  faster per core than what is measured here.
- Source modifications to the clone (bench plumbing only, no prover/AIR logic changed):
  `crates/examples/src/blake/mod.rs` (`mod air` -> `pub mod air`), `crates/examples/src/blake/air.rs`
  (`stark_proof` field made `pub`; added `pub fn blake_log_sizes(log_size)` that returns the existing private
  `BlakeStatement0::log_sizes()`), `crates/examples/Cargo.toml` (+`serde_json` dev-dep for serialized proof size).
  New file `crates/examples/examples/blake_bench.rs`: calls `prove_blake::<Blake2sMerkleChannel>(log_n, config)`
  then `verify_blake`, times both, reads VmHWM from /proc/self/status, and installs a tracing layer that sums the
  prover's own `span!` timings. Runs via `<scratch>/exp-stwo/run.sh <log_n> <config> <tag>` which wraps the binary
  in `flock <scratch>/bench.lock`, logs `uptime` before/after.
- The upstream way to run the same thing is the ignored test
  `LOG_N_INSTANCES=16 cargo test --release --features parallel test_simd_blake_prove -- --ignored --nocapture`
  (uses `PcsConfig::default()`, see security caveat below).

## What the example proves

`crates/examples/src/blake/` is a BLAKE2s **compression function** AIR (the module doc says "AIR for blake2s and
blake3"; `N_ROUNDS = 10`, `ROUND_LOG_SPLIT = [3, 1]`, i.e. rounds are split into an 8-round component and a
2-round component; the comment says "change these for blake3").

- `log_n_instances = L` means 2^L independent BLAKE2s compressions, each on a 16-word state `v` and a 16-word
  (64 B) message `m`. I count 64 B of hash input per compression.
- Three components with LogUp lookups between them:
  - Scheduler (2^L rows, 384 base columns): holds message (16 u32 = 32 felts, u32 split into two 16-bit felts) and
    11 states (input + after each of 10 rounds), and emits one `Round(in, out, msg_permuted)` lookup per round.
  - Round (2^(L+3) + 2^(L+1) rows = 10 * 2^L rows, 384 base + 260 interaction columns per row): one full BLAKE round
    (8 G functions). Per G: 2 add3 + 2 add2 (2 witness cols + boolean/ternary carry constraint each, no lookup) and
    4 xor-rotr (4 limb-split cols + 4 xor-output cols + 4 xor lookups into the 12/9/8/7/4-bit XOR tables).
  - XOR tables (preprocessed + multiplicity + LogUp): xor12 is 2^16 rows x 256 multiplicity columns
    (+512 interaction columns) = ~50M cells of FIXED overhead regardless of L; xor9/8/7/4 are small.
- Cells per compression (all trees, before 2x blowup): 384 + 10*384 base + 24 + 10*260 interaction = **6,848
  M31 cells (~27 KB) per 64 B block** = 107 cells per input byte. Composition polynomial is degree 2 (max constraint
  log-degree bound = log_size + 1). Merkle hasher: Blake2s (vcs_lifted).

Trace shape printed by the wrapper for L=16 (tree 0 = preprocessed, 1 = base, 2 = LogUp interaction):
```
tree 0: 15 columns;   (log_rows, n_cols) = [(8,3),(10,3),(12,3),(14,3),(16,3)]
tree 1: 1457 columns; (log_rows, n_cols) = [(8,1),(10,16),(12,16),(14,16),(16,640),(17,384),(19,384)]
tree 2: 1156 columns; (log_rows, n_cols) = [(8,4),(10,32),(12,32),(14,32),(16,568),(17,260),(19,260)]
total base-field trace cells (all trees, before blowup): ~500M (~1.9 GiB as u32)
```

## PCS / FRI parameters

`PcsConfig::default()` (what the upstream test uses) is **pow_bits=10, log_blowup=1, n_queries=3,
log_last_layer_degree_bound=0, fold_step=1** -> `security_bits()` = pow + log_blowup*n_queries = **13 bits**.
That is a test parameter, not a real security level. `verify_blake` only asserts `security_bits >= 5`.

I also ran a "prod96" config I chose to match what stwo-cairo ships: **pow_bits=26, log_blowup=1, n_queries=70**
-> 96 bits by stwo's own formula (this is the conjectured/"proximity-gap" accounting: 1 bit per query at
blowup 2; under the proven-soundness regime the same parameters would be worth less). Both configs were measured.

## Measurements (16 rayon threads unless noted; peak RSS = VmHWM of the process)

All runs held the shared `bench.lock`. Several other agents were building/benchmarking concurrently; the 1-min
load average at lock acquisition is given. Runs labelled r3 (load 2-13) are the cleanest; earlier passes at load
33-51 were up to 6x slower for the same size and are listed only to show the noise floor.

| log_n | compressions | input bytes | config  | prove wall s | verify s | proof (bytes, size_estimate / JSON) | peak RSS | load | compr/s | input B/s |
|------:|-------------:|------------:|---------|-------------:|---------:|-----------------------------------:|---------:|-----:|--------:|----------:|
| 12 | 4,096   | 256 KiB | default(13b) | **0.712** | 0.0025 | 91,640 / 281,209     | 0.81 GiB | 4.1  | 5,753  | 368 KB/s |
| 12 | 4,096   | 256 KiB | prod96       | 1.819 (1.10 s of it is the 2^26 PoW grind) | 0.0061 | 988,248 / 2,779,275 | 0.81 GiB | 2.3 | 2,252 | 144 KB/s |
| 14 | 16,384  | 1 MiB   | default(13b) | **1.443** | 0.0024 | 92,248 / 283,589     | 1.60 GiB | 4.2  | 11,351 | 726 KB/s |
| 14 | 16,384  | 1 MiB   | prod96       | **1.636** | 0.0063 | 1,020,264 / 2,895,780 | 1.60 GiB | 7.9 | 10,012 | 641 KB/s |
| 16 | 65,536  | 4 MiB   | default(13b) | **5.019** | 0.0026 | 98,008 / 304,213     | 5.02 GiB | 9.9  | 13,057 | 836 KB/s |
| 16 | 65,536  | 4 MiB   | prod96       | **5.560** | 0.0066 | 1,088,504 / 3,143,939 | 5.02 GiB | 10.6 | 11,786 | 754 KB/s |
| 18 | 262,144 | 16 MiB  | default(13b) | 22.09 | 0.0024 | 102,360 / 320,421    | 19.2 GiB | 7.3  | 11,865 | 759 KB/s |
| 18 | 262,144 | 16 MiB  | prod96       | **21.20** | 0.0072 | 1,161,896 / 3,408,930 | 19.2 GiB | 10.7 | 12,364 | 791 KB/s |
| 14 | 16,384  | 1 MiB   | prod96, RAYON_NUM_THREADS=1 | 7.033 | 0.0062 | 1,020,264 | 1.58 GiB | 13.4 | 2,330 | 149 KB/s |

Noisy earlier passes (same binary): L=12 default 1.09 s @load 50 and 6.25 s @load 51; L=14 default 14.65 s @load 18
and 11.37 s @load 51; L=16 default 32.3 s @load 29 and 11.8 s @load 43; L=16 prod96 8.29 s @load 34.

Headline: **~12-13k BLAKE2s compressions/s = ~0.75-0.84 MB/s (~730-815 KiB/s) of BLAKE2s input at 96-bit-claimed
parameters, 16 threads, on this AVX2 box.** Proof size at 96 bits is ~1.0-1.2 MB (70 queries x ~2,600 columns of
Merkle decommitments; this is a flat/un-recursed STARK proof).

Fixed vs per-row: fitting prove_s vs N on the r3 default runs gives ~73 us/compression marginal (14->16) and
~80 us/compression (16->18, prod96), with a fixed part of only ~0.25-0.5 s (twiddles, xor12 table commit, PoW grind,
FRI tail). At L>=14 the run is >85% per-row work; at L=12 roughly 35-60% is fixed (and the PoW grind is a random
0.2-1.1 s). Memory scales linearly (~77 KiB RSS per compression, ~19 GiB at 2^18); 2^20 compressions (64 MiB) would
need ~80 GiB, so bigger inputs must be chunked into multiple proofs (or recursed).

Where the time goes (prover's own spans, L=18 prod96, 21.2 s total; nested spans overlap): trace generation
9.5 s (Round Generation 5.3 s, Scheduler 0.8 s), commitments 8.5 s (Extension/LDE 2.8 s + Merkle 3.1 s + interpolation),
LogUp interaction trace 5.6 s, composition/constraint eval 3.1 s, OODS + FRI quotients ~2 s, grind 0.5 s.
Witness generation is ~45% of the total, i.e. the SIMD trace generator (which runs the real BLAKE G function in
u32x16 and records limbs) is as expensive as the polynomial commitment work.

Parallel scaling: 1 thread 7.03 s vs 16 threads 1.64 s at L=14 (4.3x on 8 physical cores; the single-thread run
had load 13 so this understates it somewhat).

## Derived: SHA-256 and combined SHA-256 + BLAKE3 estimate

Assumptions: same framework (M31 Circle STARK, 16-bit limbs for u32, XOR via 8/12-bit lookup tables, LogUp), same
per-cell cost as measured here (~80 us per 6,848-cell compression = ~11.7 ns per trace cell all-in at 16 threads,
including witness gen, LDE, Merkle, LogUp and constraint evaluation), and that cost is proportional to trace cells.

- BLAKE3 compression: 7 rounds of the identical G function (7 x 644 = 4,508 round cells) + a smaller scheduler
  (8 states instead of 11, ~300 cells) = ~4,800 cells / 64 B block, i.e. **0.70x the BLAKE2s cost**
  -> ~17.5k blocks/s = ~1.1 MB/s standalone. BLAKE3 over a large F also needs parent-node compressions
  (1 per 2 KiB of input, +3%) and the chaining/counter wiring, which I ignore.
- SHA-256 per round in the same style: Sigma1 and Sigma0 (3 rotations XORed: ~5 limb-split cols + 12 xor lookups
  with 6-8-bit limbs -> ~40 cells each), Ch and Maj (3 bitwise ops x 4 8-bit limbs via AND/XOR tables -> ~36 cells
  each), message schedule sigma0/sigma1 (~80 cells, amortized 48/64 -> 60), five 32-bit adds (~12 cells),
  state carry-through per round row (8 words = 16 felts + round lookup, ~20 cells): **~245-270 cells/round x 64
  rounds = ~16-17k cells / 64 B block = ~2.4-2.5x BLAKE2s.** Rough sanity check from op counts: a SHA-256 round has
  ~10 bitwise 32-bit ops + ~8 adds vs a BLAKE round's 32 xor-rotr + 48 adds, i.e. ~1/3 of a BLAKE round, times 64
  rounds ~= 21 BLAKE-round equivalents vs 10, which gives the same 2.1-2.5x. So SHA-256 alone: ~5.0-5.5k
  blocks/s = **~330 KB/s** on this box.
- Combined statement "SHA-256(F) and BLAKE3(F)" over the same bytes: (2.45 + 0.70) x BLAKE2s cost = ~3.15x
  -> ~3.9k 64-B blocks/s = **~250 KB/s (~245 KiB/s) at 96-bit parameters, 16 threads, AVX2.** Range given the
  fit spread and estimate error: 200-300 KB/s.

Versus the baselines (RESEARCH.md): SP1 v6.2.2 monolithic ~1.6 KiB/s; SP1 decomposed 1 MiB / 2300 s = 0.45 KiB/s;
RISC Zero 1 MiB / 3591 s = 0.29 KiB/s. A hand-written Stwo AIR for both hashes lands at roughly **150x the SP1
monolithic number** (comfortably past the 100x = 160 KiB/s target) but about **6x short of the 1000x = 1.6 MiB/s
target** on this CPU. Measured BLAKE2s-only is ~500x SP1. An AVX-512 machine or a GPU backend (stwo-gpu exists
for NVIDIA; nothing for the RX 7900 XT here) would move this; so would cheaper bitwise decomposition
(e.g. proving SHA-256 in a binary-field system instead), but that is outside this experiment.

## Honest assessment / caveats

- **Example-grade code.** `crates/examples` is described upstream as "intentionally minimal" reference circuits.
  The prover library itself (`crates/stwo`) is production (it backs SHARP / stwo-cairo), so the per-cell prover
  cost is representative; the AIR is not.
- **The statement is not bound to any public input.** `verify_blake` only checks that all LogUp claimed sums add
  to zero and that the STARK verifies. The scheduler's "blake" relation (input state, output state, message) is
  added with multiplicity `E::EF::zero()` and the code carries `TODO(alont): Remove blake interaction` and
  `TODO(shahars): Add inputs to sum, and constraint them`. So the proof shows "2^L rows exist whose round
  relations are consistent", not "BLAKE2s(m) = h for these public m, h". Binding inputs/outputs, chaining blocks
  (Merkle-Damgard for SHA-256, chunk/parent tree for BLAKE3) and initial-state constants would add a few columns
  per row and equality/lookup constraints; cost impact is small (a few %), soundness impact is essential.
- **Inputs are synthetic and independent**: `v` and `m` are `i + 2j` patterns, not a real hash of a real file;
  rows are not chained. Each row does run a genuine BLAKE2s round in the trace generator.
- The round component uses `add2/add3_u32_unchecked` whose comments say the caller is responsible for range
  checks that the subsequent XOR lookups are asserted to provide ("These also guarantee that all elements are in
  range"). I did not audit whether every intermediate is actually covered; the comment `// TODO: Separate lookups
  by w.` in `xor2` also hints at unfinished parts. Treat constraint completeness as unverified.
- **Default parameters are insecure** (13 bits; the verifier accepts >= 5). All headline numbers are from my
  96-bit config (pow 26, blowup 2, 70 queries), which costs only ~10% more prover time but ~11x the proof size.
  The 96 bits are by stwo's conjectured accounting.
- Timing noise: other agents ran concurrently; load at lock acquisition was 2-13 for the reported runs. Earlier
  passes at load 30-50 were 2-6x slower. The numbers above are effectively best-of-2/3 under moderate load, so
  the true quiet-machine figures are likely a bit better, not worse.
- `proof_size_estimate_bytes` is stwo's own byte estimate of the proof struct; the JSON size is given only as
  an upper bound (a compact binary encoding would be close to the estimate).
- Could not measure: SHA-256 or BLAKE3 directly (no such AIR in stwo; the estimate above is a cell-count
  extrapolation), GPU, AVX-512, recursion/aggregation cost, and any size above 2^18 compressions (RSS ~19 GiB;
  2^20 would not fit).

## Files

- Clone + modifications: `<scratch>/exp-stwo/stwo/` (diff: `crates/examples/src/blake/{mod,air}.rs`,
  `crates/examples/Cargo.toml`, new `crates/examples/examples/blake_bench.rs`).
- Binary: `<scratch>/exp-stwo/target/release/examples/blake_bench <log_n> <default|prod96|prod96b2|prod128>`.
- Runner: `<scratch>/exp-stwo/run.sh`; logs: `<scratch>/exp-stwo/run_<log_n>_<config>_<tag>.log` (r3 = clean pass).
  `<scratch>` = `/tmp/claude-1000/-home-schlarpc-waddup/4dfd21de-19a0-4f53-b05a-3d664b78352f/scratchpad`.
