//! an experimental MIR (mid-level-ir)
#![allow(dead_code)]

mod opts;
mod state;

use std::fmt::{Debug, Formatter};

use bumpalo::Bump;

use crate::{
    hir::{Hir, StmtKind as HirStmtKind},
    mir::state::{MemoryState, Store, StoreInner},
    parse::Span,
    BumpVec,
};

#[derive(Debug, Clone)]
pub struct Mir<'mir> {
    stmts: BumpVec<'mir, Stmt<'mir>>,
}

#[derive(Clone)]
struct Stmt<'mir> {
    kind: StmtKind<'mir>,
    state: MemoryState<'mir>,
    span: Span,
}

impl Debug for Stmt<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stmt")
            .field("kind", &self.kind)
            .field("state", &self.state)
            .finish()
    }
}

#[derive(Debug, Clone)]
enum StmtKind<'mir> {
    /// Add or sub, the value has the valid range -255..=255
    AddSub(i32, i16, Store),
    /// Sets the current cell to 0 and adds that value of the cell to another cell at `offset`
    MoveAddTo {
        offset: i32,
        store_set_null: Store,
        store_move: Store,
    },
    /// Left or Right pointer move (`<>`)
    PointerMove(i32),
    Loop(Mir<'mir>),
    Out,
    In(Store),
    SetN(u8, Store),
}

#[tracing::instrument(skip(alloc, hir))]
pub fn optimized_mir<'mir>(alloc: &'mir Bump, hir: &Hir<'_>) -> Mir<'mir> {
    let mut mir = hir_to_mir(alloc, hir);
    opts::passes(alloc, &mut mir);
    mir
}

/// compiles hir down to a minimal mir
fn hir_to_mir<'mir>(alloc: &'mir Bump, hir: &Hir<'_>) -> Mir<'mir> {
    let mut stmts = Vec::new_in(alloc);
    let iter = hir.stmts.iter().map(|hir_stmt| {
        let kind = match *hir_stmt.kind() {
            HirStmtKind::Add(offset, n) => StmtKind::AddSub(offset, i16::from(n), Store::unknown()),
            HirStmtKind::Sub(offset, n) => {
                StmtKind::AddSub(offset, -i16::from(n), Store::unknown())
            }
            HirStmtKind::MoveAddTo { offset } => StmtKind::MoveAddTo {
                offset,
                store_set_null: Store::unknown(),
                store_move: Store::unknown(),
            },
            HirStmtKind::Right(n) => StmtKind::PointerMove(i32::try_from(n).unwrap()),
            HirStmtKind::Left(n) => StmtKind::PointerMove(-i32::try_from(n).unwrap()),
            HirStmtKind::Loop(ref body) => StmtKind::Loop(hir_to_mir(alloc, body)),
            HirStmtKind::Out => StmtKind::Out,
            HirStmtKind::In => StmtKind::In(StoreInner::Unknown.into()),
            HirStmtKind::SetN(n) => StmtKind::SetN(n, Store::unknown()),
        };
        Stmt {
            kind,
            span: hir_stmt.span,
            state: MemoryState::empty(alloc),
        }
    });
    stmts.extend(iter);

    Mir { stmts }
}
