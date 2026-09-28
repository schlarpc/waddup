# exp-flock: Flock (binary-field sum-check) SHA-256 / BLAKE3 proving on Ryzen 7 5800X

Date: 2026-09-28. Machine: AMD Ryzen 7 5800X (Zen 3, 8c/16t; AVX2, PCLMULQDQ, VPCLMULQDQ, SHA-NI; **no AVX-512, no GFNI**), 64 GiB RAM, NixOS, Linux 7.2.

## What was built

| Repo | Commit | Toolchain | Build |
|---|---|---|---|
| https://github.com/succinctlabs/flock | `b684b1258e4b1f202bec24afd660ace851b09e5e` (shallow, HEAD 2026-09-28) | Rust stable 1.93.1 (repo has no `rust-toolchain`; edition 2024, stable is enough) | `RUSTFLAGS="-C target-cpu=native" cargo bench --no-run -p flock-prover --bench hash_throughput --bench sha2_proof --bench blake3_proof` (bench profile = release + debuginfo). 3 m 22 s. |
| https://github.com/microsoft/Spartan2 | `c0ee259053cd12eaf43ed71b5cde375452b3ee4d` | – | Cloned and inspected only, **not benchmarked** (see "Spartan2 / BinarySpartan" below). |

Benchmarks run: `crates/flock-prover/benches/sha2_proof.rs` and `blake3_proof.rs` (no-harness binaries; each size = 1 untimed warm-up + best-of-3 timed `prove_fast`, then one extra prove for heap-peak/verify/proof-size). Both use the **real Fiat-Shamir challenger** (`FsChallenger`, SHA-256 transcript), not the dev-only `RandomChallenger`, and the verifier is run on every size (`expect("verify failed")` — all passed). Exact command per run (from `run_bench.sh`):

```
flock <bench.lock> env RAYON_NUM_THREADS=<T> SHA2_LOG2S="12 14 16" BLAKE3_LOG2S="12 14 16" PCS_TRACE=1 \
  /run/current-system/sw/bin/time -v target/release/deps/{sha2,blake3}_proof-<hash> --bench
```

Raw logs: `runs/*.log` in this directory.

## Which SIMD path Flock actually uses on this CPU

Backend selection is **entirely compile-time `cfg(target_feature = ...)`** — there is no runtime `is_x86_feature_detected!` anywhere in the workspace, and no Cargo feature flag to pick a path. With `-C target-cpu=native` on Zen 3 (`avx2 pclmulqdq vpclmulqdq aes sha_ni`, no `avx512f`, no `gfni`) the gates resolve to:

| Kernel | Gate (x86_64) | Path selected on 5800X |
|---|---|---|
| GF(2^128) scalar `Mul` (`flock-field/src/gf2_128.rs`, `gf2_128/x86_64.rs`) | `target_feature = "pclmulqdq"` | **128-bit PCLMULQDQ** `ghash_mul_binius` (the Binius64 CLMUL port; `#[target_feature(enable = "pclmulqdq,sse4.1")]`). This is the only non-portable kernel that engages. |
| 4-wide `ghash_mul_x4`, `f128x4_*`, unreduced accumulators (`gf2_128/x86_64.rs:215-400`) | `avx512f && vpclmulqdq` | **compiled out** — VPCLMULQDQ is only used through 512-bit `__m512i`; the 256-bit VPCLMULQDQ this CPU has is never used. |
| Slice fold `f128_slice::fold_pairs` | `avx512f && vpclmulqdq` else portable | **portable** |
| Additive NTT (`flock-core/src/ntt/additive_ntt_f128/kernels.rs`) | `avx512f && vpclmulqdq` else portable | **portable** |
| Zerocheck multilinear kernels (`zerocheck/multilinear/kernels.rs`) | `aarch64` or `avx512f && vpclmulqdq` only | **no x86 module at all → generic code** |
| Univariate-skip zerocheck round 1 (`zerocheck/univariate_skip_optimized/kernels.rs`) | `accumulate_convert_with_s_hat_v`: `avx512f && vpclmulqdq`; `bit_transpose_64bytes`: `avx512f+bw+vbmi`; `shift_reduce_inner_ab`: `gfni` (SSE or AVX-512 variants) | **all three portable/scalar** (no AVX-512, no GFNI) |
| AG-skip zerocheck (`prove_fast_union_ag`) | `cfg(target_arch = "aarch64")` | not available; benches print `zerocheck: rs` (the Rothblum-style round-1 arm). |
| Merkle hashing | `sha2` crate with `asm` feature | SHA-NI via the sha2 crate (the one place this CPU is on par with the published box). |

