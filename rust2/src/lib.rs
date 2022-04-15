#![feature(allocator_api, let_else)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(rust_2018_idioms)]

use crate::parse::ParseError;
use bumpalo::Bump;
use owo_colors::OwoColorize;
use std::fmt::Display;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

pub mod codegen;
pub mod codegen_interpreter;
pub mod opts;
pub mod parse;

#[derive(clap::Parser, Default)]
#[clap(author, about)]
pub struct Args {
    #[clap(short, long)]
    pub profile: bool,
    #[clap(long)]
    pub dump: Option<DumpKind>,
    pub file: PathBuf,
}

pub enum DumpKind {
    Ast,
    Ir,
    Code,
}

impl FromStr for DumpKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ast" => Ok(Self::Ast),
            "ir" => Ok(Self::Ir),
            "code" => Ok(Self::Code),
            other => Err(format!("Invalid IR level: '{other}'")),
        }
    }
}

type BumpVec<'a, T> = Vec<T, &'a Bump>;

pub enum UseProfile {
    Yes,
    No,
}

pub fn run<R, W>(src: &str, stdout: W, stdin: R, config: &Args) -> Result<(), ParseError>
where
    W: Write,
    R: Read,
{
    let ast_alloc = Bump::new();

    let parsed = parse::parse(&ast_alloc, src.bytes().enumerate())?;

    if let Some(DumpKind::Ast) = config.dump {
        println!("{parsed:#?}");
        return Ok(());
    }

    let ir_alloc = Bump::new();

    let optimized_ir = opts::optimize(&ir_alloc, &parsed);

    if let Some(DumpKind::Ir) = config.dump {
        println!("{optimized_ir:#?}");
        return Ok(());
    }

    drop(parsed);
    drop(ast_alloc);

    let cg_alloc = Bump::new();

    let code = codegen::generate(&cg_alloc, &optimized_ir);

    if let Some(DumpKind::Code) = config.dump {
        println!("{code:#?}");
        return Ok(());
    }

    drop(optimized_ir);
    drop(ir_alloc);

    match config.profile {
        true => {
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
        false => {
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
    use crate::Args;

    #[test]
    fn fizzbuzz() {
        let str = include_str!("fizzbuzz.bf");
        let mut stdout = Vec::new();
        let stdin = [];

        super::run(str, &mut stdout, stdin.as_slice(), &Args::default()).unwrap();

        insta::assert_debug_snapshot!(String::from_utf8(stdout));
    }
}
