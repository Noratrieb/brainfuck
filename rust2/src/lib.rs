#![feature(allocator_api, let_else)]
#![warn(rust_2018_idioms)]

use crate::parse::ParseError;
use bumpalo::Bump;
use std::io::{Read, Write};

pub mod ir_interpreter;
pub mod opts;
pub mod parse;

pub fn run<R, W>(bytes: impl Iterator<Item = u8>, stdout: W, stdin: R) -> Result<(), ParseError>
where
    W: Write,
    R: Read,
{
    let ast_alloc = Bump::new();

    let parsed = parse::parse(&ast_alloc, bytes.enumerate())?;

    let ir_alloc = Bump::new();

    let optimized_ir = opts::optimize(&ir_alloc, &parsed);

    drop(parsed);
    drop(ast_alloc);

    ir_interpreter::run(&optimized_ir, stdout, stdin);

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn fizzbuzz() {
        let str = include_str!("fizzbuzz.bf");
        let mut stdout = Vec::new();
        let stdin = [];

        super::run(str.bytes(), &mut stdout, stdin.as_slice()).unwrap();

        insta::assert_debug_snapshot!(String::from_utf8(stdout));
    }
}