`hash_throughput.rs` would label this build "AVX2", but that label is only a `cfg!` print in the bench; **no kernel in Flock has an AVX2 variant** (`grep -rn avx2 crates/` hits only that print). So on Zen 3 Flock is effectively "scalar PCLMULQDQ field mul + portable everything else".

## Measured results (Flock, `LigeritoProfile::Fast`, default)

Trace shape: SHA-256 rows are 2^15 bits (K_LOG=15, 25,470 useful bits, 22.3 % padding), BLAKE3 rows are 2^14 bits (11,707 useful). Witness bits `m = K_LOG + log2(batch)`. All runs at 100 % slot fill. Proof = `R1csProofBundleLigerito` bincode bytes (commitment + proof). "peak heap" is the bench's own allocator high-water mark over one prove; "RSS" is GNU `time -v` max RSS of the whole process (all sizes in that sweep, so it reflects the largest size).

### SHA-256 compressions (64 B each)

| batch | m | threads | best prove (incl. witgen) | compressions/s | **input bytes/s** | verify | proof bytes | peak heap (prove) | load avg @ lock |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 2^12 = 4096 | 27 | 1 | 683.4 ms | 5,994 | 384 KB/s (375 KiB/s) | 27.0 ms | 374,456 (366 KiB) | 875 MB | 3.3 |
| 2^14 | 29 | 1 | 2.56 s | 6,388 | 409 KB/s (399 KiB/s) | 26.9 ms | 334,160 (326 KiB) | 1.72 GB | 3.3 |
| 2^16 | 31 | 1 | 9.79 s | 6,695 | 428 KB/s (418 KiB/s) | 26.8 ms | 461,032 (450 KiB) | 5.14 GB (RSS 7.3 GB) | 3.3 |
| 2^12 | 27 | 16 | 93.8 ms | 43,666 | 2.79 MB/s (2.67 MiB/s) | 9.1 ms | 374,456 | 904 MB | 4.2 |
| 2^14 | 29 | 16 | 326.1 ms | 50,250 | 3.22 MB/s (3.07 MiB/s) | 8.9 ms | 334,160 | 1.75 GB | 4.2 |
| 2^16 | 31 | 16 | 1.24 s | 52,822 | 3.38 MB/s (3.22 MiB/s) | 9.0 ms | 461,032 | 5.18 GB | 4.2 |
| 2^18 | 33 | 16 | 4.92 s | 53,304 | 3.41 MB/s (3.25 MiB/s) | 9.4 ms | 491,640 (480 KiB) | 18.9 GB (RSS 16.0 GB) | 4.2 |

(A first 16-thread SHA-256 sweep taken while other agents were compiling at load avg ≈ 40 gave 15.7k / 18.1k / 26.6k compressions/s — 2–3x worse. Discarded in favour of the low-load rerun above; kept in `runs/sha2_t16_l12-14-16.log` as evidence that these numbers are contention-sensitive.)

### BLAKE3 compressions (64 B each)

