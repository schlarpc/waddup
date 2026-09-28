# Literature survey: prover throughput for SHA-256 / BLAKE3 / Keccak in purpose-built proof systems (2023–2026)

Question: could a purpose-built prover for "SHA-256(F)=H1 AND BLAKE3(F)=H2" reach 100x (~160 KiB/s) or 1000x (~1.6 MiB/s) over the measured zkVM baselines (SP1 monolithic 1.6 KiB/s; RISC Zero ~0.3 KiB/s) on CPU-class hardware?

Short answer: yes, and by a much larger margin than 1000x. The 2026 binary-field sum-check provers (Flock, BinarySpartan) prove SHA-256 at ~400k compressions/s = ~25 MB/s and BLAKE3 at ~870k compressions/s = ~56 MB/s on a 12-core laptop, i.e. ~15,000x the SP1 baseline. Even the older Binius64 (2025) and Plonky3 BabyBear AIRs are 500–2,000x above the baseline. The binding constraints for this project are not prover speed but (a) prover memory (~1 KB of RAM per input byte in every sum-check system), which forces chunking for files above a few MiB, and (b) proof size per chunk (~0.3–0.5 MB), which forces some form of aggregation for multi-GB files because none of the fast binary-field systems ships recursion yet.

Units used throughout: 1 SHA-256 compression = 64 B, 1 BLAKE3 compression = 64 B, 1 Keccak-f[1600] = 136 B (SHA3-256 rate). "MB" = 10^6 bytes. Targets: 160 KiB/s = 164 KB/s; 1.6 MiB/s = 1.68 MB/s.

---

## 1. Purpose-built systems: measured numbers

### 1.1 Binary-field sum-check provers (the current frontier)

**BinarySpartan** (Setty, Microsoft Research; ePrint 2026/1656, Aug 2026). Spartan R1CS over GF(2) with Ligerito PCS + Diamond–Posen ring-switching. Transparent, hash-based, no ZK, 100-bit security. Circuits: SHA-256 = 30,376 R1CS constraints per compression, BLAKE3 = 14,000, Keccak-f = 38,400. Hardware: MacBook Pro M4 Max, 12 P-cores, no GPU. End-to-end including witness generation (Table 1 of the paper):

| Hash | hashes/s | bytes/s | batch at peak | peak RAM |
|---|---|---|---|---|
| SHA-256 | 401,000 | 25.7 MB/s | 2^17 compressions (8 MiB input) | 7.56 GB |
| BLAKE3 | 872,000 | 55.8 MB/s | 2^17 | 3.92 GB |
| Keccak-f | 287,000 | 39 MB/s | 12,288 perms | 1.25 GB |

Proof size 519 KiB at 2^14 BLAKE3 (422 KiB gzip; the paper notes Merkle paths are not deduplicated). Verifier 2.7–4.1 ms. Within 10% of peak rate already at 2^14 SHA-256 compressions (1 MiB input) in ≤1.3 GB RAM. Single-SHA-256 latency (EF client-side benchmark, 12 threads): 3.5 ms for 128 B, 6.2 ms for 2 KiB. Source: https://eprint.iacr.org/2026/1656.pdf ; code is the Spartan2 repo (research code, unaudited): https://github.com/microsoft/Spartan2

**Flock** (Bünz, Rothblum, Wang; Succinct/NYU; arXiv 2607.27491 / ePrint 2026/1329, July 2026). Batch Boolean R1CS, zerocheck + lincheck, Ligerito over GF(2^128), 100-bit. Ships SHA-256 and BLAKE3 provers plus "IO circuits" for hash-chaining and Merkle trees (exactly the structure this project needs). Numbers:
- M4 Max, 12 threads (as re-measured by BinarySpartan, same harness): SHA-256 400,928/s (25.7 MB/s), BLAKE3 802,899/s (51 MB/s), Keccak 288,315/s.
- M4 Max, single core (Flock paper): SHA-256 42k/s (2.7 MB/s), BLAKE3 82k/s (5.2 MB/s), Keccak 30k/s. Paper states ~250x overhead vs native BLAKE3 on one core.
- x86, AMD Threadripper 7970X (32 cores, Zen 4, AVX-512/VPCLMULQDQ) from the repo README: SHA-256 31.0k/s single-thread (2.0 MB/s), 305k/s at 32 threads (19.5 MB/s); BLAKE3 64.8k/s 1T, 630k/s 32T (40 MB/s), batch 262,144.
- Proof < 450 KB for 2^30 AND gates; verify "a few ms". EF benchmark: SHA-256 of 128 B in 32 ms, 45 MB RAM, 381 KB proof, 5 ms verify on an 8-core M1.
- Kernels: ARM64 NEON and x86-64 AVX-512/VPCLMULQDQ, with "portable fallbacks" (AVX2-only Zen 3 performance is not published: flag).
- Apache-2.0/MIT. Sources: https://arxiv.org/html/2607.27491 ; https://github.com/succinctlabs/flock

