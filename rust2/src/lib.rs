#![feature(allocator_api, let_else)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(rust_2018_idioms)]
#![allow(dead_code)]

use crate::parse::ParseError;
use bumpalo::Bump;
use std::fmt::Display;
use std::io::{Read, Write};

pub mod codegen;
pub mod codegen_interpreter;
pub mod opts;
pub mod parse;

type BumpVec<'a, T> = Vec<T, &'a Bump>;

pub enum UseProfile {
    Yes,
    No,
}

pub fn run<R, W>(str: &str, stdout: W, stdin: R, use_profile: UseProfile) -> Result<(), ParseError>
where
    W: Write,
    R: Read,
{
    let ast_alloc = Bump::new();

    let parsed = parse::parse(&ast_alloc, str.bytes().enumerate())?;

    let ir_alloc = Bump::new();

    let optimized_ir = opts::optimize(&ir_alloc, &parsed);

    drop(parsed);
    drop(ast_alloc);

    let cg_alloc = Bump::new();

    let code = codegen::generate(&cg_alloc, &optimized_ir);

    drop(optimized_ir);
    drop(ir_alloc);

    match use_profile {
        UseProfile::Yes => {
            // let profile = ir_interpreter::run_profile(&optimized_ir, stdout, stdin);
            // let max = profile.iter().max().copied().unwrap_or(0);
            // println!("---------------- Profile ----------------");
            // for (i, char) in str.bytes().enumerate() {
            //     print!("{}", color_by_profile(char as char, profile[i], max));
            // }
            println!("no supported lol");
        }
        UseProfile::No => {
            codegen_interpreter::run(&code, stdout, stdin);
        }
    }

    Ok(())
}

fn color_by_profile(char: char, value: u64, max: u64) -> impl Display {
    use owo_colors::OwoColorize;

    let percentage = ((max as f64) / (value as f64) * 100.0) as u64;

    match percentage {
        0..=1 => char.default_color().to_string(),
        2..=10 => char.green().to_string(),
        11..=30 => char.yellow().to_string(),
        31..=90 => char.red().to_string(),
        91..=100 => char.bright_red().to_string(),
        _ => char.default_color().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::UseProfile;

    #[test]
    fn fizzbuzz() {
        let str = include_str!("fizzbuzz.bf");
        let mut stdout = Vec::new();
        let stdin = [];

        super::run(str, &mut stdout, stdin.as_slice(), UseProfile::No).unwrap();

        insta::assert_debug_snapshot!(String::from_utf8(stdout));
    }
}
