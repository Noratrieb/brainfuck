use std::fmt::{Display, Formatter};
use std::fmt;
use std::io::{stdin, stdout, Write};
use crate::interpreter::{minify, parse, parsed, Memory, MEM_SIZE};
use crate::interpreter::optimized::PrintMode;

pub struct BrainfuckState {
    pub memory: Memory,
    pub pointer: usize,
}

impl Display for BrainfuckState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", display_state(self))
    }
}

fn display_state(state: &BrainfuckState) -> String {
    let start = if state.pointer < 5 {
        0
    } else if state.pointer > MEM_SIZE - 10 {
        MEM_SIZE - 10
    } else {
        state.pointer - 5
    };

    format!("{}-\n|{}|\n{}-\n{}|\n{}|\n{}|\n{}-\n   {}^^^^",
            "----------".repeat(10),
            {
                let mut out = String::new();
                let end = start + 10;
                for i in start..end {
                    out.push_str(&*format!("   {: >5}  ", i));
                }
                out
            },
            "----------".repeat(10),
            "|         ".repeat(10),
            {
                let mut out = String::new();
                let end = start + 10;
                for i in start..end {
                    out.push_str(&*format!("|   {: >3}   ", state.memory[i]));
                }
                out
            },
            "|         ".repeat(10),
            "----------".repeat(10),
            "          ".repeat(state.pointer - start))
}

pub fn start_repl() {
    println!("Brainfuck REPL");

    let mut state = BrainfuckState {
        memory: [0; MEM_SIZE],
        pointer: 0,
    };

    println!("Enter Brainfuck programs and they will be executed immediatly.");
    println!("State is kept.");
    println!("{}", state);
    loop {
        print!(">> ");
        stdout().flush().unwrap();
        match read_line() {
            Ok(s) => {
                match &*s {
                    ":q" => break,
                    ":?" | "help" | "?" => print_help(),
                    ":r" => {
                        reset(&mut state);
                        println!("{}", state);
                    },
                    _ => {
                        print!("Output: ");
                        println!();
                        parse_input(s, &mut state);
                        println!("{}", state);
                    }
                }
            }
            Err(why) => println!("Error reading input: {}\nPlease try again.", why)
        }
    }
}

fn reset(state: &mut BrainfuckState) {
    state.pointer = 0;
    state.memory = [0; MEM_SIZE];
}

fn print_help() {
    println!("Brainfuck REPL help
   :q => quit
   :? => help
   :r => reset state");
}

fn parse_input(pgm: String, state: &mut BrainfuckState) {
    let pgm = minify(&*pgm);
    let pgm = parse(pgm.chars(), PrintMode::DirectPrint);
    parsed::interpret_with_state(&*pgm, state);
}

pub fn read_line() -> Result<String, std::io::Error> {
    let mut buf = String::new();
    stdin().read_line(&mut buf)?;
    buf.pop();
    Ok(buf)
}