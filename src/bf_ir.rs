//! OLD VERSION

use std::fmt::{Debug, Display};
use std::num::Wrapping;
use std::ops::Deref;
use std::sync::Arc;

use crate::bf::{BfParser, BfScope};

/// The maximum number of cells which is supported for a program.
/// Accessing any cell beyond this index results in undefined behavior
pub const MAX_CELL_COUNT: usize = 1024 * 1024;

// pub struct InsOffset(pub isize);

/// A cheaply clonable list of BFIR tokens
#[derive(Clone)]
pub struct BfIrScope {
    toks: Arc<[BfIrTok]>,
}

impl BfIrScope {
    pub fn parse(b: impl std::io::Read) -> anyhow::Result<Self> {
        Ok(Self::from_bf(BfParser::new(b).parse()?))
    }
    pub fn parse_sl(b: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        Self::parse(b.as_ref())
    }
    pub fn from_bf(bf: BfScope) -> Self {
        use crate::bf::BfTok;
        let toks = bf.to_vec();
        Self {
            toks: toks
                .into_iter()
                .map(|tok| match tok {
                    BfTok::Read => BfIrTok::Read,
                    BfTok::Write => BfIrTok::Write,
                    BfTok::ValInc => BfIrTok::Add(Wrapping(1)),
                    BfTok::ValDec => BfIrTok::Add(Wrapping(-1)),
                    BfTok::PtrInc => BfIrTok::PtrAdd(1),
                    BfTok::PtrDec => BfIrTok::PtrAdd(-1),
                    BfTok::Loop(s) => {
                        let s = Self::from_bf(s);
                        BfIrTok::Loop(s)
                    }
                })
                .collect(),
        }
    }
    #[must_use]
    pub fn modify(self, f: impl FnOnce(&mut Vec<BfIrTok>)) -> Self {
        // For some reason, this measured consistently faster than `self.toks.to_vec()`,
        // about %10 optimization time improvement
        let mut toks = Vec::from_iter(self.toks.iter().cloned());
        f(&mut toks);
        Self {
            toks: Arc::from(toks),
        }
    }
    /// Gets the number of tokens in
    pub fn len_flat(&self) -> usize {
        self.toks.len()
    }
    /// The recursive total number of tokens of this scope
    ///
    /// Each loop token is `1 + len(loop_body)`
    pub fn len(&self) -> usize {
        let mut i = 0;
        for tok in self.toks.iter() {
            match tok {
                BfIrTok::Loop(sc) => i += sc.len(),
                _ => (),
            }

            i += 1;
        }

        i
    }

    /// Returns the subscope of `self` (including `self`) with the largest `len_flat`
    pub fn largest_subscope(&self) -> BfIrScope {
        let mut largest = self.clone();
        for tok in &self[..] {
            match tok {
                BfIrTok::Loop(inner) => {
                    let inner = inner.largest_subscope();
                    if inner.len_flat() > largest.len_flat() {
                        largest = inner;
                    }
                }
                _ => (),
            }
        }
        largest
    }
}

impl Debug for BfIrScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.toks.iter()).finish()
    }
}

impl Display for BfIrScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let delta_indent = f.width().unwrap_or(2);
        let indent = f.precision().map(|p| p + delta_indent).unwrap_or(0);
        let pad = " ".repeat(indent);

        let s = self
            .toks
            .iter()
            .map(|tok| {
                format!(
                    "{pad}{tok:width$.precision$}",
                    width = delta_indent,
                    precision = indent
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "{s}")
    }
}

impl<B> From<B> for BfIrScope
where
    B: Into<Box<[BfIrTok]>>,
{
    fn from(value: B) -> Self {
        Self {
            toks: Arc::from(value.into()),
        }
    }
}

impl AsRef<[BfIrTok]> for BfIrScope {
    fn as_ref(&self) -> &[BfIrTok] {
        self.toks.as_ref()
    }
}

impl Deref for BfIrScope {
    type Target = [BfIrTok];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

/// A cheaply clonable BFIR token
#[derive(Debug, Clone)]
pub enum BfIrTok {
    Set(u8),
    Add(Wrapping<i8>),
    PtrAdd(isize),
    Read,
    Write,
    Loop(BfIrScope),
}

impl Display for BfIrTok {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BfIrTok::Set(n) => write!(f, "={n}")?,
            BfIrTok::Add(n) => write!(f, "+{n}")?,
            BfIrTok::PtrAdd(n) => write!(f, ">{n}")?,
            BfIrTok::Read => f.write_str(",")?,
            BfIrTok::Write => f.write_str(".")?,
            BfIrTok::Loop(lp) => f.write_fmt(format_args!(
                "{lp:w$.p$}",
                w = f.width().unwrap_or(2),
                p = f.precision().unwrap_or(0)
            ))?,
        };

        Ok(())
    }
}
