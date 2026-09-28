# exp-plonky3: Plonky3 hash-AIR prover throughput on the Ryzen 7 5800X

Goal: measure how fast Plonky3's purpose-built hash AIRs prove on this box, to estimate what a
hand-written STARK for "SHA-256(F) and BLAKE3(F)" could reach versus the zkVM baselines
(SP1 monolithic ~1.6 KiB/s; SP1 decomposed 1 MiB/2300 s ≈ 0.45 KiB/s; RISC Zero 1 MiB/3591 s ≈ 0.29 KiB/s).

## TL;DR

* Plonky3 `main` already contains a **SHA-256 compression AIR** (`sha256-air/`, one row per compression,
  7728 columns) in addition to `blake3-air` (one row per compression, 9168 columns) and `keccak-air`
  (24 rows per permutation, 2633 columns). No extrapolation for SHA-256 was needed: I measured it directly.
  There are also **binary-field (characteristic-2, Boolean-trace) variants** of all of them,
  driven by `examples/examples/prove_hash_binary.rs`.
* Prime-field uni-STARK (KoalaBear, FRI, 100-bit *conjectured* security, 16 threads), 2^18 compressions = 16 MiB:
  * SHA-256: prove **36.1 s** → **454 KiB/s** (517 KiB/s at 2^16).
  * BLAKE3: prove **42.5 s** → **386 KiB/s** (434 KiB/s at 2^16).
  * Both hashes over the same bytes (sum of the two traces, 16 896 cells/64 B): **≈ 78.6 s per 16 MiB ≈ 210 KiB/s**
    (≈ 190 KiB/s if trace generation is included). That is ~130x SP1-monolithic, ~470x SP1-decomposed.
* Binary-field STARK (GF(2^128) Boolean trace, Plonky3 `binary-pcs` folding PCS, 16 threads), 2^18 compressions = 16 MiB:
  * SHA-256: prove **8.44 s** → **1.94 MiB/s** (1.60 MiB/s incl. witness generation).
  * BLAKE3: prove **3.52 s** → **4.65 MiB/s**.
  * Both hashes together: **≈ 12.0 s per 16 MiB ≈ 1.34 MiB/s** (≈ 1.18 MiB/s incl. witness) — within ~20 % of the
    "1000x" target (1.6 MiB/s), ~850x SP1-monolithic. Caveat: composed security at 2^18 is 98.5 bits, not 100
    (field-size cap; see below), and the binary-field stack is the newest part of Plonky3.
* Verification is 20–60 ms in every configuration. Proof sizes: ~3.0–3.7 MB (prime-field FRI, 87 queries) and
  ~0.7–1.3 MB (binary folding PCS).
* Per-cell cost model for the prime-field prover: **~56–64 M trace cells/s** (KoalaBear, blowup 2, Poseidon2 Merkle),
  ~90 % of prove time is "commit to trace data" (LDE + Merkle). Fixed overhead is < 0.3 s; scaling is linear to
  slightly super-linear (2^16 → 2^18: 4.56x time for 4x rows).

## What was built

* Repo: https://github.com/Plonky3/Plonky3 (`main`), shallow clone.
  Commit **72a69337dd89aac73d2eb52cc6df50024b735b7c** (2026-09-28), workspace version 0.8.0.
* Toolchain: Rust stable 1.93.1 (`nix develop /home/schlarpc/waddup`), `RUSTFLAGS="-C target-cpu=native"`,
  `cargo build --release -p p3-examples --features parallel --example prove_prime_field_31 --example prove_hash_binary`.
  `CARGO_TARGET_DIR=<scratch>/exp-plonky3/target`. Rayon parallelism verified active (17 threads, 300–1100 % CPU).
  jemalloc is used by the examples crate on Linux.
* Local patch (only in my scratch clone, `Plonky3/examples/`): added a `sha-256-compressions` objective wired to
  `p3_sha256_air::Sha256Air` in `examples/src/airs.rs` (`ProofObjective::Sha256`), `examples/src/parsers.rs`, and
  `examples/examples/prove_prime_field_31.rs`, plus a println of the column counts and of the derived FRI parameters
  (`examples/src/proofs.rs`). No changes to any AIR or prover code.

### Trace shapes (from the code, confirmed at runtime)

