use crate::codegen::{Code, Stmt};
use std::io::{Read, Write};
use std::num::Wrapping;

const MEM_SIZE: u32 = 32_000;

type Memory = [Wrapping<u8>; MEM_SIZE as usize];

// TODO maybe repr(C) to prevent field reordering?
struct Interpreter<'c, W, R> {
    code: &'c Code<'c>,
    ip: u32,
    ptr: u32,
    stdout: W,
    stdin: R,
    mem: Memory,
}

pub fn run<W, R>(code: &Code<'_>, stdout: W, stdin: R)
where
    W: Write,
    R: Read,
{
    let mut interpreter = Interpreter {
        code,
        ip: 0,
        ptr: 0,
        stdout,
        stdin,
        mem: [Wrapping(0u8); MEM_SIZE as usize],
    };

    // SAFETY: `Code` can only be produced by the `crate::codegen` module, which is trusted to not
    // produce out of bounds jumps and put the `End` at the end
    unsafe {
        interpreter.execute();
    }
}

impl<'c, W: Write, R: Read> Interpreter<'c, W, R> {
    unsafe fn execute(&mut self) {
        let stmts = self.code.stmts();
        loop {
            // SAFETY: If the code ends with an `End` and there are no out of bounds jumps,
            // `self.ip` will never be out of bounds
            // Removing this bounds check speeds up execution by about 40%
            debug_assert!((self.ip as usize) < stmts.len());
            let instr = unsafe { *stmts.get_unchecked(self.ip as usize) };
            self.ip += 1;
            match instr {
                Stmt::Add(n) => {
                    *self.elem_mut() += n;
                }
                Stmt::Sub(n) => {
                    *self.elem_mut() -= n;
                }
                Stmt::Right(n) => {
                    self.ptr += n;
                    if self.ptr >= MEM_SIZE {
                        self.ptr = 0;
                    }
                }
                Stmt::Left(n) => {
                    if self.ptr < n {
                        let diff = n - self.ptr;
                        self.ptr = MEM_SIZE - 1 - diff;
                    } else {
                        self.ptr -= n;
                    }
                }
                Stmt::Out => {
                    let char = self.elem() as char;
                    write!(self.stdout, "{char}").unwrap();
                    self.stdout.flush().unwrap();
                }
                Stmt::In => {
                    let mut buf = [0; 1];
                    self.stdin.read_exact(&mut buf).unwrap();
                    *self.elem_mut() = Wrapping(buf[0]);
                }
                Stmt::SetNull => {
                    *self.elem_mut() = Wrapping(0);
                }
                Stmt::JmpIfZero(pos) => {
                    if self.elem() == 0 {
                        self.ip = pos;
                    }
                }
                Stmt::JmpIfNonZero(pos) => {
                    if self.elem() != 0 {
                        self.ip = pos;
                    }
                }
                Stmt::End => break,
            }
        }
    }

    fn elem_mut(&mut self) -> &mut Wrapping<u8> {
        &mut self.mem[self.ptr as usize]
    }

    fn elem(&self) -> u8 {
        self.mem[self.ptr as usize].0
    }
}
