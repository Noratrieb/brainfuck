use bumpalo::Bump;
use criterion::{black_box, criterion_main, Criterion};
use std::io::{Read, Write};

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

fn run_bf(bf: &str) {
    let bump = Bump::new();
    let ast = brainfuck::parse::parse(&bump, bf.bytes().enumerate()).unwrap();
    let ir = brainfuck::opts::optimize(&bump, &ast);
    let code = brainfuck::codegen::generate(&bump, &ir);
    brainfuck::codegen_interpreter::run(&code, MockReadWrite, MockReadWrite, |_| {});
}

fn optimized(c: &mut Criterion) {
    let fizzbuzz = include_str!("fizzbuzz.bf");
    let bench = include_str!("bench.bf");
    let twinkle = include_str!("twinkle.bf");
    let bottles = include_str!("bottles.bf");
    let mandelbrot = include_str!("mandelbrot.bf");
    let hanoi = include_str!("hanoi.bf");

    c.bench_function("fizzbuzz", |b| b.iter(|| run_bf(black_box(fizzbuzz))));
    c.bench_function("bench", |b| b.iter(|| run_bf(black_box(bench))));
    c.bench_function("twinkle", |b| b.iter(|| run_bf(black_box(twinkle))));
    c.bench_function("bottles", |b| b.iter(|| run_bf(black_box(bottles))));
    c.bench_function("mandelbrot", |b| b.iter(|| run_bf(black_box(mandelbrot))));
    c.bench_function("hanoi", |b| b.iter(|| run_bf(black_box(hanoi))));
}

pub fn benches() {
    // we need such a low sample size because the benches take ages otherwise
    let mut c = Criterion::default().configure_from_args().sample_size(30);
    optimized(&mut c);
}

criterion_main!(benches);
