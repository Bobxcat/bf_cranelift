//! Contains values used by a BF host running/using BF modules
//!

use std::{
    any::type_name,
    collections::{HashMap, HashSet},
    io::{Read, Write},
    thread::{self, JoinHandle},
};

use bimap::BiHashMap;
use byte_chan_active::{byte_chan, ByteRx, ByteTx};
use smol_str::SmolStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BfImportId(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BfExportId(u16);

pub struct BfImportHandler {
    id: BfImportId,
    func: Box<dyn FnMut() -> ()>,
}

pub struct BfImports {
    funcs: HashMap<BfImportId, BfImportHandler>,
    names: BiHashMap<BfImportId, SmolStr>,
}

pub struct BfExports {
    funcs: HashSet<BfImportId>,
    names: BiHashMap<BfImportId, SmolStr>,
}

/// A trait describing a runner for a generic BF program
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
    imports: BfImports,
    exports: BfExports,
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
        let (runner_tx, rx) = byte_chan();
        let (tx, runner_rx) = byte_chan();

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
    // pub fn get_function(&self, name: impl AsRef<str>)
}
