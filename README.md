# waddup — ZK proof-of-concept benchmark

Benchmarks RISC Zero and SP1 zkVM proving and verification for the waddup
hash-binding proof:
"I know a file F such that `ALG(F) = H_content` and `BLAKE3(F) = H_swarm`."

Three guest programs (mirrored across both provers):
- `--both` — SHA-256 (precompile) + BLAKE3 (software). This is the actual waddup proof.
- `--sha256` — SHA-256 only (precompile). Isolates precompile performance.
- `--sha3` — SHA3-256 only (keccak precompile). Comparison point.

All modes validate journal hashes against natively-computed values and report
per-phase cycle counts.

See [`RESEARCH.md`](RESEARCH.md) for the head-to-head comparison and the AMD-GPU
acceleration story, [`RESEARCH-PURPOSE-BUILT.md`](RESEARCH-PURPOSE-BUILT.md) for
purpose-built provers (Flock, Plonky3, Binius64, Stwo, GKR) and Lean
verification, and [`DESIGN-MIDSTATE.md`](DESIGN-MIDSTATE.md) for the adopted
non-ZK midstate-chain design that replaces the proof layer.

## Prerequisites

- [Nix](https://nixos.org/) with flakes enabled
- For RISC Zero: `rzup`
- For SP1: `sp1up` (and `nix-ld` enabled system-wide, or run via `cargo prove
  build --docker`)

## Setup

```
# RISC Zero toolchain
curl -L https://risczero.com/install | bash
nix develop --command rzup install rust

# SP1 toolchain (requires nix-ld on NixOS; sp1-specific shell adds rustup)
curl -L https://sp1up.succinct.xyz | bash
nix develop .#sp1 --command bash -c 'rustup default stable && ~/.sp1/bin/sp1up'
```

## Run

### RISC Zero

```
nix develop --command cargo run --release -- [--both|--sha256|--sha3] <sizes...>
```

### SP1

```
nix develop .#sp1 --command cargo run --release --manifest-path sp1/script/Cargo.toml \
    -- [--both|--sha256|--sha3] <sizes...>
```

Sizes accept `k` and `m` suffixes (e.g. `10k`, `1m`). Defaults to `10k` if
omitted. First build takes several minutes per prover.

### Full matrix

```
scripts/run-benches.sh           # writes RESULTS_RAW.md
```

## GPU backends

For RISC Zero, set `RISC0_PROVER`:

```
RISC0_PROVER=cuda cargo run --release -- 10k    # NVIDIA (real speedup, ~4-7x)
RISC0_PROVER=metal cargo run --release -- 10k   # Apple (slower than CPU in practice)
```

CUDA requires `rzup install cpp`. Metal requires macOS. **AMD/ROCm/HIP is not
supported by either RISC Zero or SP1 today** — see `RESEARCH.md` Part 1.
