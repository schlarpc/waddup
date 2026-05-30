use sp1_build::build_program_with_args;

fn main() {
    build_program_with_args("../chunk-program", Default::default());
    build_program_with_args("../root-program", Default::default());
}
