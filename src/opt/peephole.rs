use std::{any::type_name, time::Instant};

use crate::{
    bf::BfTok,
    bf_ir::{BfIrScope, BfIrTok},
};

pub enum PeepholeApply {
    /// Replace the first `count` instructions with `new`
    ///
    /// If there are not `count` instructions remaining, this may cause a panic
    Replace { count: usize, new: Vec<BfIrTok> },
    /// Do not replace any instructions
    Pass,
}

pub trait PeepholePass {
    /// The minimum number of tokens which `apply` will be called with.
    /// Loops are counted as a single instruction
    ///
    /// By default, this is `1`
    #[inline]
    fn min_tokens(&self) -> usize {
        1
    }

    fn apply<'a, 'b>(&'a mut self, instructions: &'b [BfIrTok]) -> PeepholeApply;
}

pub struct DataAddFold;

impl PeepholePass for DataAddFold {
    #[inline]
    fn min_tokens(&self) -> usize {
        2
    }

    #[inline]
    fn apply(&mut self, instructions: &[BfIrTok]) -> PeepholeApply {
        return todo!();
        #[cfg(not)]
        match instructions {
            [BfIrTok::Add(a), BfIrTok::Add(b), ..] => PeepholeApply::Replace {
                count: 2,
                new: vec![BfIrTok::Add(a + b)],
            },
            _ => PeepholeApply::Pass,
        }
    }
}

pub struct PtrAddFold;

impl PeepholePass for PtrAddFold {
    #[inline]
    fn min_tokens(&self) -> usize {
        2
    }

    #[inline]
    fn apply(&mut self, instructions: &[BfIrTok]) -> PeepholeApply {
        return todo!();
        #[cfg(not)]
        // As far as measured, there's not much of a performance gain to folding more than 2 at once
        match instructions {
            [BfIrTok::PtrAdd(a), BfIrTok::PtrAdd(b), ..] => PeepholeApply::Replace {
                count: 2,
                new: vec![BfIrTok::PtrAdd(
                    // Wrapping should never be needed
                    *a + *b,
                )],
            },
            _ => PeepholeApply::Pass,
        }
    }
}

/// Maps `[-]` to `set 0`
pub struct LoopSet0;

impl PeepholePass for LoopSet0 {
    #[inline]
    fn apply<'a, 'b>(&'a mut self, instructions: &'b [BfIrTok]) -> PeepholeApply {
        return todo!();
        #[cfg(not)]
        {
            let [BfIrTok::Loop(inner), ..] = instructions else {
                return PeepholeApply::Pass;
            };

            // Only allowed if this is the only token in the scope
            let [BfIrTok::Add(delta)] = &inner[..] else {
                return PeepholeApply::Pass;
            };

            // Does cell wrap from any value to `0` eventually, after enough additions (otherwise this could go on forever)
            // In other words, does `delta` share a factor with `256` (is `delta` a multiple of `2`)

            let does_terminate = match delta.0 {
                0 => false,
                1 => true,
                n => n % 2 == 0,
            };

            if does_terminate {
                PeepholeApply::Replace {
                    count: 1,
                    new: vec![BfIrTok::Set(0)],
                }
            } else {
                PeepholeApply::Pass
            }
        }
    }
}

/// Folds together adjacent `Set(x)` and `Add(y)` appropriately:
/// * `Set(x), Add(y)` => `Set(x + y)`
/// * `Add(x), Set(y)` => `Set(y)`
pub struct DataAddSetFold;

impl PeepholePass for DataAddSetFold {
    fn min_tokens(&self) -> usize {
        2
    }
    #[inline]
    fn apply<'a, 'b>(&'a mut self, instructions: &'b [BfIrTok]) -> PeepholeApply {
        return todo!();
        #[cfg(not)]
        match instructions {
            [BfIrTok::Add(_discard), BfIrTok::Set(y), ..] => PeepholeApply::Replace {
                count: 2,
                new: vec![BfIrTok::Set(*y)],
            },
            [BfIrTok::Set(x), BfIrTok::Add(y), ..] => PeepholeApply::Replace {
                count: 2,
                new: vec![BfIrTok::Set(x.wrapping_add_signed(y.0))],
            },
            _ => PeepholeApply::Pass,
        }
    }
}

/// Apply a peephole pass on every window of tokens in the program
fn apply_pass<P>(mut toks: BfIrScope, pass: &mut P) -> BfIrScope
where
    P: PeepholePass,
{
    let mut i = 0;
    loop {
        if i >= toks.len() {
            return toks;
        }

        // Apply repeatedly on this index until no more changes are made
        loop {
            let remaining_toks = toks.len() - i;
            if remaining_toks < pass.min_tokens() {
                return toks;
            }

            let Some(PeepholeApply::Replace { count, new }) =
                toks.get(i..).map(|toks| pass.apply(toks))
            else {
                break;
            };

            // println!("i={i},count={count},new={new:?}");

            if count > remaining_toks {
                println!(
                    "WARNING: `apply_pass` of `{}` returned a count of {count} with tokens: `len={},{:?}`",
                    type_name::<P>(),
                    remaining_toks,
                    &toks[i..]
                );
                break;
            }

            toks = toks.modify(|v| {
                let old = v.clone();

                let _replaced_elems = v.splice(i..i + count, new).collect::<Vec<_>>();

                if false {
                    println!(
                        "  i={i}\n  old={:?}\n  new={:?}",
                        old.get(0..3),
                        v.get(0..3)
                    );
                }
            });
        }

        // We want to allow running a peephole opt starting from and going across a loop. But, we also recursively apply the optimization
        if let Some(BfIrTok::Loop(inner)) = toks.get(i) {
            let new_loop = BfIrTok::Loop(apply_pass(inner.clone(), pass));
            toks = toks.modify(|v| v[i] = new_loop);
        }

        i += 1;
    }
}

/// Apply a peephole pass on every window of tokens in the program
///
/// Prints performance information
fn apply_pass_bench<P>(toks: BfIrScope, pass: &mut P) -> BfIrScope
where
    P: PeepholePass,
{
    let start = Instant::now();

    let sc = apply_pass(toks, pass);

    println!("PASS={}", type_name::<P>());
    println!("  ELAPSED={:?}", start.elapsed());

    sc
}

pub fn default_peephole_opt(toks: BfIrScope) -> BfIrScope {
    let start = Instant::now();

    let toks = apply_pass_bench(toks, &mut DataAddFold);
    let toks = apply_pass_bench(toks, &mut PtrAddFold);
    let toks = apply_pass_bench(toks, &mut LoopSet0);
    let toks = apply_pass_bench(toks, &mut DataAddSetFold);

    println!("OPT_ELAPSED={:?}", start.elapsed());

    toks
}
