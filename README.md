# waddup — ZK proof-of-concept benchmark

Benchmarks RISC Zero zkVM proving and verification for the waddup hash-binding proof:
"I know a file F such that `ALG(F) = H_content` and `BLAKE3(F) = H_swarm`."

Three guest programs:
- `--both` — SHA-256 (precompile) + BLAKE3 (software). This is the actual waddup proof.
- `--sha256` — SHA-256 only (precompile). Isolates precompile performance.
- `--sha3` — SHA3-256 only (keccak precompile). Comparison point.

All modes validate journal hashes against natively-computed values and report per-phase
cycle counts from inside the guest via `env::cycle_count()`.

## Prerequisites

- [Nix](https://nixos.org/) with flakes enabled
- RISC Zero toolchain (`rzup`)

## Setup

```
# Install rzup (RISC Zero toolchain manager)
curl -L https://risczero.com/install | bash

# Enter the nix dev shell (provides Rust, pkg-config, openssl)
nix develop

# Install the RISC Zero guest Rust toolchain
rzup install rust
```

## Run

```
nix develop --command cargo run --release -- [--both|--sha256|--sha3] <sizes...>
```

Sizes accept `k` and `m` suffixes (e.g. `10k`, `1m`). Defaults to `10k` if omitted.

```
# SHA-256 + BLAKE3 at 1k, 10k, 100k
nix develop --command cargo run --release -- --both 1k 10k 100k

# SHA-256 only at 10k
nix develop --command cargo run --release -- --sha256 10k

# SHA3-256 only at 10k
nix develop --command cargo run --release -- --sha3 10k
```

First build takes several minutes (compiles RISC Zero + guest programs to RISC-V).

## GPU backends

Set `RISC0_PROVER` to use a GPU backend:

```
RISC0_PROVER=cuda cargo run --release -- 10k    # NVIDIA (real speedup, ~4-7x)
RISC0_PROVER=metal cargo run --release -- 10k   # Apple (slower than CPU in practice)
```

CUDA requires `rzup install cpp`. Metal requires macOS.