| batch | m | threads | best prove | compressions/s | **input bytes/s** | verify | proof bytes | peak heap | load avg |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 2^12 | 26 | 1 | 335.1 ms | 12,225 | 782 KB/s (764 KiB/s) | 20.5 ms | 345,096 (337 KiB) | 677 MB | 2.2 |
| 2^14 | 28 | 1 | 1.33 s | 12,294 | 787 KB/s (768 KiB/s) | 21.2 ms | 283,104 (276 KiB) | 1.09 GB | 2.2 |
| 2^16 | 30 | 1 | 4.56 s | 14,383 | 921 KB/s (899 KiB/s) | 21.8 ms | 416,368 (407 KiB) | 2.86 GB (RSS 4.0 GB) | 2.2 |
| 2^12 | 26 | 16 | 51.9 ms | 78,878 | 5.05 MB/s (4.81 MiB/s) | 8.2 ms | 345,096 | 690 MB | 1.4 |
| 2^14 | 28 | 16 | 173.4 ms | 94,480 | 6.05 MB/s (5.77 MiB/s) | 8.2 ms | 283,104 | 1.10 GB | 1.4 |
| 2^16 | 30 | 16 | 576.5 ms | 113,689 | 7.28 MB/s (6.94 MiB/s) | 9.6 ms | 416,368 | 2.87 GB | 1.4 |
| 2^18 | 32 | 16 | 2.10 s | 124,614 | 7.98 MB/s (7.61 MiB/s) | 8.8 ms | 460,552 (450 KiB) | 7.69 GB (RSS 9.9 GB) | 7.9 |

### Combined SHA-256 AND BLAKE3 over the same bytes (harmonic: t = 1/a + 1/b per 64 B)

| batch | 1 thread | 16 threads |
|---:|---:|---:|
| 2^12 | 4,023 pairs/s → 257 KB/s (**251 KiB/s**) | 28,106 pairs/s → 1.80 MB/s (**1.72 MiB/s**) |
| 2^14 | 4,203 → 269 KB/s (263 KiB/s) | 32,806 → 2.10 MB/s (2.00 MiB/s) |
| 2^16 | 4,568 → 292 KB/s (**285 KiB/s**) | 36,064 → 2.31 MB/s (**2.20 MiB/s**) |
| 2^18 | – | 37,335 → 2.39 MB/s (2.28 MiB/s) |

### Against the targets and baselines

- Project baseline (same box): SP1 v6.2.2 monolithic 1.6 KiB/s; SP1 decomposed 1 MiB / 2300 s ≈ 0.45 KiB/s; RISC Zero 1 MiB / 3591 s ≈ 0.29 KiB/s.
- **100x target (160 KiB/s)**: met even single-threaded, for both hashes combined (251–285 KiB/s, 1.6–1.8x over target).
- **1000x target (1.6 MiB/s)**: met at 16 threads for both hashes combined at every batch size ≥ 2^12 (1.72–2.28 MiB/s; 1.1–1.4x over). SHA-256 alone: 3.2 MiB/s (2x over); BLAKE3 alone: 6.9–7.6 MiB/s (4.3–4.8x over).
- vs SP1 monolithic: combined 16T ≈ **1,400x**; SHA-256-only ≈ 2,100x; BLAKE3-only ≈ 4,500x.
- For a 1 MiB input (2^14 compressions of each hash) the *proving* time would be ≈ 0.33 s (SHA) + 0.17 s (BLAKE3) ≈ 0.5 s at 16 threads, vs 2300 s (SP1 decomposed) / 3591 s (RISC Zero) — but see the caveats: this is a batch of *independent* compressions, not a chain with the wiring proven.

### The AVX2 penalty (vs published numbers)

| System / box | SHA-256 1T | SHA-256 MT | BLAKE3 1T | BLAKE3 MT |
|---|---:|---:|---:|---:|
| Flock README, Threadripper 7970X (Zen 4, AVX-512+VPCLMULQDQ), 2026-07-17, 32 cores | 32.0k (2^16) | 271k (2^16), 305k (2^18) | 64.8k | 540k (2^16), 630k (2^18) |
| **Flock, this box** (Zen 3, AVX2 only, 8c/16t) | **6.7k** | **52.8k (2^16), 53.3k (2^18)** | **14.4k** | **114k (2^16), 125k (2^18)** |
| ratio TR/5800X | 4.8x per thread | 5.1–5.7x (4x more cores) | 4.5x per thread | 4.7–5.0x |
| BinarySpartan paper, M4 Max 12 P-cores | – | 401k | – | 872k |
| ratio M4/5800X (MT) | – | 7.6x | – | 7.0x |

So per thread Zen 3 with the portable kernels is ~4.5–5x slower than Zen 4 with the AVX-512 kernels (some of which is IPC/clock, most is the missing 4-wide CLMUL/NTT/zerocheck kernels), and the Flock README warns the Zen 4 figures predate an encoder shrink, so the true gap is somewhat larger. The 16-thread number here is 7.9x the 1-thread number for both hashes at 2^16 (8 physical cores, SMT contributes little), so scaling is essentially perfect to the core count.

