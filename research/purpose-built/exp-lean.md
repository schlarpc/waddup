# exp-lean: Feasibility probe of the formal-verification side (Lean 4)

Date: 2026-09-28. Machine: Ryzen 7 5800X, 64 GiB, NixOS. Load average during this
probe was 12-50 (other agents building/benchmarking), so all build timings below are
pessimistic. Nothing under /home/schlarpc/waddup was touched. All work lives in
/home/schlarpc/waddup-scratch/exp-lean/ (clones, elan toolchains, Mathlib cache ~3 GB).

## 0. TL;DR

* **Clean** (Verified-zkEVM/clean, zkSecurity) already contains **sorry-free, machine-checked
  `FormalCircuit`s for one SHA-256 block compression (`CompressBlock`) and one BLAKE3
  compression (`Compress`), plus an `InductiveTable` (multi-row AIR) for BLAKE3 chunk
  processing**, each proved sound *and* complete against an executable Lean spec that is
  itself checked by `native_decide` against known test vectors. This is far more of the
  "waddup verification story" than expected.
* Clean's soundness notion is exactly the one we want: for **every** witness assignment
  (`Environment`), if all polynomial constraints evaluate to 0 (and lookup rows are in the
  table), then `Spec input output` holds. Completeness: the default witness generator
  satisfies the constraints for every input satisfying `Assumptions`.
* Clean is **generic over the prime field `F p`** with per-gadget size side conditions.
  SHA-256 gadgets need `p > 2^33` (bit-level Add32 carry argument), BLAKE3/Keccak need
  `p > 2^16 + 2^8` (byte lookups). So the SHA-256 gadgets as written **do not instantiate on
  BabyBear/KoalaBear/M31** (2^31-ish); a Goldilocks/BN254-style field or a re-proved Add32
  is needed. BLAKE3 gadgets work on BabyBear/M31.
* Clean **connects to real provers only as PoC**: a Plonky3 backend (Rust, "NOT-PRODUCTION-READY
  POC", BabyBear, `create_test_fri_params` = insecure test parameters, only Fibonacci and
  FemtoCairo fixtures) and a Circom/snarkjs WASM+R1CS backend written in Lean. The Rust
  side (constraint evaluator, FRI prover/verifier) is **unverified**; Lean only certifies
  the arithmetization.
* **ArkLib** (Nethermind) is a ~280k-LOC Lean formalization of IORs / sum-check / FRI / STIR /
  Binius with **225 `sorry`s in 76 files**. The only sorry-free protocol slice is the new
  "Interaction" sum-check layer. **FRI's oracle reductions are literally `:= sorry`**,
  Fiat-Shamir completeness is `sorry`, the BCS transform is commented out, and there is no
  Merkle-tree formalization and **no STARK/AIR end-to-end theorem at all**. ArkLib gives us
  vocabulary and some lemmas, not a usable "FRI-STARK is sound" theorem.
* **Aeneas** (Rust -> Lean) is real and used for crypto (Microsoft SymCrypt ML-KEM), supports
  loops (but **no nested loops**), slices, wrapping u32/u64 ops; `nix build
  github:AeneasVerif/aeneas` works (551 s on the second try; see 5).
* Clean's key theorems were re-checked here with `#print axioms`: SHA-256 `CompressBlock`
  soundness/completeness use only `propext/Classical.choice/Quot.sound`; the BLAKE3 table
  additionally trusts one `bv_decide` native axiom. No `sorryAx` anywhere in the checked set.
* Bottom line: with Clean we could get a **machine-checked arithmetization** of
  "SHA-256(F)=H1 AND BLAKE3(F)=H2" with moderate effort (weeks, not months); what does *not*
  exist anywhere in Lean today is a verified FRI/STARK verifier or a verified Rust verifier
  binary, and building that is a multi-person-year project. See section 6.

## 1. What I installed / built (with timings)

