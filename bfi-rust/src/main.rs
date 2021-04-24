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

    run(path);
}

fn run(path: String) {
    println!("Path: {}", path);
    let program = fs::read_to_string(path).unwrap();
    let start0 = SystemTime::now();
    let out0 = interpreter::o0::run(&*program);
    let end0 = start0.elapsed().unwrap();
    let start1 = SystemTime::now();
    let out = interpreter::o1::run(&*program);
    let end1 = start1.elapsed().unwrap();
    let start2 = SystemTime::now();
    let out2 = interpreter::o2::run(&*program).unwrap();
    let end2 = start2.elapsed().unwrap();
    assert_eq!(out, out2);
    assert_eq!(out0, out2);
    println!("{}\nFinished execution. Took o0: {}ms, o1: {}ms, o2: {}ms", out, end0.as_millis(), end1.as_millis(), end2.as_millis());
}