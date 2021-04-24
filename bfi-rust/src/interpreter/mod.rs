pub mod o0;
pub mod o1;
pub mod o2;

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

#[cfg(test)]
mod tests {
    use crate::interpreter::{parse, minify};
    use crate::interpreter::Statement::{Dec, In, Inc, L, Loop, Out, R};

    #[test]
    fn minify_test() {
        let program = "sdahf+saga-46<sgbv>a[r]r.hr,e";
        let expected = "+-<>[].,";
        assert_eq!(String::from(expected), minify(program));
    }

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