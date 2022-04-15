use crate::codegen::{Code, Stmt};
use std::io::{Read, Write};
use std::num::Wrapping;

const MEM_SIZE: usize = 32_000;

type Memory = [Wrapping<u8>; MEM_SIZE];

// TODO maybe repr(C) to prevent field reordering?
struct Interpreter<'c, W, R> {
    code: &'c Code<'c>,
    ip: usize,
    ptr: usize,
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
        mem: [Wrapping(0u8); MEM_SIZE],
    };

    interpreter.execute();
}

impl<'c, W: Write, R: Read> Interpreter<'c, W, R> {
    fn execute(&mut self) {
        loop {
            let instr = self.code.stmts[self.ip];
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
        &mut self.mem[self.ptr]
    }

    fn elem(&self) -> u8 {
        self.mem[self.ptr].0
    }
}
