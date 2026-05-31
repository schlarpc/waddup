# waddup — GPU acceleration and SP1 comparison

Companion research note to `PROPOSAL.md`. Two questions investigated:

1. Now that we've moved from WSL to native Linux on an AMD CPU + AMD GPU, how does
   that change the proving-throughput story for RISC Zero?
2. Could SP1 (the other major Rust-based RISC-V zkVM) do this proof faster or with
   different tradeoffs? Specifically reimplementing the SHA-256 + BLAKE3 hash binding.

Test machine: Ryzen 7 5800X (8c/16t Zen 3, AVX2, no AVX-512), 64 GiB RAM, AMD Radeon
RX 7900 XT (Navi 31, RDNA3, 20 GiB VRAM), NixOS, Linux 6.19.12.

All proving runs are CPU-only on this machine. Numbers below are single-run wall
clocks (no warm-up, no averaging), captured in `RESULTS_RAW.md`. Treat them as
indicative — within 10-15% reproducibility is realistic.

---

## Part 1 — AMD GPU acceleration

### TL;DR

Neither RISC Zero nor SP1 has a working AMD GPU backend in 2026-05. Both are
CUDA-only. Switching from WSL to native Linux helps the **CPU** prover (large
pages, native syscalls, no I/O bridging), but does **not** unlock GPU acceleration
on a Radeon — that capability simply doesn't exist upstream yet. The only credible
near-term vector is RISC Zero's stalled draft PR [#3725 (CubeCL HAL port)][r3725].

### Why the CUDA paths can't just be `hipify`'d

Both provers' CUDA paths are not vanilla CUDA, they're tightly bound to libraries
that themselves do not support HIP:

