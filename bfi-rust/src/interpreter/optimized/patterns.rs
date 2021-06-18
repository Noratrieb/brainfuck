//!
//! # Patterns find and replace
//! Pattern-match ExStatements and replace them with optimizations like add, multiply etc


use crate::interpreter::optimized::ExStatement;

///
/// Replace this: `[>>x<<-]` or `[->>x<<]` with `WhileAdd(2, x)`
fn _for_loop(to_test: ExStatement) -> ExStatement {
    match to_test {
        ExStatement::Loop(v) => {
            match v[..] {
                [ExStatement::R, ExStatement::Inc, ExStatement::L, ExStatement::Dec] => {
                    ExStatement::_ForLoop(1, Box::from(ExStatement::Inc))
                }
                _ => ExStatement::Loop(v)
            }
        },
        s => s
    }
}




#[cfg(test)]
mod test {
    use crate::interpreter::optimized::ExStatement::{Out, Loop, Inc, R, L, Dec, _ForLoop};
    use crate::interpreter::optimized::patterns::_for_loop;

    #[test]
    fn for_loop_false() {
        let statement = Loop(vec![Out, Inc]);
        assert_eq!(statement.clone(), _for_loop(statement));
    }

    #[test]
    fn for_loop_simplest() {
        let statement = Loop(vec![R, Inc, L, Dec]);
        assert_eq!(_ForLoop(1, Box::from(Inc)), _for_loop(statement));
    }
}