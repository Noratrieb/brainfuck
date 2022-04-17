// todo: we're gonna leak `Rc`s here aren't we?

use std::{
    cell::{Cell, RefCell},
    fmt::{Debug, Formatter},
    num::NonZeroU32,
    rc::Rc,
};

use bumpalo::Bump;

use crate::{mir::Offset, BumpVec};

/// The known state of a cell in the MIR
#[derive(Debug, Clone)]
pub enum CellState {
    /// The state of this cell is completely unknown and could be anything, for example after `,`
    Unknown,
    /// This cell is guaranteed to be `0` because a loop just terminated on it
    LoopNull,
    /// Some value was written to this cell classified by the `Store`, but we do not know the value
    WrittenToUnknown(Store),
    /// A known value was written to this cell
    WrittenToKnown(Store, u8),
}

/// A change in the known state of the memory caused by a single instruction
#[derive(Debug, Clone)]
pub enum MemoryStateChange {
    /// A cell value was changed to a new state.
    Change {
        offset: Offset,
        new_state: CellState,
    },
    /// The pointer was moved. This affects the `offset` calculations from previous states.
    Move(Offset),
    /// Forget everything about the memory state. This currently happens after each loop, since
    /// the loop is opaque and might clobber everything.
    Forget,
    /// Load a value from memory. This is not a direct change of the memory itself, but it does
    /// change the state in that it marks the corresponding store, if any, as alive. Loads should
    /// be eliminated whenever possible, to remove as many dead stores as possible.
    Load { offset: Offset },
}

/// The known state of memory at a specific instance in the instruction sequence
#[derive(Clone)]
pub struct MemoryState<'mir>(Rc<RefCell<MemoryStateInner<'mir>>>);

impl<'mir> MemoryState<'mir> {
    pub fn empty(alloc: &'mir Bump) -> Self {
        Self::new(None, Vec::new_in(alloc))
    }

    pub fn single(
        alloc: &'mir Bump,
        prev: MemoryState<'mir>,
        delta: MemoryStateChange,
    ) -> MemoryState<'mir> {
        let mut deltas = Vec::new_in(alloc);
        deltas.push(delta);
        Self::new(Some(prev), deltas)
    }

    pub fn double(
        alloc: &'mir Bump,
        prev: MemoryState<'mir>,
        delta1: MemoryStateChange,
        delta2: MemoryStateChange,
    ) -> MemoryState<'mir> {
        let mut deltas = Vec::new_in(alloc);
        deltas.push(delta1);
        deltas.push(delta2);
        Self::new(Some(prev), deltas)
    }

    pub fn new(
        prev: Option<MemoryState<'mir>>,
        deltas: BumpVec<'mir, MemoryStateChange>,
    ) -> MemoryState<'mir> {
        Self(Rc::new(RefCell::new(MemoryStateInner { prev, deltas })))
    }

    pub fn state_for_offset(&self, offset: Offset) -> CellState {
        self.0.borrow().state_for_offset(offset)
    }

    pub fn has_forget_delta(&self) -> bool {
        self.0
            .borrow()
            .deltas
            .iter()
            .any(|d| matches!(d, MemoryStateChange::Forget))
    }
}

impl Debug for MemoryState<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0
            .try_borrow()
            .map(|s| MemoryStateInner::fmt(&*s, f))
            .unwrap_or_else(|_| f.debug_struct("MemoryState").finish_non_exhaustive())
    }
}

/// The known state of memory relative to the pointer
#[derive(Debug, Clone)]
struct MemoryStateInner<'mir> {
    prev: Option<MemoryState<'mir>>,
    deltas: BumpVec<'mir, MemoryStateChange>,
}

impl<'mir> MemoryStateInner<'mir> {
    fn state_for_offset(&self, offset: Offset) -> CellState {
        let mut offset = offset;
        for delta in &self.deltas {
            match delta {
                MemoryStateChange::Change {
                    offset: write_offset,
                    new_state,
                } if *write_offset == offset => return new_state.clone(),
                MemoryStateChange::Move(change) => offset -= change,
                // we may not access the forbidden knowledge
                MemoryStateChange::Forget => return CellState::Unknown,
                _ => {}
            }
        }

        self.prev
            .as_ref()
            .map(|state| state.state_for_offset(offset))
            .unwrap_or(CellState::Unknown)
    }
}

/// The abstract representation of a store in memory. Corresponding loads can also hold
/// a reference to this to mark the store as alive
#[derive(Clone)]
pub struct Store(Rc<Cell<StoreInner>>);

impl Store {
    pub fn dead() -> Self {
        StoreKind::Dead.into()
    }

    pub fn id(&self) -> u64 {
        self.inner().id
    }

    pub fn add_load(&self) {
        let old = self.inner();
        let kind = match old.kind {
            StoreKind::Unknown => StoreKind::UsedAtLeast(NonZeroU32::new(1).unwrap()),
            StoreKind::UsedExact(n) => StoreKind::UsedExact(n.checked_add(1).unwrap()),
            StoreKind::UsedAtLeast(n) => StoreKind::UsedAtLeast(n.checked_add(1).unwrap()),
            StoreKind::Dead => StoreKind::UsedExact(NonZeroU32::new(1).unwrap()),
        };
        self.0.set(StoreInner { id: old.id, kind })
    }

    pub fn is_maybe_dead(&self) -> bool {
        matches!(self.inner().kind, StoreKind::Dead | StoreKind::Unknown)
    }

    pub fn mark_dead(&self) {
        let old = self.inner();
        self.0.set(StoreInner {
            id: old.id,
            kind: StoreKind::Dead,
        })
    }

    pub fn clobber(&self) {
        let old = self.inner();
        let kind = match old.kind {
            StoreKind::Unknown => StoreKind::Unknown,
            StoreKind::UsedExact(n) => StoreKind::UsedAtLeast(n),
            StoreKind::UsedAtLeast(n) => StoreKind::UsedAtLeast(n),
            StoreKind::Dead => StoreKind::Unknown,
        };
        self.0.set(StoreInner { id: old.id, kind })
    }

    fn inner(&self) -> StoreInner {
        self.0.get()
    }
}

impl Debug for Store {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner().kind.fmt(f)
    }
}

#[derive(Debug, Clone, Copy)]
struct StoreInner {
    id: u64,
    kind: StoreKind,
}

#[derive(Debug, Clone, Copy)]
enum StoreKind {
    /// No information is known about uses of the store, it has probably been clobbered
    Unknown,
    /// The exact amount of subsequent loads is known about the store, and it's this
    UsedExact(NonZeroU32),
    /// The exact amount of subsequent loads not known about this store, but it's at least this
    UsedAtLeast(NonZeroU32),
    /// The store is known to be dead
    Dead,
}

impl From<StoreKind> for Store {
    fn from(kind: StoreKind) -> Self {
        Self(Rc::new(Cell::new(StoreInner {
            id: rand::random(),
            kind,
        })))
    }
}