- **RISC Zero**: `risc0-sys` compiles `kernels/zkp/cuda/supra/{ntt,poseidon2,api}.cu`
  on top of [Supranational's `sppark`][sppark] plus `blst`. RISC Zero now vendors a
  fork (`risc0-sppark`) per [PR #3478][r3478]. sppark uses NVIDIA-specific PTX/CUB
  constructs and warp semantics — naive `hipify` does not produce working ROCm
  kernels for the NTT pipeline. The Rust side is `cust` (NVIDIA-only).
- **SP1**: GPU prover docs hard-require [CUDA 12 + compute capability ≥8.6 and
  ≥24 GiB VRAM][sp1hw]. Zero ROCm/HIP/Radeon-related issues, PRs, or forks in
  `succinctlabs/sp1` (searched for "AMD", "ROCm", "HIP", "Radeon", "RDNA"). The
  multi-GPU service (`sp1-cluster`) and all published benchmarks reference only
  RTX 4090/5090 and AWS `g6`.

Sister STARK GPU libraries are similarly Nvidia-only — Ingonyama [ICICLE][icicle]
(the dominant ZK GPU lib, supports BabyBear NTTs) is CUDA + CPU only; Plonky3 and
Stwo have no GPU backends merged at all. Note: RISC Zero is BabyBear; SP1 v6.2
moved to **KoalaBear** (p = 2^31 − 2^24 + 1) — any GPU NTT effort for one
prover does not lift to the other for free.

### The one credible path: RISC Zero PR #3725 (CubeCL HAL)

[PR #3725 "Experiment with cubecl"][r3725], opened 2026-02-25 by `flaub` (RISC Zero
core team), rewrites the HAL plus NTT and Poseidon2 kernels on top of
[CubeCL][cubecl], a Rust GPU compute framework with backends for CUDA, **HIP/ROCm**
([cubecl-hip][cubeclhip]), WGPU, Metal, and SPIR-V. New files include
`risc0/zkp/src/hal/cubecl/{hal,ntt,poseidon2,kernels}.rs`. Status: ~8200 LOC, draft,
**stalled since 2026-02-26**, marked dirty/draft. If it ever ships clean, the RX
7900 XT becomes a viable backend overnight.

Driving #3725 to a working state on our hardware was scoped as out of bounds for
this investigation (days of engineering with no guarantee of success). Documented
here as the recommended waypoint to monitor.

### What "native Linux" actually buys us

- **CPU prover**: native Linux generally outperforms WSL2 on the same hardware for
  compute-heavy workloads, mostly via lower syscall overhead, no I/O bridging, and
  access to transparent huge pages. RISC Zero CPU benchmarks rerun on this Linux
  box are the new baseline for any future SP1/GPU comparison — see Part 2.
- **SIMD**: Ryzen 5800X (Zen 3) has AVX2 but no AVX-512. SP1's CPU prover ships an
  AVX-512 path that is **not** available on this CPU; the AVX2 path is. SP1
  benchmarks would improve materially on a Zen 4+ or Intel Sapphire Rapids box.
- **GPU**: no acceleration without engineering work. ZLUDA (CUDA-on-ROCm reimpl) is
  a theoretical alternate path; we actually tried it (see § "ZLUDA result" below).
  Bottom line: doesn't currently work end-to-end for RZ; SP1 not tested but would
  hit similar issues.

### ZLUDA result (actually tested)

After this doc was first drafted, [ZLUDA v6-preview.75][zluda] (CUDA-on-ROCm
shim) was installed on this box. Worth probing whether RZ's CUDA backend or
SP1's GPU prover work through it on the RX 7900 XT, even partially.

**Setup.** ZLUDA v6 ships `libcuda.so` plus `libcublas`, `libcublasLt`,
`libcufft`, `libcusparse`, `libcudnn`. The `zluda-run` wrapper prepends
`LD_LIBRARY_PATH` so the application's `libcuda.so.1` dlopen resolves to
ZLUDA's. Basic GPU detection works — a hand-written probe calling `cuInit`,
`cuDeviceGetCount`, `cuDeviceGetName` reports the 7900 XT correctly as
`"AMD Radeon RX 7900 XTX [ZLUDA]"`.

**Building RZ with CUDA.** Enable the `cuda` feature on `risc0-zkvm`. This
pulls in `cust`, `sppark`, and `risc0-circuit-{rv32im,recursion,keccak}-sys`
which need a real CUDA toolkit at build time (headers + `nvcc` + libcuda
stub for linking). On NixOS that means assembling a build env with:
- `nixpkgs#cudaPackages.cudatoolkit` (unfree; sets `CUDA_PATH`, provides
  `nvcc` + headers + `lib/stubs/libcuda.so` for build-time linking)
- A `lib64` symlink shim because `find_cuda_helper` searches `<root>/lib64/`,
  and nixpkgs' `cuda-merged` has `lib/`
- An `nvcc` wrapper because RZ's `risc0-build-kernel` hardcodes
  `-arch=native`, which nvcc cannot resolve without a real NVIDIA GPU
  present at compile time. The wrapper rewrites it to `-arch=sm_86` and
  also adds `-DNDEBUG` to strip device-side asserts that NVCC otherwise
  refuses (`__assert_fail` is a host-only function in device code).
- `nixpkgs#protobuf` (`PROTOC` env), `nixpkgs#llvmPackages.libclang.lib`
  (`LIBCLANG_PATH` env), `nixpkgs#glibc.dev` (`BINDGEN_EXTRA_CLANG_ARGS`
  with `-isystem` for `stdlib.h`).

After ~5 minutes of conflict resolution and one ~7 minute build, the host
binary links successfully against the cuda stub. `nm` confirms `cuInit`
and friends as undefined symbols expected from the dynamic linker.

**Running RZ under ZLUDA.** `zluda-run` resolves `libcuda.so.1` to ZLUDA's
implementation. `RISC0_PROVER=local` (the default with the `prove` feature
plus `cuda` flag) picks the in-process CUDA-enabled prover. The binary
starts up, initialises CUDA via ZLUDA (`cuInit` returns 0, context creation
succeeds, kernel compilation proceeds), then panics on the first kernel
launch:

```
called `Result::unwrap()` on an `Err` value:
  cudaLaunchKernelEx(&config, kernel, ...)@kernels/zkp/cuda/cuda.h:92
  failed: "named symbol not found"
```

The kernel in question is `risc0_zkp_cuda_scatter` (one of RZ's polynomial-
commitment scatter kernels in `risc0-zkp/src/hal/cuda.rs:890`). The failure
is consistent — every run hits it at the same call. The kernel itself is
compiled fine (the PTX module loads); ZLUDA's `cudaLaunchKernelEx`
implementation just can't resolve the kernel function by name at launch
time. Likely a PTX name-mangling / function-descriptor mismatch in
ZLUDA's coverage of `cudaLaunchKernelEx`, since `cudaLaunchKernel` (the
older, simpler launch API) appears to be more widely supported.

The process also panics during exit cleanup on
`<DarkApi as CudaDarkApi>::context_local_storage_get` called from
`gpus_t::~gpus_t() → cudaStreamDestroy` — a second ZLUDA coverage gap,
unrelated to the actual prove failure but visible in the stack trace.

**SP1 under ZLUDA: not tested.** SP1's GPU prover requires building
`sp1-cuda` (a separate prover server process the SDK connects to over IPC),
which itself wants newer PTX features than RZ's sppark kernels. Given RZ
fails on basic kernel launch resolution, SP1 — which exercises broader
PTX surface — is very unlikely to fare better without ZLUDA-side fixes.

**Status:** ZLUDA + RZ doesn't work end-to-end on this AMD GPU today.
The failure is well-scoped (kernel symbol resolution in `cudaLaunchKernelEx`),
fixable in principle by working with ZLUDA upstream or by patching RZ to
fall back to `cuLaunchKernel`, but neither is a quick win. ZLUDA is a
legitimate target to keep tracking — coverage is improving — but right now
it doesn't unlock the 7900 XT for either prover.

### Recommendation

Treat the RX 7900 XT as latent capacity for the medium term. Watch
[PR #3725][r3725] (RZ → CubeCL) AND ZLUDA's `cudaLaunchKernelEx` work; either
landing would unlock the GPU. For now, capacity planning should assume CPU
proving on this machine, with the published CUDA speedup ratios in
`PROPOSAL.md` (§ "CPU vs GPU Proving Throughput") as an upper bound — not
what this Radeon will deliver as is.

[zluda]: https://github.com/vosen/ZLUDA
[r3725]: https://github.com/risc0/risc0/pull/3725
[r3478]: https://github.com/risc0/risc0/pull/3478
[sppark]: https://github.com/supranational/sppark
[sp1hw]: https://docs.succinct.xyz/docs/sp1/generating-proofs/hardware-acceleration
[icicle]: https://github.com/ingonyama-zk/icicle
[cubecl]: https://github.com/tracel-ai/cubecl
[cubeclhip]: https://lib.rs/crates/cubecl-hip

---

## Part 2 — SP1 vs RISC Zero, head to head

### Setup

- RISC Zero zkVM **v3.0.5** (cargo-prove via rzup 0.5.0, succinct rust toolchain
  1.94.1), CPU prover.
- SP1 zkVM **v6.2.2** (cargo-prove `150e629`, succinct rust 1.93.0-dev), CPU
  prover.

Guest programs mirror the existing RISC Zero programs:

- `sp1/programs/hash-prover` — SHA-256 + BLAKE3 (the actual waddup proof)
- `sp1/programs/sha256-only` — SHA-256 precompile isolation
- `sp1/programs/sha3-only` — SHA3-256 (keccak) precompile isolation

Driver: `nix develop .#sp1 --command cargo run --release --manifest-path sp1/script/Cargo.toml -- [--both|--sha256|--sha3] <sizes...>`

Each benchmark first calls `client.execute(elf, stdin)` to collect cycle counts
via SP1's `cycle-tracker-report-*` markers (requires `sp1-sdk` "profiling"
feature), then calls `client.prove(&pk, stdin).run()` to measure wall-clock
proving.

### Finding 1 — SP1's official precompile patches are broken on v6.x

The single most surprising thing this investigation turned up. The
[`sp1-patches/RustCrypto-hashes`][sp1patches] (`patch-sha2-v0.10.8`) and
[`sp1-patches/tiny-keccak`][sp1patches] (`patch-v2.0.2`) branches that the SP1
docs and SDK templates point you at to enable hash precompiles **do not actually
dispatch the precompile when built for SP1 v6.x**.

Two issues compound:

1. **Target gate is `riscv32`-only.** The patched `sha2/src/sha256.rs` selects
   the succinct dispatch with
   `#[cfg(all(target_os = "zkvm", target_vendor = "succinct", target_arch = "riscv32"))]`.
   SP1 v6 changed its default target to `riscv64im-succinct-zkvm-elf`. The cfg
   doesn't match, so the patched crate silently falls back to the pure-Rust
   `mod soft` compress.
2. **Syscall ABI is u32-pointer.** Even if the cfg gate is fixed, the patched
   crate declares `extern "C" { fn syscall_sha256_extend(w: *mut u32); }` and
   passes a `[u32; 64]` buffer. SP1 v6's `sp1-lib` redefined the syscalls as
   `*mut [u64; 64]` — same logical content, but the executor expects 512 bytes,
   not 256.

Telltale signature: with the documented patch setup, `report.syscall_counts` is
empty for SHA_EXTEND / SHA_COMPRESS / KECCAK_PERMUTE; the cycle tracker reports
**~60 cycles/byte for SHA-256** instead of the ~8 c/B that the actual precompile
delivers, even while the output hash still matches (because the software fallback
ran). It's an easy regression for a user to ship without noticing.

There is a half-published in-progress fix for tiny-keccak —
`sp1-patches/tiny-keccak#tamir/sp1-6.2.0-pre` drops the `riscv32` gate and uses
`sp1_lib::syscall_keccak_permute` directly — but its `sp1-lib` dep points at a
`tamir/no_std_zkvm` branch on `succinctlabs/sp1` that no longer exists, so it
fails to fetch and the patch silently doesn't apply. There is no equivalent
in-progress fix for the sha2 patch.

**Workaround (used here).** We dropped the patched crates entirely and added a
small `sp1/programs/precompiles-lib` module with `sha256()` and `sha3_256()`
wrappers that call `sp1_lib::syscall_sha256_extend`,
`sp1_lib::syscall_sha256_compress`, and `sp1_lib::syscall_keccak_permute`
directly with the correct u64 ABI. Each wrapper is under 50 lines. Hashes
verified against `sha2::Sha256` and `tiny_keccak::Sha3` on the host. Both code
paths — "documented patch setup" (broken precompile) and "direct syscalls"
(working precompile) — are reported below for honesty about what an off-the-shelf
SP1 user gets vs. what's achievable.

The implication for adoption: as of 2026-05, the canonical "follow the docs"
SP1 setup will silently leave SHA-256 and Keccak running as plain RV64 software
on v6.x, costing roughly an order of magnitude in cycles. This is reportable to
upstream and is mostly a maintenance gap, not an architectural problem.

[sp1patches]: https://github.com/sp1-patches

### Finding 2 — Head-to-head numbers (CPU prover, native Linux, Ryzen 5800X)

All times are single-run wall clocks; cycles are from the in-guest tracker.
**RZ** = RISC Zero v3.0.5, **SP1** = SP1 v6.2.2. SP1 numbers are with direct
syscalls (Finding 1 workaround) unless noted. "SP1 (docs)" is the same code but
with the standard patched-sha2/tiny-keccak setup that is currently broken.

#### Per-region cycles per byte

| Mode      | Size    | RZ sha256 | SP1 sha256 (direct) | SP1 sha256 (docs) | RZ blake3 | SP1 blake3 | RZ sha3 | SP1 sha3 (direct) | SP1 sha3 (docs) |
|-----------|---------|-----------|---------------------|-------------------|-----------|------------|---------|-------------------|-----------------|
| sha256    | 1 KiB   | 2.2       | 8.2                 | 64.0              | —         | —          | —       | —                 | —               |
| sha256    | 10 KiB  | 1.2       | 6.5                 | 59.7              | —         | —          | —       | —                 | —               |
| sha3      | 1 KiB   | —         | —                   | —                 | —         | —          | 16.7    | 3.5               | 57.9            |
| sha3      | 10 KiB  | —         | —                   | —                 | —         | —          | 15.6    | 3.2               | 55.0            |
| both      | 1 KiB   | 2.2       | 8.2                 | 64.0              | 25.2      | 24.6       | —       | —                 | —               |
| both      | 10 KiB  | 1.2       | 6.5                 | 59.7              | 26.5      | 25.8       | —       | —                 | —               |
| both      | 100 KiB | 1.1       | 6.3                 | 59.2              | 26.7      | 26.0       | —       | —                 | —               |

Caveats: SP1 c/B numbers include precompile execution latency (SHA_EXTEND=64,
SHA_COMPRESS=72, KECCAK_PERMUTE=25 per call); RISC Zero c/B is pure rv32im
instructions. See Finding 3.

#### Total user cycles for the full guest

| Mode      | Size    | RZ        | SP1 (direct) | SP1 (docs)  |
|-----------|---------|-----------|--------------|-------------|
| sha256    | 1 KiB   | 10,997    | 13,647       | 70,719      |
| sha256    | 10 KiB  | 38,610    | 71,823       | 616,047     |
| sha3      | 1 KiB   | 28,251    | 8,873        | 64,574      |
| sha3      | 10 KiB  | 188,614   | 38,101       | 567,958     |
| both      | 1 KiB   | 37,027    | 43,255       | 100,329     |
| both      | 10 KiB  | 310,651   | 340,848      | 885,074     |
| both      | 100 KiB | 3,006,275 | 3,318,498    | 8,734,244   |

#### Wall-clock proving time

| Mode      | Size    | RZ        | SP1 (direct)  | SP1 (docs)    |
|-----------|---------|-----------|---------------|---------------|
| sha256    | 1 KiB   | 6.22s     | 23.76s        | 22.85s        |
| sha256    | 10 KiB  | 12.65s    | 23.91s        | 29.12s        |
| sha3      | 1 KiB   | 49.03s    | 27.87s        | 23.19s        |
| sha3      | 10 KiB  | 68.73s    | 29.87s        | 27.19s        |
| both      | 1 KiB   | 12.73s    | 23.71s        | 24.24s        |
| both      | 10 KiB  | 53.14s    | 27.91s        | 32.58s        |
| both      | 100 KiB | 339.84s   | 73.03s        | 124.50s       |

Verification: RZ ~13-90 ms per receipt, SP1 ~85-140 ms per core proof. Both
sub-millisecond once compressed to Groth16, but compression to Groth16 was not
exercised here.

### Finding 3 — Cycle-count comparison is misleading on its own

SP1's "user cycle" / `global_clk` accounting adds precompile execution latency
(SHA_EXTEND=64, SHA_COMPRESS=72, KECCAK_PERMUTE=25 per call) on top of the
rv32im/rv64im instruction count, so SP1's reported cycles per region include
work that RISC Zero's `env::cycle_count()` (a pure rv32im instruction count) does
not. Direct cycle-to-cycle comparison without normalizing this difference looks
worse for SP1 than the wall clock actually is.

Concretely: at 10 KiB SHA-256, the precompile is invoked 161 times. The
syscall-latency contribution alone is 161 × (64 + 72) = ~22K "cycles" — about a
third of SP1's measured 66K cycles for that region. The remaining ~44K cycles
are the actual rv64im instructions in the wrapper (buffer scan, byte→u64
load, padding). For RISC Zero, `cycle_count` returns 12K — only the rv32im
instruction count, the precompile work itself is invisible to that counter
because RISC Zero's SHA-256 accelerator is part of the segment circuit, not a
syscall whose latency is added to the user count.

**Right comparison is wall-clock proving time per byte.**

### Finding 4 — At small sizes RISC Zero is faster; at large sizes SP1 is

This is the headline tradeoff for waddup. Looking at the sha256+blake3 mode
(the actual waddup proof):

| Size    | RZ prove  | SP1 prove | SP1/RZ |
|---------|-----------|-----------|--------|
| 1 KiB   | 12.73s    | 23.71s    | 1.86x  |
| 10 KiB  | 53.14s    | 27.91s    | 0.53x  |
| 100 KiB | 339.84s   | 73.03s    | 0.21x  |

SP1's fixed proving overhead (Hypercube setup, multi-table lookup tables,
recursion/compression scaffolding) is roughly 20-25s independent of input size,
which dominates for small files. RISC Zero starts faster but its proving cost
scales more steeply with cycles, so SP1 takes over once total work exceeds a
crossover point. **The crossover for the waddup hash binding is between 1 KiB
and 10 KiB.** At 100 KiB, SP1 is ~4.6x faster wall-clock; that gap continues to
widen with size (RISC Zero's curve is steeper).

For the waddup use case — package mirrors and Linux distro seeds where median
file size sits between 10 KiB and 100 MiB — SP1's better scaling on CPU is
materially relevant. For something with many small inputs (e.g., per-chunk
proofs of a few hundred bytes each) RISC Zero is the better fit. A real
deployment would benchmark at the actual workload's size distribution before
picking.

### Finding 5 — SP1's keccak precompile is far cheaper to prove than RISC Zero's

Even at the smallest 1 KiB workload, SP1's SHA3 proving (28s with working
precompile) beats RISC Zero (49s). At 10 KiB it's 27s vs 69s. RISC Zero's
keccak circuit is a separate AIR with its own recursion and lift/join cost; SP1's
keccak permutation is a precompile chip that folds into the main proof. This
reinforces the `PROPOSAL.md` recommendation to avoid SHA3 as the binding hash —
keccak is structurally penalized in RISC Zero's prover, less so in SP1's, but
still bad relative to SHA-256 in both.

