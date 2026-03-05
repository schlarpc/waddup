# waddup

**W**itness-**A**ttested **D**istributed **DUP**licator

## Trustless Content Hash to P2P Swarm Resolution

### Problem

Given a set of well-known file hashes (SHA-256, SHA-1, etc.) of files ranging from a few KB to tens of GB,
allow anyone who knows a hash to independently — without a central authority — join a P2P
swarm and download that exact file.

P2P systems (BitTorrent, IPFS) chunk files, hash the chunks, and derive swarm identifiers
from the chunk-level metadata. These identifiers never align with a plain content hash of the
whole file. Two people creating a torrent for the same file can get different infohashes
depending on piece size, file name, and other metadata choices.

### Core Insight

A cryptographic proof can bridge the gap between a well-known content hash and a P2P swarm
identifier, without requiring trust in any party.

The proof statement: "I know a file F such that `ALG(F) = H_content` and
`BLAKE3(F) = H_swarm`" — where ALG is the well-known hash algorithm (SHA-256, SHA-1, etc.)
and `H_swarm` is the deterministic swarm identifier used by the transfer layer.

This proof is generated once per file, by any participant who has the file, and is verified
by all subsequent participants in milliseconds.

---

## Architecture

```
Well-known hash:    ALG(file)                 (published out of band; ALG = SHA-256, SHA-1, etc.)
Swarm identifier:   BLAKE3(file)              (deterministic from file content; always BLAKE3)
Proof:              Groth16 via RISC Zero      (256-byte seal, ~1ms verification)
Transport:          iroh (QUIC via Quinn)
Transfer:           iroh-blobs (BLAKE3 + Bao verified streaming)
Discovery:          iroh discovery (DNS/Pkarr, optional Mainline DHT, mDNS)
Proof exchange:     custom iroh protocol (ProtocolHandler + ALPN)
```

### Why These Choices

**BLAKE3 + Bao** (via iroh-blobs): BLAKE3 is internally a binary Merkle tree hash with
1024-byte leaf chunks. Bao (Section 6.4 of the BLAKE3 spec) exposes the intermediate nodes
of this tree, enabling verified streaming — every chunk can be verified against the tree as
it arrives. Crucially, the Bao root hash IS the standard BLAKE3 hash — `bao hash file` and
`b3sum file` produce identical output. This means the swarm identifier is just a BLAKE3 hash,
verifiable with any BLAKE3 implementation, and Bao is purely about how data is transferred
and verified in transit.

**iroh** (not BitTorrent, not IPFS): iroh is a networking library built by n0-computer,
founded by former Protocol Labs engineers who initially built iroh as a Rust IPFS
implementation (5 releases shipped). After finding IPFS's protocol stack couldn't hit their
performance targets (~1000 messages per block vs their goal of <1, ~2000 peers required vs
their goal of ~200), they put the IPFS-compatible version in maintenance mode ("Beetle",
archived Dec 2023) and started over from scratch. iroh provides QUIC transport (via a Quinn
fork with multipath and NAT traversal extensions), content-addressed blob transfer via Bao,
pluggable discovery, and encrypted connections by default. It avoids BitTorrent's protocol
archaeology (uTP, MSE, BEP sprawl) and IPFS's overcomplexity (IPLD, UnixFS, Bitswap
inefficiency). As of Feb 2026, iroh is pre-1.0 with a release candidate (v0.97.0) published
and 1.0 targeted for Q1 2026. Half a million unique nodes have hit the public network in a
30-day span.

**Groth16 via RISC Zero** (not raw STARKs): RISC Zero runs the expensive STARK proof
internally, then compresses it to a 256-byte Groth16 seal using a trusted setup ceremony
they conducted in April-May 2024 (hosted on ceremony.pse.dev, circuit audited by Hexens,
238 contributions via p0tion/DefinitelySetup tooling). The ceremony is universal — one setup
covers all possible guest programs. The trust assumption is 1-of-N honest participants. The
verification key is a fixed artifact baked into client software.

---

## Hash Agility

### Design

The well-known content hash (H_content) is not restricted to SHA-256. The hash algorithm is
a protocol parameter that flows through discovery, proof generation, and verification:

```
H_content = ALG(file)          where ALG ∈ {sha256, sha1, sha512, ...}
H_swarm   = BLAKE3(file)       always BLAKE3 (iroh-blobs requirement, non-negotiable)
Proof:      "ALG(F) = H_content AND BLAKE3(F) = H_swarm"
DHT key:    SHA1("waddup-v1:" || alg_id || ":" || H_content)
```

The algorithm identifier (`alg_id`) is folded into:
- **DHT lookup key**: Different algorithms for the same file produce different DHT keys,
  so they form separate swarms. A file identified by its SHA-256 and the same file
  identified by its SHA-1 are two independent swarms with independent proofs.
- **Proof journal**: The journal contains the algorithm identifier alongside the hash
  value, so the verifier knows which hash function was proved.
- **Guest program**: Different hash algorithms require different guest programs (different
  image_ids). The leecher checks the image_id against a set of known-good guest programs,
  one per supported algorithm.

### Supported Algorithms

| Algorithm | Hash size | Security | Status | Use case |
|-----------|-----------|----------|--------|----------|
| SHA-256   | 32 bytes  | Full     | Primary | Modern package managers, checksums |
| SHA-512   | 64 bytes  | Full     | Supported | Some Linux distros, scientific data |
| SHA-1     | 20 bytes  | Collision-broken | Legacy | Git (older), legacy package repos |

H_swarm is always BLAKE3, regardless of the H_content algorithm. The hash agility is
entirely on the "well-known hash" side of the proof.

### Broken Hash Functions (SHA-1)

Supporting SHA-1 is a deliberate choice for legacy compatibility — many existing systems
publish only SHA-1 checksums. But SHA-1's collision vulnerability (SHAttered, 2017;
Shambles/chosen-prefix, 2020) has real protocol consequences.

**What breaks and what doesn't:**

The ZK proof remains sound. It correctly proves "this file hashes to this SHA-1 value and
this BLAKE3 value." The cryptographic weakness is in SHA-1 itself, not in the proof system.

**Collision attack** (attacker crafts two files with the same SHA-1):
- The attacker creates file_good and file_evil where `SHA1(file_good) = SHA1(file_evil)`.
- The publisher distributes `SHA1(file_good)` as the content hash.
- An honest seeder generates a valid proof binding that SHA-1 to `BLAKE3(file_good)`.
- The attacker generates a *different* valid proof binding the same SHA-1 to
  `BLAKE3(file_evil)`.
- Both proofs are cryptographically valid. The leecher looking up that SHA-1 may find
  either swarm. The ZK proof cannot distinguish them — it proves the hash binding is
  correct, and both bindings *are* correct.
- **Impact**: Integrity break. The leecher may download file_evil and believe it
  corresponds to the published SHA-1.

**Second-preimage attack** (attacker finds a different file matching an existing SHA-1):
- Given an existing file with a known SHA-1, find a different file with the same hash.
- **Not yet practical for SHA-1.** Second-preimage resistance is much stronger than
  collision resistance. No known practical second-preimage attack on SHA-1 exists.
- This means: if the legitimate file was created independently (not by the attacker), the
  attacker cannot find a collision with it using known techniques. Most real-world content
  (OS ISOs, package tarballs, source archives) falls into this category.

**Practical risk assessment:**
- SHAttered-style collision attacks require the attacker to control the creation of *both*
  files. For content that already exists independently in the wild, this is not applicable.
- The realistic threat is: a malicious publisher who crafts a file with a collision partner,
  publishes the "good" version, then later substitutes the "evil" version in the swarm.
  But if the publisher is malicious, they can publish any hash they want — the publisher
  is out of scope for the threat model anyway.
