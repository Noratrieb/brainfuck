use std::fmt::{Debug, Formatter};

use bumpalo::Bump;

use crate::{
    parse::{Ast, Instr, Span},
    BumpVec,
};

pub mod opts;

#[derive(Clone)]
pub struct Hir<'hir> {
    pub stmts: BumpVec<'hir, Stmt<'hir>>,
}

impl Debug for Hir<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.stmts.fmt(f)
    }
}

#[derive(Clone)]
pub struct Stmt<'hir> {
    pub kind: StmtKind<'hir>,
    pub span: Span,
}

impl<'hir> Stmt<'hir> {
    fn new(kind: StmtKind<'hir>, span: Span) -> Stmt<'hir> {
        Self { kind, span }
    }

    fn kind(&self) -> &StmtKind<'hir> {
        &self.kind
    }
}

impl Debug for Stmt<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub enum StmtKind<'hir> {
    Add(i32, u8),
    Sub(i32, u8),
    /// Sets the current cell to 0 and adds that value of the cell to another cell at `offset`
    MoveAddTo {
        offset: i32,
    },
    Right(usize),
    Left(usize),
    Loop(Hir<'hir>),
    Out,
    In,
    SetN(u8),
}

fn ast_to_ir<'hir>(alloc: &'hir Bump, ast: &Ast<'_>) -> Hir<'hir> {
    let mut stmts = Vec::new_in(alloc);

    let stmts_iter = ast.iter().map(|(instr, span)| {
        let kind = match instr {
            Instr::Add => StmtKind::Add(0, 1),
            Instr::Sub => StmtKind::Sub(0, 1),
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

    Hir { stmts }
}

pub fn optimized_hir<'hir>(alloc: &'hir Bump, ast: &Ast<'_>) -> Hir<'hir> {
    let mut hir = ast_to_ir(alloc, ast);
    opts::optimize(alloc, &mut hir);
    hir
}
