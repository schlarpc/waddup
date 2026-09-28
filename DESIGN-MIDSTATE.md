# waddup — midstate-chain design (decision record)

Status: decided 2026-09-28, supersedes the ZK proof layer in `PROPOSAL.md` for
Merkle–Damgård content hashes. Background and measurements are in
`RESEARCH-PURPOSE-BUILT.md`.

## Goal, restated

Download the correct file for a given well-known hash (SHA-1, MD5, SHA-256,
SHA-512) without wasting significant time or bandwidth on wrong data. A
transferable "this content hash corresponds to that swarm" certificate is
**not** a requirement.

## Decision

Replace the ZK binding proof with **SHA-family midstate chaining** verified in
reverse order. Zero proving cost, ~0.1–0.2% metadata, and a single trust
assumption (free-start preimage resistance of the hash's compression
function). The purpose-built ZK provers measured in `RESEARCH-PURPOSE-BUILT.md`
reach 100x–1400x the zkVM baseline, but for this goal even that is
unnecessary: midstates are ~10^5x beyond it and simpler to verify.

The ZK path remains the only option for sponge hashes (SHA-3), which no
target DAT format publishes. It can be added later as an optional accelerator
(a verified certificate unlocks any-order downloading) without changing the
baseline protocol.

## How it works

Notation: `f` is the compression function of the content hash (64-byte
blocks for MD5, SHA-1, SHA-256), `m_i` is the chaining state after group `i`,
groups are 16 KiB (the Bao chunk-group size iroh-blobs already uses), `L` is
the file length, `H` is the well-known content hash.

**Seeder ingest.** One streaming pass computes `H`, the BLAKE3 hash `H_swarm`
and the Bao outboard as before, and additionally records `m_i` at every
16 KiB boundary into a sidecar (20 bytes per group for SHA-1, 16 for MD5, 32
for SHA-256).

**Leecher.**

1. Derive the DHT key from `(alg, H)` as in `PROPOSAL.md`; find peers.
2. Each peer's claim is `(H_swarm, L)`. **Probe**: fetch the last group by
   Bao range request together with `m_{N-1}`; check
   `f(m_{N-1}, last_group || pad(L)) == H`. Drop the peer on failure.
   Cost: one connection plus ≤ 16 KiB.
3. Download groups from the end backward. Each group response carries its
   preceding midstate. A group is **verified** when
   `f(m_i, group_i) == m_{i+1}` and `m_{i+1}` is already verified (or is the
   anchor from step 2). Verified data is kept regardless of which swarm served
   it.
4. At the start of the file check `m_0 == IV`.
5. On completion compute `BLAKE3(F)` locally and announce that as the
   canonical `H_swarm`; check any secondary DAT hashes (MD5, CRC32) locally.

Groups may be *fetched* in any order from any number of peers, but are only
*committed* as the verified frontier reaches them. Unverified groups wait in a
bounded buffer.

## Why it is sound

Every verified link starts from a state already tied to `H`. Producing
different bytes for any group requires a pair `(m', g')` with
`f(m', g') = m_{i+1}`: a free-start preimage on `f`. Best known attacks:
SHA-256 pseudo-preimage 52 of 64 steps at 2^255; MD5 full preimage ~2^116;
SHA-1 none practical. Collision attacks on SHA-1/MD5 (SHAttered, Shambles)
require the attacker to craft both files and are out of scope exactly as in
`PROPOSAL.md`. CRC32 is linear and cannot be used.

Forward links prove nothing (anyone can hash garbage forward from the IV),
which is why verification order is fixed. The full midstate list published
by a peer is not trusted; only links reachable backward from `H` are.

## Time-waste bounds

| Vector | Bound under midstates | Under the ZK design |
|---|---|---|
| Fake claim in the DHT | 1 connection + 16 KiB probe | 1 connection + proof fetch (256 B with Groth16; 0.3–16 MB with a purpose-built prover unless wrapped) |
| Fake swarm that passes the probe | Must serve *real* data down to its junction (kept); waste = unverified buffer above the junction, ≤ that peer's window | Cannot exist (no valid proof for wrong content) |
| Slow / stalling peer | throughput thresholds, parallel peers | same |
| Scarce honest seeders | same | same |
| Eclipse / DHT poisoning | same | same |

