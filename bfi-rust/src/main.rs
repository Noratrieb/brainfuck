mod interpreter;

use std::{env, fs};
use std::time::SystemTime;


fn main() {
    let path = env::args().skip(1).next();
    let path = match path {
        Some(p) => p,
        None => {
            println!("Please specify a path");
            return;
        }
    };
    let program = match fs::read_to_string(path) {
        Ok(p) => p,
        Err(e) => {
            println!("Error reading file: {}", e);
            return;
        }
    };

    run(program);
}

fn run(program: String) {
/*
    let start1 = SystemTime::now();
    let out = interpreter::o1::run(&*program);
    let end1 = start1.elapsed().unwrap();*/
    let start2 = SystemTime::now();
    let out2 = interpreter::o2::run(&*program).unwrap();
    let end2 = start2.elapsed().unwrap();
    //assert_eq!(out, out2);
    //println!("{}\nFinished execution. Took o1: 18008ms (for hanoi), o2: {}ms", out2/*, end1.as_millis()*/, end2.as_millis());
    println!("{}\nFinished execution. Took {}ms", out2/*, end1.as_millis()*/, end2.as_millis());
}