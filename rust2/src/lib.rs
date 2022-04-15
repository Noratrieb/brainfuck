#![feature(allocator_api, let_else)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(rust_2018_idioms)]
#![allow(dead_code)]

use crate::parse::ParseError;
use bumpalo::Bump;
use owo_colors::OwoColorize;
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

pub fn run<R, W>(src: &str, stdout: W, stdin: R, use_profile: UseProfile) -> Result<(), ParseError>
where
    W: Write,
    R: Read,
{
    let ast_alloc = Bump::new();

    let parsed = parse::parse(&ast_alloc, src.bytes().enumerate())?;

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
            let mut code_profile_count = vec![0; code.debug().len()];

            codegen_interpreter::run(&code, stdout, stdin, |ip| unsafe {
                *code_profile_count.get_unchecked_mut(ip) += 1;
            });

            let mut src_profile_count = vec![0u64; src.len()];

            for (stmt_span, stmt_count) in code.debug().iter().zip(&code_profile_count) {
                for i in &mut src_profile_count[stmt_span.start()..stmt_span.end()] {
                    *i += stmt_count;
                }
            }

            let max = src_profile_count.iter().max().copied().unwrap_or(0);
            println!("---------------- Profile ----------------");
            for (char, value) in src.bytes().zip(src_profile_count) {
                print!("{}", color_by_profile(char as char, value, max));
            }
        }
        UseProfile::No => {
            codegen_interpreter::run(&code, stdout, stdin, |_| {});
        }
    }

    Ok(())
}

fn color_by_profile(char: char, value: u64, max: u64) -> impl Display {
    let max = max as f64;
    let value = value as f64;
    let ratio = value / max;
    let logged = -ratio.log10();
    let logged = (logged * 100.) as u64;

    match logged {
        0..=15 => char.bright_red().to_string(),
        16..=70 => char.yellow().to_string(),
        71..=300 => char.green().to_string(),
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
