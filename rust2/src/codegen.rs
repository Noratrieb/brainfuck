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

use crate::opts::{Ir, Stmt as IrStmt, StmtKind};
use crate::parse::Span;
use crate::BumpVec;
use bumpalo::Bump;

#[derive(Debug, Clone, Copy)]
pub enum Stmt {
    Add(u8),
    Sub(u8),
    Right(u32),
    Left(u32),
    Out,
    In,
    SetNull,
    JmpIfZero(u32),
    JmpIfNonZero(u32),
    End,
}

const _: [(); 8] = [(); std::mem::size_of::<Stmt>()];

#[derive(Debug, Clone)]
pub struct Code<'c> {
    stmts: BumpVec<'c, Stmt>,
    debug: BumpVec<'c, Span>,
}

impl Code<'_> {
    pub fn stmts(&self) -> &[Stmt] {
        &self.stmts
    }

    pub fn debug(&self) -> &[Span] {
        &self.debug
    }
}

pub fn generate<'c>(alloc: &'c Bump, ir: &Ir<'_>) -> Code<'c> {
    let stmts = Vec::new_in(alloc);
    let debug = Vec::new_in(alloc);
    let mut code = Code { stmts, debug };

    generate_stmts(&mut code, &ir.stmts);
    code.stmts.push(Stmt::End);
    code.debug.push(Span::default());

    assert_eq!(code.stmts.len(), code.debug.len());

    code
}

fn generate_stmts<'c>(code: &mut Code<'c>, ir: &[IrStmt<'_>]) {
    for ir_stmt in ir {
        ir_to_stmt(code, ir_stmt);
    }
    debug_assert_eq!(code.stmts.len(), code.debug.len());
}

fn ir_to_stmt<'c>(code: &mut Code<'c>, ir_stmt: &IrStmt<'_>) {
    let stmt = match &ir_stmt.kind {
        StmtKind::Add(n) => Stmt::Add(*n),
        StmtKind::Sub(n) => Stmt::Sub(*n),
        StmtKind::Right(n) => Stmt::Right(u32::try_from(*n).unwrap()),
        StmtKind::Left(n) => Stmt::Left(u32::try_from(*n).unwrap()),
        StmtKind::Out => Stmt::Out,
        StmtKind::In => Stmt::In,
        StmtKind::SetNull => Stmt::SetNull,
        StmtKind::Loop(instr) => {
            let skip_jmp_idx = code.stmts.len();
            code.stmts.push(Stmt::JmpIfZero(0)); // placeholder
            code.debug.push(ir_stmt.span);

            // compile the loop body now
            generate_stmts(code, &instr.stmts);
            // if the loop body is empty, we jmp to ourselves, which is an infinite loop - as expected
            let first_loop_body_idx = skip_jmp_idx + 1;
            code.stmts
                .push(Stmt::JmpIfNonZero(first_loop_body_idx.try_into().unwrap()));
            code.debug.push(ir_stmt.span);

            // there will always at least be an `End` instruction after the loop
            let after_loop_idx = code.stmts.len();

            // fix the placeholder with the actual index
            code.stmts[skip_jmp_idx] = Stmt::JmpIfZero(after_loop_idx.try_into().unwrap());

            return;
        }
    };

    code.stmts.push(stmt);
    code.debug.push(ir_stmt.span);
}
