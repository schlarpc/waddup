# Shared rules for experiment agents (read fully before starting)

Machine: Ryzen 7 5800X (8c/16t, Zen 3: AVX2, PCLMULQDQ, VPCLMULQDQ, SHA-NI; NO AVX-512, NO GFNI),
64 GiB RAM (ZFS ARC may hold ~40 GiB; it shrinks under pressure), NixOS, Linux 7.2.
No NVIDIA GPU. AMD RX 7900 XT exists but no prover supports it; CPU only.

Toolchain:
- Rust stable 1.93.1 via:  nix develop /home/schlarpc/waddup --command <cmd>
  (first invocation may take ~1 min to evaluate; it is cached after that).
- If you need nightly Rust, use the sp1 shell which has rustup:
    nix develop /home/schlarpc/waddup#sp1 --command bash -c 'rustup toolchain install nightly --profile minimal && cargo +nightly ...'
  (~/.rustup already has stable/succinct/risc0 toolchains. rustup needs network; it is available.)
- gcc is on PATH. `git`, `curl` are available inside the nix shells. `nix shell nixpkgs#<pkg>` for anything else
  (e.g. `nix shell nixpkgs#time --command \time -v ...` for peak RSS; or poll /proc/<pid>/status VmHWM).
- Set CARGO_TARGET_DIR inside your own scratch dir to avoid clobbering others. Work ONLY under your own
  subdirectory of /tmp/claude-1000/-home-schlarpc-waddup/4dfd21de-19a0-4f53-b05a-3d664b78352f/scratchpad/<your-name>/ .
  /tmp is a 32 GiB tmpfs with ~20 GiB free: keep clones shallow (`git clone --depth 1`) and clean `target/` of
  debug builds you don't need. If you need more space, use /home/schlarpc/waddup-scratch/<your-name>/ (create it).
- NEVER modify anything under /home/schlarpc/waddup (the project repo). Never git commit there.

Timing discipline (IMPORTANT — several agents build and benchmark concurrently on this one box):
- Builds may run concurrently (contention is fine for builds).
- Every TIMED benchmark run must hold the shared lock so only one benchmark runs at a time:
    flock /tmp/claude-1000/-home-schlarpc-waddup/4dfd21de-19a0-4f53-b05a-3d664b78352f/scratchpad/bench.lock <your benchmark command>
  Run `uptime` right before and record the 1-min load average alongside each timing. If load avg > 3 when you
  acquire the lock, wait ~60 s (sleep is blocked in your shell; use a busy `flock`-guarded `timeout 60 cat`?? no —
  simply proceed and note the load; other agents' builds may still be running and the lock only serializes benches).
  Record wall-clock with `date +%s.%N` before/after or the program's own timing output. Prefer the prover's own
  reported prove/verify times when it prints them.
- Keep each individual benchmark run under ~10 minutes and your whole task under ~90 minutes of wall time.
  Prefer sizes that finish in 5 s – 3 min. Scale: 2^12, 2^14, 2^16 hash compressions (=256 KiB, 1 MiB, 4 MiB of
  input at 64 B/compression) are ideal points; go bigger only if memory and time allow.
- Report: exact commit hash of every repo you built, the exact command line, trace shape (rows × columns or
  constraint counts) if visible, prove wall time, verify wall time, proof size in bytes, peak RSS, threads used,
  and the derived metric: **bytes of hash input proven per second of prover wall time** (compressions/s × 64).
  Also report what fraction of the run is fixed overhead vs per-row work if you can (two sizes suffice).

Baseline to compare against (measured on this same box in the project, see /home/schlarpc/waddup/RESEARCH.md):
- SP1 v6.2.2 monolithic with SHA-256 + BLAKE3 precompiles: 100 KiB in 63 s (~1.6 KiB/s). 1 MiB OOMs.
- SP1 decomposed: 1 MiB in 2300 s. RISC Zero v3.0.5: 1 MiB in 3591 s, 100 KiB in 340 s.
- Targets: 100x = ~160 KiB/s; 1000x = ~1.6 MiB/s of hashed input per second (for SHA-256 AND BLAKE3 together
  over the same bytes; if you can only measure one hash, say so and report per-hash throughput).

Output: write <your-dir>/RESULTS.md with all of the above plus a short honest assessment (what worked, what did
not, what you could not measure, whether the code you ran is a toy/example vs production-grade, and any
soundness caveats visible in the code such as missing constraints or "insecure test parameters" like low
security bits / small blowup / few queries). Return the same content as your final message.