### Fixed overhead vs per-row work

Linear fit of best prove time, 16 threads: SHA-256 ≈ 16 ms + 18.8 µs/compression (fixed part = 17 % at 2^12, 5 % at 2^14, 1 % at 2^16); BLAKE3 ≈ 18 ms + 8.2 µs/compression (35 % at 2^12, 10 % at 2^14, 3 % at 2^16). Per-phase breakdown (`PCS_TRACE=1`, one traced prove per size, 16T, SHA-256 / BLAKE3 at 2^16): commit 210 / 85 ms, boolean zerocheck+lincheck 578 / 255 ms, Ligerito open 167 / 121 ms, C-claim fold 15 / 5 ms; the traced `prove_union` total is 1.05 s / 0.46 s vs 1.24 s / 0.58 s best-of-3 end to end, the difference (~190 / 115 ms) being witness generation, which runs before the traced region. At 1 thread the zerocheck+lincheck phase dominates (71 %). Verify is ~9 ms at 16T and ~21–27 ms at 1T (the verifier is also rayon-parallel) and is flat across sizes.

### Memory scaling and the largest single proof on 64 GiB

Peak heap grows linearly with batch: SHA-256 ≈ 70 KB per compression above a ~0.6 GB base (≈ 18x the 4 KiB witness row, i.e. ≈ 1.1 KB of RAM per byte of hashed input); BLAKE3 ≈ 24.5 KB per compression (≈ 15x the 2 KiB row). Extrapolation: SHA-256 2^19 (32 MiB input) ≈ 37 GB, 2^20 (64 MiB) ≈ 73 GB → **max single SHA-256 proof ≈ 2^19 compressions = 32 MiB of input** (and only if the ZFS ARC yields); BLAKE3 2^20 ≈ 26 GB, 2^21 (128 MiB) ≈ 52 GB → ≈ 2^21 compressions. Beyond that, split into several proofs (each ~450–480 KiB) and aggregate (Flock has a BLAKE3-recursion tower for that, aarch64-tuned and not measured here). Measured RSS was 15–25 % below the bench's own heap high-water count.

## Spartan2 / BinarySpartan — why it was not benchmarked

- `microsoft/Spartan2` HEAD (`c0ee259`) is now **"vega-prover"**: a prime-field / elliptic-curve system (`T256HyraxEngine`, Pallas/Vesta/P-256/BN254 engines, Hyrax PCS), optimized for **client-side ZK latency** on 1–2 KiB signed credentials. Its only benches are `sha256_vega_sc` / `sha256_vega_mc_zkp` (SHA-256 of a 1 KiB / 2 KiB preimage, 17–33 sequential compressions). There is no binary-field, GF(2^128), Ligerito, or BLAKE3 code in `src/` or on any of its 27 branches (checked `neutron`, `benchmark`). Flock's own competitor harness (`benchmarks/spartan2/setup.sh`) pins this same curve-based code at `0d4f1409`. It is not the system in the ePrint.
- **BinarySpartan (ePrint 2026/1656, Setty) has no public code**: the ePrint page carries no repository link, `GET api.github.com/search/repositories?q=BinarySpartan` returns 0 results, and `microsoft/{BinarySpartan,binary-spartan,binaryspartan,Spartan3}` all 404. The paper describes it as "a single Rust crate" over F_2^128 mod x^128+x^7+x^2+x+1 with Ligerito recursing to 8 variables, measured on an M4 Max (NEON); nothing is said about x86 SIMD paths.
- Given both facts, running the curve-based Spartan2 benches would have produced a number for a different (slower, ZK, latency-oriented) system and been misleading; I stopped after inspection.