| Step | Result |
|---|---|
| elan 4.2.4 via `nix shell nixpkgs#elan` | 8 s |
| Clean clone `fba2a29f5e36420d797c1de118ac9f11f23b819e` (2026-09-16), `lean-toolchain` = `leanprover/lean4:v4.33.1`, deps: mathlib `v4.33.1`, CompPoly `v4.33.1` | shallow clone 3.6 MB |
| `elan toolchain install v4.33.1` + `lake exe cache get` (8690 Mathlib olean files) | **251 s**, `.lake` = 2.9 GB |
| Clean targeted build (`lake build Clean.Gadgets.SHA256.SHA256Compress Clean.Tables.BLAKE3.ProcessBlocksInductive Clean.Gadgets.BLAKE3.FinalizeChunk Clean.Gadgets.Keccak.Permutation Clean.Specs.SHA256`, `taskset -c 0-5 nice -n 10`, box load 22-50) | **Success, exit 0, 1245 s wall (20.8 min)**; 1807 Lake jobs of which 106 Clean modules compiled from source (rest served by Mathlib cache); slowest single modules `Clean.Utils.Tactics.ProvableStructDeriving` 670 s and `Clean.Circuit.Channel` 622 s (meta code, heavily contended CPU). `SHA256Compress` itself: 28 s, BLAKE3 table 22 s, Keccak permutation 23 s. 0 errors. |
| `lake env lean Axioms.lean` (`#print axioms` on the key theorems) | 20 s. `Gadgets.SHA256.CompressBlock.soundness/completeness`, `Gadgets.Addition32.soundness`, `InductiveTable.table_soundness` depend only on `[propext, Classical.choice, Quot.sound]` (**no `sorryAx`**). `Tables.BLAKE3.ProcessBlocksInductive.soundness/completeness` additionally depend on one `bv_decide` native axiom (`Gadgets.ByteDecomposition.Theorems.soundness._native.bv_decide.ax_1_6`, i.e. trust in the compiled Lean code + SAT solver LRAT check used by `bv_decide`), still no `sorryAx`. |
| ArkLib clone `7653a901ed466c88a2e61a3075011b73d7bb2316` (2026-09-26), `leanprover/lean4:v4.34.0`, deps: mathlib v4.34.0, VCVio `d7089e46`, PolyFun `3710d71b`, CompPoly `v4.34.0-patch2`, doc-gen4, cslib | shallow clone 17 MB |
| ArkLib `elan toolchain install v4.34.0` + `lake exe cache get` (8906 files; two attempts because the first ran without the toolchain's `lake` on PATH) | 773 s + 361 s; `.lake` = 3.1 GB |
| `lake build ArkLib.ProofSystem.Sumcheck.Interaction.Soundness` (`taskset -c 0-5 nice -n 10`, 20-min cap) | **Success, 268 s wall**, 3301 Lake jobs, 213 modules compiled from source (67 `ArkLib.*`, the rest VCVio/PolyFun/CompPoly), 0 errors. `#print axioms` on `Sumcheck.Interaction.SingleRound.executeCommitted_soundness` and `executeRandomCommitment_soundness`: `[propext, Classical.choice, Quot.sound]` only (sorry-free, as the status doc claims). |
| Aeneas clone `b86120db3183b0107eb5f2637b11c424cd06ef1c` (2026-09-25); `nix build github:AeneasVerif/aeneas` | 1st attempt killed at the 900 s cap (nix eval-cache lock contention + load 50); **2nd attempt (`--max-jobs 2 --cores 6`, 25-min cap) succeeded in 551 s**: builds 4 derivations (charon 0.1.273, charon wrapper, ocaml domainslib, `ocaml5.2.1-aeneas-0.1.0`); result has `bin/aeneas` (57 MB, `aeneas -version` → `b86120d`) and `bin/charon` symlink. Not exercised on a crate (needs Charon's pinned Rust nightly). |

Notes: this Lake (5.0.0 / Lean 4.33) has no `-j` flag; I pinned the build to 6 CPUs with
`taskset -c 0-5` instead. The bench.lock was never held.

## 2. Clean inventory

Repo: 183 `.lean` files, 48,750 LOC under `Clean/`. `sorry` count: **11, all in
`Clean/Utils/Test/TestCircuitProofStart.lean`** (tactic unit tests), i.e. **the gadget and
framework code is sorry-free**. Website: clean.zksecurity.xyz; funded by the Verified-zkEVM grant.

### 2.1 Framework layers

| Layer | Files | What it is |
|---|---|---|
| `Clean/Circuit/*` | 25 files | Monadic circuit DSL: `witness`, `assert (e = 0)`, `lookup`, `interact` (channel), `subcircuit`. `FormalCircuit` = main + Assumptions + Spec + soundness + completeness. Witness-generation IR (`WitnessIR`) for compiling witgen to backends. |
| `Clean/Table/*` | 6 files | Older "classic AIR" layer: `InductiveTable` (row i+1 = step(row i, input i)), with `table_soundness` lifting per-row soundness to whole-trace soundness. |
| `Clean/Air/*` | 7 files | New "flat AIR" layer: one-row components + **channels (LogUp-style balanced multisets)** for lookups / VM state / public IO. `FormalEnsemble` bundles ensemble + Spec + soundness proof (**completeness is TODO**). Ordered-channel and VM soundness theorems. |
| `Clean/Gadgets/*` | 12 dirs | See 2.2. |
| `Clean/Specs/*` | SHA256, BLAKE3, Keccak256, Poseidon(+Optimized, Constants) | Executable specs on `ℕ`/`UInt32`. |
| `Clean/Tables/*` | Fibonacci8/32, Addition8, KeccakInductive, **BLAKE3/ProcessBlocksInductive** | Multi-row examples. |
| `Clean/Backends/Circom` (Lean) + `backends/plonky3` (Rust) | | Export paths, see 2.5. |

### 2.2 Gadget inventory (all `FormalCircuit`, sorry-free)

| Gadget dir | LOC | Field side-condition | Notes |
|---|---|---|---|
| `Addition8`, `Addition32` (+`Addition32Full` with carry) | 371 / 355 | `p > 512` | U32 = 4 bytes (`U32 F`, `Normalized` = each limb < 256), add mod 2^32 with byte carries |
| `Xor` (Xor32/Xor64 via `ByteXorTable`) | 274 | `p > 2^16+2^8` | byte-XOR **lookup table** (65536 rows) |
| `And`, `Or`, `Not` | 233/255/73 | `p > 512` | byte lookup tables |
| `Rotation32`, `Rotation64` (bits + bytes variants) | 381 / 410 | `p > 512` | rotate by constant |
| `ByteDecomposition`, `ByteLookup`, `Bits`, `Boolean`, `IsZero`, `IsEqual`, `Equality`, `Conditional` | small | | |
| **`Keccak`** (Theta, Rho/Pi, Chi, Iota, Round, `Permutation` = Keccak-f[1600], `AbsorbBlock`) | 856 | `p > 2^16+2^8` | Spec: `out.value = keccakPermutation state.value` |
| **`BLAKE3`** (`BLAKE3G`, `Round`, `Permute`, `ApplyRounds` (7 rounds), `FinalStateUpdate`, `Compress`, `FinalizeChunk`) | 1,416 | `p > 2^16+2^8` | U32-as-4-bytes representation, byte lookups. **BabyBear/M31-compatible.** |
| **`SHA256`** (`BitwiseOps`, `Xor32`, `And32`, `Ch32`, `Maj32`, `LowerSigma0/1`, `UpperSigma0/1`, `Add32`, `SHA256Round`, `SHA256Schedule`, `SHA256Compress`) | 3,657 | **`p > 2^33`** | **Bit-level** (each word = `fields 32` booleans, LSB first), "R1CS structure, no lookup tables". 455 witnesses per round; 227 per schedule word. Introduces its own prime `pSHA256 = 2^34 + 25` in `Utils/Primes.lean`. |

No Poseidon2 gadget (spec only). No BLAKE2.

### 2.3 Specs

* `Clean/Specs/SHA256.lean` (261 LOC): FIPS 180-4 on `ℕ` words: `sha256Round`, `messageSchedule`,
  `sha256Compress`, `compressBlock` (Davies-Meyer), `pad`, `sha256 : List ℕ → Vector ℕ 8`
  (**full hash incl. padding**). Five `example : sha256 [...] = #v[...] := by native_decide`
  test vectors ("", "abc", "a", 2 longer ones).
* `Clean/Specs/BLAKE3.lean` (368 LOC): `g`, `round`, `permute`, `applyRounds`, `compress`,
  `ChunkState`, `updateChunk`, `finalizeChunk`, `processChunk`. **Single-chunk only (≤ 1024 B):
  no parent/tree nodes, no root output beyond one chunk.** For the waddup use case (F is
  MiB-scale) the BLAKE3 spec must be extended with the tree (parent CVs, `leftLen = 2^floor(log2(n-1))`
  split, ROOT flag). `argumentcomputer/Blake3.lean` (`Blake3/Pure.lean`, Lean v4.34.0, 155 LOC,
  commit `c32002ee` 2026-09-23) has exactly that (`parentOutput`, `subtree`, `digest`) plus a
  `Proofs.lean` with ~30 structural theorems; it is a good candidate reference spec to prove
  Clean's `processChunk` equivalent to.
* `Clean/Specs/Keccak256.lean`, `Poseidon*.lean`: not needed here.

### 2.4 The soundness / completeness notions (exact text)

`Clean/Circuit/Formal.lean`:

```lean
def Soundness (F : Type) [FiniteField F] (main : Var Input F → Circuit F (Var Output F))
    [elaborated : ElaboratedCircuit F Input Output main]
    (Assumptions : Input F → Prop) (Spec : Input F → Output F → Prop) :=
  -- for all environments that determine witness assignments
  ∀ offset : ℕ, ∀ env : Environment F,
  -- for all inputs that satisfy the assumptions
  ∀ input_var : Var Input F, ∀ input : Input F, eval env input_var = input →
  Assumptions input →
  -- if the constraints hold
  ConstraintsHold.Soundness env (main input_var |>.operations offset) →
  -- the spec holds on the input and output
  let output := eval env (ElaboratedCircuit.output main input_var offset)
  Spec input output ∧
  Operations.Requirements env (main input_var |>.operations offset)

def Completeness (F : Type) [FiniteField F] (main : Var Input F → Circuit F (Var Output F))
    (Assumptions : Input F → Prop) :=
  -- for all prover environments which use the default witness generators for local variables
  ∀ offset : ℕ, ∀ env : ProverEnvironment F, ∀ input_var : Var Input F,
  env.UsesLocalWitnessesCompleteness offset (main input_var |>.operations offset) →
  ∀ input : Input F, eval env input_var = input →
  Assumptions input →
  ConstraintsHold.Completeness env (main input_var |>.operations offset)
```

`ConstraintsHold` (`Clean/Circuit/Operations.lean`) is literally "every `assert e` has
`env e = 0`, every `lookup l` has its row in the table, every channel interaction's guarantee
holds; subcircuits are replaced by `Assumptions → Spec`" (with `Foundations.lean` proving that
this modular form is justified by the flat form). Lookup soundness is **assumed** as an
axiom of the model (`l.Soundness env` = the looked-up row is in the table); Clean does not
prove the LogUp/lookup argument itself (that is on their roadmap: "prove lookup protocols
end to end", "compose with ArkLib").

Field: `F p := ZMod p` with `[Fact p.Prime]`; each gadget adds `[Fact (p > N)]`.
Known primes: `p1009`, `pBabybear = 15·2^27+1`, `pMersenne = 2^31-1`, `pSHA256 = 2^34+25`.
No Goldilocks/KoalaBear instance declared (trivial to add with `native_decide`).

Table-level (multi-row) soundness, `Clean/Table/Inductive.lean`:

```lean
theorem table_soundness (table : InductiveTable F State Input) (input output : State F)
  (N : ℕ+) (trace : TraceOfLength F (ProvablePair State Input) N) (env : TableEnvironments F) :
  table.Spec input [] 0 rfl input env.data →
  TableConstraintsHold (table.tableConstraints input output) trace env →
    table.Spec input (traceInputs trace.tail) (N-1) (traceInputs_length trace.tail) output env.data
```

Ensemble-level (flat AIR + channels), `Clean/Air/FlatEnsemble.lean`:

```lean
def Statement (ens : Ensemble F PublicIO) (publicInput : PublicIO F) : Prop :=
  ∃ witness : EnsembleWitness ens,
    witness.publicInput = publicInput ∧ witness.Constraints ∧ witness.BalancedChannels
def Soundness (ens) (Assumptions Spec : PublicIO F → Prop) : Prop :=
  ∀ publicInput, Assumptions publicInput → ens.Statement publicInput → Spec publicInput
structure FormalEnsemble ... where
  ensemble : Ensemble F PublicIO
  Assumptions : PublicIO F → Prop := fun _ => True
  Spec : PublicIO F → Prop
  soundness : ensemble.Soundness Assumptions Spec
  -- completeness : ensemble.Completeness Assumptions Spec   -- TODO in upstream
```

### 2.5 Representative gadgets

**(a) `Gadgets.Addition32` (`Clean/Gadgets/Addition32/Addition32.lean`, 54 LOC + 159 Full + 142 Theorems)**

```lean
def Assumptions (input : Inputs (F p)) := let ⟨x, y⟩ := input; x.Normalized ∧ y.Normalized
def Spec (input : Inputs (F p)) (z : U32 (F p)) :=
  let ⟨x, y⟩ := input; z.value = (x.value + y.value) % 2^32 ∧ z.Normalized
theorem soundness : Soundness (F p) main Assumptions Spec
theorem completeness : Completeness (F p) main Assumptions
def circuit : FormalCircuit (F p) Inputs U32 where main elaborated Assumptions Spec soundness completeness
```
Field condition `p > 512`. Sorry-free. Proof is ~4 lines of `simp_all [circuit_norm, ...]`
on top of `Addition32Full`.

**(b) `Gadgets.SHA256.CompressBlock` (`Clean/Gadgets/SHA256/SHA256Compress.lean`, 490 LOC; whole SHA256 dir 3,657 LOC)**

```lean
variable {p : ℕ} [Fact p.Prime] [Fact (p > 2^33)]
structure Inputs (F : Type) where
  state : SHA256State F      -- Vector (fields 32) 8, bit-level
  block : SHA256Block F      -- Vector (fields 32) 16
def main (input : Var Inputs (F p)) : Circuit (F p) (Var SHA256State (F p)) := do
  let w ← MessageSchedule.circuit input.block
  let state' ← SHA256Rounds.circuit ⟨input.state, w⟩
  Circuit.mapFinRange 8 fun (i : Fin 8) => Add32.circuit ⟨input.state[i], state'[i]⟩
def Assumptions (input : Inputs (F p)) : Prop :=
  (∀ i : Fin 8, Normalized input.state[i]) ∧ (∀ i : Fin 16, Normalized input.block[i])
def Spec (input : Inputs (F p)) (out : SHA256State (F p)) : Prop :=
  out.map valueBits =
    Specs.SHA256.compressBlock (input.state.map valueBits) (input.block.map valueBits)
  ∧ ∀ i : Fin 8, Normalized out[i]
theorem soundness : Soundness (F p) main Assumptions Spec
theorem completeness : Completeness (F p) main Assumptions
def circuit : FormalCircuit (F p) Inputs SHA256State := { main, elaborated, Assumptions, Spec, soundness, completeness }
```
Sorry-free. Cost: 64 rounds × 455 witnesses + 48 schedule words × 227 witnesses ≈ 40k
witness cells per block, all boolean-constrained (this is an R1CS-style, lookup-free
encoding — expensive for an AIR prover; a byte-lookup version like the BLAKE3 one would be
~10× fewer cells). **Missing for waddup:** a multi-block table (padding, chaining over N
blocks, initial `H0`, final digest bytes). The spec `sha256 : List ℕ → Vector ℕ 8` exists,
but no `InductiveTable` for SHA-256 exists (only for BLAKE3).

**(c) `Gadgets.BLAKE3.Compress` + `Tables.BLAKE3.ProcessBlocksInductive` (56 + 637 + 422 LOC)**

```lean
-- Compress: Spec output = Specs.BLAKE3.compress cv block counter blockLen flags  (values of U32 limbs)
-- Table (17 rows: up to 16 blocks + final):
def Spec (initialState : ProcessBlocksState (F p)) (inputs : List (BlockInput (F p))) i
  (_ : inputs.length = i) (state : ProcessBlocksState (F p)) (_ : ProverData (F p)) :=
    inputs.length < 2^32 →
    initialState.Normalized ∧ (∀ input ∈ inputs, input.Normalized) ∧
    let validBlocks := inputs |>.filter (·.block_exists = 1)
    let blockWords := validBlocks.map (fun b => b.block_data.map (·.value))
    let finalState := processBlocksWords initialState.toChunkState blockWords
    state.toChunkState.blocks_compressed < inputs.length ∧
    state.toChunkState = finalState ∧ state.Normalized
lemma soundness : InductiveTable.Soundness (F p) ProcessBlocksState BlockInput Spec step
lemma completeness : InductiveTable.Completeness (F p) ProcessBlocksState BlockInput InputAssumptions InitialStateAssumptions Spec step
def table : InductiveTable (F p) ProcessBlocksState BlockInput
```
Field `p > 2^32` for the table (`p > 2^16+2^8` for the gadgets). Sorry-free. This is the
closest existing artifact to "a verified AIR that hashes a message": one 1 KiB chunk.

### 2.6 Connection to real provers

* `backends/plonky3` (Rust, 12 files): imports a JSON dump of a Clean circuit
  (`Clean/Circuit/Json.lean`, `WitnessExport.lean`) into a `p3_uni_stark` AIR with LogUp
  lookups, BabyBear, `create_test_fri_params(challenge_mmcs, 2)` (**test/insecure FRI
  params**). README: "THIS IS A NOT-PRODUCTION-READY POC". Fixtures: Fibonacci and FemtoCairo
  only. Roadmap item: "generate constraint evaluation Rust code".
* `Clean/Backends/Circom`: Lean-written compiler to Circom-ABI WASM witness generator +
  `.r1cs` for snarkjs/Groth16 (BN254). Compiled *from* the same Lean definitions, but the
  compiler itself is not verified (tested against Lean ground truth).
* Nothing in the repo proves anything about a prover or verifier implementation; the trust
  boundary is: Lean-certified constraint list ⟶ (unverified) exporter ⟶ (unverified) STARK/SNARK.

## 3. ArkLib inventory

Repo: 920 `.lean` files, **279,656 LOC** (of which `Data/CodingTheory` = 126,768 and
`ToMathlib` = 24,001). **`sorry`: 225 occurrences in 76 files.** README status
("Active formalizations, last updated 7 Aug 2025"): Sum-check, Spartan, Merkle trees, FRI +
coding theory, STIR/WHIR, Binius — "The library is currently in development."
`docs/design/00-current-status.md` (2026-09-26): "No declaration under `ArkLib/Interaction/`
or `ArkLib/ProofSystem/Sumcheck/Interaction/` uses `sorry`" (I confirmed: 0 sorries in
12,679 LOC there).

| Component | Where | LOC | sorry | Status |
|---|---|---|---|---|
| IOR core (ProtocolSpec, prover/verifier, composition, lifting) | `OracleReduction/` | 22,909 | 59 | Definitions solid; `Security/Implications.lean` 12 sorries, `LiftContext` 10, `Composition` 7 |
| Security notions (soundness, RBR knowledge soundness, special soundness, state restoration, rewinding) | `OracleReduction/Security/` | 9,341 | 15 | |
| **Sum-check (new typed "Interaction" layer)** | `ProofSystem/Sumcheck/Interaction/` | part of 12,679 | **0** | `executeCommitted_soundness : Pr[accept ∧ claim false] ≤ deg/|F|` per round; multivariate transport; perfect completeness |
| Sum-check (older Spec) | `ProofSystem/Sumcheck/Spec/SingleRound.lean` | 6,228 total | 14 | RBR knowledge soundness theorems are `:= sorry`; several `oracleReduction.*` defs are `sorry` |
| **FRI** | `ProofSystem/Fri/` | 1,205 | 4 | `RoundConsistency.lean` has only a completeness lemma; in `Spec/SingleRound.lean` the folding/final **oracle reductions themselves are `:= sorry`** (4×). No soundness theorem. |
| Batched FRI | `ProofSystem/BatchedFri/` | 1,169 | 6 | `Security.lean` 3 sorries |
| **STIR** | `ProofSystem/Stir/` | 1,370 | 4 | `stir_rbr_soundness` (Lemma 5.4 of STIR) is stated in full but proved with `sorry` (2 in `MainThm.lean`) |
| WHIR | — | 0 | — | not present despite README |
| Binius / ring switching | `ProofSystem/Binius`, `RingSwitching` | 6,769 + 3,407 | 32 + 18 | active WIP |
| Spartan, Plonk | 430 / 46 | 8 / 0 | skeletons |
| **Merkle trees** | — | — | — | **No Merkle-tree file in ArkLib**; only mentions in FRI comments and `BCS/Basic.lean`. (VCVio has a "shared-ROM Merkle extraction" primitive per the status doc.) |
| **BCS transform** | `OracleReduction/BCS/Basic.lean` | 84 | 2 | Doc-comment + **commented-out** `BCSTransform` definitions. Not usable. |
| **Fiat-Shamir** | `OracleReduction/FiatShamir/` | 1,767 | 18 | Transform defined; `fiatShamir_completeness := sorry`; duplex-sponge security files have 12 sorries. No FS soundness theorem. |
| Commitments (KZG, Hachi) | `Commitments/Functional/` | 20,405 | 21 | KZG binding etc.; no hash-based vector commitment |
| Coding theory (RS codes, proximity gaps BCIKS20, list decoding) | `Data/CodingTheory/` | 126,768 | 32 | large; proximity-gap results partially sorried |
| Hash | `Data/Hash/` | 1,693 | 0 | Poseidon2 over KoalaBear (executable spec), duplex sponge, domain separator. **No SHA-256/BLAKE.** |
| **End-to-end STARK / AIR soundness** | — | — | — | **Absent.** `grep -i stark\|\bAIR\b` hits only one unrelated comment. |

ArkLib ships its own axiom/sorry sweep (`scripts/AxiomSweep.lean`) and a blueprint; the
project is honest about status. The abstraction (IOR + oracle interfaces) is the right one to
eventually state "FRI-based STARK over AIR `A` is RBR-sound with error ε", but nothing near
that is proved today, and Merkle/BCS/FS (the parts a concrete verifier binary actually
executes) are the least developed.

Build: ArkLib needs Lean v4.34.0 + a second Mathlib cache (3.1 GB) + VCVio/PolyFun/CompPoly
compiled from source. The sum-check `Interaction.Soundness` module and its 212 dependencies
built in 268 s on 6 cores under nice; its two soundness theorems are kernel-checked with
only the three standard axioms. A full `lake build ArkLib` was not attempted (280k LOC).

## 4. Existing machine-checked SHA-256 / BLAKE3 specs in Lean 4

| Source | What | Verified? |
|---|---|---|
| **Clean `Clean/Specs/SHA256.lean`** | Full SHA-256 (padding, N blocks) on `ℕ` words, 261 LOC | 5 `native_decide` test vectors; **gadget soundness proofs target it** |
| **Clean `Clean/Specs/BLAKE3.lean`** | BLAKE3 compress + single chunk (≤1024 B), 368 LOC | `native_decide` unit tests vs. Python-derived values; gadget proofs target it; **no tree hashing** |
| `argumentcomputer/Blake3.lean` `Blake3/Pure.lean` (+ C/Rust FFI) | Full BLAKE3 `digest : List UInt8 → Digest` incl. tree (`parentOutput`, `subtree`, `leftLen`), Lean v4.34.0 | `Proofs.lean`: ~30 theorems (byte/word round trips, schedule permutation, `leftLen` uniqueness, chunk-loop framing, no `sorry` seen); equivalence to FFI is tested, not proved |
| `argumentcomputer/ix` `Ix/Aiur/Library/Blake3.lean`, `Ix/IxVM/Sha256.lean` | zkVM-flavoured implementations | unknown |
| `ethereum/cryptography-specs` `EthCryptographySpecs/Bls/Sha256.lean` (Lean v4.29.1) | 115-LOC `UInt32` transcription of FIPS 180-4 (`hash : ByteArray → ByteArray`) | executable spec, EF-maintained; `Xmss/Blake2s.lean` also there |
| `ethereum/ssz-specs`, `powdr-labs/evm-semantics`, `lambdaclass/concrete`, `tachyon-zcash/ragu`, `Verilean/lean-tea`, … | further SHA-256 transcriptions found by code search | executable specs only |
| `Verified-zkEVM/zkLean` (Galois) | separate circuit DSL with Std.Do Hoare triples; has a SHA-3 circuit + spec + proof | different framework than Clean; Keccak not SHA-256 |
| Mathlib | no hash functions | — |

Conclusion: reuse Clean's SHA-256 spec directly (it is what the gadgets are proved against);
for BLAKE3, extend Clean's spec with the tree layer and prove it equal to
`Blake3.Pure.digest` (or adopt the latter as the top-level reference), then check both against
the BLAKE3 test vectors by `native_decide`.

## 5. Aeneas (Rust → Lean)

Commit `b86120db` (2026-09-25); Charon pin `4bd5a29f`. README/docs (`docs/user/src/README.md`):

* Works on **safe** Rust; backends F*, Coq, HOL4, **Lean**. Standard flow:
  `charon cargo --preset=aeneas` → `.llbc` → `aeneas -backend lean` → Lean files importing the
  `Aeneas` Lean library (`backends/lean`, its own `lean-toolchain`).
* Stated limitations: **"loops: no nested loops for now"**, no closures/fn pointers (WIP),
  limited type parametricity with borrows, no nested borrows in signatures, no interior
  mutability, no concurrency.
* Supported and exercised in `tests/src` (120 test crates): `loops.rs` (33 loops),
  `slices.rs`, `arrays.rs`, `array_slice_index.rs`, `bitwise.rs`, `chunks_exact.rs`,
  `wrapping_add`/`wrapping_mul` (`demo.rs`, `target-features.rs`, `assert-cfg.rs`),
  `curve25519.rs`, `hashmap.rs`, AVL/BST. u32/u64 arithmetic lands in the `Result` monad
  (overflow ⇒ failure) with `core.num.U32.wrapping_*` modelled as total functions.
* `documentation/crypto-verification.md` describes the Microsoft SymCrypt ML-KEM verification
  (NTT, Montgomery/Barrett, compress/encode) done with this exact pipeline — evidence that a
  moderately sized, loop-heavy, bit-twiddling crate is within reach, at the cost of
  restructuring nested loops (extract inner loop into a function) and writing an "auxiliary
  spec" layer.
* Build: the flake builds cleanly from cache.nixos.org + 4 local derivations in ~9 min
  (see table in section 1); `aeneas` and `charon` binaries are usable from
  `/home/schlarpc/waddup-scratch/exp-lean/aeneas-build/result-aeneas/bin/`.

Relevance: a small Rust STARK **verifier** (Merkle path checks, FRI folding, constraint
evaluation over BabyBear) is exactly the shape Aeneas handles (no allocation-heavy generics
needed if written plainly); `Verified-zkEVM/rust-lean` already has `aeneas-FRI/` (fold_step,
fold_arity from Plonky3) and `hax-merkle/` (RISC Zero Merkle inclusion) as precedents.

## 6. Grounded assessment: how much of the waddup verification story exists?

Target theorem (informal): "if the verifier accepts (H1, H2, π) then ∃ F with
SHA-256(F) = H1 ∧ BLAKE3(F) = H2" (soundness, modulo hash/ROM assumptions), plus
"an honest prover always convinces the verifier" (completeness).

Decompose into layers and score each:

| Layer | Needed | Exists | Gap / effort |
|---|---|---|---|
| L0 Reference specs | SHA-256 (full), BLAKE3 (full tree) executable in Lean, test-vector checked | SHA-256: yes (Clean). BLAKE3: single chunk (Clean) / full (Blake3.lean) | Small: extend Clean BLAKE3 spec with parent/root nodes, prove = Blake3.Pure or check vectors. ~1-2 weeks. |
| L1 Per-block gadgets, sound & complete | SHA-256 compress, BLAKE3 compress, add/xor/rotate | **Yes, sorry-free in Clean** (SHA-256 bit-level, BLAKE3 byte-lookup) | If we want a *fast* AIR we would likely re-implement SHA-256 in the byte-lookup style (like Clean's BLAKE3) and re-prove: the BLAKE3 dir took ~1.4k LOC; expect ~2-4k LOC for a lookup-based SHA-256 and a few weeks of proof work. Also need a Goldilocks/KoalaBear `Fact` instance or the `p > 2^33` lift. |
| L2 Multi-row table: chaining N blocks, padding, both hashes over the *same* bytes, public H1/H2 | `InductiveTable` (classic AIR) or `Flat.Ensemble` + channels | BLAKE3 has a 17-row chunk table; SHA-256 has none; no "two hashes share input bytes" table | Medium: one table per hash + a channel/lookup tying the byte stream to both, plus a verifier circuit for public IO. Clean's `Air/` layer is designed for this but `FormalEnsemble` completeness is TODO upstream. ~1-2 months. |
| L3 Arithmetization export = what the Rust prover/verifier actually evaluate | Verified compiler from Clean ops to Rust/Plonky3 AIR, or Lean-side re-check | PoC JSON exporter + Rust importer, unverified, test FRI params | Either (a) trust a small exporter and diff-test (what Clean does), or (b) prove the exporter. (a) is days; (b) is months. |
| L4 Lookup argument (LogUp) soundness | Clean *assumes* lookups are sound | not proved in Clean; ArkLib has no LogUp | Would have to be written; ArkLib's sum-check layer helps. Months. |
| L5 STARK core: AIR → RS-codewords, quotient, FRI/STIR low-degree test, RBR soundness with concrete ε | ArkLib: sum-check sound (sorry-free), STIR theorem *stated*, FRI reductions `sorry`, proximity gaps partial | **Nothing usable end-to-end.** | Multi-person-year research-grade effort; this is the Verified-zkEVM programme's own goal and is not close. |
| L6 Merkle + Fiat-Shamir (BCS) → non-interactive; ROM assumptions | ArkLib: FS defined, completeness `sorry`, no soundness; BCS commented out; no Merkle | absent | Same as L5. |
| L7 Verified verifier binary (Rust) | Aeneas/hax translation of the Rust verifier + proof it implements the L5/L6 verifier | precedents for FRI fold and Merkle checks in `rust-lean` | Feasible for a small hand-written verifier (weeks-months) *if* the L5/L6 spec existed to prove against. |

So: **L0-L2 ("the circuit is a correct encoding of SHA-256 ∧ BLAKE3") is achievable with
Clean today** and roughly half of it is already written and sorry-free. **L4-L6 ("the STARK
proof system is sound") does not exist in Lean** for any proof system; ArkLib is the right
home but is years away. The honest claim we could make after a few months of work: "The
AIR constraints are machine-checked to admit a satisfying trace iff SHA-256(F)=H1 ∧
BLAKE3(F)=H2 (for a prime field with p > 2^33), and the Rust verifier's constraint
evaluator is differentially tested against the Lean-exported constraint list; the
FRI/Merkle/Fiat-Shamir layers rely on published (pen-and-paper) security proofs and on
Plonky3's implementation." That is a materially stronger statement than what SP1/RISC Zero
offer (whose ISA circuits are not machine-checked), but it is not "the ZK proof is proven
secure in Lean".

Caveats visible in code: Clean's Plonky3 backend uses `create_test_fri_params` (insecure,
for tests); Clean's SHA-256 is bit-level R1CS style (slow as an AIR); Clean's BLAKE3 spec is
one chunk only; ArkLib's FRI oracle reductions and STIR main theorem are `sorry`.

## 7. Files

* Clones + toolchains + caches: `/home/schlarpc/waddup-scratch/exp-lean/{clean,ArkLib,aeneas,aeneas-build,elan}` (8.3 GB total on /home, nothing on /tmp; delete when done)
* Build logs: `/home/schlarpc/waddup-scratch/exp-lean/build-targeted.log`, `build-arklib.log`, `aeneas-build/aeneas-build.log`
* Axiom-check scripts: `/home/schlarpc/waddup-scratch/exp-lean/Axioms.lean`, `ArkAxioms.lean`
* This report: `/tmp/claude-1000/-home-schlarpc-waddup/4dfd21de-19a0-4f53-b05a-3d664b78352f/scratchpad/exp-lean/RESULTS.md`
