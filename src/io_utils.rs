use std::{
    default,
    io::{self, sink, stderr, stdin, stdout, Read, Write},
    iter::Cycle,
    marker::PhantomData,
};

/// Returns a reader that cycles over the bytes over the slice
pub fn cycle_slice<'a>(sl: &'a [u8]) -> impl Read + 'a {
    ReadIter::new(sl.iter().copied().cycle())
}

pub trait ReadIterNew {
    fn new(i: impl Iterator<Item = u8>) -> ReadIter<impl Iterator<Item = u8>> {
        ReadIter(i)
    }
    fn empty() -> ReadIter<impl Iterator<Item = u8>> {
        ReadIter::new(std::iter::empty())
    }
    /// Emits a `0` for every read
    fn zero() -> ReadIter<impl Iterator<Item = u8>> {
        ReadIter::new(std::iter::repeat(0))
    }
}

impl ReadIterNew for ReadIter<()> {}

pub struct ReadIter<I>(I);

impl<I: Iterator<Item = u8>> Read for ReadIter<I> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read = 0;
        for b in buf {
            *b = match self.0.next() {
                Some(r) => r,
                None => break,
            };
            read += 1;
        }
        Ok(read)
    }
}

/// Creates a `StdioTriple` implementer which uses stdio for `in`, `out`, and `err`
pub fn stdio_triple() -> impl ProgramIO {
    ProgramStdio {
        stdin: stdin(),
        stdout: stdout(),
        stderr: stderr(),
        _p: PhantomData::<EOIEmit<0>>,
    }
}

pub fn io_triple<I: Read, O: Write, E: Write>(
    stdin: I,
    stdout: O,
    stderr: E,
) -> impl ProgramIO<Stdin = I, Stdout = O, Stderr = E> {
    ProgramStdio {
        stdin,
        stdout,
        stderr,
        _p: PhantomData::<EOIEmit<0>>,
    }
}

/// * `stdin` always emits a `0` byte
/// * `stdout`/`stderr` route to a sink
pub fn void() -> impl ProgramIO {
    ProgramStdio {
        stdin: ReadIter::zero(),
        stdout: sink(),
        stderr: sink(),
        _p: PhantomData::<EOIEmit<0>>,
    }
}

/// When EOI is detected, trigger a panic
pub struct EOIPanic;

impl ProgramEOI for EOIPanic {
    #[inline]
    fn get() -> Option<u8> {
        None
    }
}

/// When EOI is detected, continue by emitting `N` continuously.
///
/// The default `ProgramEOI` when not specified will always be `EOIEmit<0>`
pub struct EOIEmit<const N: u8>;

impl<const N: u8> ProgramEOI for EOIEmit<N> {
    #[inline]
    fn get() -> Option<u8> {
        Some(N)
    }
}

pub trait ProgramEOI {
    fn get() -> Option<u8>;
}

/// A bundle of std-`in/out/err` which is used for determing the inputs and outputs of a BF program
///
/// The `Read` impl reads from `stdin`, with customizable EOI behavior
///
/// The `Write` impl writes to `stdout`
///
/// To get a `Write`r for `stderr`, call `StdioTriple::stderr()`
pub trait ProgramIO: Read + Write + priv_impl::PrivateImpl + Sized {
    type Stdin: Read;
    type Stdout: Write;
    type Stderr: Write;

    fn stderr(&mut self) -> &mut Self::Stderr;

    fn with_eoi<EOI: ProgramEOI>(self) -> impl ProgramIO;

    fn with_stdin(self, stdin: impl Read) -> impl ProgramIO {
        self.map_stdin(|_| stdin)
    }
    fn with_stdout(self, stdout: impl Write) -> impl ProgramIO {
        self.map_stdout(|_| stdout)
    }
    fn with_stderr(self, stderr: impl Write) -> impl ProgramIO {
        self.map_stderr(|_| stderr)
    }

    fn map_stdin<R: Read>(self, stdin: impl FnOnce(Self::Stdin) -> R) -> impl ProgramIO;
    fn map_stdout<O: Write>(self, stdin: impl FnOnce(Self::Stdout) -> O) -> impl ProgramIO;
    fn map_stderr<E: Write>(self, stdin: impl FnOnce(Self::Stderr) -> E) -> impl ProgramIO;
}

mod priv_impl {
    /// You cannot implement this trait outside this crate
    pub trait PrivateImpl {}
}

struct ProgramStdio<I, O, E, EOI: ProgramEOI> {
    stdin: I,
    stdout: O,
    #[allow(unused)]
    stderr: E,
    _p: PhantomData<EOI>,
}

impl<I, O, E, EOI: ProgramEOI> priv_impl::PrivateImpl for ProgramStdio<I, O, E, EOI> {}

impl<I: Read, O, E, EOI: ProgramEOI> Read for ProgramStdio<I, O, E, EOI> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.stdin.read(buf) {
            Ok(0) => (),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => (),
            Ok(read) => return Ok(read),
            Err(e) => Err(e)?,
        }

        let byte = EOI::get().ok_or::<io::Error>(io::ErrorKind::UnexpectedEof.into())?;

        buf.fill(byte);
        Ok(buf.len())
    }
}

impl<I, O: Write, E, EOI: ProgramEOI> Write for ProgramStdio<I, O, E, EOI> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.flush()
    }
}

impl<I: Read, O: Write, E: Write, EOI: ProgramEOI> ProgramIO for ProgramStdio<I, O, E, EOI> {
    type Stdin = I;
    type Stdout = O;
    type Stderr = E;

    fn stderr(&mut self) -> &mut Self::Stderr {
        &mut self.stderr
    }

    fn with_eoi<U: ProgramEOI>(self) -> impl ProgramIO {
        ProgramStdio {
            stdin: self.stdin,
            stdout: self.stdout,
            stderr: self.stderr,
            _p: PhantomData::<U>,
        }
    }

    fn map_stdin<R: Read>(self, f: impl FnOnce(Self::Stdin) -> R) -> impl ProgramIO {
        ProgramStdio {
            stdin: f(self.stdin),
            stdout: self.stdout,
            stderr: self.stderr,
            _p: PhantomData::<EOI>,
        }
    }

    fn map_stdout<U: Write>(self, f: impl FnOnce(Self::Stdout) -> U) -> impl ProgramIO {
        ProgramStdio {
            stdin: self.stdin,
            stdout: f(self.stdout),
            stderr: self.stderr,
            _p: PhantomData::<EOI>,
        }
    }

    fn map_stderr<U: Write>(self, f: impl FnOnce(Self::Stderr) -> U) -> impl ProgramIO {
        ProgramStdio {
            stdin: self.stdin,
            stdout: self.stdout,
            stderr: f(self.stderr),
            _p: PhantomData::<EOI>,
        }
    }
}