- The residual risk unique to SHA-1 is: a third party (not the publisher) who participated
  in creating the original file could have embedded a collision. This is the classic
  "malicious PDF with a collision partner" scenario.

### Mitigations for Broken Hashes

1. **Algorithm tagging in the UI/API.** When a leecher requests content by SHA-1, the
   client should clearly indicate that the content hash uses a collision-vulnerable
   algorithm. Not a silent fallback — an explicit acknowledgment.

2. **Multiple hash binding.** If the publisher provides both SHA-1 and SHA-256 for the same
   file, the leecher can look up the SHA-256 swarm (collision-resistant) and verify that
   the downloaded file also matches the SHA-1. This doesn't require a separate proof —
   the leecher can compute SHA-1 locally after download and confirm it matches.

3. **Prefer stronger algorithms.** If the same file has swarms under multiple algorithms,
   the client should prefer the strongest available. SHA-1 is a fallback for when nothing
   better is published.

4. **No algorithm downgrade in proof exchange.** A peer cannot convince a leecher to accept
   a SHA-1-based proof when the leecher asked for SHA-256. The algorithm is part of the
   DHT key and the proof journal — they either match the leecher's request or they don't.

### Guest Program Implications

Each supported hash algorithm requires its own guest program (or a single parameterized
guest program with a different image_id per algorithm configuration):

| Algorithm | Guest program | image_id | Precompile available? |
|-----------|---------------|----------|-----------------------|
| SHA-256   | `waddup-sha256` | deterministic from ELF | Yes (built-in zkVM accelerator) |
| SHA-512   | `waddup-sha512` | deterministic from ELF | No (software, but could use app-defined precompile) |
| SHA-1     | `waddup-sha1`   | deterministic from ELF | No (software) |

All guest programs also compute BLAKE3 (for H_swarm). The image_id is checked by the
leecher during proof verification — the leecher maintains a hardcoded set of accepted
image_ids, one per supported algorithm. A proof produced by an unknown guest program is
rejected.

Adding a new hash algorithm to waddup requires:
1. Writing (or parameterizing) the guest program
2. Building it deterministically (`cargo risczero build`)
3. Distributing the new image_id in a client update
4. No new trusted setup ceremony (the Groth16 circuit is universal)

### Proof Composition Alternative

RISC Zero's proof composition feature (env::verify inside a guest) could reduce per-algorithm
proving cost by pivoting through SHA-256 as a shared binding hash, rather than requiring
every guest to compute BLAKE3 directly:

```
waddup-sha256:       SHA256(F) + BLAKE3(F)   → commits (H_sha256, H_blake3) to journal
waddup-sha1-comp:    SHA1(F) + SHA256(F)     → commits (H_sha1, H_sha256) to journal
waddup-sha512-comp:  SHA512(F) + SHA256(F)   → commits (H_sha512, H_sha256) to journal
composer:            env::verify both receipts, check H_sha256 matches
                     → outputs (H_content, H_blake3)
```

The key insight is that two composed proofs cannot directly assert they operated on the same
private input (the file F). However, if both sub-guests compute a shared pivot hash over F,
the composer can check equality of the pivot values across journals. Since SHA-256 is
collision-resistant, matching pivot values guarantees the same file (up to SHA-256 collision
resistance). This binds the content hash to BLAKE3 transitively without the non-primary
guest ever computing BLAKE3.

#### Why This Matters: SHA-1/MD5-Only Sources

Many real-world hash sources (legacy package repositories, older distribution mirrors) only
publish SHA-1 or MD5 checksums. Binding these weak hashes to BLAKE3 is the critical path —
not a secondary concern. The seeder has the file and can compute any hash natively, but
proving inside the zkVM is where cost matters.

SHA-256 is the only hash function with a built-in zkVM precompile that is both relevant and
efficient. The zkVM also has a Keccak (SHA-3) precompile, but benchmarks show it is 13x
slower per byte than SHA-256 (15.6 vs 1.2 cycles/byte) and has disproportionate prover
overhead — see Performance Considerations for measured results. No other hash relevant to
waddup (SHA-1, SHA-512, MD5, BLAKE3) has any precompile.

SHA-256 as the pivot hash is an internal implementation detail, not exposed in the protocol.
Keccak was considered as an alternative pivot for construction diversity (sponge vs
Merkle-Damgård), but the 13x per-byte cost penalty provides no practical benefit for an
implementation detail that only needs collision resistance.

This makes SHA-256 the natural pivot hash for composition:

| Guest                  | Hashes computed in zkVM           | Expensive?                         |
|------------------------|-----------------------------------|------------------------------------|
| Monolithic `waddup-sha1` | SHA1 (software) + BLAKE3 (software) | Yes — BLAKE3 dominates (~10-100x more cycles than accelerated SHA-256) |
| Composed `waddup-sha1-comp` | SHA1 (software) + SHA256 (precompile) | Much cheaper — avoids BLAKE3 entirely |
| Base `waddup-sha256`   | SHA256 (precompile) + BLAKE3 (software) | Yes, but generated once per file and reused for all algorithms |

The composition approach means the expensive BLAKE3 proving cost is paid exactly once per
file (in the `waddup-sha256` guest), regardless of how many content-hash algorithms need to
bind to that file's BLAKE3 swarm identifier. Every additional algorithm only needs to prove
its own hash + the cheap SHA-256 pivot, then compose.

#### Benefits

- The `waddup-sha256` receipt (SHA256 + BLAKE3) is generated once per file and reused as a
  pivot for all other algorithms. Only one guest ever pays the BLAKE3-in-zkVM cost.
- Non-primary algorithm guests (SHA-1, SHA-512, MD5, etc.) compute only their own hash +
  SHA-256. The SHA-256 precompile makes the pivot hash nearly free relative to any
  software-only hash computation.
- Adding a new content-hash algorithm never requires implementing BLAKE3 in the new guest.
- For SHA-1/MD5-only sources (a primary use case), this dramatically reduces proving cost
  compared to the monolithic approach.

#### Tradeoffs

- Adds a composition/resolve step to the proving pipeline for non-SHA-256 algorithms.
- The `waddup-sha256` proof must exist before other algorithms can compose against it (or
  both must be generated and composed together).
- More complex verification: the leecher must accept composed proofs with multiple image_ids.
- If a BLAKE3 application-defined precompile is built (see Performance Considerations),
  the cost advantage largely disappears — BLAKE3 becomes cheap in every guest, and the
  monolithic approach is simpler.

Given that many target sources only publish weak hashes, and BLAKE3 has no zkVM precompile,
composition via SHA-256 pivot is likely the right default architecture for non-SHA-256
algorithms rather than an optimization to revisit later.

---

## Participant Flows

### File Ingest (common to all seeders)

When a seeder adds a file, a single streaming pass over the file produces all required
artifacts:

```
1. Stream file bytes through both hash functions simultaneously:
     - ALG(file) = H_content            (well-known content hash, algorithm-specific)
     - BLAKE3(file) = H_swarm          (swarm identifier)
2. iroh-blobs stores:
     - The file data (or references existing data on disk)
     - The Bao outboard (.obao4) — flattened BLAKE3 Merkle tree, 16 KiB chunk groups
       (~0.4% space overhead, e.g., ~4 MB outboard for a 1 GB file)
3. Store the mapping: H_content <-> H_swarm
```

The BLAKE3 hash and Bao outboard are computed together — the outboard is just the
intermediate tree nodes that BLAKE3 computed internally anyway. iroh-blobs handles this
natively. The SHA-256 is computed in the same read pass to avoid reading the file twice.

### Seeder (first seeder, no proof exists yet)

