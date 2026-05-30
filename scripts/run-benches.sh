#!/usr/bin/env bash
# Run RISC Zero + SP1 benchmarks across the chosen size matrix and tee output to
# RESULTS_RAW.md. Re-runnable; appends with timestamped section headers.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

OUT="$REPO_ROOT/RESULTS_RAW.md"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

SIZES_SMALL="${SIZES_SMALL:-1k 10k}"
SIZES_LARGE="${SIZES_LARGE:-100k}"

run_risc0() {
    local mode="$1"; shift
    local sizes="$*"
    echo "=== RISC Zero | mode=$mode | sizes=$sizes | $TS ===" | tee -a "$OUT"
    nix develop --command bash -c \
        "cargo run --release --manifest-path Cargo.toml -- --${mode} ${sizes}" 2>&1 \
        | tee -a "$OUT"
    echo "" | tee -a "$OUT"
}

run_sp1() {
    local mode="$1"; shift
    local sizes="$*"
    echo "=== SP1 | mode=$mode | sizes=$sizes | $TS ===" | tee -a "$OUT"
    nix develop .#sp1 --command bash -c \
        "RUST_LOG=warn ./sp1/target/release/sp1-bench --${mode} ${sizes}" 2>&1 \
        | tee -a "$OUT"
    echo "" | tee -a "$OUT"
}

echo "# Benchmark run $TS" | tee -a "$OUT"
echo "" | tee -a "$OUT"

# Small/medium across all modes for both provers, head to head
for mode in sha256 sha3 both; do
    run_risc0 "$mode" $SIZES_SMALL
done
for mode in sha256 sha3 both; do
    run_sp1 "$mode" $SIZES_SMALL
done

# Large size only on the canonical mode (sha256 + blake3) since these runs are long
run_risc0 both $SIZES_LARGE
run_sp1 both $SIZES_LARGE
