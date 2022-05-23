use std::cmp::Ordering;

use bumpalo::Bump;
use tracing::trace;

use crate::{
    hir::{Hir, Stmt, StmtKind},
    BumpVec,
};

pub fn optimize<'hir>(alloc: &'hir Bump, hir: &mut Hir<'hir>) {
    pass_group(alloc, hir);
    pass_find_set_null(hir);
    pass_set_n(hir);
    pass_cancel_left_right_add_sub(hir);
    pass_add_sub_offset(hir);
    pass_move_add_to(hir);
    // pass_unroll_loops(hir);
    // pass_cancel_left_right_add_sub(hir);
}

/// pass that replaces things like `Sub(1) Sub(1)` with `Sub(2)`
// TODO: This pass is really slow, speed it up please
#[tracing::instrument(skip(alloc, ir_param))]
fn pass_group<'hir>(alloc: &'hir Bump, ir_param: &mut Hir<'hir>) {
    let empty_ir = Hir {
        stmts: Vec::new_in(alloc),
    };

    let ir = std::mem::replace(ir_param, empty_ir);

    let new_stmts = Vec::new_in(alloc);
    let stmts =
        ir.stmts
            .into_iter()
            .fold(new_stmts, |mut stmts: BumpVec<'hir, Stmt<'hir>>, next| {
                let Some(old) = stmts.last_mut() else {
                if let StmtKind::Loop(mut body) = next.kind {
                    pass_group(alloc, &mut body);
                    stmts.push(Stmt::new(
                         StmtKind::Loop(body),
                        next.span,
                    ));
                } else {
                    stmts.push(next);
                }
                return stmts;
            };

                match (&mut old.kind, next.kind) {
                    (StmtKind::Add(offset_a, a), StmtKind::Add(offset_b, b))
                        if *a < 255 && *offset_a == offset_b =>
                    {
                        old.span = old.span.merge(next.span);
                        *a += b;
                    }
                    (StmtKind::Sub(offset_a, a), StmtKind::Sub(offset_b, b))
                        if *a < 255 && *offset_a == offset_b =>
                    {
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
                    (_, StmtKind::Loop(mut body)) => {
                        pass_group(alloc, &mut body);
                        stmts.push(Stmt {
                            span: next.span,
                            kind: StmtKind::Loop(body),
                        });
                    }
                    (_, kind) => {
                        stmts.push(Stmt::new(kind, next.span));
                    }
                }

                stmts
            });

    *ir_param = Hir { stmts };
}

/// pass that replaces `Loop([Sub(_)])` to `SetNull`
#[tracing::instrument(skip(ir))]
fn pass_find_set_null(ir: &mut Hir<'_>) {
    pass_find_set_null_inner(ir)
}

fn pass_find_set_null_inner(ir: &mut Hir<'_>) {
    for stmt in &mut ir.stmts {
        if let Stmt {
            kind: StmtKind::Loop(body),
            span,
        } = stmt
        {
            if let [Stmt {
                kind: StmtKind::Sub(0, _),
                ..
            }] = body.stmts.as_slice()
            {
                trace!(?span, "Replacing Statement with SetNull");
                *stmt = Stmt::new(StmtKind::SetN(0), *span);
            } else {
                pass_find_set_null_inner(body);
            }
        }
    }
}

