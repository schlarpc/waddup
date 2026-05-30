# Benchmark run 2026-05-30T07:02:08Z

=== RISC Zero | mode=sha256 | sizes=1k 10k | 2026-05-30T07:02:08Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
   Compiling methods v0.1.0 (/home/schlarpc/waddup/methods)
   Compiling host v0.1.0 (/home/schlarpc/waddup/host)
    Finished `release` profile [optimized] target(s) in 59.59s
     Running `target/release/host --sha256 1k 10k`
waddup ZK Proof-of-Concept Benchmark
=====================================
Prover: CPU (local)
Mode:   SHA-256 only (accelerated)

--- SHA-256 only (accelerated) | 1 KiB (1024 bytes) ---
  SHA-256 (journal):   785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 (native):    785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 match:       OK
  Guest cycle breakdown:
    startup+read:      4615 cycles
    sha256:            2217 cycles (2.2 cycles/byte)
  Proving time:        6.215589525s
  Verification time:   12.570342ms
  Total segments:      1
  Total cycles:        65536
  User cycles:         10997

--- SHA-256 only (accelerated) | 10 KiB (10240 bytes) ---
  SHA-256 (journal):   e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 (native):    e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 match:       OK
  Guest cycle breakdown:
    startup+read:      22196 cycles
    sha256:            12249 cycles (1.2 cycles/byte)
  Proving time:        12.645333335s
  Verification time:   13.283405ms
  Total segments:      1
  Total cycles:        131072
  User cycles:         38610


=== RISC Zero | mode=sha3 | sizes=1k 10k | 2026-05-30T07:02:08Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
   Compiling methods v0.1.0 (/home/schlarpc/waddup/methods)
   Compiling host v0.1.0 (/home/schlarpc/waddup/host)
    Finished `release` profile [optimized] target(s) in 59.36s
     Running `target/release/host --sha3 1k 10k`
waddup ZK Proof-of-Concept Benchmark
=====================================
Prover: CPU (local)
Mode:   SHA3-256 only (keccak precompile)

--- SHA3-256 only (keccak precompile) | 1 KiB (1024 bytes) ---
  SHA3-256 (journal):  b6c70631c6ff932b9f380d9cde8750eb9bea393817a9aea410c2119eb7b9b870
  SHA3-256 (native):   b6c70631c6ff932b9f380d9cde8750eb9bea393817a9aea410c2119eb7b9b870
  SHA3-256 match:      OK
  Guest cycle breakdown:
    startup+read:      4570 cycles
    sha3:              17114 cycles (16.7 cycles/byte)
  Proving time:        49.032969969s
  Verification time:   24.985838ms
  Total segments:      1
  Total cycles:        65536
  User cycles:         28251

--- SHA3-256 only (keccak precompile) | 10 KiB (10240 bytes) ---
  SHA3-256 (journal):  9decd08dec8ee6a9b092f56fc86ef044836a8dd32fa80a3c80ac7a5b9dfeeed8
  SHA3-256 (native):   9decd08dec8ee6a9b092f56fc86ef044836a8dd32fa80a3c80ac7a5b9dfeeed8
  SHA3-256 match:      OK
  Guest cycle breakdown:
    startup+read:      22153 cycles
    sha3:              159894 cycles (15.6 cycles/byte)
  Proving time:        68.730374078s
  Verification time:   26.335137ms
  Total segments:      1
  Total cycles:        262144
  User cycles:         188614


=== RISC Zero | mode=both | sizes=1k 10k | 2026-05-30T07:02:08Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
   Compiling methods v0.1.0 (/home/schlarpc/waddup/methods)
   Compiling host v0.1.0 (/home/schlarpc/waddup/host)
    Finished `release` profile [optimized] target(s) in 59.57s
     Running `target/release/host --both 1k 10k`
waddup ZK Proof-of-Concept Benchmark
=====================================
Prover: CPU (local)
Mode:   SHA-256 (accelerated) + BLAKE3 (software)

--- SHA-256 (accelerated) + BLAKE3 (software) | 1 KiB (1024 bytes) ---
  SHA-256 (journal):   785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 (native):    785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 match:       OK
  BLAKE3  (journal):   882179b8dbccd285cda241d968cfcccb3156c5edac2fa3761bb6eda7ff8cb172
  BLAKE3  (native):    882179b8dbccd285cda241d968cfcccb3156c5edac2fa3761bb6eda7ff8cb172
  BLAKE3  match:       OK
  Guest cycle breakdown:
    startup+read:      4763 cycles
    sha256:            2218 cycles (2.2 cycles/byte)
    blake3:            25822 cycles (25.2 cycles/byte)
  Proving time:        12.729350983s
  Verification time:   13.220561ms
  Total segments:      1
  Total cycles:        131072
  User cycles:         37027

