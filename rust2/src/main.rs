#![feature(allocator_api, let_else)]
#![warn(rust_2018_idioms)]

use bumpalo::Bump;
use std::{env, fs, process};

mod ir_interpreter;
mod opts;
mod parse;

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("error: Provide a path as input.");
        process::exit(1);
    };

    let file = fs::read_to_string(path).unwrap_or_else(|err| {
        eprintln!("error: Failed to read file: {err}");
        process::exit(1);
    });

    let ast_alloc = Bump::new();

    let parsed = parse::parse(&ast_alloc, file.bytes()).unwrap_or_else(|_| {
        eprintln!("Failed to parse brainfuck code.");
        process::exit(1);
    });

    let ir_alloc = Bump::new();

    let optimized_ir = opts::optimize(&ir_alloc, &parsed);

    drop(parsed);
    drop(ast_alloc);

    ir_interpreter::run(&optimized_ir);
}