```
1. Ingest file (see above) -> have H_content, H_swarm, outboard
2. Generate RISC Zero proof:
     "ALG(file) = H_content AND BLAKE3(file) = H_swarm"
   Compress STARK to Groth16 via RISC Zero's published ceremony artifacts
   (This is the expensive step — minutes to hours depending on file size.
    Done ONCE per file, by ONE participant, amortized across all future peers.
    See "Proof Generation Parallelism" below.)
3. Store proof alongside file metadata
4. Announce on discovery layer under H_content
5. Serve file via iroh-blobs (Bao outboard enables verified streaming)
6. Serve Groth16 proof to discovering peers via custom protocol
```

### Leecher (has only the content hash)

```
1. Has H_content (from a website, package manager, advisory, etc.)
2. Derive discovery key from H_content
3. Discovery lookup -> find peer addresses
4. Connect to a peer, receive:
     - H_swarm (the BLAKE3 hash / swarm identifier)
     - groth16_seal (~256 bytes)
     - receipt_claim (contains image_id, journal, etc.)
5. Verify locally (~1ms):
     a. Verify groth16_seal against hardcoded verification key
     b. Confirm the claim was produced by expected guest program (image_id check)
     c. Extract H_content and H_swarm from the journal; confirm they match
6. Request blob by H_swarm via iroh-blobs
7. Every chunk group (16 KiB) is verified against the Bao Merkle tree as it streams in
8. File is complete — verified by both Bao (chunk integrity) and the ZK proof (content hash binding)
```

### Nth Seeder (has the file, proof already exists in swarm)

```
1. Ingest file (see above) -> have H_content, H_swarm, outboard
2. Join swarm, receive existing Groth16 proof from peers
3. Verify proof locally (milliseconds)
4. Announce under both H_swarm and H_content
5. Serve file + relay proof to new peers
```

---

## The ZK Proof Layer

### What Is Being Proved

"I know a file F such that `ALG(F) = H_content` and `BLAKE3(F) = H_swarm`."

Where ALG is the well-known hash algorithm (SHA-256, SHA-1, etc.). The proof is generated
using RISC Zero's zkVM. The guest program reads the file bytes, computes both hashes, and
commits the results (including the algorithm identifier) to the journal. The host verifies
the guest executed correctly and the journal contains the expected hash pair. Each supported
algorithm has its own guest program with a distinct image_id (see Hash Agility).

### RISC Zero Proof Pipeline

```
Guest program execution (ALG + BLAKE3 of file bytes)
  -> RISC-V execution trace, split into segments
  -> STARK proof per segment (SegmentReceipt, each >100 KB)
  -> Collection of segment proofs (CompositeReceipt, potentially MBs)
  -> Lift: each SegmentReceipt -> SuccinctReceipt (via Recursion Circuit)
  -> Join: binary tree reduction of SuccinctReceipts into one (~200 KB)
  -> identity_p254: re-prove using Poseidon254 hash (BN254-field-friendly)
  -> Groth16 compression (circom circuit over BN254, using ceremony artifacts)
  -> Groth16Receipt: 256-byte seal + claim metadata
```

### Groth16 Seal Structure

The seal is exactly 256 bytes, containing three standard Groth16 proof elements:

| Element | Type      | Size     |
|---------|-----------|----------|
| `a`     | G1 point  | 64 bytes |
| `b`     | G2 point  | 128 bytes|
| `c`     | G1 point  | 64 bytes |
| **Total** |         | **256 bytes** |

The full `Groth16Receipt` also contains the `ReceiptClaim` (journal with the hash pair,
image_id, exit status, etc.) and `verifier_parameters`. The claim is small for this use
case — two 32-byte hashes plus a file length — so the total receipt is still compact.

Public inputs to the Groth16 verifier circuit (5 field elements):
- `control_root` (2 elements) — identifies allowed recursion programs
- `claim_digest` (2 elements) — hash of ReceiptClaim (contains image_id + journal)
- `bn254_control_id` (1 element) — identifies the identity_p254 program

The guest program's `image_id` and the journal contents (our hash pair) flow through as:
`image_id + journal` -> `ReceiptClaim` -> `claim_digest` -> Groth16 public input.

### Why Groth16 (Not Raw STARKs)

| Property             | Raw STARK (Succinct)  | Groth16 (wrapped)           |
|----------------------|-----------------------|-----------------------------|
| Proof size           | ~200 KB               | 256 bytes                   |
| Verification         | ~10-100ms             | ~1ms                        |
| Trusted setup        | None                  | RISC Zero's (already done)  |
| Post-quantum         | Yes                   | No (BN254)                  |
| Trust assumption     | Hash security         | + 1-of-N ceremony honest    |

For a practical system, the Groth16 wrapper is strictly better: smaller proofs, faster
verification, and the ceremony is already done. The raw SuccinctReceipt exists as an
intermediate artifact during proving and could be used as a protocol option if post-quantum
concerns ever become concrete.

### Proof Non-Determinism

RISC Zero proofs are **not deterministic across proving runs**. The prover draws randomness
from the host system's CSPRNG (e.g., `/dev/urandom`) for ZK blinding — random noise rows
are appended to the execution trace columns before polynomial interpolation. This randomness
propagates through the Fiat-Shamir transcript, so two provers with the same file and same
RISC Zero version will produce different valid proof bytes.

What IS deterministic:
- Guest program builds (via `cargo risczero build` with Docker)
- Image IDs (hash of the guest ELF binary)
- Guest execution traces (same program + same inputs = same trace before ZK blinding)

What is NOT deterministic:
- STARK proofs (random noise rows differ each run)
- Groth16 seals (derived from non-deterministic STARKs)

Protocol implications:
- There is no single canonical proof per file — any valid proof works
- Multiple seeders independently generating proofs will get different bytes
- Proof deduplication must be by semantic validity (verified against the claim), not by
  content-addressing the proof bytes
- This is fine for the protocol: the first valid proof seen by any peer is sufficient,
  and all valid proofs bind the same hash pair

### Trusted Setup Details

