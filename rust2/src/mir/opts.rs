use std::collections::{hash_map::Entry, HashMap};

use bumpalo::Bump;
use tracing::info;

use crate::mir::{
    state::{CellState, MemoryState, MemoryStateChange, Store},
    Mir, Offset, StmtKind,
};

/// this pass fills out as much state info for all statements as possible
#[tracing::instrument(skip(alloc, mir))]
pub fn passes<'mir>(alloc: &'mir Bump, mir: &mut Mir<'mir>) {
    pass_fill_state_info(alloc, mir);
    pass_const_propagation(mir);
    pass_dead_store_elimination(mir);
}
/// this pass fills out as much state info for all statements as possible
#[tracing::instrument(skip(alloc, mir))]
pub fn pass_fill_state_info<'mir>(alloc: &'mir Bump, mir: &mut Mir<'mir>) {
    let empty_state = MemoryState::empty(alloc);
    pass_fill_state_info_inner(alloc, mir, empty_state);
}

#[tracing::instrument(skip(alloc, mir))]
fn pass_fill_state_info_inner<'mir>(
    alloc: &'mir Bump,
    mir: &mut Mir<'mir>,
    mut outer: MemoryState<'mir>,
) {
    for stmt in &mut mir.stmts {
        let state = match &mut stmt.kind {
            StmtKind::AddSub { offset, n, store } => {
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
                pass_fill_state_info_inner(alloc, body, MemoryState::empty(alloc));
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

/// This pass eliminates dead stores. It should probably be run multiple times between other passes
/// for cleanup
#[tracing::instrument(skip(mir))]
fn pass_dead_store_elimination(mir: &mut Mir<'_>) {
    pass_dead_store_elimination_mark_dead_stores(mir)
}

fn pass_dead_store_elimination_mark_dead_stores(mir: &mut Mir<'_>) {
    fn mark_store(
        potential_dead_stores: &mut HashMap<Offset, Store>,
        offset: Offset,
        store: &Store,
    ) {
        match potential_dead_stores.entry(offset) {
            Entry::Occupied(mut entry) => {
                let old = entry.insert(store.clone());
                if old.is_maybe_dead() {
                    // it's certainly dead
                    info!("We have a dead one!!!");
                    old.mark_dead();
                } else {
                    // it's alive and well, drop it and keep it marked alive
                    drop(old);
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(store.clone());
            }
        }
    }

    let mut potential_dead_stores = HashMap::new();
    let mut current_offset = 0;

    for stmt in &mir.stmts {
        match &stmt.kind {
            StmtKind::AddSub { store, offset, .. } => {
                mark_store(&mut potential_dead_stores, current_offset + offset, store);
            }
            StmtKind::MoveAddTo {
                offset,
                store_move,
                store_set_null,
            } => {
                mark_store(&mut potential_dead_stores, current_offset, store_set_null);
                mark_store(
                    &mut potential_dead_stores,
                    current_offset + offset,
                    store_move,
                );
            }
            StmtKind::PointerMove(offset) => {
                current_offset -= offset; // ???
            }
            StmtKind::Loop(_) | StmtKind::Out => {
                let store = potential_dead_stores.get(&current_offset);
                if let Some(store) = store {
                    store.add_load();
                }
            }
            StmtKind::In(store) | StmtKind::SetN(_, store) => {
                mark_store(&mut potential_dead_stores, current_offset, store);
            }
        }

        if stmt.state.has_forget_delta() {
            // they might all have loads now
            for store in potential_dead_stores.values_mut() {
                // TODO STOP: WE MUTATE THE STATE HERE!!! ALL COOL DEAD STORES WILL BE CLOBBERED
                store.clobber();
            }
        }
    }
}

// test pass
#[tracing::instrument(skip(mir))]
fn pass_const_propagation(mir: &mut Mir<'_>) {
    pass_const_propagation_inner(mir)
}

fn pass_const_propagation_inner(mir: &mut Mir<'_>) {
    for stmt in &mut mir.stmts {
        match &mut stmt.kind {
            StmtKind::Out => {
                let state = stmt.state.state_for_offset(0);
                // we could now insert a `SetN` before the `Out`, to mark the previous store
                // as dead.
            }
            StmtKind::Loop(body) => {
                let state = stmt.state.state_for_offset(0);
                // we could now insert a `SetN` before the `Loop`, to mark the previous store
                // as dead.
                pass_const_propagation_inner(body);
            }
            _ => {}
        }
    }
}
