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

=== SP1+PR2615 BLAKE3 precompile bench === 2026-05-30T08:48:17Z
waddup SP1 BLAKE3-precompile bench (PR #2615 ported to v6.2.2)
===============================================================

--- SHA-256 (precompile) + BLAKE3 (PR #2615 precompile) | 1 KiB (1024 bytes) ---
  SHA-256 (journal):   785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 (native):    785b0751fc2c53dc14a4ce3d800e69ef9ce1009eb327ccf458afe09c242c26c9
  SHA-256 match:       OK
  BLAKE3  (journal):   882179b8dbccd285cda241d968cfcccb3156c5edac2fa3761bb6eda7ff8cb172
  BLAKE3  (native):    882179b8dbccd285cda241d968cfcccb3156c5edac2fa3761bb6eda7ff8cb172
  BLAKE3  match:       OK
  Total user cycles:   21954
  Cycle tracker regions:
    blake3:            3383 cycles (3.30 cycles/byte)
    sha256:            8673 cycles (8.47 cycles/byte)
  Syscall counts (non-zero):
    BLAKE3_COMPRESS_INNER      16
    SHA_COMPRESS               17
    SHA_EXTEND                 17
  Execute time:        9.251827ms
  Setup time:          1.968282889s
  Proving time:        26.580917511s
  Verification time:   132.870679ms

--- SHA-256 (precompile) + BLAKE3 (PR #2615 precompile) | 10 KiB (10240 bytes) ---
  SHA-256 (journal):   e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 (native):    e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  SHA-256 match:       OK
  BLAKE3  (journal):   398d42d5677b8bc40eb177ea45565bea3fc2fec2927c80304424c244435f9a78
  BLAKE3  (native):    398d42d5677b8bc40eb177ea45565bea3fc2fec2927c80304424c244435f9a78
  BLAKE3  match:       OK
  Total user cycles:   111232
  Cycle tracker regions:
    blake3:            33045 cycles (3.23 cycles/byte)
    sha256:            68289 cycles (6.67 cycles/byte)
  Syscall counts (non-zero):
    BLAKE3_COMPRESS_INNER      169
    SHA_COMPRESS               161
    SHA_EXTEND                 161
  Execute time:        13.916183ms
  Setup time:          1.987808024s
  Proving time:        28.645180254s
  Verification time:   139.140916ms

--- SHA-256 (precompile) + BLAKE3 (PR #2615 precompile) | 100 KiB (102400 bytes) ---
  SHA-256 (journal):   27783e87963a4efb6829b531c9ba57b44f45797f6770bd637fbf0d807cbdbae0
  SHA-256 (native):    27783e87963a4efb6829b531c9ba57b44f45797f6770bd637fbf0d807cbdbae0
  SHA-256 match:       OK
  BLAKE3  (journal):   9f05d49a48b2b787c8f7197ef8d5605978cae6eb8a445b46426b155e0accce44
  BLAKE3  (native):    9f05d49a48b2b787c8f7197ef8d5605978cae6eb8a445b46426b155e0accce44
  BLAKE3  match:       OK
  Total user cycles:   995732
  Cycle tracker regions:
    blake3:            321385 cycles (3.14 cycles/byte)
    sha256:            664449 cycles (6.49 cycles/byte)
  Syscall counts (non-zero):
    BLAKE3_COMPRESS_INNER      1699
    SHA_COMPRESS               1601
    SHA_EXTEND                 1601
  Execute time:        52.946461ms
  Setup time:          1.993677897s
  Proving time:        63.194825302s
  Verification time:   130.242468ms


# 1 MiB runs 2026-05-30T09:23:07Z
=== SP1 (direct syscalls, no BLAKE3 PC) | mode=both | size=1m ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
waddup SP1 ZK Proof-of-Concept Benchmark
=========================================
Prover: cpu
Mode:   SHA-256 (precompile) + BLAKE3 (software)

--- SHA-256 (precompile) + BLAKE3 (software) | 1 MiB (1048576 bytes) ---
  SHA-256 (journal):   fbbab289f7f94b25736c58be46a994c441fd02552cc6022352e3d86d2fab7c83
  SHA-256 (native):    fbbab289f7f94b25736c58be46a994c441fd02552cc6022352e3d86d2fab7c83
  SHA-256 match:       OK
  BLAKE3  (journal):   64479cf7293960210547db8d982359e0c4ce054525ed7086cf93030828fc0533
  BLAKE3  (native):    64479cf7293960210547db8d982359e0c4ce054525ed7086cf93030828fc0533
  BLAKE3  match:       OK
  Total user cycles:   33889038
  Cycle tracker regions:
    blake3:            27258340 cycles (26.00 cycles/byte)
    sha256:            6621105 cycles (6.31 cycles/byte)
  Syscall counts (non-zero):
    SHA_COMPRESS               16385
    SHA_EXTEND                 16385
  Execute time:        948.021469ms
[2m2026-05-30T09:23:58.183508Z[0m [33m WARN[0m Memory usage is high: 89.95%, we recommend using the prover network
[2m2026-05-30T09:24:08.186876Z[0m [33m WARN[0m Memory usage is high: 90.06%, we recommend using the prover network
[2m2026-05-30T09:24:18.187293Z[0m [33m WARN[0m Memory usage is high: 89.86%, we recommend using the prover network
[2m2026-05-30T09:24:28.187429Z[0m [33m WARN[0m Memory usage is high: 90.07%, we recommend using the prover network
[2m2026-05-30T09:24:38.189335Z[0m [33m WARN[0m Memory usage is high: 91.58%, we recommend using the prover network
[2m2026-05-30T09:24:48.189467Z[0m [33m WARN[0m Memory usage is high: 91.84%, we recommend using the prover network
[2m2026-05-30T09:24:58.189586Z[0m [33m WARN[0m Memory usage is high: 91.99%, we recommend using the prover network
[2m2026-05-30T09:25:08.189705Z[0m [33m WARN[0m Memory usage is high: 92.03%, we recommend using the prover network
[2m2026-05-30T09:25:18.189827Z[0m [33m WARN[0m Memory usage is high: 92.15%, we recommend using the prover network
[2m2026-05-30T09:25:28.189951Z[0m [33m WARN[0m Memory usage is high: 92.02%, we recommend using the prover network
[2m2026-05-30T09:25:38.190074Z[0m [33m WARN[0m Memory usage is high: 92.80%, we recommend using the prover network
[2m2026-05-30T09:25:48.190195Z[0m [33m WARN[0m Memory usage is high: 92.89%, we recommend using the prover network
[2m2026-05-30T09:25:58.190308Z[0m [33m WARN[0m Memory usage is high: 94.69%, we recommend using the prover network
[2m2026-05-30T09:26:08.191317Z[0m [33m WARN[0m Memory usage is high: 96.62%, we recommend using the prover network
=== SP1 + PR2615 BLAKE3 PC | size=1m | 2026-05-30T19:27:01Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
waddup SP1 BLAKE3-precompile bench (PR #2615 ported to v6.2.2)
===============================================================

--- SHA-256 (precompile) + BLAKE3 (PR #2615 precompile) | 1 MiB (1048576 bytes) ---
  SHA-256 (journal):   fbbab289f7f94b25736c58be46a994c441fd02552cc6022352e3d86d2fab7c83
  SHA-256 (native):    fbbab289f7f94b25736c58be46a994c441fd02552cc6022352e3d86d2fab7c83
  SHA-256 match:       OK
  BLAKE3  (journal):   e97ecfc21ba4c968fb4d39a42b3d5f05eef33a56221a85c1cd50de06988edd62
  BLAKE3  (native):    64479cf7293960210547db8d982359e0c4ce054525ed7086cf93030828fc0533
  BLAKE3  match:       MISMATCH!

thread 'main' (1076774) panicked at bench/script/src/main.rs:57:5:
assertion `left == right` failed
  left: [233, 126, 207, 194, 27, 164, 201, 104, 251, 77, 57, 164, 43, 61, 95, 5, 238, 243, 58, 86, 34, 26, 133, 193, 205, 80, 222, 6, 152, 142, 221, 98]
 right: [100, 71, 156, 247, 41, 57, 96, 33, 5, 71, 219, 141, 152, 35, 89, 224, 196, 206, 5, 69, 37, 237, 112, 134, 207, 147, 3, 8, 40, 252, 5, 51]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

=== SP1 + PR2615 BLAKE3 PC | size=1m (RERUN after BLAKE3 wrapper fix) | 2026-05-30T19:29:20Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
waddup SP1 BLAKE3-precompile bench (PR #2615 ported to v6.2.2)
===============================================================

--- SHA-256 (precompile) + BLAKE3 (PR #2615 precompile) | 1 MiB (1048576 bytes) ---
  SHA-256 (journal):   fbbab289f7f94b25736c58be46a994c441fd02552cc6022352e3d86d2fab7c83
  SHA-256 (native):    fbbab289f7f94b25736c58be46a994c441fd02552cc6022352e3d86d2fab7c83
  SHA-256 match:       OK
  BLAKE3  (journal):   64479cf7293960210547db8d982359e0c4ce054525ed7086cf93030828fc0533
  BLAKE3  (native):    64479cf7293960210547db8d982359e0c4ce054525ed7086cf93030828fc0533
  BLAKE3  match:       OK
  Total user cycles:   10084485
  Cycle tracker regions:
    blake3:            3289562 cycles (3.14 cycles/byte)
    sha256:            6785025 cycles (6.47 cycles/byte)
  Syscall counts (non-zero):
    BLAKE3_COMPRESS_INNER      17407
    SHA_COMPRESS               16385
    SHA_EXTEND                 16385
  Execute time:        511.913746ms
[2m2026-05-30T19:30:41.395379Z[0m [33m WARN[0m Memory usage is high: 94.46%, we recommend using the prover network
[2m2026-05-30T19:30:51.395742Z[0m [33m WARN[0m Memory usage is high: 94.44%, we recommend using the prover network
[2m2026-05-30T19:31:01.395878Z[0m [33m WARN[0m Memory usage is high: 94.46%, we recommend using the prover network
[2m2026-05-30T19:31:11.396016Z[0m [33m WARN[0m Memory usage is high: 96.07%, we recommend using the prover network
[2m2026-05-30T19:31:21.401534Z[0m [33m WARN[0m Memory usage is high: 96.00%, we recommend using the prover network
[2m2026-05-30T19:31:31.401682Z[0m [33m WARN[0m Memory usage is high: 95.92%, we recommend using the prover network
[2m2026-05-30T19:31:41.401817Z[0m [33m WARN[0m Memory usage is high: 95.91%, we recommend using the prover network
[2m2026-05-30T19:31:51.401947Z[0m [33m WARN[0m Memory usage is high: 95.95%, we recommend using the prover network
[2m2026-05-30T19:32:01.402408Z[0m [33m WARN[0m Memory usage is high: 98.71%, we recommend using the prover network
[2m2026-05-30T19:32:11.408321Z[0m [33m WARN[0m Memory usage is high: 98.73%, we recommend using the prover network
[2m2026-05-30T19:32:21.408476Z[0m [33m WARN[0m Memory usage is high: 98.88%, we recommend using the prover network
=== Decomposed | size=128k (2 chunks) | 2026-05-30T20:33:25Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
waddup decomposed bench
  file size:        131072 bytes (0 MiB)
  proof chunk size: 65536 bytes
  num chunk proofs: 2
  blake3 leaves:    128 (64 per chunk)
  expected SHA-256: 59f410ae5e17962412e2aed4f815918f634932f2abf084f00bb638c4db017850
  expected BLAKE3:  24e9cbe9f25292cd57991ccab25ccbfcd965456b59098d3c3f7ff68db31ab941

Setup phase:
  ProverClient init: 23.789391435s
  setup chunk pk:    2.212846575s
  setup root pk:     2.137044285s

Chunk proving phase (2 chunks):
  chunk   1/  2: 119.298470733s
  chunk   2/  2: 122.570730439s
  total: 241.869201172s, avg per chunk: 120.934600586s

Root proving phase:
  root prove: 151.667587716s

Results:
  H_content (journal): 59f410ae5e17962412e2aed4f815918f634932f2abf084f00bb638c4db017850
  H_content (native):  59f410ae5e17962412e2aed4f815918f634932f2abf084f00bb638c4db017850
  H_content match:     OK
  H_swarm   (journal): 24e9cbe9f25292cd57991ccab25ccbfcd965456b59098d3c3f7ff68db31ab941
  H_swarm   (native):  24e9cbe9f25292cd57991ccab25ccbfcd965456b59098d3c3f7ff68db31ab941
  H_swarm   match:     OK
  root verify: 38.418886ms

Wall-clock summary:
  setup:        28.139282295s
  chunk total:  241.869201172s
  chunk avg:    120.934600586s
  root prove:   151.667587716s
  root verify:  38.418886ms
  grand total:  421.714490069s
=== Decomposed | size=1m (16 chunks) | 2026-05-30T20:40:49Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
waddup decomposed bench
  file size:        1048576 bytes (1 MiB)
  proof chunk size: 65536 bytes
  num chunk proofs: 16
  blake3 leaves:    1024 (64 per chunk)
  expected SHA-256: fbbab289f7f94b25736c58be46a994c441fd02552cc6022352e3d86d2fab7c83
  expected BLAKE3:  64479cf7293960210547db8d982359e0c4ce054525ed7086cf93030828fc0533

Setup phase:
  ProverClient init: 24.521199152s
  setup chunk pk:    2.080419028s
  setup root pk:     2.146379407s

Chunk proving phase (16 chunks):
  chunk   1/ 16: 121.412607371s
  chunk   2/ 16: 118.871084365s
  chunk   3/ 16: 118.319165417s
  chunk   4/ 16: 118.182329753s
  chunk   5/ 16: 117.440821589s
  chunk   6/ 16: 117.836379225s
  chunk   7/ 16: 116.467164685s
  chunk   8/ 16: 117.861142867s
  chunk   9/ 16: 116.826810839s
  chunk  10/ 16: 117.68114738s
  chunk  11/ 16: 117.628871019s
  chunk  12/ 16: 118.528126706s
  chunk  13/ 16: 116.628721861s
  chunk  14/ 16: 117.837657413s
  chunk  15/ 16: 117.105202005s
  chunk  16/ 16: 116.868106591s
  total: 1885.495339086s, avg per chunk: 117.843458692s

Root proving phase:
[2m2026-05-30T21:14:40.013675Z[0m [33m WARN[0m Memory usage is high: 81.87%, we recommend using the prover network
[2m2026-05-30T21:15:20.014190Z[0m [33m WARN[0m Memory usage is high: 81.39%, we recommend using the prover network
[2m2026-05-30T21:15:30.014308Z[0m [33m WARN[0m Memory usage is high: 80.49%, we recommend using the prover network
[2m2026-05-30T21:15:40.014438Z[0m [33m WARN[0m Memory usage is high: 80.74%, we recommend using the prover network
[2m2026-05-30T21:15:50.016557Z[0m [33m WARN[0m Memory usage is high: 86.31%, we recommend using the prover network
  root prove: 385.947903104s

Results:
  H_content (journal): fbbab289f7f94b25736c58be46a994c441fd02552cc6022352e3d86d2fab7c83
  H_content (native):  fbbab289f7f94b25736c58be46a994c441fd02552cc6022352e3d86d2fab7c83
  H_content match:     OK
  H_swarm   (journal): 64479cf7293960210547db8d982359e0c4ce054525ed7086cf93030828fc0533
  H_swarm   (native):  64479cf7293960210547db8d982359e0c4ce054525ed7086cf93030828fc0533
  H_swarm   match:     OK
  root verify: 37.767563ms

Wall-clock summary:
  setup:        28.747997587s
  chunk total:  1885.495339086s
  chunk avg:    117.843458692s
  root prove:   385.947903104s
  root verify:  37.767563ms
  grand total:  2300.22900734s
=== RISC Zero | mode=both | size=1m | 2026-05-30T22:07:04Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
   Compiling methods v0.1.0 (/home/schlarpc/waddup/methods)
   Compiling host v0.1.0 (/home/schlarpc/waddup/host)
    Finished `release` profile [optimized] target(s) in 59.53s
     Running `target/release/host --both 1m`
waddup ZK Proof-of-Concept Benchmark
=====================================
Prover: CPU (local)
Mode:   SHA-256 (accelerated) + BLAKE3 (software)

--- SHA-256 (accelerated) + BLAKE3 (software) | 1 MiB (1048576 bytes) ---
  SHA-256 (journal):   fbbab289f7f94b25736c58be46a994c441fd02552cc6022352e3d86d2fab7c83
  SHA-256 (native):    fbbab289f7f94b25736c58be46a994c441fd02552cc6022352e3d86d2fab7c83
  SHA-256 match:       OK
  BLAKE3  (journal):   64479cf7293960210547db8d982359e0c4ce054525ed7086cf93030828fc0533
  BLAKE3  (native):    64479cf7293960210547db8d982359e0c4ce054525ed7086cf93030828fc0533
  BLAKE3  match:       OK
  Guest cycle breakdown:
    startup+read:      2395427 cycles
    sha256:            1143602 cycles (1.1 cycles/byte)
    blake3:            27992503 cycles (26.7 cycles/byte)
  Proving time:        3590.735067289s
  Verification time:   576.839142ms
  Total segments:      36
  Total cycles:        36831232
  User cycles:         31535756

=== Decomposed | size=10k | 2026-05-30T23:08:12Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
waddup decomposed bench
  file size:        10240 bytes
  total BLAKE3 leaves: 10
  num chunk proofs: 1
  leaves per chunk: [10]
  expected SHA-256: e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  expected BLAKE3:  398d42d5677b8bc40eb177ea45565bea3fc2fec2927c80304424c244435f9a78

Setup phase:
  ProverClient init: 23.936711661s
  setup chunk pk:    2.022997091s
  setup root pk:     2.025449385s

Chunk proving phase (1 chunks):
  chunk   1/  1 ( 10 leaves): 94.469225436s
  total: 94.469225436s, avg per chunk: 94.469225436s

Root proving phase:
  root prove: 114.469163024s

Results:
  H_content (journal): e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  H_content (native):  e96760a87768717bcebcfd25ddc7d46b4dbc95a4b0014def080c08539f7d90d0
  H_content match:     OK
  H_swarm   (journal): 398d42d5677b8bc40eb177ea45565bea3fc2fec2927c80304424c244435f9a78
  H_swarm   (native):  398d42d5677b8bc40eb177ea45565bea3fc2fec2927c80304424c244435f9a78
  H_swarm   match:     OK
  root verify: 37.228257ms

Wall-clock summary:
  setup:        27.985158137s
  chunk total:  94.469225436s
  chunk avg:    94.469225436s
  root prove:   114.469163024s
  root verify:  37.228257ms
  grand total:  236.960774854s
=== Decomposed | size=100k | 2026-05-30T23:12:23Z ===
warning: Git tree '/home/schlarpc/waddup' has uncommitted changes
waddup decomposed bench
  file size:        102400 bytes
  total BLAKE3 leaves: 100
  num chunk proofs: 2
  leaves per chunk: [64, 36]
  expected SHA-256: 27783e87963a4efb6829b531c9ba57b44f45797f6770bd637fbf0d807cbdbae0
  expected BLAKE3:  9f05d49a48b2b787c8f7197ef8d5605978cae6eb8a445b46426b155e0accce44

Setup phase:
  ProverClient init: 23.787111165s
  setup chunk pk:    2.016536016s
  setup root pk:     2.007238156s

Chunk proving phase (2 chunks):
  chunk   1/  2 ( 64 leaves): 115.716878313s
  chunk   2/  2 ( 36 leaves): 105.909860555s
  total: 221.626738868s, avg per chunk: 110.813369434s

Root proving phase:
  root prove: 146.183980112s

Results:
  H_content (journal): 27783e87963a4efb6829b531c9ba57b44f45797f6770bd637fbf0d807cbdbae0
  H_content (native):  27783e87963a4efb6829b531c9ba57b44f45797f6770bd637fbf0d807cbdbae0
  H_content match:     OK
  H_swarm   (journal): 9f05d49a48b2b787c8f7197ef8d5605978cae6eb8a445b46426b155e0accce44
  H_swarm   (native):  9f05d49a48b2b787c8f7197ef8d5605978cae6eb8a445b46426b155e0accce44
  H_swarm   match:     OK
  root verify: 38.599631ms

Wall-clock summary:
  setup:        27.810885337s
  chunk total:  221.626738868s
  chunk avg:    110.813369434s
  root prove:   146.183980112s
  root verify:  38.599631ms
  grand total:  395.660203948s