RISC Zero's ceremony (April-May 2024):
- Hosted on ceremony.pse.dev (Ethereum Foundation's PSE team platform)
- Uses p0tion / DefinitelySetup open source tooling
- Phase 1: Reuses Hermez rollup Powers of Tau (`powersOfTau28_hez_final_23.ptau`)
- Phase 2: Circuit-specific MPC, 238 contributions
- Circuit audited by Hexens (Stark2Snark Circuit audit Feb-Mar 2024, SNARK Verifier
  Contract audit May-Jun 2024). RISC Zero has undergone 18 security audits total across
  Hexens, Veridise, and zkSecurity.
- Artifacts published on ceremony.pse.dev and mirrored on S3
- The Groth16 circuit is universal — it verifies any RISC Zero STARK proof, regardless
  of guest program. The guest program identity flows through as part of the `claim_digest`
  public input, not as part of the circuit. One ceremony covers all use cases. The
  `control_root` (identifying allowed recursion programs) can be updated when the RISC-V
  circuit changes without requiring a new ceremony.

### Performance Considerations

#### Measured Precompile Performance (10 KiB input, CPU prover)

Benchmarks on a 10 KiB input using optimized I/O and sequential runs:

| Metric         | SHA-256 only      | SHA3-256 only       | SHA-256 + BLAKE3              |
|----------------|-------------------|---------------------|-------------------------------|
| Hash cycles    | 12,263 (1.2 c/B)  | 159,893 (15.6 c/B)  | 12,264 + 271,788 (1.2 + 26.5 c/B) |
| Startup + read | 22,396            | 22,231              | 22,236                        |
| User cycles    | 38,897            | 188,863             | 310,810                       |
| Total cycles   | 131,072           | 262,144             | 524,288                       |
| Segments       | 1                 | 1                   | 1                             |
| Proving time   | 15.9s             | 86.9s               | 60.9s                         |
| Verification   | 14ms              | 27ms                | 15ms                          |

Key observations:
- **SHA-256 precompile** is extremely efficient: 1.2 cycles/byte.
- **SHA3-256 (Keccak) precompile** is 13x slower than SHA-256 at 15.6 cycles/byte, despite
  also being a built-in precompile. Proving time is disproportionately high (86.9s for 262K
  total cycles vs 60.9s for 524K total cycles with SHA-256+BLAKE3), suggesting the Keccak
  precompile generates additional circuit proof segments not reflected in the rv32im cycle
  count but still requiring proving work.
- **BLAKE3 (no precompile)** is 22x slower than SHA-256 at 26.5 cycles/byte and dominates
  the combined SHA-256+BLAKE3 guest's cycle count (271K of 284K hash cycles).

These numbers confirm SHA-256 as the right pivot hash for proof composition (see Proof
Composition Alternative in Hash Agility). SHA3-256/Keccak was considered as an alternative
pivot for construction diversity (sponge vs Merkle-Damgård), but at 13x the per-byte cost
and disproportionate prover overhead, it offers no practical advantage. The pivot hash is an
internal implementation detail not exposed in the protocol.

#### CPU vs GPU Proving Throughput

Our CPU proving times are roughly 1.2-1.6x slower per rv32im cycle than RISC Zero's
published benchmark CPU, which is reasonable for different hardware.

GPU speedup ratios from RISC Zero benchmarks (Linux CPU vs NVIDIA RTX A5000, 8 KiB input):

| Workload  | CPU (benchmark) | GPU A5000 | Speedup |
|-----------|-----------------|-----------|---------|
| SHA-256   | 38.5s           | 5.7s      | 6.7x   |
| Keccak    | 118.2s          | 51.6s     | 2.3x   |
| BLAKE3    | 21.5s           | 5.2s      | 4.1x   |

Applying these ratios to our measured CPU times for 10 KiB:

| Mode (10 KiB)    | Our CPU (Linux) | Est. CUDA A5000 | Est. Metal M2 Pro   |
|-------------------|-----------------|-----------------|---------------------|
| SHA-256 only      | 15.9s           | ~2.4s           | ~35s (worse than CPU) |
| SHA3-256 only     | 86.9s           | ~38-43s         | ~90-120s            |
| SHA-256 + BLAKE3  | 60.9s           | ~15s            | ~100-150s           |

The Keccak case benefits least from GPU acceleration because the keccak circuit proving
(not the rv32im circuit) is the bottleneck, and it does not parallelize on GPU as well.
This further reinforces SHA-256 over Keccak as the composition pivot hash — the Keccak
penalty is even worse on GPU than on CPU.

#### Application-Defined BLAKE3 Precompile

RISC Zero (as of zkVM 1.2) supports **application-defined precompiles** — custom accelerators
that ship with the application, not the zkVM itself. Reference results from early adopters
show dramatic improvements — Automata Network reduced their guest program from 39M to 217K
cycles (~180x reduction) using an application-defined precompile.

A BLAKE3 precompile for waddup would:
- Replace the interpreted BLAKE3 guest code (26.5 cycles/byte) with a dedicated circuit
- Potentially bring BLAKE3 cycle counts to the same order of magnitude as the built-in
  SHA-256 accelerator (1.2 cycles/byte)
- Eliminate BLAKE3 as the proving bottleneck, making SHA-256 and BLAKE3 roughly equal
  contributors to total proving time

BLAKE3's structure is relatively circuit-friendly: simple compression function (based on
ChaCha), binary Merkle tree, no complex scheduling. This is more tractable than something
like Keccak (which has a built-in precompile but still shows high prover overhead).

Writing the precompile is real cryptographic engineering work — it requires a circuit that
proves BLAKE3 was computed correctly, not just code that computes BLAKE3. But this is a
bounded, well-defined problem with a fixed specification to implement against.

#### Proving Time Extrapolation

Based on the measured 10 KiB per-byte rates, estimated hash cycle counts at larger file
sizes (without a BLAKE3 precompile):

| File size | SHA-256 cycles (1.2 c/B) | BLAKE3 cycles (26.5 c/B) | Combined   |
|-----------|--------------------------|--------------------------|------------|
| 10 KiB    | 12K                      | 272K                     | 284K       |
| 1 MB      | 1.3M                     | 27M                      | 28M        |
| 100 MB    | 126M                     | 2.7B                     | 2.8B       |
| 1 GB      | 1.3B                     | 27B                      | 28B        |

BLAKE3 accounts for ~96% of hash cycles. With a BLAKE3 precompile bringing it to parity
with SHA-256 (~1.2 c/B), total hash cycles would drop by ~22x.

#### Prover Backends

RISC Zero does **not require a GPU**. The prover supports multiple backends:

| Backend  | Hardware             | Speedup vs CPU                    | Notes                          |
|----------|----------------------|-----------------------------------|--------------------------------|
| CPU      | Any x86/ARM          | 1x (baseline)                     | Default, works everywhere      |
| Metal    | Apple GPU (macOS)    | **0.4-0.7x (slower than CPU)**    | Via `RISC0_PROVER=metal`       |
| CUDA     | NVIDIA GPU           | 2.3-6.7x (measured, RTX A5000)    | Via `RISC0_PROVER=cuda`        |

**Metal is slower than CPU.** RISC Zero benchmarks (Oct 2025, M2 Pro) show Metal
consistently underperforms the same machine's CPU across all hash workloads:

| Workload (8 KiB) | macOS CPU | Metal M2 Pro | CUDA A5000 |
|-------------------|-----------|--------------|------------|
| SHA-256           | 34.7s     | 85.7s (0.4x) | 5.7s (6.7x) |
| BLAKE3            | 62.4s     | 120.5s (0.5x)| ~5.2s (4.1x)|
| Keccak            | 133.1s    | 181.3s (0.7x)| 51.6s (2.3x)|