**Binius64** (Irreducible, 2025). 64-bit-word constraint system over binary towers; ZK; 96-bit security.
- M4 Max 12 threads (BinarySpartan harness): SHA-256 45,474/s (2.9 MB/s), BLAKE3 55,488/s (3.6 MB/s), Keccak 45,786/s (6.2 MB/s).
- binius.xyz/benchmarks, AWS c7i.16xlarge (32-core Sapphire Rapids, AVX-512+GFNI): Keccak-256, 1,365 permutations: 28 ms witness + 142 ms prove = 8,000 perms/s (1.1 MB/s), proof 304 KiB, verify 29 ms. Blake2s, 2,048 compressions: 18 + 223 ms = 8,500/s (0.54 MB/s), proof 360 KiB, verify 72 ms. These are small batches; peak rates (above) are 5x higher. Note: the site's phrase "about 1 MiB of total input data" does not match 1,365 x 136 B = 185 KB; I could not reconcile it (flag).
- EF client-side benchmark (M1, 8 cores): SHA-256 2 KiB in 67 ms, 248 MB RAM, 415 KB proof, 59 ms verify.
- Irreducible's Binius64 announcement contains no hash-throughput numbers and retracted earlier zkVM comparisons as not apples-to-apples. Its 2026 "Reinventing Irreducible" post pivots from custom silicon to commodity CPUs/mobile; the older FPGA/ASIC-for-binary-towers work has no published hash-proving throughput. Sources: https://www.binius.xyz/benchmarks/ ; https://www.irreducible.com/posts/announcing-binius64 ; https://github.com/binius-zk/binius64

**Hashcaster** (Soukhanov, 2024; GF(2) GKR-style boolcheck; Keccak only): 72,535 perms/s at 12 threads on M4 Max (9.9 MB/s), ~17k/s single core (Flock paper). Not extensible to SHA-256/BLAKE3 without handling u32 addition.

### 1.2 Prime-field STARK AIRs

**Plonky3** (Polygon; BabyBear/KoalaBear/M31 with FRI/STIR/WHIR). At 100-bit matched security on M4 Max 12 threads (BinarySpartan harness): BLAKE3 53,634/s (3.4 MB/s), Keccak 7,604/s (1.0 MB/s). On c7i.16xlarge (binius.xyz): Keccak 1,365 perms in 265 ms prove + 17 ms witness = 4,800/s (0.66 MB/s), proof 1.86 MB, verify 16–39 ms; trace 2,633 columns x 24 rows per permutation. Plonky3's newer binary-field track (2026, GF(2^128) multilinear STARK + BinaryPcs/WHIR): SHA-256 binary AIR = 23,712 bit-columns per compression (one row per compression, degree 2); Apple M2, 96-bit: 2^10 compressions proved in 0.16 s (6.4k/s = 0.41 MB/s, small batch, thread count unstated), proof 856 KB, verify 75 ms; BLAKE3 binary AIR 11,536 bit-columns; Keccak binary AIR 1,625 bit-columns x 24 rows. Sources: https://github.com/Plonky3/Plonky3/pull/2207 ; https://github.com/Plonky3/Plonky3/pull/2164 ; https://hackmd.io/@tcoratger/HJFGugU0Ze

