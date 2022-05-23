#![feature(allocator_api, let_else)]
#![feature(nonzero_ops)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(rust_2018_idioms)]

use std::{
    fmt::Display,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

use bumpalo::Bump;
use owo_colors::OwoColorize;

use crate::parse::ParseError;

pub mod hir;
pub mod lir;
mod mir;
pub mod parse;

#[derive(clap::Parser, Default)]
#[clap(author, about)]
pub struct Args {
    /// Print colored source code depending on how often it was run.
    /// Makes the interpreter ~30% slower.
    #[clap(short, long)]
    pub profile: bool,
    /// Dump the IR info (ast, hir, mir, lir)
    #[clap(long)]
    pub dump: Option<DumpKind>,
    /// Use experimental mid-level IR
    #[clap(long)]
    pub mir: bool,
    /// The file to run
    pub file: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DumpKind {
    Ast,
    Hir,
    Mir,
    Lir,
}

impl FromStr for DumpKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ast" => Ok(Self::Ast),
            "hir" => Ok(Self::Hir),
            "mir" => Ok(Self::Mir),
            "lir" => Ok(Self::Lir),
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

    let hir_alloc = Bump::new();

    let optimized_hir = hir::optimized_hir(&hir_alloc, &parsed);

    if let Some(DumpKind::Hir) = config.dump {
        println!("{}", dbg_pls::color(&optimized_hir));
        return Ok(());
    }

    drop(parsed);
    drop(ast_alloc);

    if config.dump == Some(DumpKind::Mir) || config.mir {
        let mir_alloc = Bump::new();
        let mir = mir::optimized_mir(&mir_alloc, &optimized_hir);
        if config.dump == Some(DumpKind::Mir) {
            println!("{mir:#?}");
        }
    }

    let cg_alloc = Bump::new();

    let lir = lir::generate(&cg_alloc, &optimized_hir);

    if let Some(DumpKind::Lir) = config.dump {
        println!("{lir:#?}");
        return Ok(());
    }

    drop(optimized_hir);
    drop(hir_alloc);

    match config.profile {
        true => {
            let mut code_profile_count = vec![0; lir.debug().len()];

            lir::interpreter::run(&lir, stdout, stdin, |ip| unsafe {
                *code_profile_count.get_unchecked_mut(ip) += 1;
            });

            let mut src_profile_count = vec![0u64; src.len()];

            for (stmt_span, stmt_count) in lir.debug().iter().zip(&code_profile_count) {
                for i in &mut src_profile_count[stmt_span.start()..stmt_span.end()] {
                    *i += stmt_count;
                }
            }

            let max = src_profile_count.iter().max().copied().unwrap_or(0);
            println!("\n\n---------------- Profile ----------------");
            for (char, value) in src.bytes().zip(src_profile_count) {
                print!("{}", color_by_profile(char as char, value, max));
            }
        }
        false => {
            lir::interpreter::run(&lir, stdout, stdin, |_| {});
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
        let str = include_str!("../benches/fizzbuzz.bf");
        let mut stdout = Vec::new();
        let stdin = [];

        super::run(str, &mut stdout, stdin.as_slice(), &Args::default()).unwrap();

        insta::assert_debug_snapshot!(String::from_utf8(stdout));
    }

    #[test]
    fn mandelbrot() {
        let str = include_str!("../benches/mandelbrot.bf");
        let mut stdout = Vec::new();
        let stdin = [];

        super::run(str, &mut stdout, stdin.as_slice(), &Args::default()).unwrap();

        insta::assert_debug_snapshot!(String::from_utf8(stdout));
    }
}
