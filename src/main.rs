use std::{
    fs::File,
    io::{empty, sink, Read, Write},
    path::PathBuf,
};

use bf_cranelift::{
    bf::{BfParser, BfScope},
    bf_ir::{BfIrScope, BfIrTok},
    bf_ir2,
    interpret::Interpreter,
    io_utils::{self, void, ProgramIO, ReadIter, ReadIterNew},
    opt::peephole,
};

fn opt_run(b: impl AsRef<[u8]>, io: impl ProgramIO) {
    let program = BfIrScope::parse_sl(b).unwrap();
    println!("parsed!");
    println!("  len={}", program.len());
    println!("  largest_scope={}", program.largest_subscope().len_flat());

    let program = peephole::default_peephole_opt(program);

    println!("optimized!");
    println!("  len={}", program.len());
    println!("  largest_scope={}", program.largest_subscope().len_flat());

    print!("stdout:");
    let mut interp = Interpreter::new(program, io);
    interp.run();
    println!("=====\n");
}

const TEST_1: &[u8] = include_bytes!("../bf_programs/test_1.bf");
const AWIB: &[u8] = include_bytes!("../bf_programs/awib-0.4.bf");
const AWIB_AS_C: &str = "./bf_programs/target/awib-0.4.c";
const AWIB_TARG: &str = "./bf_programs/target/awib-0.4_target.c";

const EASY_OPT: &[u8] = include_bytes!("../bf_programs/EasyOpt.b");

fn main() {
    let p = bf_ir2::BfIrScope::parse_sl(TEST_1).unwrap();
    println!("{p}");

    return;

    opt_run(TEST_1, void());
    opt_run(
        AWIB,
        io_utils::io_triple(
            ReadIter::new(TEST_1.iter().copied()),
            File::create(AWIB_TARG).unwrap(),
            sink(),
        ),
    );
    opt_run(
        AWIB,
        io_utils::io_triple(
            ReadIter::new(AWIB.iter().copied()),
            File::create(AWIB_AS_C).unwrap(),
            sink(),
        ),
    );
}