**Stwo** (StarkWare, Circle STARK over M31, SIMD prover). Headline record is Poseidon2 (500k–620k hashes/s on 4-core i7 / M3 Pro), not relevant for standard hashes. For Blake2s on c7i.16xlarge (binius.xyz): 2,048 compressions in 442 ms prove + 926 ms witness = 1,500/s end-to-end (96 KB/s), 4,600/s prover-only (0.30 MB/s); proof 92.5 KiB; verify 2.5 ms. SHA-256 is not upstream; the third-party "rookie-numbers" M31 SHA-256 AIR (one row per compression, 9 lookup tables) proves a 128 B SHA-256 in 4.8 s / 3.4 GB on M1 in the EF benchmark, i.e. not yet throughput-tuned. GPU: ICICLE-Stwo reports 3.25–7x over the SIMD CPU backend (Feb 2025); NitrooZK-stwo (RTX 4090 vs 16-core EPYC) reports up to 22.6x on wide-Fibonacci with a Blake2s channel at 2^23 rows. Neither publishes Blake-AIR hashes/s. Sources: https://starkware.co/blog/starkware-new-proving-record/ ; https://www.ingonyama.com/post/introducing-icicle-stwo-a-gpu-accelerated-stwo-prover ; https://github.com/AntChainOpenLabs/NitrooZK-stwo

### 1.3 GKR provers

**Polyhedra Expander** (GKR, Keccak): 4,500 Keccak-f/s on M3 Max and ~4,700 on 7950X3D (2024, Expander blog); "up to 15,000 Keccak-f/s on consumer hardware" (2025, polyhedra.network/expander; 2.0 MB/s), 16 MB RAM per Keccak. GPU figures of 110k/s (RTX 4090, 15 MB/s) and 150k/s (H100, 20 MB/s) appeared only in a search summary of Polyhedra material; I could not load the primary page (HTTP 530) — treat as unverified. Cysic's C1 ASIC (64x64 PE array, TSMC 12 nm) is claimed at 1.31M Keccak-f/s (178 MB/s) with Polyhedra's GKR, vs 15k/s on a 7950X3D (vendor claim, HackMD write-up). Sources: https://blog.polyhedra.network/introducing-expander-the-fastest-gkr-proof-system-to-date/ ; https://www.polyhedra.network/expander ; https://hackmd.io/@Cysic/r1LoS0MmJg

### 1.4 Polynomial commitment schemes alone (calibration for the "commit floor")

- **Ligerito** (Novakovic–Angeris, 2025; used by Flock and BinarySpartan): Julia prover, M1 MacBook Pro, 8 threads, 32-bit binary field, 100-bit: 2^24 coefficients (64 MiB) in 1.3 s, proof 255 KiB; 2^30 coefficients (4 GiB) in 80 s, proof 420 KiB. That is ~50 MiB/s of committed data in an unoptimized Julia implementation. https://angeris.github.io/papers/ligerito.pdf
- **WHIR** (2024): 2^24-degree RS PCS prover 3.44 s at 32 threads vs 9.04 s for FRI/STIR. https://eprint.iacr.org/2024/1586
- **Blaze** (EUROCRYPT 2025): binary-field RAA-code MLPCS; commit = 8n XORs + one Merkle tree; "faster than all prior schemes except Brakedown" with much smaller proofs than Brakedown; no absolute numbers in the abstract. https://eprint.iacr.org/2024/1609
- Brakedown/Ligero are strictly dominated here: Ligerito gives the same linear-time prover with log^2-size proofs (hundreds of KB) instead of tens of MB.

### 1.5 Classic systems (calibration)

- **Groth16/circom**: circomlib SHA-256 ≈ 30k R1CS constraints per compression (2 KiB message = 1.1M constraints). EF benchmark on M1: 128 B in 356 ms (≈360 B/s), 2 KiB in 4.8 s (≈430 B/s), proof ~1 KB, verify 40 ms; rapidsnark ~1.1M constraints in 7 s on 8 cores (older). Trusted setup, tiny proof, ~0.4 KB/s.
- **Halo2 / plonky2 / starky** (Sladuca 2022, M1): starky 142 SHA-256 compressions/s (9 KB/s); halo2 ~38x slower (~0.24 KB/s); plonky2 (EF benchmark, M1): 2 KiB in 24.3 s (84 B/s), 9 GB RAM.
- **Nova / folding**: native Nova + bellperson SHA-256 on M1 Max, single fold: 64 KB preimage (27.4M constraints) in 7.7 s (8.5 KB/s); recursion adds ~2 s per step. NeutronNova folds a 2^20-constraint SHA-256 circuit in 91 ms (Flock paper characterizes it at ~84 compressions/s ≈ 5 KB/s). Folding is 3–4 orders of magnitude below sum-check batching for this workload.
- Ethereum Foundation client-side proving benchmark (SHA-256, AWS mac2.metal M1 8-core, 16 GB, updated 29 Jul 2026), 128 B message: flock 32 ms, expander 46 ms, binius64 46 ms, spartan2 73 ms, circom 356 ms, barretenberg 610 ms, provekit 677 ms, jolt 1.5 s, plonky2 2.9 s, rookie-numbers 4.8 s, risc0 18.5 s. https://ethproofs.org/csp-benchmarks

