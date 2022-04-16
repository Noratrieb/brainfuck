//! codegen to flat code
//!
//! ```bf
//! ++[-].
//! ```
//! compiles down to
//! ```text
//! Add | Add | JmpIfZero | Sub | JumpIfNotZero | Out | End
//!                  |       ^           |         ^
//!                  +-------|-----------|---------|
//!                          +-----------+
//! ```
//!
//! technically, the `JumpIfNotZero` would be an unconditional Jmp to the `JmpIfZero`, but that's
//! a needless indirection.
//!
//! this module must not produce out of bounds jumps and always put the `End` instruction at the
//! end

pub mod interpreter;

use std::fmt::{Debug, Formatter};

use bumpalo::Bump;

use crate::{
    hir::{Hir, Stmt as HirStmt, StmtKind as HirStmtKind},
    parse::Span,
    BumpVec,
};

#[derive(Debug, Clone, Copy)]
pub enum Stmt {
    Add(u8),
    Sub(u8),
    AddOffset { offset: i32, n: u8 },
    SubOffset { offset: i32, n: u8 },
    MoveAddTo { offset: i32 },
    Right(u32),
    Left(u32),
    Out,
    In,
    SetN(u8),
    JmpIfZero(u32),
    JmpIfNonZero(u32),
    End,
}

const _: [(); 8] = [(); std::mem::size_of::<Stmt>()];

#[derive(Clone)]
pub struct Lir<'lir> {
    stmts: BumpVec<'lir, Stmt>,
    debug: BumpVec<'lir, Span>,
}

impl Debug for Lir<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.stmts.fmt(f)
    }
}

impl Lir<'_> {
    pub fn stmts(&self) -> &[Stmt] {
        &self.stmts
    }

    pub fn debug(&self) -> &[Span] {
        &self.debug
    }
}

pub fn generate<'lir>(alloc: &'lir Bump, ir: &Hir<'_>) -> Lir<'lir> {
    let stmts = Vec::new_in(alloc);
    let debug = Vec::new_in(alloc);
    let mut lir = Lir { stmts, debug };

    hir_to_lir(&mut lir, &ir.stmts);
    lir.stmts.push(Stmt::End);
    lir.debug.push(Span::default());

    assert_eq!(lir.stmts.len(), lir.debug.len());

    lir
}

fn hir_to_lir<'lir>(lir: &mut Lir<'lir>, ir: &[HirStmt<'_>]) {
    for ir_stmt in ir {
        hir_stmt_to_lir_stmt(lir, ir_stmt);
    }
    debug_assert_eq!(lir.stmts.len(), lir.debug.len());
}

fn hir_stmt_to_lir_stmt<'lir>(lir: &mut Lir<'lir>, ir_stmt: &HirStmt<'_>) {
    let stmt = match &ir_stmt.kind {
        HirStmtKind::Add(0, n) => Stmt::Add(*n),
        HirStmtKind::Sub(0, n) => Stmt::Sub(*n),
        HirStmtKind::Add(offset, n) => Stmt::AddOffset {
            offset: *offset,
            n: *n,
        },
        HirStmtKind::Sub(offset, n) => Stmt::SubOffset {
            offset: *offset,
            n: *n,
        },
        HirStmtKind::MoveAddTo { offset } => Stmt::MoveAddTo { offset: *offset },
        HirStmtKind::Right(n) => Stmt::Right(u32::try_from(*n).unwrap()),
        HirStmtKind::Left(n) => Stmt::Left(u32::try_from(*n).unwrap()),
        HirStmtKind::Out => Stmt::Out,
        HirStmtKind::In => Stmt::In,
        HirStmtKind::SetN(n) => Stmt::SetN(*n),
        HirStmtKind::Loop(instr) => {
            let skip_jmp_idx = lir.stmts.len();
            lir.stmts.push(Stmt::JmpIfZero(0)); // placeholder
            lir.debug.push(ir_stmt.span);

            // compile the loop body now
            hir_to_lir(lir, &instr.stmts);
            // if the loop body is empty, we jmp to ourselves, which is an infinite loop - as expected
            let first_loop_body_idx = skip_jmp_idx + 1;
            lir.stmts
                .push(Stmt::JmpIfNonZero(first_loop_body_idx.try_into().unwrap()));
            lir.debug.push(ir_stmt.span);

            // there will always at least be an `End` instruction after the loop
            let after_loop_idx = lir.stmts.len();

            // fix the placeholder with the actual index
            lir.stmts[skip_jmp_idx] = Stmt::JmpIfZero(after_loop_idx.try_into().unwrap());

            return;
        }
    };

    lir.stmts.push(stmt);
    lir.debug.push(ir_stmt.span);
}