--- SHA-256 (accelerated) + BLAKE3 (software) | 10 KiB (10240 bytes) ---
  SHA-256 (journal):   e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 (native):    e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 match:       OK
  BLAKE3  (journal):   398d42d5677b8bc40eb177ea45565bea3fc2fec2927c80304424c244435f9a78
  BLAKE3  (native):    398d42d5677b8bc40eb177ea45565bea3fc2fec2927c80304424c244435f9a78
  BLAKE3  match:       OK
  Guest cycle breakdown:
    startup+read:      22406 cycles
    sha256:            12250 cycles (1.2 cycles/byte)
    blake3:            271771 cycles (26.5 cycles/byte)
  Proving time:        53.139165861s
  Verification time:   14.937248ms
  Total segments:      1
  Total cycles:        524288
  User cycles:         310651


=== SP1 | mode=sha256 | sizes=1k 10k | 2026-05-30T07:02:08Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
waddup SP1 ZK Proof-of-Concept Benchmark
=========================================
Prover: cpu
Mode:   SHA-256 only (precompile)

--- SHA-256 only (precompile) | 1 KiB (1024 bytes) ---
  SHA-256 (journal):   785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 (native):    785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 match:       OK
  Total user cycles:   70719
  Cycle tracker regions:
    sha256:            65509 cycles (63.97 cycles/byte)
  Syscall counts:
    BLS12381_ADD               0
    BLS12381_DECOMPRESS        0
    BLS12381_DOUBLE            0
    BLS12381_FP2_ADD           0
    BLS12381_FP2_MUL           0
    BLS12381_FP2_SUB           0
    BLS12381_FP_ADD            0
    BLS12381_FP_MUL            0
  Execute time:        8.791622ms
  Setup time:          2.142457697s
  Proving time:        22.847084107s
  Verification time:   89.133916ms

--- SHA-256 only (precompile) | 10 KiB (10240 bytes) ---
  SHA-256 (journal):   e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 (native):    e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 match:       OK
  Total user cycles:   616047
  Cycle tracker regions:
    sha256:            610837 cycles (59.65 cycles/byte)
  Syscall counts:
    BLS12381_ADD               0
    BLS12381_DECOMPRESS        0
    BLS12381_DOUBLE            0
    BLS12381_FP2_ADD           0
    BLS12381_FP2_MUL           0
    BLS12381_FP2_SUB           0
    BLS12381_FP_ADD            0
    BLS12381_FP_MUL            0
  Execute time:        25.101397ms
  Setup time:          2.093695977s
  Proving time:        29.121872867s
  Verification time:   88.769057ms


=== SP1 | mode=sha3 | sizes=1k 10k | 2026-05-30T07:02:08Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
waddup SP1 ZK Proof-of-Concept Benchmark
=========================================
Prover: cpu
Mode:   SHA3-256 only (keccak precompile)

--- SHA3-256 only (keccak precompile) | 1 KiB (1024 bytes) ---
  SHA3-256 (journal):  b6c70631c6ff932b9f380d9cde8750eb9bea393817a9aea410c2119eb7b9b870
  SHA3-256 (native):   b6c70631c6ff932b9f380d9cde8750eb9bea393817a9aea410c2119eb7b9b870
  SHA3-256 match:      OK
  Total user cycles:   64574
  Cycle tracker regions:
    sha3:              59323 cycles (57.93 cycles/byte)
  Syscall counts:
    BLS12381_ADD               0
    BLS12381_DECOMPRESS        0
    BLS12381_DOUBLE            0
    BLS12381_FP2_ADD           0
    BLS12381_FP2_MUL           0
    BLS12381_FP2_SUB           0
    BLS12381_FP_ADD            0
    BLS12381_FP_MUL            0
  Execute time:        12.238718ms
  Setup time:          2.096014072s
  Proving time:        23.187105059s
  Verification time:   86.92876ms