### 1.6 zkVMs with precompiles (what the baseline looks like elsewhere)

- babybear-labs CPU-only benchmark (48-core EPYC Rome, Jul 2025): SHA-256 of 262,144 B takes 257–2,338 s across RISC Zero, SP1, OpenVM, Jolt, Stone, S-two, i.e. 0.11–1.0 KB/s — consistent with your 1.6 KiB/s SP1 baseline. https://github.com/babybear-labs/benchmark
- SP1 (Hypercube): SHA-256 precompile = 2 chips, 80 + 48 rows per compression at ~210/~100 columns, i.e. ~22–25k KoalaBear cells (~700 kbit) per 64-byte block (survey by tcoratger). Ethereum-block latency numbers only (16 x RTX 5090 < 12 s). No hash-throughput publication.
- RISC Zero: R0VM 2.0 inlines SHA-256 as 73 rows in the RV32IM trace; docs quote 68 cycles per 64-byte block for the accelerator (from a search excerpt of dev.risczero.com; flag). Only Ethereum-block latencies published (44 s).
- Jolt: SHA-256 inline is 1,938 rows per 64-byte block (a16z/jolt PR #1863 discussion). Lattice Jolt (Sept 2026 press): 2M RISC-V cycles/s on a laptop CPU, 10M+ on a MacBook Metal GPU, proofs < 100 KB — "not independently verified". Derived, assuming 1 row ≈ 1 cycle and no other overhead: ~66 KB/s CPU, ~330 KB/s GPU. a16z also projects ~5,000 Keccak-f/s single-threaded on a c7i.16xlarge once optimized (unverified). So the best zkVM route lands at roughly 50–300 KB/s, 100x below purpose-built.
- Airbender: 21.8 MHz on one H100 with Blake2s/Blake3 "delegation" circuits; ZisK: SHA-256 as 72 Goldilocks rows per compression (~7,350 cells); Pico Prism: 6 s Ethereum blocks on 64 x 5090. None publishes hash bytes/s.

### 1.7 GPU / FPGA / ASIC

- Purpose-built binary-field provers (Flock, BinarySpartan, Binius64, Hashcaster): no GPU implementation exists. Ingonyama's open-binius targets FPGA building blocks (binary tower NTT), no end-to-end numbers. Irreducible shelved custom silicon in 2026.
- Prime-field STARKs on GPU: ICICLE-Stwo 3.25–7x, NitrooZK-stwo up to 22x (generic AIR workloads, RTX 4090); SP1/RISC Zero/Airbender/ZisK/Pico all run on 5090/H100 but report only block latencies.
- GKR on GPU/ASIC: Expander 110k–150k Keccak/s (4090/H100, unverified); Cysic C1 ASIC 1.31M Keccak/s (vendor claim).
- Fabric VPU, Ingonyama–Cornami: no hash-proving throughput published.

---

## 2. Theoretical floor: committed bits per compression, and bits/s the provers actually handle

Witness/trace size per 64-byte SHA-256 compression in real arithmetizations:

| Arithmetization | Field | Cells per compression | ≈ committed bits |
|---|---|---|---|
| BinarySpartan Boolean R1CS | GF(2) (packed 128:1) | 30,376 constraints (~30k witness bits) | ~30 kbit |
| Plonky3 sha256 binary AIR | GF(2) bits | 23,712 columns x 1 row | 23.7 kbit |
| Plonky3 sha256-air (prime) | BabyBear (31-bit) | 7,488 columns x 1 row | ~232 kbit |
| ZisK | Goldilocks (64-bit) | ~7,350 cells (72 rows) | ~470 kbit |
| rookie-numbers | M31 | ~7,840 cells + 9 lookup tables | ~243 kbit + lookups |
| SP1 | KoalaBear | ~22–25k cells (128 rows, 2 chips) | ~700 kbit |
| circom/Groth16 | BN254 (254-bit) | ~30k R1CS | ~7.6 Mbit (curve ops dominate) |

BLAKE3: 14,000 constraints (BinarySpartan), 11,536 bit-columns (Plonky3 binary). Keccak-f: 38,400 AND gates ≈ 1,600 bits x 24 rounds (Plonky3 binary: 1,625 columns x 24 rows = 39k bits); Plonky3 prime keccak-air: 2,633 x 24 = 63k BabyBear cells (~2 Mbit).

So the information-theoretic-ish floor for a bit-level arithmetization is ~470 committed bits per input byte for SHA-256 (30 kbit / 64 B), ~220 for BLAKE3, ~290 for Keccak. Prime-field AIRs pay 8–25x more committed bits because each Boolean lives in a 31–64-bit cell (the "embedding overhead" Binius/Flock are built to remove).

Committed data rate observed: BinarySpartan at 401k SHA-256/s commits 401k x 30.4 kbit ≈ 12 Gbit/s ≈ 1.5 GB/s of packed witness (as 95M GF(2^128) elements/s) on 12 cores, and that is not the bottleneck — sum-check evaluation is. Ligerito's Julia reference commits at ~50 MiB/s on 8 M1 threads; WHIR (prime field, 32 threads) at 2^24 x 4 B / 3.44 s ≈ 20 MB/s. Rule of thumb from these numbers: a tuned binary-field sum-check prover costs ~250x native hashing on one core (Flock), and prime-field STARK AIRs cost ~10–40x more than that at matched security (BinarySpartan Table 1: 16–38x vs Plonky3).

Bound for this project's statement: per input byte you must commit ~470 (SHA-256) + ~220 (BLAKE3, incl. ~6% for parent nodes) ≈ 700 bits. At BinarySpartan's measured rates the two hashes in sequence cost 1/(1/25.7 + 1/55.8) MB/s ≈ 17.6 MB/s on an M4 Max (12 P-cores) — ~10,000x your SP1 baseline — or ~1.8 MB/s single-core (Flock 1T rates combined: 1/(1/2.7+1/5.2)).

---

## 3. Large inputs and memory

- Every sum-check/STARK prover above holds the whole batch's witness and its sum-check tables in RAM: BinarySpartan 7.56 GB at 2^17 SHA-256 compressions (8 MiB of input) ≈ 0.9–1 KB RAM per input byte; Flock 7.1 GB at the same batch, 14.3 GB at 2^18; Plonky3 10.6 GB at its peak batch. Your SP1 OOM at 1 MiB / 64 GiB and RISC Zero's numbers are the same phenomenon with a 25x larger witness. A 64 GB box therefore caps a single proof at ~2^18–2^19 SHA-256 compressions (16–32 MiB) with these systems.
- Throughput is nearly flat with batch size above 2^14 (BinarySpartan within 10% of peak at 2^14, i.e. 1 MiB per proof in ≤1.3 GB), so chunking costs little prover time. Proof size is the cost: ~0.3–0.5 MB per chunk regardless of chunk size (Ligerito is log^2). At 8 MiB chunks, a 1 GiB file is 128 proofs ≈ 50–60 MB; 10 GiB ≈ 0.5 GB. That exceeds your "several MB" budget by 10–100x for multi-GB files, so aggregation is needed above roughly 100 MiB.
- Aggregation options: (i) none of Flock/BinarySpartan/Binius64 implements recursion; a Ligerito verifier is ~150 Merkle-path openings per level plus a small sum-check, i.e. on the order of 10^4–10^5 hash compressions and few field ops — cheap for these very provers (tens of ms per verified chunk at 400k hashes/s) but it must be written; (ii) Plonky3 has recursion infrastructure (this is what SP1 uses) but is 16–38x slower on hashes; (iii) streaming/low-memory provers: Scribe (USENIX Sec 2026) proves 2^28-gate circuits in <750 MB RAM at +10% time by read-write streaming to disk (HyperPlonk-based, prime field); Gemini (elastic, log-memory mode is quasilinear time); HOBBIT (ePrint 2025/1214). None is binary-field or hash-tuned, but the technique (stream the witness, keep sum-check tables on disk) transfers. (iv) Folding/IVC (Nova, Nexus, Sonobe): constant memory, but 5–10 KB/s.
- Proofs of retrievability / proofs of data possession (Juels–Kaliski 2007 etc.) are a different problem (server proves it still stores F using sampled challenges); they cannot bind two hash digests of a hidden preimage and are not applicable.

---

## 4. Structural shortcuts for "two hashes of one preimage"

- I found no paper or repo specifically about proving two different standard hash digests of the same preimage. The building blocks exist: LegoSNARK-style commit-and-prove (commit to the message once, open it in both sub-proofs), and, more simply, shared witness columns inside one batched circuit.
- In a bit-level batched circuit (Flock IO circuits, BinarySpartan's SuperSpartan "next MLE" linking, Plonky3 multi-stark cross-table lookups/logUp), the message words of SHA-256 block i and BLAKE3 chunk-compression j are the same witness bits, wired by linear constraints that commit nothing extra. Endianness differences (SHA-256 big-endian words, BLAKE3 little-endian words) are free relabelings at bit level; in word-level systems (Binius64) they cost byte-swap constraints only.
- SHA-256's Merkle–Damgård chain forces sequential state passing (Flock's hash-chain IO circuit; BinarySpartan's tail matrix pins padding and final digest). BLAKE3's tree is embarrassingly parallel: 16 compressions per 1 KiB chunk, chaining values feed a parent tree of ~n/16 extra compressions (~6% overhead), and chunk proofs can expose chaining values publicly so the top of the tree is a tiny separate proof. For chunked proving, each chunk proof exposes (sha_state_in, sha_state_out, blake3_cv[]) as public IO; the verifier checks the chain and the BLAKE3 parent tree itself in O(chunks) time (~1 ms per chunk at 3–60 ms verify per proof), keeping within a ~1 s verify budget up to a few hundred chunks.
- For SHA-1/MD5/SHA-512 variants: SHA-1 and MD5 are cheaper per byte than SHA-256 (fewer/lighter rounds, same ARX structure); SHA-512 processes 128-byte blocks with 64-bit adds, roughly 2x SHA-256 work per block, so similar bytes/s; Binius64's 64-bit word model fits SHA-512 natively (its README example: SHA-512 of 64 KiB in ~129 ms, ≈0.5 MB/s, unspecified hardware).

---

## 5. Summary table

| System (year) | Hash | Throughput (bytes/s) | Hardware | Proof size | Verify | Source |
|---|---|---|---|---|---|---|
| BinarySpartan (2026) | SHA-256 | 25.7 MB/s (401k/s) | M4 Max 12 P-cores, CPU | ~0.4–0.5 MB | 2.7 ms | eprint 2026/1656 |
| BinarySpartan | BLAKE3 | 55.8 MB/s (872k/s) | same | 519 KiB @2^14 | 2.7 ms | same |
| BinarySpartan | Keccak-f | 39 MB/s (287k/s) | same | — | 4.1 ms | same |
| Flock (2026) | SHA-256 | 25.7 MB/s 12T; 2.7 MB/s 1T | M4 Max | 381 KB (128 B) / <450 KB | ~5 ms | arXiv 2607.27491 |
| Flock | SHA-256 | 19.5 MB/s 32T; 2.0 MB/s 1T | Threadripper 7970X (AVX-512) | — | — | github succinctlabs/flock |
| Flock | BLAKE3 | 51 MB/s 12T (M4 Max); 40 MB/s 32T (7970X) | as above | — | — | same |
| Binius64 (2025) | SHA-256 | 2.9 MB/s (45k/s) | M4 Max 12T | ~415 KB | 59 ms | BinarySpartan Tbl 1; ethproofs |
| Binius64 | Keccak-256 | 1.1 MB/s (8.0k/s, 1,365-perm batch) | c7i.16xlarge 32c | 304 KiB | 29 ms | binius.xyz |
| Binius64 | Blake2s | 0.54 MB/s (8.5k/s, 2,048 batch) | c7i.16xlarge | 360 KiB | 72 ms | binius.xyz |
| Hashcaster (2024) | Keccak-f | 9.9 MB/s (72.5k/s) | M4 Max 12T | — | — | BinarySpartan Tbl 1 |
| Plonky3 BabyBear (2025/26) | BLAKE3 | 3.4 MB/s (53.6k/s @100-bit) | M4 Max 12T | — | — | BinarySpartan Tbl 1 |
| Plonky3 BabyBear | Keccak-f | 0.66–1.0 MB/s (4.8k–7.6k/s) | c7i.16xlarge / M4 Max | 1.86 MB | 16–39 ms | binius.xyz; BinarySpartan |
| Plonky3 binary SHA-256 AIR (2026) | SHA-256 | 0.41 MB/s (6.4k/s, 2^10 batch) | Apple M2 | 856 KB | 75 ms | P3 PR #2207 |
| Stwo (2025) | Blake2s | 0.30 MB/s prove-only; 0.10 MB/s e2e | c7i.16xlarge | 92.5 KiB | 2.5 ms | binius.xyz |
| Expander GKR (2025) | Keccak-f | 2.0 MB/s (15k/s) | consumer CPU (7950X3D) | — | — | polyhedra.network |
| Expander GPU (unverified) | Keccak-f | 15 / 20 MB/s (110k / 150k/s) | RTX 4090 / H100 | — | — | search summary only |
| Cysic C1 ASIC (claim) | Keccak-f | 178 MB/s (1.31M/s) | 12 nm ASIC | — | — | hackmd @Cysic |
| Ligerito PCS only (2025) | commit | ~50 MiB/s of 32-bit coeffs | M1 8T, Julia | 420 KiB @2^30 | — | Ligerito paper |
| Jolt Lattice + SHA inline (2026, derived) | SHA-256 | ~66 KB/s CPU, ~330 KB/s Metal GPU | laptop | <100 KB | — | a16z press; PR #1863 |
| Nova (2023) | SHA-256 | 8.5 KB/s (single fold, 64 KB) | M1 Max | — | — | oskarth hackmd |
| Groth16/circom | SHA-256 | ~0.4 KB/s | M1 8c | ~1 KB | 40 ms | ethproofs csp |
| starky / halo2 (2022) | SHA-256 | 9 KB/s / 0.24 KB/s | M1 | — | — | Sladuca |
| zkVMs w/ precompiles (CPU, 2025) | SHA-256 | 0.11–1.0 KB/s | 48c EPYC | 0.2–1.3 MB | 23–236 ms | babybear-labs |
| Your baselines | SHA-256+BLAKE3 | SP1 1.6 KiB/s; RISC Zero 0.3 KiB/s | Ryzen 5800X | — | — | this project |

---

## 6. Assessment

**Is 100x (160 KiB/s) plausible on CPU?** Yes, with a >100x margin. Binary-field sum-check provers demonstrate 2–2.7 MB/s per core for SHA-256 and 5 MB/s per core for BLAKE3 on Apple M4 / Zen 4 cores. Even the slower prime-field route (Plonky3 BabyBear AIRs, 3.4 MB/s BLAKE3 on 12 cores) clears 160 KiB/s by 20x, and even Binius64 (2.9 MB/s SHA-256) by 18x.

**Is 1000x (1.6 MiB/s) plausible on CPU?** Yes for the binary-field systems. Combined SHA-256+BLAKE3 at BinarySpartan/Flock rates is ~17.6 MB/s on 12 M4 Max cores and ~1.8 MB/s on one core. For a Ryzen 7 5800X (8 Zen 3 cores, AVX2 + 128-bit PCLMULQDQ, no AVX-512/VPCLMULQDQ/GFNI) the published kernels do not apply directly: Flock/Binius64 ship AVX-512/VPCLMULQDQ kernels with "portable fallbacks", and no AVX2-only numbers are published (flag). Assuming a pessimistic 3–5x per-core penalty vs the tuned kernels, 8 cores give roughly 0.4–0.7 MB/s per core x 8 ≈ 3–5 MB/s combined, i.e. 2,000–3,000x the SP1 baseline; even a 10x penalty leaves ~1.5 MB/s ≈ 1000x. Plonky3-prime-field would land around 1–2 MB/s combined on that box (roughly 700–1,200x), close to the line. GPU: no binary-field GPU prover exists; prime-field STARK GPU backends give 3–20x over CPU SIMD, and Expander's GKR reportedly reaches 15–20 MB/s Keccak on a 4090/H100 (unverified), so a GPU route to 1000x is plausible but would require porting work that no one has published for SHA-256/BLAKE3. FPGA/ASIC: only Cysic's 1.31M Keccak/s claim; not a practical option here.

**What it would take.**
1. Adopt Flock (Apache/MIT, has SHA-256, BLAKE3 and hash-chaining IO circuits, x86 AVX-512 + NEON kernels) or BinarySpartan/Spartan2 (MIT, research code). Both are 2026 research prototypes without ZK (not needed here) and without audits.
2. Write the joint SIMD-R1CS circuit: SHA-256 chain + BLAKE3 chunk/parent tree, sharing message bits; add padding/length pinning (already in both codebases for SHA-256) and public IO for chunk chaining values.
3. Add an AVX2/PCLMULQDQ GF(2^128) kernel path (or measure the fallback) for Zen 3; expected 3–10x below Zen 4 AVX-512 per core.
4. Chunk at ~2^15–2^17 compressions (2–8 MiB) to stay under 2–8 GB RAM, ~10% throughput loss; proofs ~0.3–0.5 MB per chunk (Merkle-path dedup and Johnson-radius parameters could roughly halve this). Verify cost 3–60 ms per chunk.
5. For files above ~100 MiB, proof size (not time) becomes the problem: either build a Ligerito-verifier circuit for one-level aggregation (cheap for these provers but unimplemented anywhere), or accept tens of MB of proof per GB, or stream (Scribe-style) with larger batches.

**Numbers I could not verify or that need caveats:** Expander GPU 110k/150k Keccak/s (search summary only; primary page unreachable); Cysic 1.31M/s (vendor); Lattice Jolt 2M/10M cycles/s (press release, "not independently verified") and my derived KB/s from it; RISC Zero "68 cycles per block" (search excerpt); a16z "5,000 Keccak/s single-thread" projection; binius.xyz "1 MiB of input" phrasing vs its own permutation counts; Plonky3 PR #2207 thread count; Flock/Binius64 performance on AVX2-only CPUs (unpublished); Stwo has no published Blake-AIR throughput beyond the binius.xyz small-batch run.

Primary sources: https://eprint.iacr.org/2026/1656.pdf ; https://arxiv.org/html/2607.27491 ; https://github.com/succinctlabs/flock ; https://www.binius.xyz/benchmarks/ ; https://www.irreducible.com/posts/announcing-binius64 ; https://github.com/Plonky3/Plonky3/pull/2207 ; https://github.com/Plonky3/Plonky3/pull/2164 ; https://hackmd.io/@tcoratger/HJFGugU0Ze ; https://angeris.github.io/papers/ligerito.pdf ; https://eprint.iacr.org/2024/1586 ; https://eprint.iacr.org/2024/1609 ; https://ethproofs.org/csp-benchmarks ; https://github.com/babybear-labs/benchmark ; https://hackmd.io/@oskarth/rJHXGY6Mn ; https://github.com/Sladuca/sha256-prover-comparison ; https://blog.polyhedra.network/introducing-expander-the-fastest-gkr-proof-system-to-date/ ; https://www.polyhedra.network/expander ; https://hackmd.io/@Cysic/r1LoS0MmJg ; https://starkware.co/blog/starkware-new-proving-record/ ; https://www.ingonyama.com/post/introducing-icicle-stwo-a-gpu-accelerated-stwo-prover ; https://github.com/AntChainOpenLabs/NitrooZK-stwo ; https://a16zcrypto.com/posts/article/jolt-6x-speedup/ ; https://thequantuminsider.com/2026/09/09/a16z-crypto-rebuilds-jolt-proof-system-with-post-quantum-security-and-faster-performance/ ; https://eprint.iacr.org/2024/1970 (Scribe) ; https://eprint.iacr.org/2022/420 (Gemini) ; https://eprint.iacr.org/2025/1214 (HOBBIT) ; https://eprint.iacr.org/2024/1606 (NeutronNova)
