//! # optimization time
//!  some better optimizations like set null, repeating and doing more stuff with simplifying stuff
//!

mod patterns;

use std::io::{Read, stdin, Write};

use crate::interpreter::{minify, parse, Statement, Memory, MEM_SIZE};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::ops::Deref;

#[derive(PartialOrd, PartialEq, Ord, Eq, Clone, Debug)]
enum ExStatement {
    Inc,
    Dec,
    R,
    L,
    Out,
    DOut,
    In,
    Loop(Vec<ExStatement>),
    SetNull,
    Repeat(Box<ExStatement>, usize),
}

impl From<Statement> for ExStatement {
    fn from(s: Statement) -> Self {
        match s {
            Statement::L => ExStatement::L,
            Statement::R => ExStatement::R,
            Statement::Inc => ExStatement::Inc,
            Statement::Dec => ExStatement::Dec,
            Statement::In => ExStatement::In,
            Statement::Out => ExStatement::Out,
            Statement::Loop(v) => ExStatement::Loop(
                v.into_iter().map(|s| ExStatement::from(s)).collect()
            ),
            Statement::DOut => ExStatement::DOut
        }
    }
}

#[derive(Debug)]
pub struct BfErr {
    msg: &'static str
}

impl BfErr {
    pub fn new(msg: &'static str) -> BfErr {
        BfErr { msg }
    }
}

impl Display for BfErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Error interpreting brainfuck code: {}", self.msg)
    }
}

impl Error for BfErr {}


pub fn run(pgm: &str, direct_print: bool) -> Result<String, BfErr> {
    let pgm = minify(pgm);
    if pgm.len() < 1 { return Err(BfErr::new("no program found")); };
    let pgm = parse(pgm.chars().collect(), direct_print);
    let pgm = optimize(&pgm);
    let out = interpret(&pgm);
    Ok(out)
}

fn optimize(code: &Vec<Statement>) -> Vec<ExStatement> {
    let code = o_set_null(code);
    let code = o_repeat(code);
    code
}

fn o_set_null(code: &Vec<Statement>) -> Vec<ExStatement> {
    code.iter().map(|s| {
        match s {
            Statement::Loop(v) => {
                if let [Statement::Dec] = v[..] {
                    ExStatement::SetNull
                } else {
                    ExStatement::Loop(optimize(v))
                }
            }
            Statement::Inc => ExStatement::Inc,
            Statement::Dec => ExStatement::Dec,
            Statement::R => ExStatement::R,
            Statement::L => ExStatement::L,
            Statement::Out => ExStatement::Out,
            Statement::DOut => ExStatement::DOut,
            Statement::In => ExStatement::In,
        }
    }).collect()
}

fn o_repeat(code: Vec<ExStatement>) -> Vec<ExStatement> {
    let mut amount = 0;
    let mut result: Vec<ExStatement> = vec![];

    for i in 0..code.len() {
        if code.get(i) == code.get(i + 1) {
            amount += 1;
        } else if amount == 0 {
            result.push(code[i].clone())
        } else {
            amount += 1;
            result.push(ExStatement::Repeat(Box::new(code[i].clone()), amount as usize));
            amount = 0;
        }
    }

    result
}

fn interpret(pgm: &Vec<ExStatement>) -> String {
    let mut out = String::new();
    let mut pointer: usize = 0;
    let mut mem: [u8; MEM_SIZE] = [0; MEM_SIZE];

    for s in pgm {
        execute(s, &mut mem, &mut pointer, &mut out)
    }

    out
}

fn execute(statement: &ExStatement, mem: &mut Memory, pointer: &mut usize, out: &mut String) {
    match statement {
        ExStatement::R => if *pointer == MEM_SIZE - 1 { *pointer = 0 } else { *pointer += 1 },
        ExStatement::L => if *pointer == 0 { *pointer = MEM_SIZE - 1 } else { *pointer -= 1 },
        ExStatement::Inc => mem[*pointer] = mem[*pointer].wrapping_add(1),
        ExStatement::Dec => mem[*pointer] = mem[*pointer].wrapping_sub(1),
        ExStatement::SetNull => mem[*pointer] = 0,
        ExStatement::Out => out.push(mem[*pointer] as u8 as char),
        ExStatement::DOut => {
            print!("{}", mem[*pointer] as u8 as char);
            std::io::stdout().flush().unwrap();
        }
        ExStatement::In => {
            let mut in_buffer = [0, 1];
            stdin().read(&mut in_buffer).unwrap();
            mem[*pointer] = in_buffer[0] as u8;
        }
        ExStatement::Loop(vec) => {
            while mem[*pointer] != 0 {
                for s in vec {
                    execute(&s, mem, pointer, out);
                }
            }
        }
        ExStatement::Repeat(statement, amount) => {
            match statement.deref() {
                ExStatement::R => {
                    *pointer += amount;
                    if *pointer > MEM_SIZE {
                        *pointer %= MEM_SIZE
                    }
                }
                ExStatement::L => *pointer = (*pointer).wrapping_sub(*amount),
                ExStatement::Inc => mem[*pointer] = mem[*pointer].wrapping_add(*amount as u8),
                ExStatement::Dec => mem[*pointer] = mem[*pointer].wrapping_sub(*amount as u8),
                ExStatement::Loop(v) => {
                    for _ in 0..*amount {
                        execute(&ExStatement::Loop(v.clone()), mem, pointer, out)
                    }
                }
                s => {
                    for _ in 0..*amount {
                        execute(s, mem, pointer, out)
                    }
                }
            }
        }
    };
}


#[cfg(test)]
mod test {
    use crate::interpreter::optimized::{run, o_repeat};
    use crate::interpreter::optimized::ExStatement::{Inc, Repeat, R, L};
    use crate::interpreter::Statement::Dec;

    #[test]
    fn run_loop() {
        let program = "++++++++++[>++++++++++<-]>.";
        let out = run(program, false).unwrap();
        assert_eq!(out, String::from("d"));
    }

    #[test]
    fn hello_world() {
        let program = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let out = run(program, false).unwrap();
        assert_eq!(out, String::from("Hello World!\n"));
    }

    #[test]
    fn o_repeat_simple() {
        let code = vec![Inc, Inc, Inc, R];
        let expected = vec![Repeat(Box::new(Inc), 3), R];
        println!("{}", code.len());
        assert_eq!(expected, o_repeat(code));
    }

    #[test]
    fn o_repeat_long() {
        let code = vec![Inc, Inc, Inc, R, L, L, L, Dec, L, L, Dec];
        let expected = vec![Repeat(Box::new(Inc), 3), R, Repeat(Box::new(L), 3), Dec, Repeat(Box::new(L), 2), Dec];
        assert_eq!(expected, o_repeat(code));
    }
}