--- SHA3-256 only (keccak precompile) | 10 KiB (10240 bytes) ---
  SHA3-256 (journal):  9decd08dec8ee6a9b092f56fc86ef044836a8dd32fa80a3c80ac7a5b9dfeeed8
  SHA3-256 (native):   9decd08dec8ee6a9b092f56fc86ef044836a8dd32fa80a3c80ac7a5b9dfeeed8
  SHA3-256 match:      OK
  Total user cycles:   567958
  Cycle tracker regions:
    sha3:              562707 cycles (54.95 cycles/byte)
  Syscall counts:
    BLS12381_ADD               0
    BLS12381_DECOMPRESS        0
    BLS12381_DOUBLE            0
    BLS12381_FP2_ADD           0
    BLS12381_FP2_MUL           0
    BLS12381_FP2_SUB           0
    BLS12381_FP_ADD            0
    BLS12381_FP_MUL            0
  Execute time:        24.008714ms
  Setup time:          2.110805707s
  Proving time:        27.194990128s
  Verification time:   85.686125ms


=== SP1 | mode=both | sizes=1k 10k | 2026-05-30T07:02:08Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
waddup SP1 ZK Proof-of-Concept Benchmark
=========================================
Prover: cpu
Mode:   SHA-256 (precompile) + BLAKE3 (software)

--- SHA-256 (precompile) + BLAKE3 (software) | 1 KiB (1024 bytes) ---
  SHA-256 (journal):   785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 (native):    785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 match:       OK
  BLAKE3  (journal):   882179b8dbccd285cda241d968cfcccb3156c5edac2fa3761bb6eda7ff8cb172
  BLAKE3  (native):    882179b8dbccd285cda241d968cfcccb3156c5edac2fa3761bb6eda7ff8cb172
  BLAKE3  match:       OK
  Total user cycles:   100329
  Cycle tracker regions:
    blake3:            25221 cycles (24.63 cycles/byte)
    sha256:            65509 cycles (63.97 cycles/byte)
  Syscall counts:
    BLS12381_ADD               0
    BLS12381_DECOMPRESS        0
    BLS12381_DOUBLE            0
    BLS12381_FP2_ADD           0
    BLS12381_FP2_MUL           0
    BLS12381_FP2_SUB           0
    BLS12381_FP_ADD            0
    BLS12381_FP_MUL            0
  Execute time:        9.391134ms
  Setup time:          2.140290659s
  Proving time:        24.243368352s
  Verification time:   87.387239ms

--- SHA-256 (precompile) + BLAKE3 (software) | 10 KiB (10240 bytes) ---
  SHA-256 (journal):   e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 (native):    e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 match:       OK
  BLAKE3  (journal):   398d42d5677b8bc40eb177ea45565bea3fc2fec2927c80304424c244435f9a78
  BLAKE3  (native):    398d42d5677b8bc40eb177ea45565bea3fc2fec2927c80304424c244435f9a78
  BLAKE3  match:       OK
  Total user cycles:   885074
  Cycle tracker regions:
    blake3:            264638 cycles (25.84 cycles/byte)
    sha256:            610837 cycles (59.65 cycles/byte)
  Syscall counts:
    BLS12381_ADD               0
    BLS12381_DECOMPRESS        0
    BLS12381_DOUBLE            0
    BLS12381_FP2_ADD           0
    BLS12381_FP2_MUL           0
    BLS12381_FP2_SUB           0
    BLS12381_FP_ADD            0
    BLS12381_FP_MUL            0
  Execute time:        30.19746ms
  Setup time:          2.119671321s
  Proving time:        32.576180457s
  Verification time:   94.961367ms


=== RISC Zero | mode=both | sizes=100k | 2026-05-30T07:02:08Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
   Compiling methods v0.1.0 (/home/schlarpc/waddup/methods)
   Compiling host v0.1.0 (/home/schlarpc/waddup/host)
    Finished `release` profile [optimized] target(s) in 59.97s
     Running `target/release/host --both 100k`
waddup ZK Proof-of-Concept Benchmark
=====================================
Prover: CPU (local)
Mode:   SHA-256 (accelerated) + BLAKE3 (software)

