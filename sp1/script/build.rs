use sp1_build::build_program_with_args;

fn main() {
    build_program_with_args("../programs/hash-prover", Default::default());
    build_program_with_args("../programs/sha256-only", Default::default());
    build_program_with_args("../programs/sha3-only", Default::default());
}
