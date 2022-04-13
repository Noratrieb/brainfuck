//! codegen to flat code
//!
//! ```bf
//! ++[-].
//! ```
//! compiles down to
//! ```text
//! Add | Add | JmpIfZero | Out | End | Sub | JmpIfNonZero | Jmp
//!                  |       |           ^         |          |
//!                  +-------------------+---------|----------+
//!                          +---------------------+
//! ```

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
    Jmp(usize),
    End,
}

#[derive(Debug, Clone)]
pub struct Code<'c> {
    pub stmts: Vec<Stmt, &'c Bump>,
    pub debug: Vec<Span, &'c Bump>,
}

struct UnlinkedCode<'u> {
    pub stmts: Vec<Vec<Stmt, &'u Bump>, &'u Bump>,
    pub debug: Vec<Vec<Span, &'u Bump>, &'u Bump>,
}

pub fn generate<'c>(alloc: &'c Bump, ir: &Ir<'_>) -> Code<'c> {
    let unlinked_alloc = Bump::new();

    let stmts = Vec::new_in(&unlinked_alloc);
    let debug = Vec::new_in(&unlinked_alloc);
    let mut unlinked = UnlinkedCode { stmts, debug };

    generate_stmts(&unlinked_alloc, &mut unlinked, &ir.stmts);

    link(alloc, &unlinked)
}

fn generate_stmts<'u>(alloc: &'u Bump, code: &mut UnlinkedCode<'u>, ir: &[IrStmt<'_>]) {
    for ir_stmt in ir {
        ir_to_stmt(alloc, code, ir_stmt, 0);
    }
    assert_eq!(code.stmts.len(), code.debug.len());
}

fn ir_to_stmt<'u>(
    alloc: &'u Bump,
    code: &mut UnlinkedCode<'u>,
    ir_stmt: &IrStmt<'_>,
    current_block: usize,
) {
    let stmt = match &ir_stmt.kind {
        StmtKind::Add(n) => Stmt::Add(*n),
        StmtKind::Sub(n) => Stmt::Sub(*n),
        StmtKind::Right(n) => Stmt::Right(*n),
        StmtKind::Left(n) => Stmt::Left(*n),
        StmtKind::Out => Stmt::Out,
        StmtKind::In => Stmt::In,
        StmtKind::SetNull => Stmt::SetNull,
        StmtKind::Loop(instr) => {
            let new_block = Vec::new_in(alloc);
            let new_block_debug = Vec::new_in(alloc);
            code.stmts.push(new_block);
            code.stmts.push(new_block_debug);

            let current_block = code.stmts.len() - 1;
            return;
        }
    };

    code.stmts[current_block].push(stmt);
    code.debug[current_block].push(ir_stmt.span);
}

fn link<'c>(alloc: &'c Bump, code: &UnlinkedCode<'_>) -> Code<'c> {
    todo!()
}
