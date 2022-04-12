use bumpalo::Bump;
use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

struct MockReadWrite;

impl Read for MockReadWrite {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        buf.fill(b'A');
        Ok(buf.len())
    }
}

impl Write for MockReadWrite {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn get_bf(path_from_bench: impl AsRef<str>) -> String {
    let file = PathBuf::from(file!())
        .parent()
        .unwrap()
        .join(path_from_bench.as_ref());

    fs::read_to_string(file).unwrap()
}

fn run_bf_bench(bf: &str) {
    let bump = Bump::new();
    let parsed = brainfuck::parse::parse(&bump, bf.bytes()).unwrap();
    let optimized = brainfuck::opts::optimize(&bump, &parsed);
    brainfuck::ir_interpreter::run(&optimized, MockReadWrite, MockReadWrite);
}

fn optimized(c: &mut Criterion) {
    let fizzbuzz = get_bf("fizzbuzz.bf");
    let hello_world = get_bf("hello.bf");
    let bench = get_bf("bench.bf");
    let loopremove = get_bf("loopremove.bf");

    c.bench_function("fizzbuzz", |b| b.iter(|| run_bf_bench(&fizzbuzz)));
    c.bench_function("hello_world", |b| b.iter(|| run_bf_bench(&hello_world)));
    c.bench_function("bench", |b| b.iter(|| run_bf_bench(&bench)));
    c.bench_function("loopremove", |b| b.iter(|| run_bf_bench(&loopremove)));
}

criterion_group!(benches, optimized);
criterion_main!(benches);