/// pass that replaces `SetN(n) Add(m)` with `SetN(n + m)`
#[tracing::instrument(skip(ir))]
fn pass_set_n(ir: &mut Hir<'_>) {
    pass_set_n_inner(ir)
}
fn pass_set_n_inner(ir: &mut Hir<'_>) {
    window_pass(ir, pass_set_n_inner, |[a, b]| {
        if let StmtKind::SetN(before) = a.kind() {
            let new = match b.kind() {
                StmtKind::Add(0, n) => StmtKind::SetN(before.wrapping_add(*n)),
                StmtKind::Sub(0, n) => StmtKind::SetN(before.wrapping_sub(*n)),
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
#[tracing::instrument(skip(ir))]
fn pass_cancel_left_right_add_sub(ir: &mut Hir<'_>) {
    pass_cancel_left_right_add_sub_inner(ir)
}

fn pass_cancel_left_right_add_sub_inner(ir: &mut Hir<'_>) {
    window_pass(ir, pass_cancel_left_right_add_sub_inner, |[a, b]| {
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
            (StmtKind::Add(offset_a, r), StmtKind::Sub(offset_b, l))
            | (StmtKind::Sub(offset_a, l), StmtKind::Add(offset_b, r))
                if offset_a == offset_b =>
            {
                let new = match r.cmp(l) {
                    Ordering::Equal => return WindowPassAction::RemoveAll,
                    Ordering::Less => StmtKind::Sub(*offset_a, l - r),
                    Ordering::Greater => StmtKind::Add(*offset_a, r - l),
                };

                WindowPassAction::Merge(new)
            }
            _ => WindowPassAction::None,
        }
    })
}

/// pass that replaces `Right(9) Add(5) Left(9)` with `AddOffset(9, 5)`
#[tracing::instrument(skip(ir))]
fn pass_add_sub_offset(ir: &mut Hir<'_>) {
    pass_add_sub_offset_inner(ir)
}
fn pass_add_sub_offset_inner(ir: &mut Hir<'_>) {
    window_pass(ir, pass_add_sub_offset_inner, |[a, b, c]| {
        match (a.kind(), b.kind(), c.kind()) {
            (StmtKind::Right(r), StmtKind::Add(0, n), StmtKind::Left(l)) if r == l => {
                WindowPassAction::Merge(StmtKind::Add(i32::try_from(*r).unwrap(), *n))
            }
            (StmtKind::Left(l), StmtKind::Add(0, n), StmtKind::Right(r)) if r == l => {
                WindowPassAction::Merge(StmtKind::Add(-i32::try_from(*r).unwrap(), *n))
            }
            (StmtKind::Right(r), StmtKind::Sub(0, n), StmtKind::Left(l)) if r == l => {
                WindowPassAction::Merge(StmtKind::Sub(i32::try_from(*r).unwrap(), *n))
            }
            (StmtKind::Left(l), StmtKind::Sub(0, n), StmtKind::Right(r)) if r == l => {
                WindowPassAction::Merge(StmtKind::Sub(-i32::try_from(*r).unwrap(), *n))
            }
            _ => WindowPassAction::None,
        }
    })
}

/// pass that replaces `Loop([Sub(1) AddOffset(o, 1)])` with `MoveAddTo(o)`
#[tracing::instrument(skip(ir))]
fn pass_move_add_to(ir: &mut Hir<'_>) {
    pass_move_add_to_inner(ir)
}

fn pass_move_add_to_inner(ir: &mut Hir<'_>) {
    for stmt in &mut ir.stmts {
        if let Stmt {
            kind: StmtKind::Loop(body),
            span,
        } = stmt
        {
            if let [Stmt {
                kind: StmtKind::Sub(0, 1),
                ..
            }, Stmt {
                kind: StmtKind::Add(offset, 1),
                ..
            }]
            | [Stmt {
                kind: StmtKind::Add(offset, 1),
                ..
            }, Stmt {
                kind: StmtKind::Sub(0, 1),
                ..
            }] = body.stmts.as_slice()
            {
                trace!(?span, ?offset, "Replacing Statement with MoveAddTo");
                *stmt = Stmt::new(StmtKind::MoveAddTo { offset: *offset }, *span);
            } else {
                pass_move_add_to_inner(body);
            }
        }
    }
}

#[tracing::instrument(skip(ir))]
fn pass_unroll_loops(ir: &mut Hir<'_>) {
    let alloc = Bump::new();
    pass_unroll_loops_inner(&alloc, ir);
}

fn pass_unroll_loops_inner(alloc: &Bump, ir: &mut Hir<'_>) {
    window_pass(ir, pass_unroll_loops, |[a, b]| {
        if let (StmtKind::SetN(n), StmtKind::Loop(body)) = (a.kind(), b.kind()) {
            let mut stmts_vec = BumpVec::new_in(alloc);

            let stmts = std::iter::repeat(body.stmts.iter())
                .take(usize::from(*n))
                .flatten()
                .cloned();
            stmts_vec.extend(stmts);

            WindowPassAction::MergeMany(stmts_vec)
        } else {
            WindowPassAction::None
        }
    })
}

enum WindowPassAction<'hir, 'pass> {
    None,
    Merge(StmtKind<'hir>),
    MergeMany(BumpVec<'pass, Stmt<'hir>>),
    RemoveAll,
}

fn window_pass<'hir, 'pass, P, F, const N: usize>(ir: &mut Hir<'hir>, pass_recur: P, action: F)
where
    P: Fn(&mut Hir<'hir>),
    F: Fn([&Stmt<'hir>; N]) -> WindowPassAction<'hir, 'pass>,
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
            WindowPassAction::MergeMany(new) => {
                trace!(?elements, ?new, "Merging many");
                for _ in 0..N {
                    stmts.remove(i);
                }
                for stmt in new.into_iter().rev() {
                    stmts.insert(i, stmt);
                }
            }
        }
    }
}
