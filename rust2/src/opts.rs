use crate::parse::{Instr, Span};
use crate::BumpVec;
use bumpalo::Bump;

#[derive(Debug, Clone)]
pub struct Ir<'ir> {
    pub stmts: BumpVec<'ir, Stmt<'ir>>,
}

#[derive(Debug, Clone)]
pub struct Stmt<'ir> {
    pub kind: StmtKind<'ir>,
    pub span: Span,
}

impl<'ir> Stmt<'ir> {
    fn new(kind: StmtKind<'ir>, span: Span) -> Stmt<'ir> {
        Self { kind, span }
    }

    fn kind(&self) -> &StmtKind<'ir> {
        &self.kind
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
    SetN(u8),
}

pub fn optimize<'ir>(alloc: &'ir Bump, instrs: &[(Instr<'_>, Span)]) -> Ir<'ir> {
    let ir = ast_to_ir(alloc, instrs);
    let mut ir = pass_group(alloc, ir);
    pass_find_set_null(&mut ir);
    pass_set_n(&mut ir);
    ir
}

fn ast_to_ir<'ir>(alloc: &'ir Bump, ast: &[(Instr<'_>, Span)]) -> Ir<'ir> {
    let mut stmts = Vec::new_in(alloc);

    let stmts_iter = ast.iter().map(|(instr, span)| {
        let kind = match instr {
            Instr::Add => StmtKind::Add(1),
            Instr::Sub => StmtKind::Sub(1),
            Instr::Right => StmtKind::Right(1),
            Instr::Left => StmtKind::Left(1),
            Instr::Out => StmtKind::Out,
            Instr::In => StmtKind::In,
            Instr::Loop(body) => {
                let ir_body = ast_to_ir(alloc, body);
                StmtKind::Loop(ir_body)
            }
        };
        Stmt::new(kind, *span)
    });

    stmts.extend(stmts_iter);

    Ir { stmts }
}

/// pass that replaces things like `Sub(1) Sub(1)` with `Sub(2)`
fn pass_group<'ir>(alloc: &'ir Bump, ir: Ir<'ir>) -> Ir<'ir> {
    let new_stmts = Vec::new_in(alloc);
    let stmts = ir
        .stmts
        .into_iter()
        .fold(new_stmts, |mut stmts: BumpVec<'ir, Stmt<'ir>>, next| {
            let Some(old) = stmts.last_mut() else {
                if let StmtKind::Loop(body) = next.kind {
                    let new_body = pass_group(alloc, body);
                    stmts.push(Stmt::new(
                         StmtKind::Loop(new_body),
                        next.span,
                    ));
                } else {
                    stmts.push(next);
                }
                return stmts;
            };

            match (&mut old.kind, next.kind) {
                (StmtKind::Add(a), StmtKind::Add(b)) if *a < 255 => {
                    old.span = old.span.merge(next.span);
                    *a += b;
                }
                (StmtKind::Sub(a), StmtKind::Sub(b)) if *a < 255 => {
                    old.span = old.span.merge(next.span);
                    *a += b;
                }
                (StmtKind::Right(a), StmtKind::Right(b)) if *a < 255 => {
                    old.span = old.span.merge(next.span);
                    *a += b;
                }
                (StmtKind::Left(a), StmtKind::Left(b)) if *a < 255 => {
                    old.span = old.span.merge(next.span);
                    *a += b;
                }
                (_, StmtKind::Loop(body)) => {
                    let new_body = pass_group(alloc, body);
                    stmts.push(Stmt {
                        span: next.span,
                        kind: StmtKind::Loop(new_body),
                    });
                }
                (_, kind) => {
                    stmts.push(Stmt::new(kind, next.span));
                }
            }

            stmts
        });

    Ir { stmts }
}

/// pass that replaces `Loop([Sub(_)])` to `SetNull`
fn pass_find_set_null(ir: &mut Ir<'_>) {
    for stmt in &mut ir.stmts {
        if let Stmt {
            kind: StmtKind::Loop(body),
            ..
        } = stmt
        {
            if let [Stmt {
                kind: StmtKind::Sub(_),
                span,
            }] = body.stmts.as_slice()
            {
                *stmt = Stmt::new(StmtKind::SetNull, *span);
            } else {
                pass_find_set_null(body);
            }
        }
    }
}

/// pass that replaces `SetNull Add(5)` with `SetN(5)`
fn pass_set_n(ir: &mut Ir<'_>) {
    let stmts = &mut ir.stmts;
    let mut i = 0;
    while i < stmts.len() - 1 {
        let a = &stmts[i];
        if let StmtKind::SetNull = a.kind() {
            let b = &stmts[i + 1];
            let new = match b.kind() {
                StmtKind::Add(n) => StmtKind::SetN(*n),
                StmtKind::Sub(n) => StmtKind::SetN(0u8.wrapping_sub(*n)),
                _ => {
                    i += 1;
                    continue;
                }
            };
            let span = a.span.merge(b.span);
            stmts.remove(i + 1);
            stmts[i] = Stmt::new(new, span);
        }
        i += 1;
    }
}
