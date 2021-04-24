const MEM_SIZE: usize = 0xFFFF;

type Memory = [u8; MEM_SIZE];


#[derive(Debug, PartialOrd, PartialEq, Ord, Eq, Clone)]
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


///
/// # optimization time
///
/// first parse the bf so that it can be executed faster
/// most importantly: loop jumps should be immediate
///
pub(crate) mod o1 {
    use std::io::{Read, stdin};

    use crate::interpreter::{MEM_SIZE, Memory, minify, parse, Statement};

    pub fn run(pgm: &str) -> String {
        let pgm = minify(pgm);
        let pgm = parse(pgm.chars().collect());
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
        }
    }


    #[cfg(test)]
    mod test {
        use crate::interpreter::o1::{execute, run, Statement};

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
pub mod o2 {
    use std::io::{Read, stdin};

    use crate::interpreter::{minify, parse, Statement};
    use std::error::Error;
    use std::fmt::{Display, Formatter};
    use std::fmt;
    use std::ops::Deref;

    const MEM_SIZE: usize = 0xFFFF;

    type Memory = [u8; MEM_SIZE];

    #[derive(PartialOrd, PartialEq, Ord, Eq, Clone, Debug)]
    enum ExStatement {
        Inc,
        Dec,
        R,
        L,
        Out,
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


    pub fn run(pgm: &str) -> Result<String, BfErr> {
        let pgm = minify(pgm);
        if pgm.len() < 1 { return Err(BfErr::new("no program found")); };
        let pgm = parse(pgm.chars().collect());
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
                    ExStatement::Out => {
                        for _ in 0..*amount {
                            execute(&ExStatement::Out, mem, pointer, out)
                        }
                    }
                    ExStatement::In => {
                        for _ in 0..*amount {
                            execute(&ExStatement::In, mem, pointer, out)
                        }
                    }
                    ExStatement::Loop(v) => {
                        for _ in 0..*amount {
                            execute(&ExStatement::Loop(v.clone()), mem, pointer, out)
                        }
                    }
                    _ => panic!("Invalid statement in repeat: {:?}", *statement)
                }
            }
        };
    }


    #[cfg(test)]
    mod test {
        use crate::interpreter::o2::{run, o_repeat};
        use crate::interpreter::o2::ExStatement::{L, R, Inc, Dec, Repeat};

        #[test]
        fn run_loop() {
            let program = "++++++++++[>++++++++++<-]>.";
            let out = run(program).unwrap();
            assert_eq!(out, String::from("d"));
        }

        #[test]
        fn hello_world() {
            let program = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
            let out = run(program).unwrap();
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
}

#[cfg(test)]
mod tests {
    use crate::interpreter::parse;
    use crate::interpreter::Statement::{Dec, In, Inc, L, Loop, Out, R};

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