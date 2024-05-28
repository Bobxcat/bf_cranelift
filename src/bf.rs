use std::{fmt::Debug, io::Read, ops::Deref, sync::Arc};

use utf8_read::{Char, Reader};

/// A cheaply clonable list of BF tokens
#[derive(Clone)]
pub struct BfScope {
    toks: Arc<[BfTok]>,
}

impl Debug for BfScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.toks.iter()).finish()
    }
}

impl<B> From<B> for BfScope
where
    B: Into<Box<[BfTok]>>,
{
    fn from(value: B) -> Self {
        Self {
            toks: Arc::from(value.into()),
        }
    }
}

impl AsRef<[BfTok]> for BfScope {
    fn as_ref(&self) -> &[BfTok] {
        self.toks.as_ref()
    }
}

impl Deref for BfScope {
    type Target = [BfTok];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

/// A cheaply clonable BF token
#[derive(Debug, Clone)]
pub enum BfTok {
    Read,
    Write,
    ValInc,
    ValDec,
    PtrInc,
    PtrDec,
    Loop(BfScope),
}

/// A streaming BF parser, which wraps a `io::Read` instance
pub struct BfParser<R: Read> {
    src: Reader<R>,
}

impl<R: Read> BfParser<R> {
    pub fn new(src: R) -> Self {
        Self {
            src: Reader::new(src),
        }
    }
    pub fn parse(mut self) -> anyhow::Result<BfScope> {
        parse_stream(&mut self.src)
    }
}

fn parse_stream<R: Read>(chars: &mut Reader<R>) -> anyhow::Result<BfScope> {
    let mut toks = vec![];
    loop {
        let c = match chars.next_char()? {
            Char::Eof | Char::NoData => break,
            Char::Char(c) => c,
        };

        match c {
            '+' => toks.push(BfTok::ValInc),
            '-' => toks.push(BfTok::ValDec),
            '>' => toks.push(BfTok::PtrInc),
            '<' => toks.push(BfTok::PtrDec),
            '.' => toks.push(BfTok::Write),
            ',' => toks.push(BfTok::Read),
            // We don't need to keep track of depth for loops
            '[' => {
                let body = parse_stream(chars)?;
                toks.push(BfTok::Loop(body))
            }
            ']' => {
                break;
            }
            _ => (),
        }
    }

    Ok(toks.into())
}
