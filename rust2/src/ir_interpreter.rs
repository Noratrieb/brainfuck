use crate::opts::Stmt;
use std::io::{Read, Write};
use std::num::Wrapping;

const MEM_SIZE: usize = 32_000;

type Memory = [Wrapping<u8>; MEM_SIZE];

pub fn run(instrs: &[Stmt<'_>]) {
    let mut mem = [Wrapping(0u8); MEM_SIZE];
    let mut ptr = 0;

    execute(&mut mem, &mut ptr, instrs);
}

fn execute(mem: &mut Memory, ptr: &mut usize, instrs: &[Stmt<'_>]) {
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
                print!("{char}");
                std::io::stdout().flush().unwrap();
            }
            Stmt::In => {
                let mut buf = [0; 1];
                std::io::stdin().read_exact(&mut buf).unwrap();
                mem[*ptr] = Wrapping(buf[0]);
            }
            Stmt::Loop(body) => {
                while mem[*ptr] != Wrapping(0) {
                    execute(mem, ptr, body);
                }
            }
            Stmt::SetNull => {
                mem[*ptr] = Wrapping(0);
            }
        }
    }
}