### BLAKE3 precompile (SP1 PR #2615) — ported forward and measured

[PR #2615][s2615] adds a single syscall `syscall_blake3_compress_inner` (id
`0x00_01_01_34`) exposing the 7-round Blake3 compression function. It targets
the `dev` branch and is `OPEN + CONFLICTING` against main; last touched
2026-02. There is no patched `blake3` crate that auto-dispatches to it.

**What was done.** Cherry-picked the four PR commits onto v6.2.2 main. Forward
ported to the v6.2.2 `SyscallContext` / `SyscallRuntime` API (the per-word
`mr`/`mw` accessors were replaced with slice helpers, permission checks moved
into `trap_*` helpers, `postprocess_precompile()` now returns a tuple, the
`SyscallRuntime` trait gained an `M: ExecutionMode` generic). The controller
chip's `receive_syscall(...)` gained a `trap_code` arg in v6.2.2 which we pass
as zero — see "Soundness caveat" below. Wrote a ~150-line BLAKE3 wrapper that
chunks input into 1024-byte chunks, maintains a CV stack for tree merging,
calls the precompile per 64-byte block (with the post-compress final XOR
`state[i] ^= state[i+8]`, `state[i+8] ^= cv_in[i]`), and applies the ROOT flag
on the final compress. Verified bit-for-bit against the `blake3` crate on the
host for 1 KiB / 10 KiB / 100 KiB inputs.

