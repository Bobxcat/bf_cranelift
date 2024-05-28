use std::{
    collections::{HashMap, VecDeque},
    ffi::{OsStr, OsString},
    fs,
    io::{empty, Read},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    bf::BfParser,
    bf_ir::BfIrScope,
    interpret::Interpreter,
    io_utils::{self, ReadIter, ReadIterNew},
};

/// Parses bytes as a path and only returns the path if a file exists at the path
fn parse_bytes_as_path(b: &[u8], root: &str) -> Option<PathBuf> {
    let s = String::from_utf8_lossy(b);
    let s = &*s;
    // let s = String::from_utf8(b.to_vec()).ok()?;
    let path = PathBuf::from(root).join(PathBuf::from(&s));

    match fs::File::open(&path) {
        Ok(_) => Some(path),
        Err(e) => {
            println!(">>{}", e);
            None
        }
    }
}

pub struct TestCase {
    pub program: Vec<u8>,
    pub input: Vec<u8>,
    pub desired_out: Vec<u8>,
}

impl TestCase {
    #[track_caller]
    pub fn test(&self) {
        let program = BfIrScope::parse_sl(&self.program).unwrap();

        let mut stdout = Vec::new();
        let _res = Interpreter::new(
            program,
            io_utils::io_triple(
                ReadIter::new(self.input.iter().copied()),
                &mut stdout,
                empty(),
            ),
        )
        .run_drop();

        assert_eq!(
            self.desired_out, stdout,
            "Desired output (left) was not emitted by this test (right)"
        );
    }
}

#[test]
fn test_1_bf() {
    TestCase {
        program: include_bytes!("../bf_programs/test_1.bf").to_vec(),
        input: vec![],
        desired_out: b"\0Hello World! 255\n".to_vec(),
    }
    .test()
}

#[test]
fn _run_tests() {
    run_tests()
}

pub fn run_tests() {
    todo!("No longer valid: Need to rework `testing` folder");
    let dir = PathBuf::from("./testing/");
    // List of (`*.b`, `Option<*.in>`, `*.out`) pairs
    let mut file_pairs: Vec<(PathBuf, Option<PathBuf>, PathBuf)> = Vec::new();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        if entry.path().extension() == Some(OsStr::new("b")) {
            let entry_out_path = entry.path().with_extension("out");
            if let Ok(_entry_out) = fs::File::open(&entry_out_path) {
                let entry_in_path = {
                    let path = entry.path().with_extension("in");
                    match fs::File::open(&path) {
                        Ok(_f) => Some(path),
                        Err(_) => None,
                    }
                };
                file_pairs.push((entry.path(), entry_in_path, entry_out_path));
            }
        }
    }

    println!("Tests:\n  len={}", file_pairs.len());

    for (program, input, out) in file_pairs {
        println!(
            "Running: [program={}, in={:?}, out={}]",
            program.display(),
            input.clone().map(|x| x.display().to_string()),
            out.display()
        );
        let program = fs::read(program).unwrap();
        let input = {
            let raw = input.map(|p| fs::read(p).unwrap()).unwrap_or(vec![]);

            if let Some(path) = parse_bytes_as_path(&raw, "./testing/") {
                println!("  Found path!: {}", path.display());
                fs::read(path).unwrap()
            } else {
                raw
            }
        };
        let desired_out = fs::read(out).unwrap();

        print!("  Running...");
        TestCase {
            program,
            input,
            desired_out,
        }
        .test();

        println!("  Passed!");
    }
}
