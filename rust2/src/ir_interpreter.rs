use crate::opts::{Ir, Stmt};
use std::io::{Read, Write};
use std::num::Wrapping;

const MEM_SIZE: usize = 32_000;

type Memory = [Wrapping<u8>; MEM_SIZE];

pub fn run<W, R>(instrs: &Ir<'_>, mut stdout: W, mut stdin: R)
where
    W: Write,
    R: Read,
{
    let mut mem = [Wrapping(0u8); MEM_SIZE];
    let mut ptr = 0;

    execute(&mut mem, &mut ptr, &instrs.stmts, &mut stdout, &mut stdin);
}

fn execute<W, R>(
    mem: &mut Memory,
    ptr: &mut usize,
    instrs: &[Stmt<'_>],
    stdout: &mut W,
    stdin: &mut R,
) where
    W: Write,
    R: Read,
{
    for instr in instrs {
        match instr {
            Stmt::Add(n) => {
                mem[*ptr] += n;
            }
            Stmt::Sub(n) => {
                mem[*ptr] -= n;
            }
            Stmt::Right(n) => {
                *ptr += n;
                if *ptr >= MEM_SIZE {
                    *ptr = 0;
                }
            }
            Stmt::Left(n) => {
                if *ptr < *n {
                    let diff = *n - *ptr;
                    *ptr = MEM_SIZE - 1 - diff;
                } else {
                    *ptr -= n;
                }
            }
            Stmt::Out => {
                let char = mem[*ptr].0 as char;
                write!(stdout, "{char}").unwrap();
                stdout.flush().unwrap();
            }
            Stmt::In => {
                let mut buf = [0; 1];
                stdin.read_exact(&mut buf).unwrap();
                mem[*ptr] = Wrapping(buf[0]);
            }
            Stmt::Loop(body) => {
                while mem[*ptr] != Wrapping(0) {
                    execute(mem, ptr, &body.stmts, stdout, stdin);
                }
            }
            Stmt::SetNull => {
                mem[*ptr] = Wrapping(0);
            }
        }
    }
}

type Profile = Vec<u64>;

pub fn run_profile<W, R>(instrs: &Ir<'_>, mut stdout: W, mut stdin: R) -> Profile
where
    W: Write,
    R: Read,
{
    let mut mem = [Wrapping(0u8); MEM_SIZE];
    let mut ptr = 0;

    let size = instrs
        .spans
        .last()
        .map(|sp| sp.start() + sp.len())
        .unwrap_or(0);

    let mut profile = vec![0; size];

    execute_profile(
        &mut mem,
        &mut ptr,
        instrs,
        &mut stdout,
        &mut stdin,
        &mut profile,
    );

    profile
}

fn execute_profile<W, R>(
    mem: &mut Memory,
    ptr: &mut usize,
    ir: &Ir<'_>,
    stdout: &mut W,
    stdin: &mut R,
    profile: &mut [u64],
) where
    W: Write,
    R: Read,
{
    for (i, stmt) in ir.stmts.iter().enumerate() {
        match stmt {
            Stmt::Add(n) => {
                mem[*ptr] += n;
            }
            Stmt::Sub(n) => {
                mem[*ptr] -= n;
            }
            Stmt::Right(n) => {
                *ptr += n;
                if *ptr >= MEM_SIZE {
                    *ptr = 0;
                }
            }
            Stmt::Left(n) => {
                if *ptr < *n {
                    let diff = *n - *ptr;
                    *ptr = MEM_SIZE - 1 - diff;
                } else {
                    *ptr -= n;
                }
            }
            Stmt::Out => {
                let char = mem[*ptr].0 as char;
                write!(stdout, "{char}").unwrap();
                stdout.flush().unwrap();
            }
            Stmt::In => {
                let mut buf = [0; 1];
                stdin.read_exact(&mut buf).unwrap();
                mem[*ptr] = Wrapping(buf[0]);
            }
            Stmt::Loop(body) => {
                while mem[*ptr] != Wrapping(0) {
                    execute_profile(mem, ptr, body, stdout, stdin, profile);
                }
            }
            Stmt::SetNull => {
                mem[*ptr] = Wrapping(0);
            }
        }
        let span = ir.spans[i];
        profile[span.start()..][..span.len()]
            .iter_mut()
            .for_each(|p| *p += 1);
    }
}
