#![feature(allocator_api, let_else)]
#![warn(rust_2018_idioms)]

use brainfuck::Args;
use clap::Parser;
use std::{fs, io, process};

fn main() {
    let stdout = io::stdout();
    let stdout = stdout.lock();
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let src = fs::read_to_string(&args.file).unwrap_or_else(|err| {
        eprintln!("error: Failed to read file: {err}");
        process::exit(1);
    });

    brainfuck::run(&src, stdout, stdin, &args).unwrap_or_else(|_| {
        eprintln!("error: Failed to parse brainfuck code");
        process::exit(1);
    });
}
