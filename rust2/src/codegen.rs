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

use crate::opts::{Ir, Stmt as IrStmt, StmtKind};
use crate::parse::Span;
use bumpalo::Bump;

#[derive(Debug, Clone, Copy)]
pub enum Stmt {
    Add(u8),
    Sub(u8),
    Right(usize),
    Left(usize),
    Out,
    In,
    SetNull,
    JmpIfZero(usize),
    JmpIfNonZero(usize),
    End,
}

#[derive(Debug, Clone)]
pub struct Code<'c> {
    pub stmts: Vec<Stmt, &'c Bump>,
    pub debug: Vec<Span, &'c Bump>,
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
        StmtKind::Right(n) => Stmt::Right(*n),
        StmtKind::Left(n) => Stmt::Left(*n),
        StmtKind::Out => Stmt::Out,
        StmtKind::In => Stmt::In,
        StmtKind::SetNull => Stmt::SetNull,
        StmtKind::Loop(instr) => {
            let skip_jmp_idx = code.stmts.len();
            code.stmts.push(Stmt::JmpIfZero(usize::MAX)); // placeholder
            code.debug.push(ir_stmt.span);

            // compile the loop body now
            generate_stmts(code, &instr.stmts);
            // if the loop body is empty, we jmp to ourselves, which is an infinite loop - as expected
            let first_loop_body_idx = skip_jmp_idx + 1;
            code.stmts.push(Stmt::JmpIfNonZero(first_loop_body_idx));
            code.debug.push(ir_stmt.span);

            // there will always at least be an `End` instruction after the loop
            let after_loop_idx = code.stmts.len();

            // fix the placeholder with the actual index
            code.stmts[skip_jmp_idx] = Stmt::JmpIfZero(after_loop_idx);

            return;
        }
    };

    code.stmts.push(stmt);
    code.debug.push(ir_stmt.span);
}
