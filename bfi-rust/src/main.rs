use std::{env, fs};
use std::io::{Read, stdin};
use std::time::SystemTime;

fn main() {
    let path = env::args().skip(1).next();
    let path = match path {
        Some(p) => p,
        None => {
            println!("Please specify a path");
            return;
        }
    };

    run(path);
}

fn run(path: String) {
    println!("Path: {}", path);
    let program = fs::read_to_string(path).unwrap();
    let start = SystemTime::now();
    let out = o1::run(&*program);
    println!("{}\nFinished execution in {}ms", out, start.elapsed().unwrap().as_millis());
}

///
/// # optimization time
///
/// first parse the bf so that it can be executed faster
/// most importantly: loop jumps should be immediate
///
mod o1 {
    use std::io::{stdin, Read, stdout, Write};

    const MEM_SIZE: usize = 0xFFFF;

    type Memory = [u8; MEM_SIZE];


    pub fn run(pgm: &str) -> String {
        let pgm = minify(pgm);
        let pgm = parse(pgm.chars().collect());
        let out = interpret(&pgm);
        out
    }

    ///
    /// A single Statement, can be an instruction or a nestable loop
    #[derive(Debug, PartialOrd, PartialEq)]
    enum Statement {
        Inc,
        Dec,
        R,
        L,
        Out,
        In,
        Loop(Vec<Statement>),
    }

    fn minify(code: &str) -> String {
        let allowed: Vec<char> = vec!['>', '<', '+', '-', '.', ',', '[', ']'];
        code.chars().filter(|c| allowed.contains(c)).collect()
    }

    fn parse(chars: Vec<char>) -> Vec<Statement> {
        let mut loop_stack = vec![vec![]];

        for c in chars {
            match c {
                '+' => loop_stack.last_mut().unwrap().push(Statement::Inc),
                '-' => loop_stack.last_mut().unwrap().push(Statement::Dec),
                '>' => loop_stack.last_mut().unwrap().push(Statement::R),
                '<' => loop_stack.last_mut().unwrap().push(Statement::L),
                '.' => loop_stack.last_mut().unwrap().push(Statement::Out),
                ',' => loop_stack.last_mut().unwrap().push(Statement::In),
                '[' => loop_stack.push(vec![]),
                ']' => {
                    let statement = Statement::Loop(loop_stack.pop().unwrap());
                    loop_stack.last_mut().unwrap().push(statement);
                }
                _ => ()
            }
        }

        return loop_stack.pop().unwrap();
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
        }
    }


    #[cfg(test)]
    mod test {
        use crate::o1::{parse, Statement::{self, Inc, Dec, R, L, In, Out, Loop}, execute, run};

        #[test]
        fn parse_no_loop() {
            let program = "+-<>,.";
            let statements = vec![Inc, Dec, L, R, In, Out];
            let result = parse(program.chars().collect());

            assert_eq!(statements, result);
        }

        #[test]
        fn parse_simple_loop() {
            let program = "+[<<]-";
            let statements = vec![Inc, Loop(vec![L, L]), Dec];
            let result = parse(program.chars().collect());

            assert_eq!(statements, result);
        }

        #[test]
        fn parse_complex_loops() {
            let program = ">[<[][<[<]>]>[>]]";
            let statements = vec![R, Loop(vec![L, Loop(vec![]), Loop(vec![L, Loop(vec![L]), R]), R, Loop(vec![R])])];
            let result = parse(program.chars().collect());

            assert_eq!(statements, result);
        }


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
}