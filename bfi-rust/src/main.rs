use std::io::{stdin, Read};
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
    let program = minify(program);
    println!("Minified program: {}", program);

    let start = SystemTime::now();
    let out = interpret(program.chars().collect(), false);

    println!("Finished in {}ms: {}", start.elapsed().unwrap().as_millis(), out);
}

fn minify(program: String) -> String {
    let allowed = vec!['>', '<', '+', '-', '.', ',', '[', ']'];
    program.chars().filter(|c| allowed.contains(c)).collect()
}

const MEM_SIZE: usize = 0xFFFF;

fn interpret(pgm: Vec<char>, number_debug: bool) -> String {
    let mut out = String::new();
    let mut pointer: usize = 0;
    let mut mem: [u8; MEM_SIZE] = [0; MEM_SIZE];
    let mut in_buffer = [0; 1];
    let mut pc = 0;
    let len = pgm.len();

    while pc < len {
        match pgm[pc] {
            '>' => if pointer == MEM_SIZE - 1 { pointer = 0 } else { pointer += 1 },
            '<' => if pointer == 0 { pointer = MEM_SIZE - 1 } else { pointer -= 1 },
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
                mem[pointer] = in_buffer[0] as u8;
            }
            '[' => {
                //jump to corresponding ] so that it will get to the command after the ]
                if mem[pointer] == 0 {
                    println!("jumping forward at location {}", pc + 1);
                    let mut level = 0;
                    while pgm[pc] != ']' || level > -1 {
                        pc += 1;
                        match pgm[pc] {
                            '[' => {
                                level += 1
                            }
                            ']' => {
                                level -= 1
                            }
                            _ => (),
                        }
                    }
                    println!("found matching ] at location {}", pc + 1);
                    assert_eq!(pgm[pc], ']')
                }
            }
            ']' => {
                if mem[pointer] != 0 {
                    println!("jumping back at location {}", pc + 1);

                    //jump to corresponding [
                    let mut level = 0;
                    while pgm[pc] != '[' || level > -1 {
                        pc -= 1;
                        match pgm[pc] {
                            '[' => level -= 1,
                            ']' => level += 1,
                            _ => (),
                        }
                    }
                    println!("found matching [ at location {}", pc + 1);

                    assert_eq!(pgm[pc], '[')
                }
            }
            _ => (),
        }
        pc += 1;
    }

    out
}
