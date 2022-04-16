use crate::parse::{Instr, Span};
use crate::BumpVec;
use bumpalo::Bump;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use tracing::trace;

#[derive(Clone)]
pub struct Ir<'ir> {
    pub stmts: BumpVec<'ir, Stmt<'ir>>,
}

impl Debug for Ir<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.stmts.fmt(f)
    }
}

#[derive(Clone)]
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

impl Debug for Stmt<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub enum StmtKind<'ir> {
    Add(u8),
    Sub(u8),
    AddOffset(i32, u8),
    SubOffset(i32, u8),
    Right(usize),
    Left(usize),
    Loop(Ir<'ir>),
    Out,
    In,
    SetN(u8),
}

pub fn optimize<'ir>(alloc: &'ir Bump, instrs: &[(Instr<'_>, Span)]) -> Ir<'ir> {
    let ir = ast_to_ir(alloc, instrs);
    let mut ir = pass_group(alloc, ir);
    pass_find_set_null(&mut ir);
    pass_set_n(&mut ir);
    pass_cancel_left_right_add_sub(&mut ir);
    pass_add_sub_offset(&mut ir);
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
#[tracing::instrument]
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
#[tracing::instrument]
fn pass_find_set_null(ir: &mut Ir<'_>) {
    for stmt in &mut ir.stmts {
        if let Stmt {
            kind: StmtKind::Loop(body),
            span,
        } = stmt
        {
            if let [Stmt {
                kind: StmtKind::Sub(_),
                ..
            }] = body.stmts.as_slice()
            {
                trace!(?span, "Replacing Statement with SetNull");
                *stmt = Stmt::new(StmtKind::SetN(0), *span);
            } else {
                pass_find_set_null(body);
            }
        }
    }
}

/// pass that replaces `SetN(n) Add(m)` with `SetN(n + m)`
#[tracing::instrument]
fn pass_set_n(ir: &mut Ir<'_>) {
    window_pass(ir, pass_set_n, |[a, b]| {
        if let StmtKind::SetN(before) = a.kind() {
            let new = match b.kind() {
                StmtKind::Add(n) => StmtKind::SetN(before.wrapping_add(*n)),
                StmtKind::Sub(n) => StmtKind::SetN(before.wrapping_sub(*n)),
                _ => {
                    return WindowPassAction::None;
                }
            };
            return WindowPassAction::Merge(new);
        }
        WindowPassAction::None
    });
}

/// pass that replaces `Left(5) Right(3)` with `Left(2)`
#[tracing::instrument]
fn pass_cancel_left_right_add_sub(ir: &mut Ir<'_>) {
    window_pass(ir, pass_cancel_left_right_add_sub, |[a, b]| {
        match (a.kind(), b.kind()) {
            (StmtKind::Right(r), StmtKind::Left(l)) | (StmtKind::Left(l), StmtKind::Right(r)) => {
                let new = match r.cmp(l) {
                    Ordering::Equal => {
                        return WindowPassAction::RemoveAll;
                    }
                    Ordering::Less => StmtKind::Left(l - r),
                    Ordering::Greater => StmtKind::Right(r - l),
                };

                WindowPassAction::Merge(new)
            }
            (StmtKind::Add(r), StmtKind::Sub(l)) | (StmtKind::Sub(l), StmtKind::Add(r)) => {
                let new = match r.cmp(l) {
                    Ordering::Equal => return WindowPassAction::RemoveAll,
                    Ordering::Less => StmtKind::Sub(l - r),
                    Ordering::Greater => StmtKind::Add(r - l),
                };

                WindowPassAction::Merge(new)
            }
            _ => WindowPassAction::None,
        }
    })
}

/// pass that replaces `Right(9) Add(5) Left(9)` with `AddOffset(9, 5)`
#[tracing::instrument]
fn pass_add_sub_offset(ir: &mut Ir<'_>) {
    window_pass(ir, pass_add_sub_offset, |[a, b, c]| {
        match (a.kind(), b.kind(), c.kind()) {
            (StmtKind::Right(r), StmtKind::Add(n), StmtKind::Left(l)) if r == l => {
                WindowPassAction::Merge(StmtKind::AddOffset(i32::try_from(*r).unwrap(), *n))
            }
            (StmtKind::Left(l), StmtKind::Add(n), StmtKind::Right(r)) if r == l => {
                WindowPassAction::Merge(StmtKind::AddOffset(-i32::try_from(*r).unwrap(), *n))
            }
            (StmtKind::Right(r), StmtKind::Sub(n), StmtKind::Left(l)) if r == l => {
                WindowPassAction::Merge(StmtKind::SubOffset(i32::try_from(*r).unwrap(), *n))
            }
            (StmtKind::Left(l), StmtKind::Sub(n), StmtKind::Right(r)) if r == l => {
                WindowPassAction::Merge(StmtKind::SubOffset(-i32::try_from(*r).unwrap(), *n))
            }
            _ => WindowPassAction::None,
        }
    })
}

enum WindowPassAction<'ir> {
    None,
    Merge(StmtKind<'ir>),
    RemoveAll,
}

fn window_pass<'ir, P, F, const N: usize>(ir: &mut Ir<'ir>, pass_recur: P, action: F)
where
    P: Fn(&mut Ir<'ir>),
    F: Fn([&Stmt<'ir>; N]) -> WindowPassAction<'ir>,
{
    assert!(N > 0);

    let stmts = &mut ir.stmts;
    let mut i = 0;
    while i < stmts.len() {
        let a = &mut stmts[i];
        if let StmtKind::Loop(body) = &mut a.kind {
            pass_recur(body);
        }

        if i + N > stmts.len() {
            break; // there aren't N elements left
        }

        let mut elements = stmts[i..][..N].iter();
        let elements = [(); N].map(|()| elements.next().unwrap());

        let merged_span = elements[0].span.merge(elements.last().unwrap().span);
        let result = action(elements);

        match result {
            WindowPassAction::None => {
                // only increment i if we haven't removed anything
                i += 1;
            }
            WindowPassAction::RemoveAll => {
                trace!(?elements, "Removing all statements");
                for _ in 0..N {
                    stmts.remove(i);
                }
            }
            WindowPassAction::Merge(new) => {
                trace!(?elements, ?new, "Merging statements");
                for _ in 1..N {
                    stmts.remove(i);
                }
                stmts[i] = Stmt::new(new, merged_span);
            }
        }
    }
}