| AIR | rows per hash | columns (`NUM_*_COLS`) | cells per 64 B input | notes |
|---|---|---|---|---|
| `Blake3Air` (prime field) | 1 | **9168** | 9168 | full 7-round compression in one row; 16-bit limbs + bit columns |
| `Sha256Air` (prime field) | 1 | **7728** | 7728 | message schedule + 64 rounds in one row; a/e chains, 2×16-bit limbs for adds, bits for XOR/rot; no transition constraints |
| `KeccakAir` (prime field) | 24 | **2633** | 63 192 per perm (136 B rate) → 29 737 per 64 B | round-per-row |
| `Blake3BinaryAir` (GF(2)) | 1 | 11 536 bit-columns | 11 536 bits | Boolean trace; XOR is free, AND is a mul |
| `Sha256BinaryAir` (GF(2)) | 1 | 23 712 bit-columns | 23 712 bits | Boolean trace |

Constraint degree of `Sha256Air` ≤ 3 (documented in `sha256-air/src/lib.rs`). Poseidon2 AIRs exist too but are irrelevant here.

### Security parameters actually used

`prove_prime_field_31` (via `examples/src/proofs.rs::example_fri_parameters`) starts from
`FriParameters::new_benchmark_high_arity` and raises `num_queries` until `ConjecturedSecurity::compute_ldt_only` ≥ 100 bits.
Printed at runtime:

```
FRI params: log_blowup=1 num_queries=87 query_pow_bits=16 batch_pow_bits=10 commit_pow_bits=0 max_log_arity=3 log_final_poly_len=0
```

Every prime-field run printed: `Proven security: 59 bits (UDR: 52, LDR: 59)`, `Conjectured security: 100 bits`
(`Legacy security 103 bits`). So: **100 bits under the standard list-decoding/"up-to-capacity" FRI conjecture, only 59 bits proven**,
with rate 1/2, 87 queries, 16-bit query grinding, 10-bit batching grinding. Challenge field is the degree-4 extension
(~124-bit) of KoalaBear/BabyBear. These are *not* toy parameters (the `new_testing` config would be 2 queries / 1 PoW bit),
but a deployment wanting proven (not conjectured) 100 bits would need ~2x more queries or a lower rate, roughly 1.3–2x more prover time.

