#![feature(allocator_api, let_else)]
#![warn(rust_2018_idioms)]

use brainfuck::UseProfile;
use std::{env, fs, io, process};

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("error: Provide a path as input.");
        process::exit(1);
    };

    let file = fs::read_to_string(path).unwrap_or_else(|err| {
        eprintln!("error: Failed to read file: {err}");
        process::exit(1);
    });

    let stdout = io::stdout();
    let stdout = stdout.lock();
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let profile = env::args()
        .any(|a| a == "--profile")
        .then(|| UseProfile::Yes)
        .unwrap_or(UseProfile::No);

    brainfuck::run(&file, stdout, stdin, profile).unwrap_or_else(|_| {
        eprintln!("error: Failed to parse brainfuck code");
        process::exit(1);
    });
}
