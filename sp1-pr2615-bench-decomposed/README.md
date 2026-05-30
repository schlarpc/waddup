# sp1-pr2615-bench-decomposed

Decomposed binding-proof bench: proves `SHA256(F) = H_content AND BLAKE3(F) = H_swarm`
via N+1 sub-proofs instead of one monolithic proof, so memory stays bounded per
proof regardless of file size.

## Architecture

- `chunk-program/` — guest that processes one 64 KiB chunk of `F`. Computes
  SHA-256 update (no padding) and 64 BLAKE3 leaf CVs in parallel. Commits
  `(chunk_index, sha_state_in, sha_state_out, [leaf_cv; 64])` in fixed-size
  public values (2,116 bytes). Uses SP1 v6.2.2 SHA-256 precompiles and the PR
  #2615 BLAKE3 precompile.
- `root-program/` — guest that ingests N chunk proofs as compressed
  sub-receipts. For each: `verify_sp1_proof(chunk_vkey, sha256(pv))` against
  the witnessed sub-proof; parse `(sha_state_in, sha_state_out, leaf_cvs)`;
  enforce the chain `sha_state_out[i] == sha_state_in[i+1]` and
  `sha_state_in[0] == IV`. Absorb all `N×64` leaf CVs into a single BLAKE3
  CV stack, performing the tree merge with the precompile. Apply SHA-256 final
  padding using `file_byte_length` and the ROOT flag on the last BLAKE3
  merge. Commit `(H_content, H_swarm)`.
- `script/` — host. One `ProverClient` for the whole run. `setup()` once per
  ELF (cached). Computes SHA-256 boundary states **natively** so chunk proofs
  can be prepared (and, on a cluster, dispatched in parallel). Each chunk
  proof is generated with `.compressed()` so the root can verify it. Root is
  also `.compressed()` because `verify_sp1_proof` is wired into the recursion
  verifier.

## Why this layout

The user's intuition was correct: SHA-256 must chain serially, but each chunk
proof only needs to *verify a boundary assertion*, not *compute* the chain
state. The host precomputes boundary states natively and feeds them into the
chunk proofs; the root enforces consistency. This makes the chunk proofs
embarrassingly parallel logically (we just don't realise it on one CPU box).
BLAKE3 leaves are the chunk proof's natural side product — no second-pass
work needed.

Per-proof memory is bounded by the 64 KiB chunk size, so any file size that
fits the *file* on disk is provable. Monolithic 1 MiB OOM'd this 64 GiB box;
decomposed 1 MiB runs to completion (results in `RESULTS_RAW.md`).

## Soundness caveat (inherits from the PR2615 port)

The PR2615 BLAKE3 compress chip we ported is under-constrained in three places
(see `../sp1-pr2615-patches/README.md`). Both this composition's chunk and root
proofs depend on that chip. Same "fine for benchmarking, not for deployment"
status applies.

## Reproduce

Same prerequisites as `../sp1-pr2615-bench` — copy this directory into a
patched SP1 v6.2.2 source tree, add `bench-decomposed/script` to the workspace
members in `Cargo.toml`, then:

```
WITHOUT_VK_VERIFICATION=1 \
    nix develop /path/to/waddup#sp1 --command \
    cargo run --release --manifest-path bench-decomposed/script/Cargo.toml -- 1m
```

`WITHOUT_VK_VERIFICATION=1` opts out of SP1's verifying-key allowlist (an
`experimental`-feature dev knob). The script enables the `experimental`
feature on `sp1-sdk` for this reason.
