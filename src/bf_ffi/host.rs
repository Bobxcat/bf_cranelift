//! Contains values used by a BF host running/using BF modules
//!

use std::{
    any::type_name,
    collections::HashMap,
    io::{Read, Write},
    thread::{self, JoinHandle},
};

use bimap::BiHashMap;
use smol_str::SmolStr;

use crate::byte_chan::{self, ByteRx, ByteTx};
pub struct BfImports {
    //
}

pub struct BfFunc {
    id: BfFuncId,
    func: Box<dyn FnMut() -> ()>,
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct BfFuncId(pub u16);

pub struct BfFuncs {
    funcs: HashMap<BfFuncId, BfFunc>,
    names: BiHashMap<BfFuncId, SmolStr>,
}

pub struct BfModule {
    //
}

/// A trait describing a runner for a BF program in any format
///
///
pub trait BfRunner<Program: Send>: Send + 'static {
    type Res: Send;

    /// Run a BF program which uses the following stdin and stdout channels
    fn run(self, p: Program, bf_stdin: ByteRx, bf_stdout: ByteTx) -> Self::Res;
}

impl<P, R, F> BfRunner<P> for F
where
    F: FnOnce(P, ByteRx, ByteTx) -> R + Send + 'static,
    R: Send,
    P: Send + 'static,
{
    type Res = R;

    fn run(self, p: P, bf_stdin: ByteRx, bf_stdout: ByteTx) -> Self::Res {
        (self)(p, bf_stdin, bf_stdout)
    }
}

/// Metadata about a BF program
struct BfLibMeta {
    imports: BfFuncs,
    exports: BfFuncs,
}

impl BfLibMeta {
    fn from_handshake(bfin: ByteTx, bfout: ByteRx) -> Option<Self> {
        todo!()
    }
}

/// A BF library middleware layer over a generic BF program and runner
///
/// This struct runs a generic BF program that implements the `bf_ffi` format and exposes
pub struct BfLib<Stdin, Stdout, P: Send, R: BfRunner<P>> {
    bfin: Stdin,
    bfout: Stdout,
    runner_handle: JoinHandle<R::Res>,
    meta: BfLibMeta,
}

impl<Stdin, Stdout, P, R> BfLib<Stdin, Stdout, P, R>
where
    Stdin: Read,
    Stdout: Write,
    P: Send + 'static,
    R: BfRunner<P> + 'static,
{
    pub fn new(bfin: Stdin, bfout: Stdout, program: P, runner: R) -> Self {
        let (runner_tx, rx) = byte_chan::bounded(10_000);
        let (tx, runner_rx) = byte_chan::bounded(10_000);

        let runner_handle = thread::spawn(move || runner.run(program, runner_rx, runner_tx));

        // Do the handshake with the BF program
        let meta = BfLibMeta::from_handshake(tx, rx).expect(&format!(
            "Handshake failed with BF Program: {}",
            type_name::<R>(),
        ));

        Self {
            bfin,
            bfout,
            runner_handle,
            meta,
        }
    }
}
