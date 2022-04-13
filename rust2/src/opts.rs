use crate::parse::{Instr, Span};
use bumpalo::Bump;

#[derive(Debug, Clone)]
pub struct Ir<'ir> {
    pub stmts: Vec<Stmt<'ir>, &'ir Bump>,
}

#[derive(Debug, Clone)]
pub struct Stmt<'ir> {
    pub kind: StmtKind<'ir>,
    pub span: Span,
}

impl<'ir> Stmt<'ir> {
    fn lol(kind: StmtKind<'ir>) -> Stmt<'ir> {
        Self {
            kind,
            span: Span::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StmtKind<'ir> {
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
        return Ir { stmts };
    };

    let mut last = &first.0;
    let mut start_span = first.1;
    let mut count = 1;
    let mut end_span = start_span;

    for (next, next_span) in instr_iter {
        match last {
            Instr::Add | Instr::Sub | Instr::Right | Instr::Left if last == next => {
                count += 1;
                end_span = *next_span;
                continue;
            }
            _ => {
                end_span = *next_span;
                let new_last = match last {
                    Instr::Add => Stmt::lol(StmtKind::Add(count)),
                    Instr::Sub => Stmt::lol(StmtKind::Sub(count)),
                    Instr::Right => Stmt::lol(StmtKind::Right(count.into())),
                    Instr::Left => Stmt::lol(StmtKind::Left(count.into())),
                    Instr::Out => Stmt::lol(StmtKind::Out),
                    Instr::In => Stmt::lol(StmtKind::In),
                    Instr::Loop(body) => Stmt::lol(StmtKind::Loop(ast_to_ir(alloc, body))),
                };
                stmts.push(new_last);
                spans.push(start_span.until(end_span));
                last = next;
                start_span = *next_span;
                count = 1;
            }
        }
    }

    let new_last = match last {
        Instr::Add => Stmt::lol(StmtKind::Add(count)),
        Instr::Sub => Stmt::lol(StmtKind::Sub(count)),
        Instr::Right => Stmt::lol(StmtKind::Right(count.into())),
        Instr::Left => Stmt::lol(StmtKind::Left(count.into())),
        Instr::Out => Stmt::lol(StmtKind::Out),
        Instr::In => Stmt::lol(StmtKind::In),
        Instr::Loop(body) => Stmt::lol(StmtKind::Loop(ast_to_ir(alloc, &body))),
    };
    stmts.push(new_last);
    spans.push(start_span.until(end_span));

    Ir { stmts }
}

fn pass_find_set_null(ir: &mut Ir<'_>) {
    for stmt in &mut ir.stmts {
        if let Stmt {
            kind: StmtKind::Loop(body),
            ..
        } = stmt
        {
            if let [Stmt {
                kind: StmtKind::Sub(_),
                ..
            }] = body.stmts.as_slice()
            {
                *stmt = Stmt::lol(StmtKind::SetNull);
            } else {
                pass_find_set_null(body);
            }
        }
    }
}
