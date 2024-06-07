use std::io::Read;

use utf8_read::Reader;
use wasmparser::Parser;

use crate::bf::{BfScope, BfTok};

pub fn wasm2bf(mut wasm: impl Read) -> anyhow::Result<BfScope> {
    let mut buf = vec![];
    wasm.read_to_end(&mut buf)?;
    let wasm = buf;

    let mut parser = Parser::new(0);
    parser.parse(&wasm, true)?;

    todo!()
}
