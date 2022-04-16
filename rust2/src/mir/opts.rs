use bumpalo::Bump;
use tracing::info;

use crate::mir::{
    state::{CellState, MemoryState, MemoryStateChange},
    Mir, StmtKind,
};

/// this pass fills out as much state info for all statements as possible
#[tracing::instrument(skip(alloc, mir))]
pub fn passes<'mir>(alloc: &'mir Bump, mir: &mut Mir<'mir>) {
    pass_get_state_info(alloc, mir);
    pass_const_propagation(mir);
}
/// this pass fills out as much state info for all statements as possible
#[tracing::instrument(skip(alloc, mir))]
pub fn pass_get_state_info<'mir>(alloc: &'mir Bump, mir: &mut Mir<'mir>) {
    let empty_state = MemoryState::empty(alloc);
    pass_get_state_info_inner(alloc, mir, empty_state);
}

#[tracing::instrument(skip(alloc, mir))]
fn pass_get_state_info_inner<'mir>(
    alloc: &'mir Bump,
    mir: &mut Mir<'mir>,
    mut outer: MemoryState<'mir>,
) {
    for stmt in &mut mir.stmts {
        let state = match &mut stmt.kind {
            StmtKind::AddSub(offset, n, store) => {
                let prev_state = outer.state_for_offset(*offset);
                let new_state = match prev_state {
                    CellState::WrittenToKnown(_, prev_n) => {
                        let n = i16::from(prev_n).wrapping_add(*n);
                        let n = u8::try_from(n).unwrap();
                        CellState::WrittenToKnown(store.clone(), n)
                    }
                    _ => CellState::WrittenToUnknown(store.clone()),
                };
                MemoryState::single(
                    alloc,
                    outer,
                    MemoryStateChange::Change {
                        offset: *offset,
                        new_state,
                    },
                )
            }
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

#[tracing::instrument(skip(mir))]
fn pass_const_propagation(mir: &mut Mir<'_>) {
    pass_const_propagation_inner(mir)
}

fn pass_const_propagation_inner(mir: &mut Mir<'_>) {
    for stmt in &mut mir.stmts {
        match &mut stmt.kind {
            StmtKind::Out => {
                let state = stmt.state.state_for_offset(0);
                info!(?state, "We got the state of the output 😳😳😳");
                // we could now insert a `SetN` before the `Out`, to mark the previous store
                // as dead.
            }
            StmtKind::Loop(body) => {
                let state = stmt.state.state_for_offset(0);
                info!(?state, "We got the state of the output 😳😳😳");
                // we could now insert a `SetN` before the `Out`, to mark the previous store
                // as dead.
                pass_const_propagation_inner(body);
            }
            _ => {}
        }
    }
}
