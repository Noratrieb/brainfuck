use crate::parse::{Instr, Span};
use bumpalo::Bump;

#[derive(Debug)]
pub struct Ir<'ir> {
    pub stmts: Vec<Stmt<'ir>, &'ir Bump>,
    pub spans: Vec<Span, &'ir Bump>,
}

#[derive(Debug)]
pub enum Stmt<'ir> {
    Add(u8),
    Sub(u8),
    Right(usize),
    Left(usize),
    Loop(Ir<'ir>),
    Out,
    In,
    SetNull,
}

pub fn optimize<'ir>(alloc: &'ir Bump, instrs: &[(Instr<'_>, Span)]) -> Ir<'ir> {
    let mut ir = ast_to_ir(alloc, instrs);
    pass_find_set_null(&mut ir);
    ir
}

fn ast_to_ir<'ir>(alloc: &'ir Bump, ast: &[(Instr<'_>, Span)]) -> Ir<'ir> {
    let mut stmts = Vec::new_in(alloc);
    let mut spans = Vec::new_in(alloc);

    let mut instr_iter = ast.iter();

    let Some(first) = instr_iter.next() else {
        return Ir { stmts, spans: Vec::new_in(alloc) };
    };

    let mut last = &first.0;
    let mut last_span = first.1;
    let mut count = 1;

    for (next, next_span) in instr_iter {
        match last {
            Instr::Add | Instr::Sub | Instr::Right | Instr::Left if last == next => {
                count += 1;
                continue;
            }
            _ => {
                let new_last = match last {
                    Instr::Add => Stmt::Add(count),
                    Instr::Sub => Stmt::Sub(count),
                    Instr::Right => Stmt::Right(count.into()),
                    Instr::Left => Stmt::Left(count.into()),
                    Instr::Out => Stmt::Out,
                    Instr::In => Stmt::In,
                    Instr::Loop(body) => Stmt::Loop(ast_to_ir(alloc, body)),
                };
                stmts.push(new_last);
                spans.push(last_span.until(*next_span));
                last = next;
                last_span = *next_span;
                count = 1;
            }
        }
    }

    let new_last = match last {
        Instr::Add => Stmt::Add(count),
        Instr::Sub => Stmt::Sub(count),
        Instr::Right => Stmt::Right(count.into()),
        Instr::Left => Stmt::Left(count.into()),
        Instr::Out => Stmt::Out,
        Instr::In => Stmt::In,
        Instr::Loop(body) => Stmt::Loop(ast_to_ir(alloc, &body)),
    };
    stmts.push(new_last);
    spans.push(last_span);

    Ir { stmts, spans }
}

fn pass_find_set_null(ir: &mut Ir<'_>) {
    for stmt in &mut ir.stmts {
        if let Stmt::Loop(body) = stmt {
            if let [Stmt::Sub(_)] = body.stmts.as_slice() {
                *stmt = Stmt::SetNull;
            } else {
                pass_find_set_null(body);
            }
        }
    }
}