Reputation is therefore an efficiency knob, never a safety one: a peer that
earns a large window and then serves garbage costs at most one window.
Suggested policy: new peers get a small window near the frontier (~1 MiB);
window grows with verified groups delivered; any failed link attributable to
the peer drops it.

## Hash agility and DAT formats

MAME XML carries CRC32 + SHA-1; No-Intro and Redump carry CRC32 + MD5 + SHA-1
(SHA-256 in some newer No-Intro sets). Consequences:

- Key swarms on **SHA-1** (present everywhere); publish only the SHA-1
  sidecar. MD5 chains are a fallback for MD5-only sources. SHA-256/SHA-512
  chains for sources that publish them.
- A leecher follows one chain; other DAT hashes are checked after download.
- No per-algorithm circuits, no SHA-256 pivot composition (`PROPOSAL.md`
  § Proof Composition Alternative is obsolete).

Sidecar sizes at 16 KiB groups, versus the Bao outboard:

| Sidecar | Per group | Fraction of file | Per 1 GiB |
|---|---:|---:|---:|
| obao4 outboard | 64 B | ~0.39% | ~4 MB |
| SHA-1 midstates | 20 B | ~0.12% | ~1.3 MB |
| MD5 midstates | 16 B | ~0.10% | ~1 MB |
| SHA-256 midstates | 32 B | ~0.20% | ~2 MB |

The unit of a swarm is the individual ROM/CHD/ISO (the DAT hashes are of the
files inside archives, not the archives). Midstates matter for large files;
for tiny ROMs waste is bounded by the file size anyway, and sets should be
bundled under a manifest rather than one DHT key per file (open design item).

## Implementation verification plan

The scheme is hash math, reputation scoring and slot management, which is
amenable to proving the Rust implementation directly.

1. **Sans-IO core.** All invariant-bearing logic in a pure, deterministic state
   machine: events in (claim, group+midstate from peer P, link result,
   timeout), actions out (request, mark verified, release, drop peer). The
   tokio/iroh shell only moves bytes. ~1–2k lines.
2. **Invariants to prove** (all inductive over transitions):
   - chain: verified ⟹ link checks out and successor verified; frontier
     contiguous from the end; `m_0 = IV`; padding uses `L`; when the frontier
     reaches 0, folding `f` from IV over verified groups equals `H`
     (with `f` uninterpreted).
   - waste: unverified bytes per peer ≤ window(peer); total ≤ budget;
     released on link failure.
   - blame: a failed link is attributed only to the peers that supplied the
     bytes or the midstate.
   - re-seeding: only announce a swarm hash computed locally over the
     verified file.
   - arithmetic: `L*8` fits, partial last group, files < one group,
     Bao 1 KiB alignment.
3. **Tools.** proptest + adversarial simulator and differential test of the
   midstate fold against the `sha1`/`sha2` one-shot digest (days). **Kani**
   inductive harnesses (arbitrary state satisfying the invariant, one step,
   invariant holds) with `f` stubbed (1–3 weeks). **Verus** for full
   functional correctness of the core if time is no object (1–3 months; SMT
   trusted base; core must be in Verus's subset, hence the sans-IO split;
   tokenized state machines cover a threaded core). Cross-check Kani against
   Verus specs. Optional Lean lemma for the preimage reduction.
4. **Outside any tool.** The cryptographic step (invariants ⟹ no wrong bytes
   accepted) is the free-start preimage assumption; liveness (you eventually
   finish) depends on honest peers existing and is handled by simulation.

## What is dropped from `PROPOSAL.md`

RISC Zero/SP1 guest programs, Groth16 seals, image-id allowlists, proof
exchange protocol messages, proof non-determinism notes, trusted-setup
discussion, GPU/prover capacity planning, and the SHA-256 pivot composition.
Kept unchanged: iroh transport, iroh-blobs/Bao verified streaming, DHT keying,
threat model sections on Sybil/eclipse/privacy/censorship (with the proof
checks replaced by the probe and link checks above).
