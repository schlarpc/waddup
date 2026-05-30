# sp1-pr2615-patches

Five-commit patch series that forward-ports
[SP1 PR #2615 (BLAKE3 compress inner precompile)](https://github.com/succinctlabs/sp1/pull/2615)
from its v6.0.1 base onto SP1 v6.2.2 main.

The first four are the PR's own commits (cherry-picked with conflict
resolution for the v6.0.1→v6.2.2 chip ID, syscall enum, and machine wiring
changes). The fifth adapts the runtime + chip code to v6.2.2's new
`SyscallContext` / `SyscallRuntime` API and the `receive_syscall` signature
change.

Apply against a clean checkout of `succinctlabs/sp1` at tag `v6.2.2`:

```
git clone --branch v6.2.2 https://github.com/succinctlabs/sp1.git
cd sp1
git checkout -b port-blake3 v6.2.2
git am ../sp1-pr2615-patches/*.patch
```

## Soundness caveat — read this

The fifth commit's chip integration has **known under-constraint** in three
specific places:

1. `Blake3CompressControlChip::eval`'s `receive_syscall(...)` call passes
   `AB::Expr::zero()` for the `trap_code` arg that v6.2.2 added.
2. The chip is not generic over `SupervisorMode`/`UserMode` like the v6.2.2
   `ShaCompressControlChip` is; it effectively only supports trusted mode.
3. The PR predates v6.2.2's page-prot interaction architecture, so the chip
   doesn't emit the lookups the new prover expects on those buses.

These produce correct hash outputs but leave the proof under-constrained for
trap / user-mode / page-prot cases. Fine for benchmark numbers, NOT safe to
deploy. A real port needs to mirror what `sha256/compress/controller.rs`
does end-to-end and ideally pass an audit.