Sources: [ePrint 2026/1656](https://eprint.iacr.org/2026/1656), [github.com/microsoft/Spartan2](https://github.com/microsoft/Spartan2), [github.com/succinctlabs/flock](https://github.com/succinctlabs/flock).

## Step 4: can Flock express "SHA-256 chain over N blocks with public (state_in, state_out) AND BLAKE3 chunk CVs over the same message bits"?

What exists (read-only survey of `crates/flock-core/src/circuit*.rs`, `crates/flock-prover/src/tower/chain.rs`, `gates_blake3.rs`, `r1cs_hashes/{sha2,blake3,merkle_glue,merkle_r1cs}.rs`, `mixed.rs`):

1. **Union / multi-table model.** A proof commits a "union" of table types (rows of the SHA-256 table, rows of the BLAKE3 table, …). Per-row relations are R1CS over GF(2). The union model explicitly *forbids constraints between rows*; everything cross-row goes through the **circuit/wiring layer** (`flock_core::circuit`, `docs/circuit-wiring-design.tex`): each table type exposes an `IoWord` schema (128-bit-aligned input/output words of a row), wires are equivalence classes of such cells, and equality is proven with a permutation (grand-product GKR, `product_gkr`) plus `public_input()` / `publish()` words that the verifier sees. `mixed.rs` already ships SHA-256+BLAKE3 two-type registries (`blake3+sha2@nu7`, `@nu10`, i.e. 128 / 1024 rows per type), so **two hash tables in one commitment is supported today** — but only at those two toy capacities; a 2^14–2^16 tier is a config/wire-format addition (`Registry::new(types, nu)` is generic).
2. **BLAKE3 hash-chain IO circuit exists** (`tower/chain.rs::build_chain_shape`): `Blake3Gate` (table with `io_schema()` = cv[2 words], m[4 words], params, out_lo[2], out_hi[2]); the chain wires `out → m` of the next row with a fixed IV cv and `params` as a public word, publishing the final output. It proves H^T(x) with 64-byte state, i.e. exactly the "sequential chain with public start/end" shape, and the pattern generalizes to chunk hashing (`out_lo → next cv`, message words as public/witness inputs, `params` per position carrying counter/`CHUNK_START|CHUNK_END` flags — the params word is *in the schema precisely so a circuit can pin it*, per the doc comment). `merkle_r1cs.rs` also documents the 16-block chunk-leaf chaining. So **BLAKE3 chunk CVs over a public/witness message: expressible with existing gates**, needing only a new `ShapeBuilder` program (~50 lines like `build_chain_shape`) and a bigger registry tier. Parent-node compressions (2 CVs → 1) are just more BLAKE3 rows with `cv=IV, m = left‖right, PARENT flag`.
3. **SHA-256 has no IO schema and no gate.** `grep IoWord sha2.rs` = 0 hits; no `Sha2Gate` in `tower/`; nothing in the circuit layer references SHA-256. `Sha256HybridSetup` only proves *independent* `(h_in, m)` pairs, which is what was benchmarked. To chain SHA-256 you need: (a) an `io_schema()` for the SHA-256 table — and the row layout is already wireable: `sha2.rs:118-132` puts `H` at bits [0,256), `H_OUT` at [256,512) and `M` at [512,1024) ("a clean 4-slot region of 1024 bits at the start of each block"), the same 128-bit-aligned shape as BLAKE3's `CV_BASE=0 / OUT_LO_BASE=256 / M_BASE=512`, so the schema is 8 `IoWord`s (2 in, 4 in, 2 out) with no relayout; (b) a `Sha2Gate: GateType` (mirror of `gates_blake3.rs`, ~60 lines); (c) a chain shape wiring `h_out → h_in` with `state_in` as `public_input()` and `state_out` `publish()`ed. Effort: hours to a day for someone who knows the crate; the risk is only in getting the `GateType::eval` row-witness plumbing right. It runs on the x86 "RS" zerocheck path, which has no AVX-512/NEON kernel here anyway.
4. **Sharing message bits between the two tables.** Both hashes take a 64-byte block as 16×u32 = 4×128-bit words, and both rows pack message bit `b` of word `i` at `M_BASE + 32*i + b` (`sha2::m_bit`, `blake3::m_bit`, identical), so the four message wires of block *i* can simply be connected to both gates (`sb.gate(sha, [h_prev.., m_i..])` and `sb.gate(blake, [cv.., m_i.., params])` with the same `Wire`s) — the wiring argument is per-cell equality and does not care which table a cell belongs to. No bit-permutation glue table (like `merkle_glue::BitSpreadTable`) is needed for the message words. (One caveat: the two tables interpret the u32 words with their own endianness conventions inside the relation — SHA-256 big-endian, BLAKE3 little-endian — so "the same 64 bytes" means the wire carries the *bytes* in one fixed order and whichever table disagrees must have its witness writer/relation consume them byte-swapped; that is a fixed permutation of bit positions within each u32 and, if not already handled by the encoders, would need one small glue table.) Cost of the wiring argument itself scales with schema words × rows (≈ 11 words/row), a small fraction of the hash rows, but **it is not measured here** — the chain benches live in `#[test]` harnesses and are aarch64-tuned.
5. Bottom line: **BLAKE3 side: exists (chain + chunk semantics); SHA-256 side: table exists, wiring does not; sharing message bits across the two tables: supported by the circuit model with no new cryptography**, one new registry tier and one shape program. Reported throughput should be discounted for (i) the SHA-256 K_LOG=15 row, which would not change, (ii) the wiring/GKR overhead (unmeasured, expected ≤ 10–20 %), and (iii) the fact that at 2^16 SHA + 2^16 BLAKE3 rows in one union the dense witness is 384 MiB and peak RAM ≈ 8 GB (fine).

## Honest assessment

- **Worked:** Flock builds cleanly on stable Rust on an AVX2-only Zen 3 box in 3.5 min; every size proved and verified; numbers are the prover's own timers cross-checked with GNU time wall clock. Per-thread throughput is ~4.5–5x below the published Zen 4/AVX-512 numbers, entirely as expected from the compile-time kernel selection (only the scalar PCLMULQDQ field multiply is accelerated; VPCLMULQDQ is present on the CPU but only used through 512-bit registers). A 256-bit VPCLMULQDQ/AVX2 port of `ghash_mul_x4`, the NTT, and the zerocheck kernels is the obvious way to claw back maybe 2x on this class of CPU; nobody has written it.
- **Did not / could not measure:** BinarySpartan (no public code); the SHA-256→SHA-256 *chain* and the SHA+BLAKE3 shared-message circuit (the SHA-256 side has no wiring support; the BLAKE3 chain benches are test-only and NEON-tuned); recursion/aggregation; anything on the RX 7900 XT (Flock's GPU backend is CUDA-only).
- **What was run is the system's headline benchmark, not a toy**: the same `prove_fast` path, Fiat-Shamir transcript, proof serialization and verifier as the README table. But it proves *independent* random compressions with no public IO relation between them; the application statement needs the circuit path from Step 4.
- **Security / soundness caveats visible in code:** (i) `LigeritoProfile::Fast` (default, what was measured) is rate 1/2 in the **Johnson list-decoding regime** with 2-point OOD binding, an aggressive +2/level rate ladder, and **16-bit query proof-of-work per level; the query term targets 112 bits and PoW "supplies the rest, work-normalized to 128"** — i.e. 128 bits of *work*, not 128 bits of pure query soundness, and `profile.security_bits()` still reports the historical **100**. The conservative `Secure` profile (unique decoding, 120 bits, no OOD, no PoW) exists but was not timed; expect larger proofs and slower opening. (ii) Field is F_2^128 (GHASH polynomial); Merkle and transcript hash are SHA-256 by default. (iii) **No zero-knowledge** anywhere (the witness is committed directly; the word "zero-knowledge" does not appear in the code or docs). (iv) Research prototype: unaudited, `docs/128-bit-grinding-audit.md` and `docs/bloat-ledger.md` show the parameters were retuned as recently as 2026-08-27 with a wire-format bump; `unsafe` uninitialized-buffer allocations in hot paths (`alloc_uninit_vec`). (v) The verifier rebuilds the registry from the same code as the prover (the registry digest is bound in the transcript, so a mismatched verifier fails, but there is no separate verifier-key artifact for the hash benches). (vi) Nothing insecure was enabled in these runs: the dev-only `unsound-challenger` feature and `FLOCK_NO_GRIND` were not used.
- **Contention warning:** 16-thread results moved 2.8x depending on other agents' compile load; only the low-load reruns are tabulated. Single-thread results are much less sensitive.
