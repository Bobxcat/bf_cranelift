use std::{
    io::{stdin, stdout, Read, Stdin, Stdout, Write},
    num::Wrapping,
    time::Instant,
};

use crate::{
    bf_ir::{BfIrScope, BfIrTok, MAX_CELL_COUNT},
    io_utils::{self, ProgramIO},
};

struct RtData<IO> {
    stdio: IO,
    data: Box<[Wrapping<u8>]>,
    data_ptr: usize,
}

impl<IO: ProgramIO> RtData<IO> {
    #[inline(always)]
    fn modify_data(&mut self, f: impl FnOnce(Wrapping<u8>) -> Wrapping<u8>) {
        self.data[self.data_ptr] = f(self.data[self.data_ptr])
    }

    fn run_scope(&mut self, sc: BfIrScope) {
        let mut ins_ptr = 0;

        loop {
            let Some(ins) = sc.get(ins_ptr) else { return };

            // Do not initialize, to force an assignment of the instruction pointer in every branch
            let new_ins_ptr: usize;

            match ins {
                // BfIrTok::Set(n) => {
                //     self.modify_data(|_| Wrapping(*n));
                //     new_ins_ptr = ins_ptr + 1;
                // }
                // BfIrTok::Add(n) => {
                //     self.modify_data(|prev| Wrapping(prev.0.wrapping_add_signed(n.0)));
                //     new_ins_ptr = ins_ptr + 1;
                // }
                // BfIrTok::PtrAdd(delta) => {
                //     // We wrap `data_ptr` around
                //     self.data_ptr = self
                //         .data_ptr
                //         .checked_add_signed(*delta)
                //         .unwrap_or(self.data.len() - 1)
                //         % self.data.len();
                //     new_ins_ptr = ins_ptr + 1;
                // }
                BfIrTok::Modify { adds, ptr_delta } => {
                    for (offset, delta) in adds {
                        todo!()
                    }

                    self.data_ptr = self
                        .data_ptr
                        .checked_add_signed(*ptr_delta)
                        .unwrap_or(self.data.len() - 1)
                        % self.data.len();
                    new_ins_ptr = ins_ptr + 1;
                }
                BfIrTok::Read => {
                    let mut new_val = 0;
                    self.stdio
                        .read_exact(std::array::from_mut(&mut new_val))
                        .unwrap();
                    self.modify_data(|_| Wrapping(new_val));
                    new_ins_ptr = ins_ptr + 1;
                }
                BfIrTok::Write => {
                    let mut to_write: u8 = 0;
                    self.modify_data(|x| {
                        to_write = x.0;
                        x
                    });
                    self.stdio.write_all(&[to_write]).unwrap();

                    new_ins_ptr = ins_ptr + 1;
                }
                BfIrTok::Loop(inner) => {
                    if self.data[self.data_ptr].0 != 0 {
                        self.run_scope(inner.clone());
                        new_ins_ptr = ins_ptr;
                    } else {
                        // Don't run
                        new_ins_ptr = ins_ptr + 1;
                    }
                }
            }
            ins_ptr = new_ins_ptr;
        }
    }
}

pub struct Interpreter<IO> {
    program: BfIrScope,
    data: RtData<IO>,
}

impl Interpreter<()> {
    pub fn new_stdio(program: BfIrScope) -> Interpreter<impl ProgramIO> {
        Interpreter::new(program, io_utils::stdio_triple())
    }
}

impl<IO> Interpreter<IO> {
    pub fn new(program: BfIrScope, io: IO) -> Interpreter<IO> {
        let data = vec![Wrapping(0); MAX_CELL_COUNT].into_boxed_slice();
        Self {
            program,
            data: RtData {
                stdio: io,
                data,
                data_ptr: 0,
            },
        }
    }
}

impl<IO> Interpreter<IO>
where
    IO: ProgramIO,
{
    pub fn with_stdout<X: Write>(self, w: X) -> Interpreter<impl ProgramIO> {
        Interpreter::new(self.program, self.data.stdio.with_stdout(w))
    }
    pub fn with_stdin<X: Read>(self, r: X) -> Interpreter<impl ProgramIO> {
        Interpreter::new(self.program, self.data.stdio.with_stdin(r))
    }
    pub fn run(&mut self) {
        let start = Instant::now();
        self.data.run_scope(self.program.clone());
        println!(";");
        println!("INTERPRETER_ELAPSED={:?}", start.elapsed());
    }
    /// Calls `self.run()` and drops `self`, as a helper in case of lifetime issues
    pub fn run_drop(mut self) {
        self.run()
    }
}
