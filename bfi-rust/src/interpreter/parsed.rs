//!
//!  # optimization time
//!
//!  first parse the bf so that it can be executed faster
//!  most importantly: loop jumps should be immediate
#![allow(dead_code)]

use std::io::{Read, stdin, Write};

use crate::interpreter::{MEM_SIZE, Memory, minify, parse, Statement};

pub fn run(pgm: &str) -> String {
    let pgm = minify(pgm);
    let pgm = parse(pgm.chars().collect(), false);
    let out = interpret(&pgm);
    out
}

fn interpret(pgm: &Vec<Statement>) -> String {
    let mut out = String::new();
    let mut pointer: usize = 0;
    let mut mem: [u8; MEM_SIZE] = [0; MEM_SIZE];

    for s in pgm {
        execute(s, &mut mem, &mut pointer, &mut out)
    }

    out
}

fn execute(statement: &Statement, mem: &mut Memory, pointer: &mut usize, out: &mut String) {
    match statement {
        Statement::R => if *pointer == MEM_SIZE - 1 { *pointer = 0 } else { *pointer += 1 },
        Statement::L => if *pointer == 0 { *pointer = MEM_SIZE - 1 } else { *pointer -= 1 },
        Statement::Inc => mem[*pointer] = mem[*pointer].wrapping_add(1),
        Statement::Dec => mem[*pointer] = mem[*pointer].wrapping_sub(1),
        Statement::Out => out.push(mem[*pointer] as u8 as char),
        Statement::In => {
            let mut in_buffer = [0, 1];
            stdin().read(&mut in_buffer).unwrap();
            mem[*pointer] = in_buffer[0] as u8;
        }
        Statement::Loop(vec) => {
            while mem[*pointer] != 0 {
                for s in vec {
                    execute(&s, mem, pointer, out);
                }
            }
        }
        Statement::DOut => {
            print!("{}", mem[*pointer] as u8 as char);
            std::io::stdout().flush().unwrap();
        }
    }
}


#[cfg(test)]
mod test {
    use crate::interpreter::parsed::{execute, run, Statement};

    #[test]
    fn execute_simple() {
        let mut pointer: usize = 0;
        let mut mem: [u8; 65535] = [0; 65535];
        let mut out = String::new();

        execute(&Statement::R, &mut mem, &mut pointer, &mut out);
        assert_eq!(pointer, 1);
        execute(&Statement::L, &mut mem, &mut pointer, &mut out);
        assert_eq!(pointer, 0);
        execute(&Statement::Inc, &mut mem, &mut pointer, &mut out);
        assert_eq!(mem[pointer], 1);
        execute(&Statement::Dec, &mut mem, &mut pointer, &mut out);
        assert_eq!(mem[pointer], 0);
    }

    #[test]
    fn execute_false_loop() {
        let statement = Statement::Loop(vec![Statement::Inc, Statement::Inc, Statement::R]);
        let mut pointer: usize = 0;
        let mut mem: [u8; 65535] = [0; 65535];

        execute(&statement, &mut mem, &mut pointer, &mut String::new());
        assert_eq!(mem[0], 0);
        assert_eq!(mem[1], 0);
    }

    #[test]
    fn execute_loop() {
        let statement = Statement::Loop(vec![Statement::Inc, Statement::Inc, Statement::R]);
        let mut pointer: usize = 0;
        let mut mem: [u8; 65535] = [0; 65535];
        mem[0] = 1;

        execute(&statement, &mut mem, &mut pointer, &mut String::new());
        assert_eq!(mem[0], 3);
        assert_eq!(mem[1], 0);
    }

    #[test]
    fn run_loop() {
        let program = "++++++++++[>++++++++++<-]>.";
        let out = run(program);
        assert_eq!(out, String::from("d"));
    }

    #[test]
    fn hello_world() {
        let program = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let out = run(program);
        assert_eq!(out, String::from("Hello World!\n"));
    }
}