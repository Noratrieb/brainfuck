use std::io::{stdin, Read};
use std::{env, fs};

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
    let program = minify(program);
    println!("Minified program: {}", program);

    let out = interpret(program.chars().collect(), false);

    println!("{}", out);
}

fn minify(program: String) -> String {
    let allowed = vec!['>', '<', '+', '-', '.', ',', '[', ']'];
    program.chars().filter(|c| allowed.contains(c)).collect()
}

fn interpret(pgm: Vec<char>, number_debug: bool) -> String {
    let mut out = String::new();
    let mut pointer: usize = 0;
    let mut mem: [i8; 30000] = [0; 30000];
    let mut in_buffer = [0; 1];
    let mut pc = 0;
    let len = pgm.len();

    while pc < len {
        //println!("pc: {} instruction: {}, pointer: {}", pc, pgm[pc], pointer);
        match pgm[pc] {
            '>' => pointer += 1,
            '<' => pointer -= 1,
            '+' => mem[pointer] = mem[pointer].wrapping_add(1),
            '-' => mem[pointer] = mem[pointer].wrapping_sub(1),
            '.' => {
                if number_debug {
                    out.push_str(&(mem[pointer].to_string() + " "))
                } else {
                    out.push(mem[pointer] as u8 as char)
                }
            }
            ',' => {
                stdin().read(&mut in_buffer).unwrap();
                mem[pointer] = in_buffer[0] as i8;
            }
            '[' => {
                //jump to corresponding ] so that it will get to the command after the ]
                if mem[pointer] == 0 {
                    println!("[ found");
                    let mut level = 0;
                    while pgm[pc] != ']' || level > 0 {
                        pc += 1;
                        match pgm[pc] {
                            '[' => {
                                println!("level up");
                                level += 1
                            },
                            ']' => {
                                println!("level down");
                                level -= 1
                            },
                            _ => (),
                        }
                    }
                    assert_eq!(pgm[pc], ']')
                }
            }
            ']' => {
                if mem[pointer] != 0 {
                    //jump to corresponding [
                    let mut level = 0;
                    while pgm[pc] != '[' || level > 0 {
                        pc -= 1;
                        match pgm[pc] {
                            '[' => level -= 1,
                            ']' => level += 1,
                            _ => (),
                        }
                    }
                    assert_eq!(pgm[pc], '[')
                }
            }
            _ => (),
        }
        pc += 1;
    }

    out
}