`prove_hash_binary` targets `--security-bits 100` (composed, i.e. end-to-end) by default: it printed
`Composed security: 100.6–101.3 bits` for 2^12–2^16. At 2^18 it refuses ("security level 100 exceeds the 98 bits the field leaves
at this domain size") because the trace is committed as packed bits in GF(2^128); I re-ran 2^18 with `--security-bits 98`
(reported 98.54 / 98.83 bits). Parameters: `log_inv_rate=1`, folding=4, Merkle arity 4, Keccak-256 Merkle/transcript, PCS "Folding".

## Measurements

All timed runs were serialized with `flock <scratchpad>/bench.lock`. Peak RSS via GNU `time` (`%M`). "prove"/"verify" are the
program's own tracing-span times (they exclude trace generation, which is listed separately). The 1-minute load average was
sampled just before each run; **several other agents were building concurrently, so runs with load1 > 16 are pessimistic**
(CPU oversubscribed on a 16-thread box). I re-ran the key KoalaBear points once load dropped (`runs2/`).
Throughput = hashes × 64 B / prove-time (Keccak: × 136 B).

Command form: `prove_prime_field_31 -f <field> -o <b3|sha|kf> -l <log2 rows> -d radix-2-dit-parallel -m <poseidon2|keccak>` and
`prove_hash_binary -o <sha-256-compressions|blake-3-compressions> -l <log2 rows> [--security-bits 98]`.

### A. Prime-field uni-STARK, KoalaBear, Poseidon2 Merkle tree (best/low-load runs)

| objective | log2 rows | hashes | input | load1 | trace gen s | **prove s** | of which commit s | verify s | proof bytes | peak RSS | wall s | user s | **KiB/s** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| BLAKE3 | 12 | 4 096 | 256 KiB | 10.6 | 0.02 | **0.44** | 0.33 | 0.051 | 3 568 377 | 352 MiB | 0.52 | 4.2 | 580 |
| BLAKE3 | 14 | 16 384 | 1 MiB | 17.3 | 0.79 | **2.25** | 1.81 | 0.071 | 3 593 544 | 1 215 MiB | 3.15 | 15.7 | 455 |
| BLAKE3 | 16 | 65 536 | 4 MiB | 10.8 | 1.18 | **9.44** | 8.40 | 0.055 | 3 627 862 | 4 684 MiB | 10.7 | 105.7 | 434 |
| BLAKE3 | 18 | 262 144 | 16 MiB | 15.2 | 4.65 | **42.5** | 38.4 | 0.059 | 3 662 871 | 18 548 MiB | 47.2 | 464 | 386 |
| SHA-256 | 12 | 4 096 | 256 KiB | 10.7 | 0.01 | **0.28** | 0.19 | 0.049 | 3 023 417 | 313 MiB | 0.37 | 3.4 | 928 |
| SHA-256 | 14 | 16 384 | 1 MiB | 43.1 | 0.50 | **2.47** | 1.83 | 0.086 | 3 047 816 | 1 035 MiB | 3.19 | 9.2 | 415 (heavy load) |
| SHA-256 | 16 | 65 536 | 4 MiB | 11.8 | 0.78 | **7.92** | 6.92 | 0.054 | 3 079 510 | 3 956 MiB | 8.79 | 87.9 | 517 |
| SHA-256 | 18 | 262 144 | 16 MiB | 12.1 | 3.64 | **36.1** | 32.1 | 0.055 | 3 115 479 | 15 668 MiB | 39.8 | 396 | 454 |
| Keccak-f | 14 | 682 perms | 90.6 KiB | 33.3 | 0.21 | **0.66** | 0.45 | 0.025 | 1 112 452 | 375 MiB | 0.90 | 3.2 | 138 |
| Keccak-f | 16 | 2 730 perms | 362 KiB | 7.5 | 0.44 | **2.04** | 1.68 | 0.019 | 1 140 114 | 1 389 MiB | 2.51 | 21.8 | 178 |
| Keccak-f | 18 | 10 922 perms | 1.45 MiB | 18.5 | 1.01 | **10.7** | 9.26 | 0.019 | 1 179 827 | 5 439 MiB | 11.8 | 121 | 136 |

Heavily loaded duplicates (load1 27–50) of BLAKE3 2^12/2^16 and SHA-256 2^12 came out 2–10x slower (e.g. BLAKE3 2^16: 20.4 s at load 50
vs 9.44 s at load 11); they are in `runs/` but not used above.

### B. Prime-field variants: Keccak Merkle tree and BabyBear

| field | Merkle hash | objective | log2 rows | load1 | prove s | verify s | proof bytes | peak RSS | KiB/s |
|---|---|---|---|---|---|---|---|---|---|
| KoalaBear | Keccak | BLAKE3 | 12 | 10.7 | 0.27 | 0.015 | 3 579 362 | 349 MiB | 941 |
| KoalaBear | Keccak | BLAKE3 | 14 | 2.8 | 1.72 | 0.015 | 3 604 047 | 1 214 MiB | 595 |
| KoalaBear | Keccak | BLAKE3 | 16 | 12.5 | 8.65 | 0.016 | 3 641 806 | 4 691 MiB | 474 |
| KoalaBear | Keccak | SHA-256 | 14 | 2.3 | 1.45 | 0.018 | 3 058 594 | 1 043 MiB | 706 |
| BabyBear | Poseidon2 | BLAKE3 | 12 | 4.2 | 0.42 | 0.057 | 3 569 817 | 349 MiB | 614 |
| BabyBear | Poseidon2 | BLAKE3 | 14 | 6.8 | 2.12 | 0.057 | 3 594 760 | 1 217 MiB | 483 |
| BabyBear | Poseidon2 | BLAKE3 | 16 | 8.0 | 9.90 | 0.058 | 3 627 446 | 4 692 MiB | 414 |
| BabyBear | Poseidon2 | SHA-256 | 14 | 10.6 | 1.71 | 0.053 | 3 044 584 | 1 043 MiB | 599 |

Keccak-based Merkle trees are ~10 % faster than Poseidon2 (both are SIMD-vectorized), and verification is 3–4x faster
(15–18 ms vs 50–70 ms). BabyBear ≈ KoalaBear (±5 %). Merkle hash choice matters little because the LDE and hashing of
~7.7–9.2 k columns dominate either way.

### C. Binary-field STARK (`prove_hash_binary`, GF(2^128), Boolean trace, folding PCS, Keccak-256 Merkle, 16 threads)

| objective | log2 rows | hashes | input | load1 | width (bits) | witness s | **prove s** | verify s | proof bytes | peak RSS | wall s | composed security | **KiB/s** (prove) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| SHA-256 | 12 | 4 096 | 256 KiB | 42.5 | 23 712 | 0.09 | 0.60 | 0.068 | 985 177 | 299 MiB | 1.21 | 101.2 | 427 (heavy load) |
| SHA-256 | 14 | 16 384 | 1 MiB | 41.0 | 23 712 | 0.27 | 1.92 | 0.059 | 1 065 999 | 657 MiB | 2.62 | 101.1 | 533 (heavy load) |
| SHA-256 | 16 | 65 536 | 4 MiB | 4.1 | 23 712 | 0.38 | **2.21** | 0.037 | 1 197 457 | 2 127 MiB | 2.83 | 100.6 | **1 849** |
| SHA-256 | 18 | 262 144 | 16 MiB | 15.2 | 23 712 | 1.54 | **8.44** | 0.044 | 1 291 459 | 7 078 MiB | 10.3 | 98.5 (`--security-bits 98`) | **1 942** |
| BLAKE3 | 12 | 4 096 | 256 KiB | 33.3 | 11 536 | 0.05 | 0.31 | 0.037 | 690 021 | 158 MiB | 0.58 | 101.2 | 834 |
| BLAKE3 | 14 | 16 384 | 1 MiB | 7.5 | 11 536 | 0.01 | 0.30 | 0.022 | 798 902 | 397 MiB | 0.40 | 101.2 | 3 436 |
| BLAKE3 | 16 | 65 536 | 4 MiB | 2.3 | 11 536 | 0.01 | **0.92** | 0.021 | 894 152 | 1 100 MiB | 1.06 | 100.9 | **4 443** |
| BLAKE3 | 18 | 262 144 | 16 MiB | 12.5 | 11 536 | 0.05 | **3.52** | 0.024 | 1 011 447 | 3 484 MiB | 3.74 | 98.8 (`--security-bits 98`) | **4 650** |

The binary prover is 4–5x faster than the prime-field one for the same hash because every trace cell is one bit
(committed packed, 128 bits per field element) instead of a 31-bit field element, and XOR/rotation are free.
Its "Prove time" covers commit + zerocheck (sumcheck) + PCS opening; witness generation is separate and listed.

## Derived numbers

### Compressions/s and bytes/s (16 threads, this CPU)

| stack | hash | compressions/s (2^18) | input bytes/s | vs SP1 monolithic (1.6 KiB/s) | vs SP1 decomposed (0.45 KiB/s) |
|---|---|---|---|---|---|
| prime-field FRI STARK | SHA-256 | 7 260 | 454 KiB/s | 280x | 1 000x |
| prime-field FRI STARK | BLAKE3 | 6 170 | 386 KiB/s | 240x | 860x |
| prime-field FRI STARK | **SHA-256 + BLAKE3** (same bytes) | 3 340 | **≈ 209 KiB/s** (≈ 190 KiB/s incl. trace gen) | **130x** | **460x** |
| binary-field STARK | SHA-256 | 31 060 | 1.94 MiB/s | 1 240x | 4 300x |
| binary-field STARK | BLAKE3 | 74 500 | 4.65 MiB/s | 2 980x | 10 300x |
| binary-field STARK | **SHA-256 + BLAKE3** | 21 900 | **≈ 1.34 MiB/s** (≈ 1.18 MiB/s incl. witness) | **≈ 860x** | **≈ 3 000x** |

"SHA-256 + BLAKE3" is computed as the sum of the two separately measured prove times (equivalently a combined trace of
7728 + 9168 = 16 896 cells per 64 B at the measured ~56 M cells/s). A single multi-table proof (`p3-multi-stark`/`batch-stark`)
would share the FRI/opening phase and be marginally faster, not slower.

### Fixed overhead vs per-row work (prime field, KoalaBear/Poseidon2)

SHA-256: 0.28 s @ 2^12, 7.92 s @ 2^16, 36.1 s @ 2^18 → per-row ≈ 140–145 µs, intercept ≈ 0 (≤ 0.3 s). The prover is
essentially linear in rows and slightly super-linear at 2^18 (memory bandwidth: 15.7 GiB RSS, LDE of 2×2^18×7728 elements).
Breakdown at 2^18: commit-to-trace (LDE + Merkle) 89 %, constraint/quotient evaluation ≈ 8 % (span self-time), open/FRI 3 %.
So the cost is dominated by *how many field cells you commit*, not by constraint complexity — which is why the Keccak AIR
(2633 cols × 24 rows = 63 k cells per 136 B) is only 136 KiB/s despite cheaper constraints per cell.

Cells/s: SHA-256 2^18 = 2.03e9 cells / 36.1 s = **56 M cells/s**; BLAKE3 2^18 = 2.40e9 / 42.5 s = **56.5 M cells/s**;
Keccak 2^18 = 6.9e8 / 10.7 s = 64 M cells/s. Use ~56 M cells/s for degree-3 AIRs of this shape.

### Extrapolation for a hypothetical hand-written SHA-256 AIR (Step 4, retained for the record)

Not needed as an extrapolation since `sha256-air` exists, but for cross-checking the trace-size reasoning:

* Plonky3's `Sha256Air` spends 7728 cells per compression (64 rounds + 48 schedule extensions in one row; ~120 cells per round).
  BLAKE3's 7 rounds cost 9168 cells (~1 300 cells/round) because each BLAKE3 round is 8 quarter-rounds over 16 words with
  intermediate states committed, whereas SHA-256 only commits the new `a` and `e` per round plus 6 packed helpers. So
  "64 rounds vs 7 rounds" does *not* translate into a 9x larger SHA-256 trace; SHA-256 is ~16 % smaller than BLAKE3 here.
* SP1 (dev branch, `crates/core/machine/src/syscall/precompiles/sha256/{extend,compress}/columns.rs`, fetched 2026-09-28):
  the compress chip has ~55 struct fields (9 working/constant words as 2×16-bit limbs, 6 fixed-rotate ops, 8 XOR ops, 5 AND ops,
  1 NOT, 5 adds, 3 address-add ops, memory access, 3 phase flags, octet one-hots) over 80 rows/compression
  (8 init + 64 round + 8 finalize); the extend chip ~30 fields over 48 rows. Assuming ~150–200 columns per compress row and
  ~100–150 per extend row (I could not compute exact `size_of` values without building SP1), that is ≈ 17–23 k cells per
  compression, i.e. **2.2–3x the Plonky3 single-row AIR**, before SP1's interaction/lookup-bus and memory columns. Applying the
  measured 56 M cells/s to ~20 k cells gives ≈ 2 800 compressions/s ≈ 175 KiB/s for an SP1-shaped SHA-256 chip proven
  standalone in Plonky3 — consistent with the observation that SP1's remaining ~100x gap to these numbers is the zkVM
  machinery (CPU/memory/bus tables, per-shard recursion), not the precompile chip itself.
* Expected combined SHA-256 + BLAKE3 statement on this CPU with a hand-written prime-field STARK: ≈ 200 KiB/s (measured
  sum), ≈ 1.2–1.3 MiB/s with the binary-field stack. Assumptions: 16 threads, rate 1/2, 100-bit conjectured security, 2^18-row
  shards, the same "prove N independent compressions" statement as the examples.

## Honest assessment / caveats

**What worked.** Clean shallow clone, one 2.5-minute release build, all example runs succeeded at 2^12–2^18 rows for
prime-field and 2^12–2^18 for binary-field. Peak RSS at 2^18 was 15.7 GiB (SHA-256) / 18.5 GiB (BLAKE3) for the
prime-field prover and 7 / 3.5 GiB for the binary one; 2^20 prime-field rows would need ~64–75 GiB and was not attempted.

**Code grade.** Plonky3 is the production STARK library behind SP1, Valida, OpenVM etc.; the repo carries a Least Authority audit
report (`audits/`). The hash AIRs (`blake3-air`, `keccak-air`, `sha256-air`) are library crates with tests against the `sha2`/`blake3`
reference implementations and `check_constraints` tests, not scratch code — but they are *compression-function* AIRs meant to be
embedded in a larger system. The binary-field stack (`binary-field`, `binary-pcs`, `sumcheck`, `whir`, `*BinaryAir`) is the newest
part of the codebase (2026) and has no audit reference that I found; treat its numbers as "credible research-grade".

**What the measured statement is, and is not.** Each run proves "here are 2^k independent SHA-256 (or BLAKE3) compression
function evaluations on *random witness inputs*". It does **not**:
1. chain rows (SHA-256 `H_out[i] → H_in[i+1]`, or the BLAKE3 tree/chunk structure). `Sha256Air` explicitly has no transition
   constraints; adding the chaining is a handful of degree-1/2 equalities on already-committed bits/limbs (`h_out` is committed as
   bits, `h_in` as 16-bit limbs) and would add < 1 % cells. BLAKE3's tree needs a lookup/permutation argument between rows
   (parent chaining values = child outputs) — cheap relative to 9168 cells/row but not free, and not implemented in the AIR.
2. bind the message bytes or the final digest to public values. Committing the 16 message words is already part of the trace
   (`w[0..16]` / `inputs`); exposing the final hash needs public-value constraints. Also the *same* bytes feeding both hashes
   would need cross-table equality (a lookup/bus) in a combined proof.
3. handle padding/length encoding (SHA-256 final block, BLAKE3 flags) — trivially small.
   So the numbers are an upper bound on throughput for the real "SHA-256(F) and BLAKE3(F)" statement, with the expected
   overhead of the missing pieces in the few-percent range, not a multiple.

**Soundness/parameter caveats visible in the code.**
* Prime-field: 100-bit security is *conjectured* (list-decoding regime), *proven* is 59 bits (UDR 52) — printed by the example
  itself. Rate 1/2, 87 queries, 16-bit query PoW, 10-bit batch PoW. The example says the challenge field is 4th-degree
  extension; `StarkSecurityParams` computed with 128-bit hash collision assumption. Not toy parameters, but not "proven 100-bit".
* Prime-field Poseidon2 Merkle-tree round constants are drawn from `SmallRng::seed_from_u64(1)` with a
  `// WARNING: Use a real cryptographic PRNG in applications!!` comment — irrelevant to timing.
* Binary-field: the `*BinaryAir::assuming_boolean_trace()` constructor drops the booleanity constraints and is sound only
  under a commitment whose alphabet is a single bit (which the Boolean PCS in `prove_boolean_air` provides, per its docs).
  At 2^18 rows the packed GF(2^128) commitment caps composed security at 98 bits (the tool refuses 100); I ran those two points
  at `--security-bits 98`. At 2^20 it would be ~96 bits; larger inputs should be sharded at 2^16–2^18 rows anyway (RSS).
* Proof sizes are 3.0–3.7 MB (prime field) / 0.7–1.3 MB (binary) per 2^18-row shard; a real system would wrap these in a
  recursive/aggregation proof, which is extra prover time not measured here.
* Timing noise: the box was shared with other agents' builds. I recorded load1 next to every run; the headline numbers all come
  from runs with load1 ≤ 16 (partially oversubscribed at worst), and the duplicated points show that the numbers reported
  are, if anything, pessimistic by up to ~10–20 %.

**Not measured.** A single combined SHA-256+BLAKE3 multi-STARK (`p3-multi-stark`) — reported as the sum of two proofs
instead; Mersenne31/Circle-STARK variant; STIR PCS variant (`-p stir`, exists in the example); WHIR binary PCS
(`--pcs whir`, exists); GPU (none supported anyway). SP1's chips were not run (per instructions).

## Files

* Clone + patch: `<scratchpad>/exp-plonky3/Plonky3/` (patched files: `examples/src/airs.rs`, `examples/src/parsers.rs`,
  `examples/src/proofs.rs`, `examples/examples/prove_prime_field_31.rs`).
* Binaries: `<scratchpad>/exp-plonky3/target/release/examples/{prove_prime_field_31,prove_hash_binary}`.
* Raw logs: `<scratchpad>/exp-plonky3/runs/` (first pass, some under heavy load), `runs2/` (re-runs + 2^18), `runs_failed/`
  (binary 2^18 at 100-bit target, refused). Scripts: `bench.sh`, `bench2.sh`, `benchbin.sh`, `queue{1..5}.sh`, `parse.py`
  (regenerates the full table from the logs).