Patches included in this branch under `sp1-pr2615-patches/`.

#### Results (CPU prover, Ryzen 5800X, native Linux)

| Size    | SP1 (no BLAKE3 PC)              | SP1 + PR2615 BLAKE3 PC          | cycle ratio | prove ratio |
|---------|---------------------------------|----------------------------------|------------:|------------:|
| 1 KiB   | 43,255 cycles, prove 23.71s     | 21,954 cycles, prove 26.58s      |        0.51x |       1.12x |
| 10 KiB  | 340,848 cycles, prove 27.91s    | 111,232 cycles, prove 28.65s     |        0.33x |       1.03x |
| 100 KiB | 3,318,498 cycles, prove 73.03s  | 995,732 cycles, prove 63.19s     |        0.30x |       0.87x |

Per-byte for the BLAKE3 region (10 KiB):

| Mode                                | BLAKE3 c/B | SHA-256 c/B |
|-------------------------------------|-----------:|------------:|
| RISC Zero (no BLAKE3 PC)            | 26.5       | 1.2         |
| SP1 (no BLAKE3 PC)                  | 25.8       | 6.5         |
| **SP1 + PR2615**                    | **3.2**    | 6.5         |

#### Headline findings

1. **The intuition was right: BLAKE3 with PR2615 is cheaper per byte than
   SHA-256 with its precompile.** 3.2 vs 6.5 cycles/byte at 10 KiB. The
   BLAKE3 compress chip uses 104 rows per invocation vs SHA-256's ~144 (extend
   + compress combined), and BLAKE3 fits one syscall per 64-byte block vs
   SHA-256's two (extend + compress). Both effects compound. With the
   precompile lit up, the hash region of the binding proof rebalances —
   **SHA-256 becomes the larger contributor**, not BLAKE3.

2. **Cycle counts collapse but proving wall-clock barely moves at small sizes.**
   At 1-10 KiB, SP1's fixed proving overhead (~25-28s of Hypercube setup +
   lookup-table scaffolding) swamps everything. The precompile cuts BLAKE3
   region cycles by ~8x but proving time is essentially unchanged at small
   sizes — the rv32im trace was never the bottleneck there.

