# sp1-pr2615-bench

Benchmark code for the SHA-256 + BLAKE3 hash-binding proof on SP1 with the
[PR #2615 BLAKE3 precompile](https://github.com/succinctlabs/sp1/pull/2615)
ported forward to v6.2.2.

The path deps assume this directory has been copied into a patched SP1 v6.2.2
source tree.

## Reproduce

```bash
# 1. Clone SP1 at the v6.2.2 tag.
git clone --branch v6.2.2 https://github.com/succinctlabs/sp1.git /tmp/sp1-port
cd /tmp/sp1-port

# 2. Apply the patch series in this repo (cherry-picks PR #2615 onto v6.2.2
#    and adapts to the v6.2.2 SyscallContext + chip APIs).
git checkout -b port-blake3 v6.2.2
git am /path/to/waddup/sp1-pr2615-patches/*.patch

# 3. Copy this bench/ in.
cp -r /path/to/waddup/sp1-pr2615-bench bench

# 4. Add "bench/script" to /tmp/sp1-port/Cargo.toml [workspace.members].

# 5. Build (needs the sp1 dev shell from waddup's flake.nix for libclang
#    and rustup).
nix develop /path/to/waddup#sp1 --command \
    cargo build --release --manifest-path bench/script/Cargo.toml

# 6. Run.
RUST_LOG=warn ./target/release/waddup-bench 1k 10k 100k
```

## What's in here

- `program/` — guest crate. Calls `syscall_blake3_compress_inner` via
  `sp1_lib`, wraps it with BLAKE3 chunk + tree-mode bookkeeping
  (`blake3_precompile.rs`), and runs SHA-256 via SP1's existing precompile
  (`sha256_precompile.rs`).
- `script/` — host driver using `sp1-sdk`. Mirrors the CLI of the main
  RISC Zero and SP1 benchmarks in this repo.

See the parent repo's `RESEARCH.md` § "BLAKE3 precompile (SP1 PR #2615) —
ported forward and measured" for the measurements and the soundness caveats.
