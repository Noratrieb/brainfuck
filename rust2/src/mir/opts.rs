use bumpalo::Bump;

use crate::mir::{
    state::{CellState, MemoryState, MemoryStateChange},
    Mir, StmtKind,
};

/// this pass fills out as much state info for all statements as possible
#[tracing::instrument]
pub fn pass_get_state_info<'mir>(alloc: &'mir Bump, mir: &mut Mir<'mir>) {
    let empty_state = MemoryState::empty(alloc);
    pass_get_state_info_inner(alloc, mir, empty_state);
}

#[tracing::instrument]
fn pass_get_state_info_inner<'mir>(
    alloc: &'mir Bump,
    mir: &mut Mir<'mir>,
    mut outer: MemoryState<'mir>,
) {
    for stmt in &mut mir.stmts {
        let state = match &mut stmt.kind {
            StmtKind::AddSub(offset, _, store) => MemoryState::single(
                alloc,
                outer,
                MemoryStateChange::Change {
                    offset: *offset,
                    new_state: CellState::WrittenToUnknown(store.clone()),
                },
            ),
            StmtKind::MoveAddTo {
                offset,
                store_set_null,
                store_move,
            } => MemoryState::double(
                alloc,
                outer,
                MemoryStateChange::Change {
                    offset: 0,
                    new_state: CellState::WrittenToKnown(store_set_null.clone(), 0),
                },
                MemoryStateChange::Change {
                    offset: *offset,
                    new_state: CellState::WrittenToUnknown(store_move.clone()),
                },
            ),
            StmtKind::PointerMove(n) => {
                MemoryState::single(alloc, outer, MemoryStateChange::Move(*n))
            }
            StmtKind::Loop(body) => {
                // TODO: we can get a lot smarter here and get huge benefits; we don't yet
                pass_get_state_info_inner(alloc, body, MemoryState::empty(alloc));
                MemoryState::double(
                    alloc,
                    outer,
                    // forget all knowledge, the opaque loop might have touched it all
                    MemoryStateChange::Forget,
                    // we certainly know that the current cell is zero, since the loop exited
                    MemoryStateChange::Change {
                        offset: 0,
                        new_state: CellState::LoopNull,
                    },
                )
            }
            StmtKind::Out => outer,
            StmtKind::In(store) => MemoryState::single(
                alloc,
                outer,
                MemoryStateChange::Change {
                    offset: 0,
                    new_state: CellState::WrittenToUnknown(store.clone()),
                },
            ),
            StmtKind::SetN(value, store) => MemoryState::single(
                alloc,
                outer,
                MemoryStateChange::Change {
                    offset: 0,
                    new_state: CellState::WrittenToKnown(store.clone(), *value),
                },
            ),
        };
        stmt.state = state.clone();
        outer = state;
    }
}

#[tracing::instrument]
fn pass_const_propagation(mir: &mut Mir<'_>) {}
