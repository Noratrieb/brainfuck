mod interpreter;
mod repl;

use std::{env, fs};
use std::time::SystemTime;
use crate::repl::start_repl;
use crate::interpreter::optimized::{PrintMode};
use std::error::Error;


fn main() {
    let path = env::args().nth(1);

    match path {
        Some(p) => {
            if let Err(why) = run_program(p) {
                eprintln!("An error occurred in the program: {}", why)
            }
        },
        None => start_repl()
    };
}

fn run_program(path: String) -> Result<(), Box<dyn Error>> {
    let program = match fs::read_to_string(path) {
        Ok(p) => p,
        Err(e) => {
            println!("Error reading file: {}", e);
            return Err(Box::from(e));
        }
    };
    let start_time = SystemTime::now();
    let out = interpreter::optimized::run(&*program, PrintMode::DirectPrint)?;
    let duration = start_time.elapsed()?;
    println!("{}\nFinished execution. Took {}ms", out, duration.as_millis());
    Ok(())
}