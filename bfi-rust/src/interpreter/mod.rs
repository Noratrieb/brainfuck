const MEM_SIZE: usize = 0xFFFF;

type Memory = [u8; MEM_SIZE];


#[derive(Debug, PartialOrd, PartialEq, Clone)]
enum SimpleStatement {
    Inc,
    Dec,
    R,
    L,
    Out,
    In,
    Loop(Vec<SimpleStatement>),
}

fn minify(code: &str) -> String {
    let allowed: Vec<char> = vec!['>', '<', '+', '-', '.', ',', '[', ']'];
    code.chars().filter(|c| allowed.contains(c)).collect()
}

fn parse(chars: Vec<char>) -> Vec<SimpleStatement> {
    let mut loop_stack = vec![vec![]];

    for c in chars {
        match c {
            '+' => loop_stack.last_mut().unwrap().push(SimpleStatement::Inc),
            '-' => loop_stack.last_mut().unwrap().push(SimpleStatement::Dec),
            '>' => loop_stack.last_mut().unwrap().push(SimpleStatement::R),
            '<' => loop_stack.last_mut().unwrap().push(SimpleStatement::L),
            '.' => loop_stack.last_mut().unwrap().push(SimpleStatement::Out),
            ',' => loop_stack.last_mut().unwrap().push(SimpleStatement::In),
            '[' => loop_stack.push(vec![]),
            ']' => {
                let statement = SimpleStatement::Loop(loop_stack.pop().unwrap());
                loop_stack.last_mut().unwrap().push(statement);
            }
            _ => ()
        }
    }

    return loop_stack.pop().unwrap();
}


///
/// # optimization time
///
/// first parse the bf so that it can be executed faster
/// most importantly: loop jumps should be immediate
///
pub(crate) mod o1 {
    use std::io::{Read, stdin};

    use crate::interpreter::{MEM_SIZE, Memory, minify, parse, SimpleStatement};

    pub fn run(pgm: &str) -> String {
        let pgm = minify(pgm);
        let pgm = parse(pgm.chars().collect());
        let out = interpret(&pgm);
        out
    }

    fn interpret(pgm: &Vec<SimpleStatement>) -> String {
        let mut out = String::new();
        let mut pointer: usize = 0;
        let mut mem: [u8; MEM_SIZE] = [0; MEM_SIZE];

        for s in pgm {
            execute(s, &mut mem, &mut pointer, &mut out)
        }

        out
    }

    fn execute(statement: &SimpleStatement, mem: &mut Memory, pointer: &mut usize, out: &mut String) {
        match statement {
            SimpleStatement::R => if *pointer == MEM_SIZE - 1 { *pointer = 0 } else { *pointer += 1 },
            SimpleStatement::L => if *pointer == 0 { *pointer = MEM_SIZE - 1 } else { *pointer -= 1 },
            SimpleStatement::Inc => mem[*pointer] = mem[*pointer].wrapping_add(1),
            SimpleStatement::Dec => mem[*pointer] = mem[*pointer].wrapping_sub(1),
            SimpleStatement::Out => out.push(mem[*pointer] as u8 as char),
            SimpleStatement::In => {
                let mut in_buffer = [0, 1];
                stdin().read(&mut in_buffer).unwrap();
                mem[*pointer] = in_buffer[0] as u8;
            }
            SimpleStatement::Loop(vec) => {
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
        use crate::interpreter::o1::{execute, run, Statement};
        use crate::interpreter::o1::Statement::{Dec, In, Inc, L, Loop, Out, R};
        use crate::interpreter::parse;
        use crate::o1::{execute, parse, run, Statement::{self, Dec, In, Inc, L, Loop, Out, R}};

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

///
/// # optimization time
/// some better optimizations like set null, repeating and doing more stuff with simplifying stuff
pub(crate) mod o2 {
    use std::io::{Read, stdin};

    use crate::interpreter::{minify, parse, SimpleStatement};

    const MEM_SIZE: usize = 0xFFFF;

    type Memory = [u8; MEM_SIZE];

    enum Statement {
        Inc,
        Dec,
        R,
        L,
        Out,
        In,
        Loop(Vec<Statement>),
        SetNull,
    }


    pub fn run(pgm: &str) -> String {
        let pgm = minify(pgm);
        let pgm = parse(pgm.chars().collect());
        let pgm = optimize(&pgm);
        let out = interpret(&pgm);
        out
    }

    fn optimize(code: &Vec<SimpleStatement>) -> Vec<Statement> {
        code.iter().map(|s| {
            match s {
                SimpleStatement::Loop(v) => {
                    if let [SimpleStatement::Dec] = v[..] {
                        Statement::SetNull
                    } else {
                        Statement::Loop(optimize(v))
                    }
                }
                SimpleStatement::Inc => Statement::Inc,
                SimpleStatement::Dec => Statement::Dec,
                SimpleStatement::R => Statement::R,
                SimpleStatement::L => Statement::L,
                SimpleStatement::Out => Statement::Out,
                SimpleStatement::In => Statement::In,
            }
        }).collect()
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
            Statement::SetNull => mem[*pointer] = 0,
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
        use crate::interpreter::o2::{execute, run};
        use crate::interpreter::parse;
        use crate::interpreter::SimpleStatement::{Dec, In, Inc, L, Loop, Out, R};
        use crate::o2::{execute, parse, run};

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

#[cfg(test)]
mod tests {
    use crate::interpreter::parse;
    use crate::interpreter::SimpleStatement::{Dec, In, Inc, L, Loop, Out, R};

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
}