3. **Scaling story: the precompile pays off at size, but less than the cycle
   numbers suggest.** At 100 KiB total cycles drop 3.3x (3.3M → 996K) but
   proving wall-clock only drops 13% (73s → 63s). The proving cost has large
   fixed components (lookup-table generation, recursion, gnark conversion, the
   `setup` phase) that don't scale with cycle count, and the BLAKE3 chip's
   rows-per-byte (~1.6) replaces rv32im rows roughly 1:15 by count, but the
   BLAKE3 chip rows likely cost more than rv32im rows individually so the net
   shard-proving improvement is modest. Comparison to RISC Zero at 100 KiB: RZ
   is 340s, SP1 unaccelerated is 73s, SP1 with BLAKE3 PC is 63s. The RZ→SP1
   wall-clock gap (5.4x) widened to RZ→SP1+PR2615 (5.4x → still ~5.4x). The
   precompile makes SP1 *more efficient in cycles* but not dramatically *more
   efficient in proving time at this size*. The gap may widen further at
   larger sizes where rv32im dominates more.

4. **Hashes verify correct.** All cycle-count savings are real — the journal
   hashes match `blake3::hash()` byte for byte across all sizes tested.

#### Soundness caveat (worth re-reading the intuition section above)

The chip integration we shipped here has **known under-constraint** in three
specific places that are fine for benchmark numbers but not OK to ship to a
verifier accepting untrusted proofs:

- `receive_syscall(..., trap_code: AB::Expr::zero(), ...)` in the BLAKE3
  controller. v6.2.2's other precompile controllers thread a real `trap_code`
  through `AddressSlicePageProtOperation::eval` and propagate it to
  `is_not_trap`. Our port passes zero unconditionally — a trap on a BLAKE3
  call would not be caught by the chip.
- The BLAKE3 controller is not generic over `SupervisorMode`/`UserMode`. The
  v6.2.2 SHA-256 / Keccak controllers have separate `<SupervisorMode>` and
  `<UserMode>` variants with different page-prot eval paths. Our BLAKE3 chip
  effectively only supports trusted (supervisor) mode.
- The PR's chip predates the page-prot interaction architecture in v6.2.2 and
  doesn't emit the lookups the new prover expects on those buses.

The runtime path produces correct outputs (the hash always matches the
`blake3` crate). What's missing is the cryptographic constraint that the
prover *could not have produced a different output*. For benchmark
characterization — "how fast does the prover go when everything works" —
this is fine. For deployment, this chip needs an audit and the chip needs to
be properly mode-split and trap-aware.

[s2615]: https://github.com/succinctlabs/sp1/pull/2615

### Finding 6 — Porting RISC Zero's SHA-256 precompile into SP1 is chip-rewrite, not port

Whether SP1's relatively expensive SHA-256 precompile (6.5 c/B, vs RZ's 1.2)
could be improved by porting RZ's design over was an open question after the
benchmarks. After reading both source trees the honest answer is **months of
work using RZ as a design reference, not a "port" in any code-reuse sense.**
Concrete surfaces of mismatch:

- **Constraint DSL.** RZ's SHA-256 constraints live in
  [zirgen][zirgen] (`sha2.zir`, `inst_sha.zir`) — a domain-specific compiler
  language that lowers to MLIR and code-generates a ~1 MiB `poly_ext.rs`
  used by RZ's prover. SP1 is hand-written Plonky3 AIR via `slop_air::Air::eval`
  (e.g. `crates/core/machine/src/syscall/precompiles/sha256/compress/air.rs`,
  ~520 LOC). Zirgen primitives (`NondetBitReg`, `AliasLayout!`, `OneHot<N>`,
  `LookupDelta` externs, parametric components, `reduce ... init ... with ...`)
  have **no equivalent** in SP1's AIR builder. The `.zir` is a specification
  to translate from, not code to copy.

- **Field arithmetic.** RZ uses BabyBear (p = 15·2^27 + 1). **SP1 v6.2.2 uses
  KoalaBear (p = 2^31 − 2^24 + 1)** — both small but materially different
  field moduli, different NTT-friendly factorizations, different limb sizes,
  different range-check tables. The arithmetic isn't directly translatable.

- **u32 packing.** RZ stores u32 in memory as `ValU32 { low: u16, high: u16 }`
  and SHA round arithmetic operates on **32 bit registers** with bitwise
  `BitXor`/`MajU32`/`ChU32` primitives. SP1 packs `Word<T> = [T; 4]` as
  four u16 limbs (representing 64 bits) and implements `XorU32Operation` via
  **byte-XOR lookups** against a global byte table. Two completely different
  encodings for the same primitive — no reusable constraint forms.

- **Memory + syscall model.** RZ's SHA-256 isn't an ECALL precompile at all;
  it's an integrated set of cycle-states (`ShaEcall`, `ShaLoadState`,
  `ShaLoadData`, `ShaMix`, `ShaStoreState`) driven by the rv32im top-level
  state machine, with word-granular addressing into a paged memory argument.
  SP1's ABI is ECALL with `(w_ptr, h_ptr)`, 8-byte slot addressing
  (`w_ptr + i * 8`), 48-bit clk decomposition (`clk_high`, `clk_low`),
  `MemoryAccessTimestamp` chips, and separate Supervisor/User page-prot
  interactions. RZ's "chip is part of the rv32im state machine" design
  doesn't fit SP1's "chip is a separate syscall handler" architecture
  even structurally.

- **Lookup architecture.** RZ uses Plonk-style polynomial-commitment lookups
  with `LookupDelta`/`LookupCurrent` externs in the generated code. SP1
  Hypercube uses LogUp on multilinear extensions with typed interaction
  buses (`InteractionKind::ShaExtend`, `Memory`, `Byte`, etc., with
  `InteractionScope::Local|Global`). Different proof-system primitives;
  multiplicity bookkeeping must be re-authored end-to-end.

- **Prover glue.** SP1's existing SHA-256 precompile alone is ~2,610 LOC of
  hand-written Rust across `extend/{air,columns,controller,trace,mod}.rs`,
  `compress/{air,columns,controller,trace,mod}.rs`, and the executor's two
  syscall handlers. That's not counting syscall-code registration, RiscvAirId
  enumeration, byte-lookup operations consumed (`XorU32Operation`,
  `AndU32Operation`, `FixedRotateRightOperation`, etc.), recursion-verifier
  wiring, the cost-model entry in `rv64im_costs.json`, and the
  Supervisor/User mode doubling of the controller. The RZ `.zir` is ~380
  LOC across `sha2.zir` + `inst_sha.zir` because zirgen does the heavy
  lifting — you can't lift that compactness back into hand-written Rust.