--- SHA-256 (accelerated) + BLAKE3 (software) | 100 KiB (102400 bytes) ---
  SHA-256 (journal):   27783e87963a4efb6829b531c9ba57b44f45797f6770bd637fbf0d807cbdbae0
  SHA-256 (native):    27783e87963a4efb6829b531c9ba57b44f45797f6770bd637fbf0d807cbdbae0
  SHA-256 match:       OK
  BLAKE3  (journal):   9f05d49a48b2b787c8f7197ef8d5605978cae6eb8a445b46426b155e0accce44
  BLAKE3  (native):    9f05d49a48b2b787c8f7197ef8d5605978cae6eb8a445b46426b155e0accce44
  BLAKE3  match:       OK
  Guest cycle breakdown:
    startup+read:      157206 cycles
    sha256:            112654 cycles (1.1 cycles/byte)
    blake3:            2732191 cycles (26.7 cycles/byte)
  Proving time:        339.84337681s
  Verification time:   64.060914ms
  Total segments:      4
  Total cycles:        3407872
  User cycles:         3006275


=== SP1 | mode=both | sizes=100k | 2026-05-30T07:02:08Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
waddup SP1 ZK Proof-of-Concept Benchmark
=========================================
Prover: cpu
Mode:   SHA-256 (precompile) + BLAKE3 (software)

--- SHA-256 (precompile) + BLAKE3 (software) | 100 KiB (102400 bytes) ---
  SHA-256 (journal):   27783e87963a4efb6829b531c9ba57b44f45797f6770bd637fbf0d807cbdbae0
  SHA-256 (native):    27783e87963a4efb6829b531c9ba57b44f45797f6770bd637fbf0d807cbdbae0
  SHA-256 match:       OK
  BLAKE3  (journal):   9f05d49a48b2b787c8f7197ef8d5605978cae6eb8a445b46426b155e0accce44
  BLAKE3  (native):    9f05d49a48b2b787c8f7197ef8d5605978cae6eb8a445b46426b155e0accce44
  BLAKE3  match:       OK
  Total user cycles:   8734244
  Cycle tracker regions:
    blake3:            2660528 cycles (25.98 cycles/byte)
    sha256:            6064117 cycles (59.22 cycles/byte)
  Syscall counts:
    BLS12381_ADD               0
    BLS12381_DECOMPRESS        0
    BLS12381_DOUBLE            0
    BLS12381_FP2_ADD           0
    BLS12381_FP2_MUL           0
    BLS12381_FP2_SUB           0
    BLS12381_FP_ADD            0
    BLS12381_FP_MUL            0
  Execute time:        244.792668ms
  Setup time:          2.119463066s
  Proving time:        124.502146965s
  Verification time:   136.293525ms



# SP1 RERUN with fixed precompile dispatch
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
=== SP1-FIXED | mode=sha256 | sizes=1k 10k | 2026-05-30T07:40:21Z ===
waddup SP1 ZK Proof-of-Concept Benchmark
=========================================
Prover: cpu
Mode:   SHA-256 only (precompile)

--- SHA-256 only (precompile) | 1 KiB (1024 bytes) ---
  SHA-256 (journal):   785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 (native):    785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 match:       OK
  Total user cycles:   13647
  Cycle tracker regions:
    sha256:            8433 cycles (8.24 cycles/byte)
  Syscall counts (non-zero):
    SHA_COMPRESS               17
    SHA_EXTEND                 17
  Execute time:        6.231906ms
  Setup time:          2.081917347s
  Proving time:        23.75981682s
  Verification time:   94.202079ms

--- SHA-256 only (precompile) | 10 KiB (10240 bytes) ---
  SHA-256 (journal):   e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 (native):    e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 match:       OK
  Total user cycles:   71823
  Cycle tracker regions:
    sha256:            66609 cycles (6.50 cycles/byte)
  Syscall counts (non-zero):
    SHA_COMPRESS               161
    SHA_EXTEND                 161
  Execute time:        11.132137ms
  Setup time:          2.090386623s
  Proving time:        23.90569148s
  Verification time:   88.046609ms


=== SP1-FIXED | mode=sha3 | sizes=1k 10k | 2026-05-30T07:41:38Z ===
waddup SP1 ZK Proof-of-Concept Benchmark
=========================================
Prover: cpu
Mode:   SHA3-256 only (keccak precompile)

--- SHA3-256 only (keccak precompile) | 1 KiB (1024 bytes) ---
  SHA3-256 (journal):  b6c70631c6ff932b9f380d9cde8750eb9bea393817a9aea410c2119eb7b9b870
  SHA3-256 (native):   b6c70631c6ff932b9f380d9cde8750eb9bea393817a9aea410c2119eb7b9b870
  SHA3-256 match:      OK
  Total user cycles:   8873
  Cycle tracker regions:
    sha3:              3609 cycles (3.52 cycles/byte)
  Syscall counts (non-zero):
    KECCAK_PERMUTE             8
  Execute time:        6.320251ms
  Setup time:          2.066448055s
  Proving time:        27.870901489s
  Verification time:   136.991754ms