Metal also generates more cycles for the same computation (e.g., big_blake3-8192: Metal
produces 1,310,720 total cycles / 1,145,836 user cycles vs CPU's 1,048,576 / 656,306).
The M2 Pro's GPU is likely not well-suited to the NTT/polynomial operations the prover
requires. Metal exists as a backend but CPU is strictly better on the same hardware.

**CUDA is the only backend that provides real acceleration.** Speedup is
workload-dependent: 6.7x for SHA-256-dominated workloads (rv32im circuit parallelizes
well), 4.1x for BLAKE3-dominated workloads, and only 2.3x for Keccak (precompile circuit
is the bottleneck, parallelizes poorly on GPU). For the SHA-256+BLAKE3 base proof, an RTX
A5000 would reduce 10 KiB proving from ~61s to ~15s.

CPU proving is viable for small files or infrequent proving. For a seeder proving a handful
of files, CPU may be acceptable. For a mirror seeding thousands of packages, CUDA hardware
makes a significant difference. With a BLAKE3 precompile reducing cycle counts, CPU proving
becomes more practical for larger files.

### Proof Generation Parallelism

Proof generation for a single file occupies one prover instance. However, a seeder with
many files to prove can run multiple independent proof jobs concurrently:

- **Across files**: Each file's proof is completely independent. A machine can run multiple
  prover processes simultaneously (one per GPU if using CUDA, or multiple CPU-bound
  processes if proving on CPU). A seeder ingesting a large corpus can saturate all
  available compute with parallel proving jobs.
- **Within a file**: RISC Zero's pipeline segments the execution trace and proves segments
  in parallel. The lift/join recursion is also parallelizable across segments. Multi-GPU
  proving (`RISC0_PROVER=actor`) distributes segment proving across GPUs for a single
  large file.
- **Across machines**: Since proof generation is a pure function of the file bytes (the
  output proof differs per run due to CSPRNG blinding, but any valid proof works), a fleet
  of proving machines can divide work by file. No coordination needed beyond "which files
  still need a proof."

For a deployment that needs to prove many files (e.g., a package mirror seeding thousands
of packages), the throughput bottleneck is total compute-hours, not serial proving time. A
single file's proof cannot be made faster than the pipeline allows, but many files can be
proved in parallel up to hardware limits. On CPU-only hardware, this means throwing more
cores/machines at the problem; with GPUs, it scales with GPU count.

---

## iroh Integration Details

### Current Architecture (post-v0.29 restructuring)

The iroh ecosystem underwent a major restructuring in v0.29 ("net is the new iroh"):

| Before v0.29                | After v0.29 (current)              |
|-----------------------------|------------------------------------|
| `iroh` = monolith           | `iroh` = networking only (was `iroh-net`) |
| `iroh-net` = networking     | `iroh-net` = **deprecated**        |
| Protocols bundled in `iroh` | `iroh-blobs`, `iroh-gossip`, `iroh-docs` = separate crates |
| "Node" abstraction          | "Router" abstraction               |
| Built-in CLI                | CLI **deprecated** (library only)  |

### Key Components for waddup

**`iroh` (v0.96.0)** — Pure networking library:
- `Endpoint`: establishes peer-to-peer QUIC connections
- `Router`: multiplexes multiple protocols on a single endpoint via ALPN
- `ProtocolHandler` trait: implement to define custom protocol behavior
- Built-in NAT traversal via relay servers + hole-punching (moving to QUIC-NAT-TRAVERSAL
  draft by Seemann/Huitema before 1.0)
- MagicSock: transparently switches between relay and direct paths
- QUIC Multipath (as of v0.95.1, Nov 2025)

**`iroh-blobs` (v0.98.0)** — Content-addressed blob transfer:
- Blobs identified by 32-byte BLAKE3 hash
- Bao verified streaming via `bao-tree` (v0.16.0) with 16 KiB chunk groups
- Stores data + outboard hash tree (`.obao4` files)
- Request-response protocol: request data by BLAKE3 hash and byte ranges
- BLAKE3 is the only supported hash function (hardcoded)
- Supports range downloads and resumption

### Custom Protocol for Proof Exchange

Custom protocols are registered on the Router with an ALPN identifier:

```rust
use iroh::{endpoint::Connection, protocol::{ProtocolHandler, AcceptError}};

const waddup_ALPN: &[u8] = b"waddup/proof-exchange/0";

#[derive(Debug, Clone)]
struct ProofExchangeProtocol { /* ... */ }

impl ProtocolHandler for ProofExchangeProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        // Handle incoming proof requests
        // connection.remote_id() returns the peer's NodeId (infallible)
        // connection.alpn() returns the negotiated ALPN (infallible)
        Ok(())
    }
}

let router = Router::builder(endpoint)
    .accept(waddup_ALPN, ProofExchangeProtocol { /* ... */ })
    .accept(iroh_blobs::ALPN, blobs_protocol)
    .spawn()
    .await?;
```

### Discovery Mechanisms

iroh supports composable discovery via `ConcurrentDiscovery`:

| Mechanism           | Default? | Notes |
|---------------------|----------|-------|
| DNS (Pkarr)         | Yes      | Publishes ed25519-signed DNS records to iroh-dns-server |
| Mainline DHT        | No       | Feature flag `discovery-pkarr-dht`; uses BitTorrent Mainline DHT |
| Local (mDNS)        | No       | Local Swarm Discovery for LAN scenarios |

For waddup, the Mainline DHT is the most relevant for permissionless operation — DNS
discovery defaults to n0-computer's server. DHT discovery allows fully decentralized peer
lookup. Multiple mechanisms can run simultaneously.

### Stability Considerations

- **Pre-1.0**: API still breaking between releases, but RC published Feb 2026
- **Wire protocol break**: v0.90+ nodes cannot communicate with v0.35.x nodes
- **Quinn fork**: iroh depends on a fork of Quinn for multipath/NAT traversal; diverges
  from upstream
- **iroh-blobs stability**: Team advises waiting "1 or 2 releases" before production
  deployment on new blobs
- **Positive signals**: 500K+ unique nodes/month, active development, formal protocol
  spec being drafted

---

## Protocol Extensions Required

### Discovery Layer

For Mainline DHT: standard Kademlia `get_peers`/`announce_peer` with arbitrary 20-byte keys.
Peers announce under `SHA1("waddup-v1:" || alg_id || ":" || H_content)` for 20-byte DHT
key compatibility. The algorithm identifier is folded into the key so that different
algorithms for the same file produce different swarms.

For DNS/Pkarr: would need a mapping from H_content to NodeIds, which could be published
as DNS records signed by seeder keys.

### Peer Protocol

One custom protocol extension for proof exchange during content-hash-based discovery:

```
content_proof_request {
    algorithm: uint8            // hash algorithm identifier (0=sha256, 1=sha1, 2=sha512, ...)
    content_hash: bytes         // H_content (length determined by algorithm)
}

content_proof_response {
    blake3_hash: bytes32        // BLAKE3 hash (= swarm identifier, always 32 bytes)
    groth16_seal: bytes         // Groth16 seal (256 bytes)
    receipt_claim: bytes        // ReceiptClaim (image_id, journal, exit status)
    file_length: uint64         // needed for Bao tree reconstruction
}
```

After verification (including checking that the proof's image_id matches the expected
guest program for the requested algorithm), the leecher uses `blake3_hash` to request
the blob via iroh-blobs' standard protocol.

---

## Cost Summary

| Cost                    | Who Pays              | When           | Magnitude                      |
|-------------------------|-----------------------|----------------|--------------------------------|
| ZK proof generation     | First seeder per file | Once ever      | Seconds to hours (CPU or GPU)  |
| ZK proof verification   | Every leecher         | Once on join   | ~1ms                           |
| BLAKE3 hash computation | Every seeder          | Once per file  | Seconds (native, no ZK)        |
| DHT announcement        | Every peer            | Periodic       | Negligible                     |
| Proof relay             | Every peer            | On connection  | ~256 bytes seal + small claim  |
| File transfer           | Standard P2P          | Ongoing        | File size                      |

The expensive operation (ZK proof generation) is paid once, by one participant, and amortized
across every future participant in the swarm for the lifetime of the file. Without a BLAKE3
precompile, proving cost is dominated by the BLAKE3 computation inside the zkVM; with an
application-defined BLAKE3 precompile (see Performance Considerations), both hash
computations would contribute roughly equally.

---

## Rust Crate Stack

| Need                | Crate              | Version | Notes                                      |
|---------------------|--------------------|---------|--------------------------------------------|
| P2P networking      | `iroh`             | 0.96.0  | QUIC, NAT traversal, Router, ProtocolHandler |
| Blob transfer       | `iroh-blobs`       | 0.98.0  | BLAKE3 + Bao verified streaming            |
| BLAKE3 hashing      | `blake3`           | 1.8+    | iroh dependency; also used standalone      |
| Bao tree            | `bao-tree`         | 0.16.0  | Used by iroh-blobs; standalone if needed   |
| ZK guest + host     | `risc0-zkvm`       | 3.0.5   | Guest-side program + host-side proving     |
| ZK build            | `risc0-build`      | 3.0.5   | Build dependency for guest ELF compilation |
| Groth16 compression | `risc0-groth16`    | 3.0.3   | STARK -> Groth16; requires x86 + Docker    |
| SHA-256             | `sha2`             | —       | Primary content hash (zkVM accelerator available) |
| SHA-1               | `sha1`             | —       | Legacy content hash (collision-broken, see Hash Agility) |

Note: `risc0-groth16` proving only works on x86 architecture. Apple Silicon is not
supported even via Docker. Verification works on all platforms. RISC Zero v5.0.0-rc.1 is
in development as of Jan 2026.

---

## Threat Model

### Scope

The publisher of H_content (package registry, website, etc.) is **out of scope** — if the
published hash is wrong, the system faithfully distributes the wrong file. The threat
model begins at the point where a leecher has a correct H_content and wants to retrieve the
file without trusting any individual peer.

Proof generation as a service (Boundless, etc.) is **out of scope** — all proving is done
locally by seeders who possess the file.

### Actors

| Actor                 | Has                        | Wants                                    |
|-----------------------|----------------------------|------------------------------------------|
| Honest seeder         | File, proof                | Distribute file to peers                 |
| Honest leecher        | H_content only             | Download the correct file                |
| Malicious peer        | Network presence           | Tamper, censor, surveil, or waste resources |
| Sybil operator        | Many fake identities       | Eclipse peers, poison discovery           |
| Network observer      | Passive tap on traffic     | Learn who downloads what                 |
| Infrastructure actor  | Relay server, DNS server   | Surveil or deny service                  |

---

### Data Integrity / Tampering

**Serve wrong file data**
- Attack: Malicious peer sends garbage bytes during blob transfer.
- Mitigation: **Fully mitigated by Bao.** Every 16 KiB chunk group is verified against the
  BLAKE3 Merkle tree rooted at H_swarm. A single wrong byte fails verification immediately.
  The leecher disconnects and tries another peer. No partial corruption is possible — Bao
  is not "check at the end," it's "check every chunk as it arrives."

**Serve wrong proof (invalid seal)**
- Attack: Malicious peer sends a fabricated Groth16 seal.
- Mitigation: **Fully mitigated by Groth16 verification.** The leecher verifies the seal
  against the hardcoded verification key in ~1ms. An invalid seal is rejected before any
  file transfer begins. Forging a valid Groth16 seal requires solving the discrete log
  problem on BN254 — computationally infeasible.

**Serve valid proof binding wrong hashes**
- Attack: Malicious peer has a legitimate proof for some other file and tries to pass it
  off as binding the leecher's H_content to some H_swarm.
- Mitigation: **Fully mitigated by journal check.** The leecher extracts H_content from
  the proof's journal and confirms it matches the H_content they started with. A proof for
  a different file will contain a different H_content in its journal. The proof cannot be
  altered without invalidating the seal.

**Serve valid proof, then serve a different file**
- Attack: Peer provides a correct proof (binding H_content to H_swarm) but then attempts
  to serve different data for the blob transfer.
- Mitigation: **Fully mitigated by Bao.** The leecher requests the blob by H_swarm. If
  the data doesn't match H_swarm, Bao chunk verification fails immediately.

**Hash collision on weak algorithm (SHA-1)**
- Attack: Attacker crafts two files with the same SHA-1 hash. Both produce valid proofs
  binding the same H_content to different H_swarm values. Leecher may find either swarm.
- Impact: Integrity break when using collision-vulnerable algorithms. See Hash Agility
  section for detailed analysis.
- Mitigation: **Partially mitigated.** Client warns when using broken algorithms. Prefer
  stronger algorithms when available. Multiple-hash binding (verify SHA-1 + SHA-256 if
  both are published). Not applicable to second-preimage (most real-world content).
- Residual risk: Accepted for legacy compatibility. SHA-256 and stronger algorithms are
  not affected.

**Tamper with proof in transit**
- Attack: MITM modifies proof bytes between peers.
- Mitigation: **Mitigated by QUIC.** All iroh connections use QUIC (TLS 1.3). Tampering
  requires breaking TLS. Additionally, any tampered proof fails Groth16 verification.

**Summary**: The combination of Groth16 proof verification (binds H_content to H_swarm) and
Bao verified streaming (binds transferred data to H_swarm) creates an unbroken verification
chain from the well-known hash to every byte received. There is no window in which a
malicious peer can serve wrong data that survives verification.

---

### Sybil Attacks

**DHT flooding (announce sybil peers for a content hash)**
- Attack: Sybil operator announces thousands of fake peers under the DHT key derived from
  H_content. Leechers connect to sybil nodes first, wasting time.
- Impact: Delays discovery. Each failed connection costs one QUIC handshake (~1 RTT) plus
  one proof exchange attempt (sybil node either sends no proof, an invalid proof, or hangs).
- Mitigation: **Proof verification as a cheap filter.** The leecher connects, requests a
  proof, and verifies in ~1ms. A sybil node that cannot produce a valid proof is discarded
  in milliseconds. The leecher should connect to multiple peers in parallel and use the
  first one that provides a valid proof.
- Residual risk: If the sybil operator controls enough DHT nodes to dominate the routing
  for a given key, the leecher may never discover real peers (eclipse attack — see below).

**Eclipse attack (isolate a leecher from real peers)**
- Attack: Sybil operator populates the DHT neighborhood around the target key with enough
  controlled nodes that honest peers' announcements are evicted or never returned.
- Impact: Leecher cannot find any honest seeder. Effective censorship of that content hash.
- Mitigation:
  - **Multi-mechanism discovery.** Use DHT + DNS/Pkarr + mDNS + peer exchange
    simultaneously. An eclipse attack must succeed across all mechanisms.
  - **Peer exchange (PEX).** Once a leecher finds ONE honest peer (via any mechanism), that
    peer can share its known peer list. This breaks the eclipse for all subsequent lookups.
  - **Out-of-band peer hints.** The publisher of H_content can optionally include a set of
    known seeder NodeIds or addresses alongside the hash, bootstrapping discovery without
    the DHT.
- Residual risk: A well-resourced attacker controlling DHT routing, DNS infrastructure, and
  all initial bootstrap peers could eclipse a leecher. This is the standard DHT threat and
  is not unique to this protocol.

**Sybil nodes that serve valid proofs but stall transfers**
- Attack: Sybil operator obtains a valid proof (by downloading it from an honest peer) and
  serves it to leechers. After proof verification succeeds, the sybil node accepts the
  blob request but sends data very slowly or hangs.
- Impact: Wastes leecher bandwidth and time. More effective than a no-proof sybil because
  the leecher commits to downloading after proof verification.
- Mitigation:
  - **Transfer timeouts and parallel downloads.** Leecher should set minimum throughput
    thresholds and drop slow peers. Connect to multiple proven peers simultaneously.
  - **iroh-blobs supports range requests.** Leecher can request different byte ranges from
    different peers, reducing dependence on any single peer.
  - **Proof-then-speed filtering.** After proof verification, rank peers by transfer speed
    and drop the slowest.
- Residual risk: Moderate. This is a resource-waste attack, not a data-integrity attack.
  It degrades performance but cannot cause wrong data to be accepted.

---

### Resource Waste / Denial of Service

**Connection flood on seeders**
- Attack: Attacker opens many connections to a seeder, exhausting its connection slots or
  bandwidth.
- Impact: Seeder becomes unavailable to honest leechers.
- Mitigation:
  - **Standard rate limiting.** iroh endpoints can limit concurrent connections.
  - **Proof exchange is cheap.** Serving a proof is ~256 bytes + claim metadata. The
    expensive resource is file transfer bandwidth, not proof serving.
  - **Multiple seeders.** If one seeder is DoS'd, others remain available.
- Residual risk: Standard P2P DoS concern. A single seeder can be overwhelmed. Swarm
  redundancy is the primary defense.

**Proof generation waste (trick a seeder into proving)**
- Attack: Could someone cause an honest seeder to waste GPU time generating a proof for a
  file nobody wants?
- Mitigation: **Not applicable.** Proof generation is a local decision by the seeder. The
  seeder decides when and whether to prove. No remote peer can trigger proof generation.
  A seeder only proves files it already has and wants to share.

**DHT announcement spam (announce for files you don't have)**
- Attack: Attacker announces on many content hash keys, making it look like they have
  everything. Leechers connect and waste a QUIC handshake.
- Impact: One wasted round-trip per fake announcement.
- Mitigation: **Proof verification filters this cheaply.** The fake peer cannot produce a
  valid proof for any file it doesn't have (and hasn't obtained the proof for). Cost to
  the leecher: one QUIC handshake (~1 RTT) + one failed proof check (~1ms).
- Residual risk: Annoyance-level. The asymmetry favors the leecher — the attacker must
  maintain many DHT entries; the leecher discards fakes in milliseconds.

**Relay abuse**
- Attack: Force all traffic through iroh relay servers to exhaust their bandwidth.
- Impact: Relay servers become bottleneck; peers that need relay (behind strict NATs) lose
  connectivity.
- Mitigation: iroh's MagicSock prefers direct connections when hole-punching succeeds.
  Self-hosted relay servers are possible. Relay is only a fallback.
- Residual risk: Peers behind symmetric NATs that cannot hole-punch depend on relay
  availability.

---

### Privacy / Surveillance

P2P systems inherently reveal participation through network-level metadata. iroh encrypts
content (QUIC/TLS 1.3) but does not hide who is communicating with whom. waddup's
approach to privacy is **defense in depth at the network layer** rather than protocol-level
anonymity: strong interface binding to prevent leaks, designed to work behind a consumer
VPN, with optional network namespace isolation for high-assurance deployments.

#### Design Position

waddup does not attempt to build anonymity into the P2P protocol itself. Instead:

1. **Strong interface binding.** The application binds all sockets (QUIC, DHT, DNS) to a
   specific network interface. If the VPN interface goes down, traffic does not fall back
   to the default route. This prevents the class of "VPN dropped and my real IP leaked"
   failures. This includes:
   - All iroh `Endpoint` traffic (QUIC connections to peers and relays)
   - All DHT queries and announcements
   - All DNS resolution (prevent OS-level DNS leaks outside the tunnel)
   - No fallback to system default route — if the bound interface is unavailable,
     operations fail rather than leak

2. **Consumer VPN as the recommended anonymity layer.** The documentation assumes users
   will run behind a VPN (Mullvad, etc.) for IP-level privacy. iroh's NAT traversal
   (relay + hole-punching) should work through most VPN tunnels without port forwarding —
   the VPN endpoint is just another NAT from iroh's perspective. MagicSock will use relay
   servers as fallback if hole-punching fails through the VPN.

3. **Network namespace isolation (Linux).** For users who want stronger guarantees than
   interface binding, the documentation covers running waddup inside a Linux network
   namespace that has only the VPN tunnel interface. This is bulletproof — if the VPN
   tunnel goes down, the namespace has no route out at all. No kernel bypass, no DNS leak,
   no fallback path. This is the recommended deployment for high-threat environments.

#### Remaining Exposure (Even With VPN)

A VPN hides the user's real IP from peers and DHT participants, but does not eliminate all
metadata exposure:

**Discovery queries reveal interest (to the DHT, not to ISP)**
- Attack: Observer participates in the DHT and sees queries for
  `SHA1("waddup-v1:" || H_content)`.
- Impact: Observer learns that *some VPN exit IP* is interested in a specific content hash.
  With a shared VPN exit IP (Mullvad, etc.), this is not directly attributable to a user.
- Residual risk: Timing correlation between VPN session and DHT query could deanonymize
  against a well-resourced adversary who controls both the VPN provider and DHT nodes.

**DHT announcements reveal possession (to the DHT, not to ISP)**
- Attack: Observer queries the DHT for a content hash and finds seeder addresses.
- Impact: Observer learns VPN exit IPs of seeders. Not directly attributable with a shared
  VPN exit.
- Residual risk: Long-lived seeder sessions may be correlatable across time even through
  VPN rotation.

**Peer connections reveal participation (to network observers)**
- Attack: Network observer sees encrypted QUIC traffic to/from the VPN tunnel.
- Impact: Observer sees that the user is communicating via VPN. Cannot see peer IPs, DHT
  queries, or content without compromising the VPN.
- Mitigation: QUIC inside VPN tunnel. ALPN is negotiated inside TLS, not visible to
  passive observers. Traffic volume reveals transfer sizes but not content identity.

**Relay servers see metadata**
- Attack: Relay operator logs which NodeIds communicate through the relay.
- Impact: Relay operator learns communication graph between NodeIds (ephemeral keys, not
  real identities), not content.
- Mitigation: Self-host relay servers. Or accept that relay sees only encrypted traffic
  metadata from a VPN exit IP.

**Content hash as surveillance key**
- A fundamental tension: H_content is public (it must be, for the system to work). Anyone
  who knows H_content can derive the DHT key, enumerate seeders, and query for leechers.
  This is not a bug — it's inherent to content-addressed P2P. The set of "people who know
  H_content" is the same set as "people who can surveil the swarm."
- Implication: If H_content is published on a public website, the swarm has zero privacy
  from any DHT-level observer. The VPN protects the mapping from "DHT participant" to
  "real human," but the swarm membership itself (by NodeId / VPN exit IP) is observable.

**Summary**: The protocol provides **data confidentiality** (encrypted transfers) and
**IP-level privacy** (via VPN + strong interface binding), but not **activity privacy**
(an observer on the DHT can see that *someone* is interested in a given content hash, just
not who). Full activity privacy in a P2P system requires an anonymity network (Tor, mixnet)
underneath the transport layer, which has severe performance implications for large file
transfers. The VPN approach is a pragmatic middle ground: it hides real IPs from all peers,
DHT participants, and relay operators, while acknowledging that the DHT itself leaks swarm
membership to observers.

---

### Censorship Resistance

The ZK proof layer provides a structural advantage for censorship resistance: the protocol
has two cheap verification steps that limit what a censor can accomplish.

**Data poisoning (serve bad data to pollute the swarm)**
- Attack: Censor joins the swarm and serves corrupted file data, hoping leechers accept it.
- Mitigation: **Impossible.** Bao verifies every chunk. Corrupted data is rejected at the
  chunk level, not at the end of transfer. The censor cannot even corrupt a single byte
  without detection.

**Proof poisoning (serve bad proofs to waste leecher time)**
- Attack: Censor serves invalid Groth16 seals, hoping leechers waste time on failed
  verification.
- Mitigation: **Negligible impact.** Groth16 verification takes ~1ms. A leecher can test
  and discard hundreds of invalid proofs per second. The censor gains almost nothing.

**Sybil censorship (eclipse real peers on the DHT)**
- Attack: Censor floods the DHT key space around a content hash with sybil nodes that
  either refuse to serve proofs or serve valid proofs but stall transfers.
- Impact: Most effective censorship strategy. Leechers who only use the DHT cannot find
  real peers.
- Mitigation:
  - **Multi-mechanism discovery** (DHT + DNS + mDNS + PEX). Censor must eclipse all
    mechanisms simultaneously.
  - **Peer exchange.** A single honest connection breaks the eclipse. Once a leecher finds
    one real peer, PEX reveals others.
  - **Out-of-band bootstrap.** Publisher includes known seeder addresses alongside H_content.
  - **Proof as peer authentication.** Even if the censor obtains a valid proof (by
    downloading it from a real peer), they still cannot serve the actual file data — Bao
    verification will fail when chunk data doesn't match H_swarm. The censor is limited to
    stalling (serving data slowly), which is detectable via throughput thresholds.

**Network-level blocking (block connections to known seeders)**
- Attack: ISP or state actor blocks IP addresses of known seeders.
- Impact: Leechers on that network cannot reach those seeders.
- Mitigation:
  - **iroh relay servers.** Connections can be relayed through relay servers, which look
    like ordinary QUIC/UDP traffic to arbitrary IPs. Blocking requires identifying and
    blocking relay servers.
  - **Relay server diversity.** Self-hosted relay servers with rotating IPs.
  - **NAT traversal / hole-punching.** Direct connections between peers behind NATs may
    evade IP-based blocking of known seeder addresses.
- Residual risk: A sufficiently motivated censor can block all known relay servers. This is
  the same arms race as Tor bridge distribution.

**Legal/takedown (DMCA, court orders targeting seeders)**
- Attack: Legal action against individual seeders forces them offline.
- Impact: Reduces swarm size.
- Mitigation: **Swarm redundancy.** Standard P2P defense — more seeders means more
  resilience. No single point of failure.
- Residual risk: Small swarms with few seeders are vulnerable to targeted takedowns.

**Cost asymmetry summary (defender vs censor)**

| Censor action            | Cost to censor              | Cost to leecher to detect/bypass |
|--------------------------|-----------------------------|----------------------------------|
| Serve invalid proof      | Network presence            | ~1ms verification                |
| Serve corrupted data     | Network + obtain proof      | Bao rejects per-chunk            |
| DHT sybil flood          | Many nodes, sustained       | Parallel connects + ~1ms filter  |
| DHT eclipse              | Dominate key neighborhood   | Multi-mechanism discovery        |
| Stall transfers          | Network + obtain proof      | Throughput timeout               |
| IP blocking              | Network control             | Relay servers                    |

The protocol's key structural advantage is that **proof verification is a 1ms gate** that
filters out the majority of sybil and poisoning attacks before any expensive operation
(file transfer) begins. An attacker must obtain a valid proof to get past this gate, and
even then, Bao prevents them from serving wrong data. The only remaining attack surface
is resource waste (stalling), which is detectable and mitigable by standard P2P techniques.

---

### Groth16 / Cryptographic Layer Threats

**Forge a Groth16 proof**
- Requires: Solving discrete log on BN254, or the trusted setup toxic waste.
- Mitigation: BN254 provides ~96 bits of security. Trusted setup used 1-of-N honest
  participant assumption across 238 contributions.
- Residual risk: Quantum computer breaks BN254. No known classical attack.

**Compromise the trusted setup**
- Attack: All 238 ceremony participants colluded or their toxic waste was recovered.
- Impact: Attacker can forge arbitrary proofs, binding any H_content to any H_swarm.
  This would allow serving wrong files that pass proof verification (though Bao would
  still verify chunks against H_swarm — the attack is making H_swarm point to a malicious
  file that genuinely hashes to that H_swarm under BLAKE3).
- Actual impact: Attacker could create a proof claiming ALG(malicious_file) = H_content
  AND BLAKE3(malicious_file) = H_swarm'. The leecher would accept this proof, download
  malicious_file, and believe it corresponds to H_content. This is a **complete integrity
  break**.
- Mitigation: 1-of-N honest participant. Ceremony conducted publicly. Hexens audited the
  circuit. The ceremony artifacts and contributions are publicly verifiable.
- Residual risk: Non-zero but extremely low. The 1-of-N assumption is strong.

**RISC Zero guest program vulnerability**
- Attack: A bug in the guest program (the Rust code computing ALG + BLAKE3) allows
  producing a valid proof for an incorrect hash binding.
- Impact: Same as setup compromise — integrity break.
- Mitigation: The guest program is simple (~20 lines of Rust: read bytes, compute two
  hashes, commit to journal). Auditable. The image_id is deterministic from the source
  code, so any party can rebuild and verify the guest binary matches expectations.
- Residual risk: Low. The guest program is simple enough to formally verify.

**RISC Zero zkVM vulnerability**
- Attack: A soundness bug in the zkVM allows forging proofs.
- Impact: Integrity break across ALL RISC Zero applications.
- Mitigation: 18 security audits across Hexens, Veridise, and zkSecurity. Formal
  verification efforts via Veridise's Picus tool. Critical vulnerability found and patched
  in Aug 2025 (v2.3.2/v3.0.3 — malicious host could write to arbitrary guest memory via
  sys_read). This demonstrates that bugs do happen and patches are shipped.
- Residual risk: Non-zero. Complex cryptographic systems have bugs. The STARK layer has
  post-quantum security; the Groth16 wrapper does not. A zkVM soundness bug would be
  catastrophic but is a shared risk with the entire RISC Zero ecosystem.

---

### Threat Summary

| Category             | Threat                        | Severity | Status              |
|----------------------|-------------------------------|----------|---------------------|
| **Integrity**        | Wrong file data               | Critical | Fully mitigated (Bao) |
|                      | Forged proof                  | Critical | Mitigated (Groth16)  |
|                      | Proof for wrong hash pair     | Critical | Mitigated (journal check) |
|                      | Hash collision (weak alg)     | High     | Warn + prefer strong alg |
|                      | Trusted setup compromise      | Critical | 1-of-N assumption    |
|                      | zkVM soundness bug            | Critical | Audited, residual risk |
| **Sybil**           | DHT flooding                  | Medium   | Cheap proof filter   |
|                      | Eclipse attack                | High     | Multi-mechanism discovery |
|                      | Stall-after-proof             | Medium   | Throughput timeouts  |
| **Resource waste**   | Connection flood on seeder    | Medium   | Rate limiting, swarm redundancy |
|                      | DHT announcement spam         | Low      | 1ms proof filter     |
| **Privacy**          | Discovery query surveillance  | High     | VPN hides real IP; DHT sees VPN exit |
|                      | Seeder enumeration            | High     | VPN hides real IP; DHT sees VPN exit |
|                      | Peer connection surveillance  | Medium   | VPN + interface binding |
|                      | IP leak on VPN drop           | High     | Interface binding + netns |
|                      | Relay metadata leakage        | Medium   | Self-host relays     |
| **Censorship**       | Data poisoning                | —        | Impossible (Bao)     |
|                      | Proof poisoning               | Low      | 1ms filter           |
|                      | DHT eclipse censorship        | High     | Multi-mechanism + PEX |
|                      | Network-level blocking        | High     | Relay servers        |

**Strongest property**: Data integrity. The Groth16 + Bao chain makes it cryptographically
infeasible to serve wrong data that survives verification.

**Weakest property**: Activity privacy. DHT-level observers can see swarm membership by
NodeId/VPN exit IP. VPN + interface binding hides real IPs but does not hide the fact that
*someone* is participating in a given swarm.

---

## Open Questions and Risks

1. **BLAKE3 precompile**: RISC Zero's zkVM 1.2 supports application-defined precompiles,
   which is the path to eliminating BLAKE3 as the proving bottleneck. Writing a BLAKE3
   precompile circuit is a bounded engineering effort (fixed spec, circuit-friendly
   structure) but is a prerequisite for practical proving times on large files. Without it,
   BLAKE3 dominates cycle counts by 10-100x over accelerated SHA-256.

2. **iroh pre-1.0 stability**: Wire protocol and API may still change before 1.0. Pinning
   to a specific version is recommended until the 1.0 release lands.

3. **Post-quantum**: The Groth16 layer depends on BN254 elliptic curve cryptography, which
   is not post-quantum secure. RISC Zero targets 96 bits of security for the on-chain
   verifier. The raw STARK path (post-quantum) exists as a fallback but with much larger
   proofs.

4. **Proof non-determinism**: Since proofs are not content-addressable, the protocol needs
   a "first valid proof wins" strategy. Multiple seeders independently proving the same
   file waste compute but cause no correctness issues.

5. **Discovery bootstrapping**: The Mainline DHT requires no infrastructure, but DNS/Pkarr
   discovery defaults to n0-computer's server. A fully decentralized deployment should
   enable the DHT backend.

6. **Privacy**: VPN + strong interface binding hides real IPs from all peers and DHT
   participants. DHT-level observers can still see swarm membership by VPN exit IP. Network
   namespace isolation provides the strongest guarantee against IP leaks. Users in
   high-threat environments should use the netns deployment model.
