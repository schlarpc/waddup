# waddup — purpose-built prover vs. zkVM, and formal verification

Companion to `PROPOSAL.md` and `RESEARCH.md`. Question investigated (2026-09-28):

> If we built a purpose-built ZK prover/verifier for the binding statement
> "SHA-256(F) = H_content AND BLAKE3(F) = H_swarm" instead of a general-purpose
> zkVM (RISC Zero, SP1), could proving be 100x ("viable") to 1000x ("useful")
> faster? And could we prove the result correct and secure, e.g. in Lean?

Method: two literature surveys, five bounded prover experiments on the same
Ryzen 7 5800X box as `RESEARCH.md` (8c/16t Zen 3, AVX2, no AVX-512/GFNI,
64 GiB), and one Lean toolchain probe. Raw reports with commands, commits,
logs and caveats are in `research/purpose-built/`. All prover numbers are
single-run wall clocks of the prover's own timers, taken under a shared lock
at 1-minute load ≤ 16 (runs under heavier build contention were discarded).

---

## TL;DR

1. **Yes to both thresholds, measured on this box.** The combined
   SHA-256 + BLAKE3 statement proves at **~2.2 MiB/s on 16 threads with Flock**
   (Succinct/NYU's 2026 binary-field sum-check prover), about **1,400x** the
   SP1-with-precompiles baseline of 1.6 KiB/s, and **~285 KiB/s on one thread**
   (~180x). Plonky3's new binary-field STARK reaches ~1.3 MiB/s (~860x);
   Binius64 ~755 KiB/s (~470x); a prime-field Plonky3 STARK ~210 KiB/s (~130x);
   Stwo ~245 KiB/s extrapolated (~150x). GKR over GF(2) is the wrong family for
   SHA-256 (2–8 KiB/s per core). 1 GiB goes from roughly a CPU-week to ~8 minutes.
2. **The cost of the proof was never the hashes; it was the zkVM.** Plonky3's
   own SHA-256 chip proves ~450 KiB/s standalone; SP1 wraps the same kind of chip
   in CPU/memory/bus tables and per-shard recursion and lands at 1.6 KiB/s.
3. **Formal verification in Lean is partly available today, but only for the
   circuit layer.** Certora/zkSecurity's Clean library already has `sorry`-free
   soundness and completeness proofs for a SHA-256 block compression and a
   BLAKE3 compression (rebuilt and kernel-checked here). No proof system has a
   machine-checked end-to-end soundness theorem in any prover: ArkLib's FRI
   reductions are literally `sorry`, Fiat-Shamir/BCS is absent. A credible
   "circuit is machine-checked, verifier binary is machine-checked, protocol
   soundness is cited" claim is ~10–20 person-months; the full stack is
   multi-person-years and gated on ArkLib.
4. **A non-ZK alternative exists for Merkle–Damgård content hashes** (SHA-256,
   SHA-512, SHA-1, MD5): publish SHA-256 midstates at Bao chunk-group boundaries
   and verify chunks in reverse order. Zero proving cost, ~0.2% metadata,
   security reduces to free-start preimage resistance of the compression
   function, and it is small enough to verify end-to-end in Lean in weeks. The
   price is no transferable upfront certificate and reverse-order verified
   streaming. It does not work for sponge hashes (SHA-3).

---

## Part 1 — How fast can a purpose-built prover go?

### Baselines (from `RESEARCH.md`, same machine)

| System | Rate for the binding statement | Notes |
|---|---:|---|
| SP1 v6.2.2 monolithic, SHA-256 + BLAKE3 precompiles | 1.6 KiB/s (100 KiB in 63 s) | OOMs at 1 MiB |
| SP1 decomposed (16 chunk proofs + root) | 0.45 KiB/s (1 MiB in 2300 s) | ~83% is per-sub-proof recursion tax |
| RISC Zero v3.0.5 monolithic | 0.29 KiB/s (1 MiB in 3591 s) | BLAKE3 unaccelerated |
| Native SHA-256 (SHA-NI, `openssl`) | 577 MB/s | measured here, 1 GiB in 1.86 s |
| Native BLAKE3 (`b3sum`, 1 thread / 16) | 1.25 GB/s / 3.8 GB/s | measured here |

Targets: 100x = 160 KiB/s; 1000x = 1.6 MiB/s.

### Measured: combined SHA-256 + BLAKE3 over the same bytes, 16 threads

"Combined" is the harmonic sum of separately measured SHA-256 and BLAKE3
proving rates (each 64-byte block costs one compression of each). All systems
were run at 2^12–2^18 compressions; the table shows the 2^16 or 2^18 point.

| Prover (family) | SHA-256 alone | BLAKE3 alone | **Combined** | vs SP1 mono | Security | Proof size | Verify | Peak RAM per input byte |
|---|---:|---:|---:|---:|---|---:|---:|---:|
| **Flock** (GF(2^128) zerocheck/lincheck + Ligerito) | 3.2 MiB/s | 6.9–7.6 MiB/s | **2.2 MiB/s** (1 thread: 285 KiB/s) | **~1,400x** | 128-bit work, Johnson regime | 330–490 KiB | 9 ms | ~1.1 KB → 32 MiB max/proof |
| **Plonky3 binary-field STARK** (GF(2) Boolean AIR, folding PCS) | 1.9 MiB/s | 4.6 MiB/s | **1.34 MiB/s** | **~860x** | 98–101 bits composed | 1.0–1.3 MB | 40 ms | ~0.45 KB |
| **Binius64** (binary tower, FRI) | 1.3 MB/s | 1.8 MB/s | **755 KiB/s** | **~470x** | 96-bit hard-coded, 232 queries | ~400 KiB | 200–400 ms | ~8 KB (frontend) |
| **Plonky3 prime-field STARK** (KoalaBear, FRI) | 454 KiB/s | 386 KiB/s | **~210 KiB/s** | **~130x** | 100 conjectured / 59 proven | 3.0–3.7 MB | 55 ms | ~1 KB |
| **Stwo** (Circle STARK, M31; BLAKE2s measured, SHA-256 extrapolated) | ~330 KB/s est. | ~1.1 MB/s est. | **~245 KiB/s est.** | **~150x** | 96-bit (stwo's accounting) | ~1 MB | 7 ms | ~1.2 KB |
| **Expander GKR** (GF(2), Keccak-oriented) | 2–8 KiB/s per core | not available | n/a | ~20–100x at 16T | 128-bit Orion | 0.8–3.9 MB | 0.2–1 s | low (streams layers) |

Details, commands, commits and caveats: `research/purpose-built/exp-*.md`.

Published numbers on newer hardware are higher still: Flock and BinarySpartan
report ~25 MB/s SHA-256 and ~50 MB/s BLAKE3 on a 12-core M4 Max or a 32-core
Zen 4 with AVX-512. On this Zen 3 box Flock compiles to scalar PCLMULQDQ plus
portable kernels (no AVX2 variants exist), which costs ~4.5–5x per thread
versus the published Zen 4 figures. An AVX2/256-bit VPCLMULQDQ port of its
kernels is the obvious ~2x lever nobody has written.

### What this means for waddup

| File size | Baseline (SP1 mono, 1.6 KiB/s) | Flock on this box (2.2 MiB/s) | Plonky3 binary (1.3 MiB/s) |
|---|---:|---:|---:|
| 1 MiB | ~11 min (or OOM) | ~0.5 s | ~0.8 s |
| 100 MiB | ~18 h | ~45 s | ~77 s |
| 1 GiB | ~7.6 days | ~8 min | ~13 min |
| 10 GiB | ~2.5 months | ~80 min | ~2.2 h |

Memory is the real constraint, not time. Every sum-check/STARK prover holds
~0.5–1.1 KB of RAM per input byte, so a single proof tops out at ~16–32 MiB of
input on 64 GiB. The binding statement decomposes cleanly (SHA-256 chain state
plus BLAKE3 subtree root per chunk, exactly as in `sp1-pr2615-bench-decomposed/`),
and because the proof rides alongside a large download, **aggregation is
optional**: 1 GiB in 32 MiB chunks is 32 proofs of ~0.5 MB each (~16 MB, 1.5%
of the file) and ~0.3 s of verification. Recursion only becomes worthwhile for
multi-GB files or if proof size must stay constant. None of the fast
binary-field systems ships recursion today; Plonky3 does, at prime-field speed.

### Why the zkVM was 100–1000x slower

- Plonky3's standalone SHA-256 AIR commits 7,728 KoalaBear cells per
  compression and proves ~56 M cells/s on this box. SP1's SHA-256 chips are
  ~2–3x larger, and then the zkVM adds CPU, memory, bus and lookup tables plus
  per-shard recursion. The precompile chip is a few percent of what SP1 proves.
- Binary-field systems commit one *bit* per trace cell (23,712 bits per
  SHA-256 compression) instead of a 31-bit field element, and XOR/rotate are
  free. That is the 4–5x gap between Plonky3 prime-field and binary-field.
- The 100x threshold is met by every purpose-built family measured; the
  1000x threshold is met on CPU only by the two binary-field sum-check/STARK
  stacks, and comfortably by them on Zen 4/Apple silicon.

### What was NOT measured, and expected overhead

Every benchmark proves N *independent* compressions on random inputs. The real
statement additionally needs: SHA-256 chaining (`H_out[i] = H_in[i+1]`),
padding and length, the BLAKE3 chunk/parent tree, public `(H_content,
H_swarm)`, and the same message bits feeding both hash tables. In Flock the
BLAKE3 chain circuit exists, the SHA-256 table exists but has no wiring
schema; message bits are laid out identically in both tables so sharing them
is a wiring-only change. In Plonky3 it is a handful of transition constraints
and a lookup. Expected cost: a few percent to ≤20%, not a multiple.
GPU: no binary-field prover has a GPU backend at all; the Radeon remains
irrelevant. All measured systems are research/pre-audit code; none provides
zero-knowledge, which this protocol does not need (the file is public).

---

## Part 2 — Could we prove it works, securely, in Lean?

"Secure" decomposes into four layers with very different maturity
(`research/purpose-built/lit-formal-verification.md`, `exp-lean.md`).

| Layer | What exists (verified on this box where stated) | Gap for waddup | Effort |
|---|---|---|---|
| **Specs** (SHA-256, BLAKE3, SHA-1/512, MD5 in Lean 4) | Clean has a full SHA-256 spec with `native_decide` test vectors; BLAKE3 single-chunk spec in Clean, full tree in `argumentcomputer/Blake3.lean` | SHA-1/SHA-512/MD5 specs; BLAKE3 tree in Clean | weeks |
| **Circuit** (constraints hold ⟹ hash spec holds, and honest witness satisfies them) | **Clean `Gadgets.SHA256.CompressBlock` and `Gadgets.BLAKE3.Compress` are proven sound and complete; rebuilt here, `#print axioms` shows no `sorryAx`.** A 17-row BLAKE3 chunk table exists. sp1-lean proved all 62 SP1 RV64 chips sound this way. | Multi-block SHA-256 table, padding, BLAKE3 tree, shared-message binding, public IO. Clean's SHA-256 gadget is bit-level over a prime field with p > 2^33 (not BabyBear/KoalaBear, not GF(2)); a fast binary-field circuit needs new gadgets. | 3–6 person-months |
| **Protocol** (FRI/Ligerito + Merkle + Fiat-Shamir soundness) | ArkLib: sum-check sorry-free (built here), **FRI oracle reductions `:= sorry`**, STIR stated but `sorry`, Fiat-Shamir completeness `sorry`, BCS commented out, no Merkle trees, no STARK theorem. A standalone Lean proof of FRI round-by-round soundness exists (zkSecurity, 2026). Isabelle has a toy end-to-end STARK. | Everything. Also note: the "up to capacity" proximity-gap conjectures were disproven in 2026; parameters must sit at the Johnson bound (Flock's default does; Plonky3 prime-field claims 100 bits conjectured, 59 proven). | cite papers now; 18–36 PM to machine-check once ArkLib's BCS layer lands |
| **Implementation** (Rust verifier = Lean spec) | Aeneas (Rust→Lean) built here from its flake; used by Microsoft to verify 16.7 KLOC of SymCrypt; Nethermind verified zkSync's PLONK verifier "honest" in 2.5 months (EasyCrypt) | Write the verifier in the Aeneas subset (no nested loops, no closures), prove equivalence to a Lean reference verifier | 6–12 PM |

**Honest achievable claim within a year (1–2 people, ~10–20 PM):** "The
constraint system is machine-checked in Lean to admit a satisfying trace iff
SHA-256(F) = H_content and BLAKE3(F) = H_swarm; the shipped verifier is
machine-checked to implement the specified verifier; soundness of the proof
system rests on published proofs under the random-oracle model at
Johnson-bound parameters." That is stronger than what RISC Zero or SP1 can say
today (SP1's chips are partly verified; neither zkVM's protocol or verifier
binary is), but it is not "the ZK proof is proven secure in Lean".

**Which prover is most verifiable vs. fastest.** They pull in opposite
directions:

- Prime-field Plonky3 STARK: every Lean tool targets it (Clean's Plonky3
  backend, sp1-lean's extraction workflow, Picus/LLZK, the FRI Lean proof), but
  it is the ~130x option.
- Plonky3 binary-field STARK and Flock: 860x and 1,400x, but the binary-field
  code is months old, unaudited, and has no formal artifacts beyond ArkLib's
  in-progress Binius work. Clean's field is `ZMod p`; instantiating at p = 2 is
  possible in principle but no GF(2) gadgets exist.
- Groth16 has the best protocol-level formalization but a trusted setup and a
  ~0.4 KB/s prover; not a fit.

Recommendation if a ZK proof is kept: prototype on Flock for speed, but write
the circuit spec in Clean from day one so Rust constraints are generated or
extracted rather than hand-mirrored, and keep the verifier in the Aeneas
subset. Treat protocol-level machine-checking as a later upgrade.

---

## Part 3 — The non-ZK alternative: midstate chains

The ZK proof exists so a leecher can trust `H_swarm` before spending
bandwidth. For Merkle–Damgård hashes that trust can be established
incrementally with no prover at all.

**Setup.** The seeder publishes, next to `H_swarm`, the SHA-256 midstate
(32 bytes) at every 16 KiB Bao chunk-group boundary: ~0.2% of the file,
produced by one native hashing pass.

**Verification.** Before downloading anything, the leecher checks two anchors:
midstate 0 equals the SHA-256 IV, and the last midstate plus the padding block
for the claimed length compresses to `H_content`. Then it downloads from the
end backwards, one Bao-verified chunk group at a time, and checks that
compressing the previous midstate with the group's bytes yields the next
midstate.

**Why it holds.** Each backward link starts from a value already tied to
`H_content`. Serving different bytes for any group requires a different
(midstate, bytes) pair compressing to that trusted value: a free-start
preimage on the SHA-256 compression function. The best known pseudo-preimage
attack reaches 52 of 64 steps at 2^255 work (Khovratovich–Rechberger–Savelieva
2012); nothing better has appeared through 2026. Forward links prove nothing
(anyone can hash garbage forward from the IV), so verification order matters.

**Attacker's options.** Publish a fake swarm with an honest suffix and garbage
prefix. The leecher verifies and keeps the honest suffix, hits the bad link,
and moves to another swarm; waste is bounded by the download lookahead
(as little as one 16 KiB group). Cost per fake swarm to the leecher is one
connection plus one chunk group, comparable to the DHT-spam annoyance
`PROPOSAL.md` already accepts. No wrong byte can ever be accepted.

**Tradeoffs versus the ZK design.**

- Works for SHA-256, SHA-512, SHA-1, MD5 (all Merkle–Damgård). Does not work
  for SHA-3/Keccak: the permutation is invertible, so a backward chain is
  forgeable until the initial state. BLAKE3 as content hash needs nothing.
- No compact transferable certificate: nobody can check "H_swarm is the swarm
  for H_content" without fetching a chunk group. Sybil filtering costs 16 KiB
  instead of 1 ms.
- Download order is constrained to reverse-sequential commitment
  (iroh-blobs supports arbitrary chunk ranges, so this is a client policy, not
  a protocol change).
- Trust assumption shrinks from "STARK + Ligerito soundness in the ROM +
  Johnson-bound proximity gaps" to "SHA-256 compression is free-start preimage
  resistant", an older and simpler assumption.
- Cost: native hashing speed, i.e. ~10^5x beyond even the 1000x target.

**Formal verification.** This is the one design that can plausibly be verified
end to end in Lean in weeks: the security argument is a short reduction to the
preimage assumption, the SHA-256 spec already exists in Clean, and the verifier
is a few hundred lines of loop-free Rust (compression function plus chain
check) well inside Aeneas's subset.

---

## Bottom line

- **100x is achieved by every purpose-built family; 1000x is achieved on this
  AVX2 desktop by Flock (1,400x) and nearly by Plonky3's binary-field STARK
  (860x)**, with 0.3–1.3 MB proofs and sub-50 ms verification. On current
  hardware (AVX-512 or Apple silicon) the same code is another 5–7x faster.
- Memory, not time, dictates architecture: chunk at 16–32 MiB and either ship
  many proofs (fine for a P2P download) or add aggregation later.
- Lean can machine-check the *circuit* now (half of it is already done in
  Clean) and the *verifier binary* with known tooling; it cannot yet
  machine-check the *proof system*. Budget 10–20 person-months for the
  credible partial claim.
- Before building any of it, decide whether a transferable certificate is
  actually required. If reverse-order verified streaming is acceptable, the
  midstate chain removes the prover entirely for every hash waddup targets
  except SHA-3, and is the only option that can be fully verified end to end
  on a short timeline.