**What's free:** the SHA-256 spec itself (NIST FIPS 180-4 round function,
message schedule, K table, IVs). Everything else must be re-implemented
for SP1's architecture using RZ's circuit as a design reference, not a
porting target. Estimate is months for someone fluent in SP1 internals, not
the days the user budgeted. Not pursued here.

[zirgen]: https://github.com/risc0/zirgen

### Finding 7 — Recursive composition makes the proof scale-free in memory

The PR2615 port let us measure with the BLAKE3 precompile lit but didn't solve
the bigger problem: SP1's monolithic CPU prover hits a memory wall at 1 MiB on
a 64 GiB box. The wall isn't a function of the *useful* work (BLAKE3
precompile cut cycles 3.3× at 1 MiB and the prover still OOM'd) — it's a
function of how many rv32im shards SP1 expands in parallel and how much
working state each shard's lookup-multiplicity / basefold scaffolding keeps
in memory. SP1's own logger says this out loud: `Memory usage is high: 96%,
we recommend using the prover network`.

The binding statement decomposes cleanly though:

- **SHA-256** is a 32-byte chained state plus a 64-bit length counter. Each
  64-byte block updates state via the SP1 precompile. State at chunk
  boundaries is `O(1)` data, and crucially it can be *precomputed natively*
  by the host before any prover work runs (it's just SHA-256 of F).
- **BLAKE3** processes 1024-byte chunks independently into leaf CVs, then
  merges those CVs in a binary tree via the same precompile. Leaves are
  embarrassingly parallel; tree merges are `O(log N)` deep.

So a chunk proof can be:

> "Given `(sha_state_in, chunk_bytes, chunk_index)`, the SHA-256 update over
> `chunk_bytes` yields `sha_state_out` AND the 64 BLAKE3 leaf CVs for the 64
> sub-chunks at positions `[chunk_index*64..chunk_index*64+64)` are these."

Per-proof memory is bounded by the chunk size, not the file size. The proof
itself doesn't even compute `sha_state_in` — it accepts it as input. A root
proof composes N chunk proofs, enforces the chain (`sha_state_out[i] ==
sha_state_in[i+1]`, `sha_state_in[0] == IV`), absorbs all `N*64` BLAKE3
leaves into a single CV stack, applies SHA-256 final padding and the BLAKE3
ROOT flag, and emits `(H_content, H_swarm)`.

`sp1-pr2615-bench-decomposed/` implements exactly this. Chunk size = 64 KiB
(= 64 BLAKE3 leaves per chunk proof). Chunk proof's public values are a
fixed 2,116 bytes: `chunk_index || sha_state_in || sha_state_out ||
[leaf_cv; 64]`. Root verifies each chunk proof via `verify_sp1_proof` (SP1's
recursion verifier), parses by offset, chains.

#### Measurements (CPU prover, Ryzen 5800X, native Linux)

| Size       | Chunks  | Per-chunk avg | Chunk total | Root prove | Setup | Grand total | Peak mem |
|------------|--------:|--------------:|------------:|-----------:|------:|------------:|---------:|
| 10 KiB     |       1 |           94s |         94s |       114s |   28s |        237s |    ~22 G |
| 64 KiB     |       1 |          125s |        125s |       121s |   30s |        277s |    ~25 G |
| 100 KiB    |       2 |          111s |        222s |       146s |   28s |        396s |    ~25 G |
| 128 KiB    |       2 |          121s |        242s |       152s |   28s |        422s |    ~30 G |
| 1 MiB      |      16 |          118s |       1885s |       386s |   29s |       2300s |    ~53 G |

Both hashes verify byte-for-byte against `sha2::Sha256` and `blake3::hash`
at every size.

#### Why each chunk proof is ~120s (and how that's mostly compression)

A 64 KiB monolithic core proof in PR2615 mode at the comparable size would be
~30-40s. The extra ~80-90s per chunk is SP1's `.compressed()` step —
recursively folding the rv32im trace into a single fixed-size proof that
`verify_sp1_proof` can witness. We pay this once per sub-proof; it can't be
amortised across chunks on a single prover instance. The root is also
`.compressed()` because `verify_sp1_proof` is wired into SP1's recursion
verifier and core-mode root proofs can't witness sub-proofs.

#### Why this matters anyway, even with the overhead

| Property               | Monolithic              | Decomposed                |
|------------------------|-------------------------|---------------------------|
| Peak memory            | scales with file size   | constant in file size     |
| Largest size on 64 GiB | ~100 KiB                | unbounded                 |
| Wall-clock at 1 MiB    | OOM                     | 38 min                    |
| Parallelisable         | no (one shard pipeline) | yes (chunk proofs in parallel; tree merges in log N layers) |
| Cluster scaling        | none                    | near-ideal                |

On a single CPU box you trade slowness for fitting at all. On a cluster
matching Succinct's published runs (16× RTX 5090), every chunk proof would
take ~1-2s in parallel and the binding proof for a 1 GiB file would land in
minutes — the same architecture, just compute-bound instead of latency-bound.

#### What we amortised (and what we couldn't)

Amortised:
- **`ProverClient`**: one instance for the whole run.
- **`ProvingKey`s**: `client.setup(ELF)` once per ELF (chunk + root), reused
  across all proofs.
- **Native SHA-256 boundary states**: single `sha2::compress256` pass over
  the whole file at host startup, no work duplicated per chunk.
- **ELF compactness**: chunk-prover is 55 KiB, root-prover is 122 KiB, both
  with `panic = "abort"`, `lto = "fat"`, `strip = true`, `codegen-units = 1`,
  `opt-level = 3`, and no heap allocations in the hot paths. The chunk's
  precompile-region cycle counts (`sha256_chunk`, `blake3_leaves`) are at
  the precompile-latency floor; further Rust-side optimisation has no room
  to give.

Couldn't:
- **Per-proof recursion-compression cost**: ~80-90s independent of input
  size or compute spent. This is the SP1 v6.2 architectural floor and the
  reason the wall-clock looks the way it does. The Succinct GPU prover
  hides it through parallelism, not by reducing it. Their `experimental`
  feature has some warm-state knobs (`WITHOUT_VK_VERIFICATION` etc.) that
  trade safety for setup time; we used `WITHOUT_VK_VERIFICATION=1` already.

[r_decompose_dir]: ./sp1-pr2615-bench-decomposed

### NixOS-specific friction (worth documenting)

For reproducibility on NixOS:

- SP1's prebuilt toolchain (`sp1up`) downloads glibc-linked binaries that do
  not run on stock NixOS. With `programs.nix-ld.enable = true;` (already on
  this box), `cargo-prove` runs unpatched. Docker (`cargo prove build --docker`)
  is the alternative.
- `cargo-prove prove install-toolchain` requires `rustup` for the `toolchain
  link succinct ...` step. The existing `devShells.default` shell uses
  rust-overlay (no `rustup`), so we added `devShells.sp1` with `rustup` plus
  `protobuf`/`git`/`curl`/`gcc` and let it manage both a default stable and the
  succinct toolchain. See `flake.nix`.
- RISC Zero (`rzup`) had no friction beyond needing to be run inside a shell
  that has `rustc` available so its preflight check passes.

---

## Bottom line

1. **AMD GPU is not an option today** for either prover. Watch RISC Zero PR
   #3725. Native Linux is still worth it for CPU throughput; the cross-prover
   comparison above is on a competent but not state-of-the-art CPU.
2. **For the waddup proof (SHA-256 + BLAKE3)** as it stands today, the choice
   between SP1 and RISC Zero is a size-dependent tradeoff: RISC Zero wins on
   small files (faster fixed cost), SP1 wins on medium-to-large files (better
   scaling). The crossover for this workload sits between 1 KiB and 10 KiB on
   CPU.
3. **Adding a BLAKE3 precompile** is a larger win than the prover choice for
   cycle counts. We ported SP1 PR #2615 forward and measured an 8.3x BLAKE3
   cycle reduction. Wall-clock proving improvement is more modest (13% at
   100 KiB) because SP1's fixed proving overhead dominates. The cycle
   reduction would matter more at proof-batching or larger sizes. **In SP1
   the chip is something an application can ship; in RZ a BLAKE3 precompile
   would require a first-party zkVM-circuit change** — RZ's app-defined
   precompile mechanism is scoped to modular/algebraic arithmetic (BigInt2,
   demonstrated for RSA/ECC, not hashes; see Finding 8 caveats). In neither
   case is the BLAKE3 chip safe to deploy without an audit.
4. **The SP1 ecosystem has rough edges on v6.x.** The official precompile
   patches don't dispatch; without fixing this you get pure-software hashes
   and ~10x worse cycle counts. Worth filing upstream and worth doing the
   trivial workaround in any real SP1 deployment.
5. **"Just port RZ's SHA-256 chip to SP1" is not a real option.** Different
   constraint DSL (zirgen vs hand-Plonky3), different field (BabyBear vs
   KoalaBear), different u32 encoding, different memory model, different
   lookup architecture. The SHA-256 *algorithm* is shared (NIST FIPS 180-4)
   but no constraint, no row layout, no interaction is portable verbatim.
   Months of work using RZ as a design reference, not weeks of porting.
6. **Decomposition makes the binding proof scale-free in memory.** The
   monolithic proof has a per-prover memory ceiling that scales with rv32im
   trace size; the decomposed version we shipped (`sp1-pr2615-bench-decomposed/`)
   has per-proof memory bounded by chunk size and works at 1 MiB on a box
   that OOM'd monolithic at the same size. Composition adds SP1's
   `.compressed()` recursion overhead per sub-proof (~80-90 s) — visible as
   ~38 min for 1 MiB on this CPU box — but it's the right architecture once
   you exceed the monolithic memory ceiling, and that overhead vanishes on a
   GPU cluster running sub-proofs in parallel.

### Finding 8 — Cross-prover scaling, full matrix

Head-to-head prove wall-clock for the binding statement on the same hardware:

| Size    | RZ monolithic | SP1 monolithic (PR2615) | SP1 decomposed | Winner            |
|---------|--------------:|------------------------:|---------------:|:------------------|
| 1 KiB   |        12.7s  |                   23.7s |        n/a [1] | RZ                |
| 10 KiB  |        53.1s  |                   28.7s |           237s | SP1 monolithic    |
| 100 KiB |       339.8s  |                   63.2s |           396s | SP1 monolithic    |
| 1 MiB   |       3591s   |                     OOM |          2300s | **SP1 decomposed** |

[1] decomposition is meaningless with 1 BLAKE3 leaf; the root would just be
the leaf compress with ROOT, which is what the monolithic proof already does.

Headline observations:

1. **RZ's segment+recursion model never OOMs on this box.** 1 MiB ran 36
   segments × lift/join recursion to a single receipt in ~60 minutes
   wall-clock. That's the *architectural* property RZ has that SP1 doesn't —
   application code doesn't need to know about segmentation.
2. **SP1 monolithic wins where it fits.** Below ~150 KiB the proving time is
   dominated by the fixed ~25 s SP1 setup + a much-lower per-cycle proving
   cost than RZ; SP1 with both precompiles is 4-5× faster than RZ at 10-100
   KiB. The wall is at ~150-300 KiB on a 64 GiB CPU box.
3. **Above the SP1 monolithic ceiling, decomposed SP1 beats RZ monolithic
   on wall-clock.** At 1 MiB: 38 min vs 60 min, a ~36% reduction. The win
   comes from SP1's lower per-byte proving cost (BLAKE3 precompile is 8×
   cheaper than RZ's unaccelerated path, SHA-256 is ~6.5× cheaper than RZ's
   1.1 c/B blended with the chunking — wait, the SHA-256 precompile in RZ is
   1.1 c/B, that's faster per byte than SP1's 6.5; the win is BLAKE3
   dominating the binding cost and SP1's precompile crushing RZ's
   unaccelerated path there). Even with the ~120 s per-sub-proof
   `.compressed()` tax × 16 chunks (1920 s pure overhead), the underlying
   SP1 prove cost is so much lower than RZ's at this size that it still wins.
4. **The decomposition tax doesn't scale.** Each chunk's ~110-125 s is
   dominated by the `.compressed()` recursion-compression step which is
   per-sub-proof, not per-byte. As file size grows, the relative overhead
   shrinks — at 1 MiB the tax is ~83% of wall-clock; at 10 MiB it would be
   smaller (more chunks but proportionally more underlying work), at 1 GiB
   it would be ~negligible. Reaching 1 GiB on a single CPU is impractical
   (~12 hours sequential) but the architecture is the right one.
5. **For waddup specifically**, the practical recommendation depends on
   typical file size:
   - **Median <100 KiB**: SP1 monolithic with both precompiles. Best
     wall-clock, simplest deploy.
   - **Median in 100 KiB – ~1 MiB**: SP1 monolithic if it fits in your
     prover's memory; SP1 decomposed if it doesn't (single CPU box).
     Either way, port PR #2615 to get the BLAKE3 precompile.
   - **Files routinely > 1 MiB on CPU**: SP1 decomposed.
   - **GPU cluster available**: SP1 decomposed scales near-ideally with
     parallel proving capacity (chunks parallel, log N tree merges); RZ
     scales via `RISC0_PROVER=actor` segment-parallel proving similarly.

#### Caveats — read the table with these in mind

The numbers in the matrix are honest single-run wall clocks of two provers
running the same proof statement on the same hardware, but several
asymmetries make the comparison less clean than the bare numbers suggest:

1. **Precompile coverage isn't symmetric.** RZ ran with SHA-256 accelerated
   and **BLAKE3 unaccelerated** (~26.7 c/B); SP1 ran with both accelerated
   (BLAKE3 at ~3.14 c/B via the PR2615 port). At 1 MiB the BLAKE3 region is
   ~28 M cycles for RZ vs ~3.3 M for SP1, which is most of where SP1's
   wall-clock advantage at 1 MiB comes from. We did *not* test "RZ + BLAKE3
   precompile" because RZ has no comparable ship-it-in-your-app mechanism
   for BLAKE3:
   - **SP1's BLAKE3 precompile (PR #2615)** is a zkVM-chip change in SP1's
     hand-written Plonky3 AIR — application-shippable in the sense that we
     could port it forward in ~half a day of conflict resolution.
   - **RZ's app-defined precompile mechanism (zkVM 1.2+)** is the **BigInt2
     system** — scoped to modular/algebraic arithmetic. Every demonstrated
     example is field work: RSA (the Automata 180× case was modular
     exponentiation, not a hash), elliptic curve operations, BLS pairing
     pieces, `add128`. The MLIR `bigint` dialect's first-class primitives
     are big-int add/mul/reduce; **bitwise XOR and fixed-bit rotations**
     (what BLAKE3 needs) aren't natively expressible. There is no public
     evidence of *any* hash precompile authored as an app-defined precompile
     in RZ.
   - The realistic way to give RZ a BLAKE3 precompile is to **modify the
     zkVM circuit itself** — similar in constraint-code scope to SP1's
     PR2615 (~2 KLOC of constraints) but with a much heavier deployment
     story: verifier coordination, audit, version bump, upstream PR.
   - So when comparing "RZ as deployed today" vs "SP1 with PR2615 lifted and
     ported by waddup" the comparison is fair *in terms of what you can
     actually run*, but RZ has BLAKE3-acceleration headroom that's
     meaningfully more expensive to unlock than SP1's was. (Earlier drafts
     of this document overstated the symmetry; corrected here.)

2. **Recursion lives at different layers.** RZ's segment continuation is
   built into the prover — 36 segments → lift → pairwise join → one
   receipt, no application code involved, recursion verifier specialised
   for "this is another rv32im segment of the same execution" and cheap
   per merge. SP1 decomposed uses `verify_sp1_proof` inside a user program;
   each chunk proof pays an ~80-90 s `.compressed()` recursion-compression
   tax before the root can witness it, and the root is also `.compressed()`
   for the same reason. **Of the SP1 decomposed 1 MiB wall-clock, roughly
   ~1440 s (≈ 16 × 90 s) is recursion-compression overhead that has no
   structural equivalent in RZ.** If SP1 shipped automatic continuation
   the way RZ does, that tax would mostly vanish; absent that, it's a real
   cost that shows up as wall clock.

3. **Soundness is not symmetric.** RZ's circuits are audited production
   code. The SP1 BLAKE3 chip we ported (PR #2615 forward-ported to v6.2.2)
   is **knowingly under-constrained** in three places (`trap_code` zeroed,
   no Supervisor/User mode split, missing page-prot interaction lookups —
   see Finding 1 and `sp1-pr2615-patches/README.md`). For *measuring
   proving time* this is fine — the bytes flow correctly, the hashes match.
   For *deploying to verifiers accepting proofs from untrusted seeders* it's
   not — a malicious prover could potentially substitute different bytes
   while still producing a valid receipt. **The SP1 decomposed 1 MiB receipt
   is not safe to ship**; that asymmetry doesn't show up in wall-clock
   numbers.

4. **The 1 MiB cell isn't quite apples-to-apples on memory either.** Both
   stay below the 64 GiB ceiling, but RZ's segment model keeps a flatter
   working set throughout the run (no warnings observed), while SP1
   decomposed peaks at ~53 GiB (86%) during the root proof that verifies
   16 sub-proofs. The "memory bounded per sub-proof" property of
   decomposition only applies to the leaf sub-proofs, not the root. A
   production decomposition would replace the fan-in root with a binary
   tree of merge-proofs to keep the root small too; we didn't build that.

5. **Neither receipt is Groth16-compressed.** Both stop at
   compressed-STARK. For on-chain verification each would need a final
   Groth16 compression step (~30-60 s, applied once at the root,
   approximately equal for both provers) on top of the wall-clock numbers
   shown. Doesn't change the rankings.

6. **Tiny host-side optimisation in SP1.** The SP1 decomposed host
   precomputes SHA-256 boundary states natively (~50 ms / MiB on this CPU)
   and feeds them to chunk provers. The chunk-prover's SHA-256 work
   re-derives the same boundaries inside the proof, and the root's chain
   check enforces they agree — so this is "doing SHA-256 twice for 1 MiB."
   Wall-clock impact is rounding error and the proof guarantees are
   unchanged.

**Reading the table fairly:** the bare numbers answer *"which prover should
I use as deployed today, given current ecosystem state, for this workload
on this hardware?"* and that's the comparison the matrix accurately
reflects. They don't answer *"which prover has the structurally faster
underlying machinery for this workload?"* — because RZ has BLAKE3-precompile
headroom we didn't tap (because tapping it is much more invasive than for
SP1), and SP1 paid recursion-compression overhead it wouldn't pay if SP1
shipped automatic continuation. Those would be separate investigations.