--- SHA3-256 only (keccak precompile) | 10 KiB (10240 bytes) ---
  SHA3-256 (journal):  9decd08dec8ee6a9b092f56fc86ef044836a8dd32fa80a3c80ac7a5b9dfeeed8
  SHA3-256 (native):   9decd08dec8ee6a9b092f56fc86ef044836a8dd32fa80a3c80ac7a5b9dfeeed8
  SHA3-256 match:      OK
  Total user cycles:   38101
  Cycle tracker regions:
    sha3:              32837 cycles (3.21 cycles/byte)
  Syscall counts (non-zero):
    KECCAK_PERMUTE             76
  Execute time:        6.450066ms
  Setup time:          2.063493738s
  Proving time:        29.87120067s
  Verification time:   142.775758ms


=== SP1-FIXED | mode=both | sizes=1k 10k | 2026-05-30T07:43:05Z ===
waddup SP1 ZK Proof-of-Concept Benchmark
=========================================
Prover: cpu
Mode:   SHA-256 (precompile) + BLAKE3 (software)

--- SHA-256 (precompile) + BLAKE3 (software) | 1 KiB (1024 bytes) ---
  SHA-256 (journal):   785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 (native):    785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 match:       OK
  BLAKE3  (journal):   882179b8dbccd285cda241d968cfcccb3156c5edac2fa3761bb6eda7ff8cb172
  BLAKE3  (native):    882179b8dbccd285cda241d968cfcccb3156c5edac2fa3761bb6eda7ff8cb172
  BLAKE3  match:       OK
  Total user cycles:   43255
  Cycle tracker regions:
    blake3:            25229 cycles (24.64 cycles/byte)
    sha256:            8433 cycles (8.24 cycles/byte)
  Syscall counts (non-zero):
    SHA_COMPRESS               17
    SHA_EXTEND                 17
  Execute time:        7.641603ms
  Setup time:          2.120769728s
  Proving time:        23.709624083s
  Verification time:   93.681996ms

--- SHA-256 (precompile) + BLAKE3 (software) | 10 KiB (10240 bytes) ---
  SHA-256 (journal):   e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 (native):    e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 match:       OK
  BLAKE3  (journal):   398d42d5677b8bc40eb177ea45565bea3fc2fec2927c80304424c244435f9a78
  BLAKE3  (native):    398d42d5677b8bc40eb177ea45565bea3fc2fec2927c80304424c244435f9a78
  BLAKE3  match:       OK
  Total user cycles:   340848
  Cycle tracker regions:
    blake3:            264646 cycles (25.84 cycles/byte)
    sha256:            66609 cycles (6.50 cycles/byte)
  Syscall counts (non-zero):
    SHA_COMPRESS               161
    SHA_EXTEND                 161
  Execute time:        16.415392ms
  Setup time:          2.109001459s
  Proving time:        27.912002805s
  Verification time:   91.167883ms


=== SP1-FIXED | mode=both | sizes=100k | 2026-05-30T07:44:26Z ===
waddup SP1 ZK Proof-of-Concept Benchmark
=========================================
Prover: cpu
Mode:   SHA-256 (precompile) + BLAKE3 (software)

--- SHA-256 (precompile) + BLAKE3 (software) | 100 KiB (102400 bytes) ---
  SHA-256 (journal):   27783e87963a4efb6829b531c9ba57b44f45797f6770bd637fbf0d807cbdbae0
  SHA-256 (native):    27783e87963a4efb6829b531c9ba57b44f45797f6770bd637fbf0d807cbdbae0
  SHA-256 match:       OK
  BLAKE3  (journal):   9f05d49a48b2b787c8f7197ef8d5605978cae6eb8a445b46426b155e0accce44
  BLAKE3  (native):    9f05d49a48b2b787c8f7197ef8d5605978cae6eb8a445b46426b155e0accce44
  BLAKE3  match:       OK
  Total user cycles:   3318498
  Cycle tracker regions:
    blake3:            2660536 cycles (25.98 cycles/byte)
    sha256:            648369 cycles (6.33 cycles/byte)
  Syscall counts (non-zero):
    SHA_COMPRESS               1601
    SHA_EXTEND                 1601
  Execute time:        106.151446ms
  Setup time:          2.21715879s
  Proving time:        73.026682282s
  Verification time:   92.023136ms

