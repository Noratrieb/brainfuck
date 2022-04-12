use crate::parse::Instr;
use std::io::{Read, Write};
use std::num::Wrapping;

const MEM_SIZE: usize = 32_000;

type Memory = [Wrapping<u8>; MEM_SIZE];

pub fn run(instrs: &[Instr<'_>]) {
    let mut mem = [Wrapping(0u8); MEM_SIZE];
    let mut ptr = 0;

    execute(&mut mem, &mut ptr, instrs);
}

fn execute(mem: &mut Memory, ptr: &mut usize, instrs: &[Instr<'_>]) {
    for instr in instrs {
        match instr {
            Instr::Add => {
                mem[*ptr] += 1;
            }
            Instr::Sub => {
                mem[*ptr] -= 1;
            }
            Instr::Right => {
                *ptr += 1;
                if *ptr >= MEM_SIZE {
                    *ptr = 0;
                }
            }
            Instr::Left => {
                if *ptr == 0 {
                    *ptr = MEM_SIZE - 1;
                } else {
                    *ptr -= 1;
                }
            }
            Instr::Out => {
                let char = mem[*ptr].0 as char;
                print!("{char}",);
                std::io::stdout().flush().unwrap();
            }
            Instr::In => {
                let mut buf = [0; 1];
                std::io::stdin().read_exact(&mut buf).unwrap();
                mem[*ptr] = Wrapping(buf[0]);
            }
            Instr::Loop(body) => {
                while mem[*ptr] != Wrapping(0) {
                    execute(mem, ptr, body);
                }
            }
        }
    }
}
