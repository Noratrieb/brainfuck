use std::{
    cell::RefCell,
    fmt::{Debug, Formatter},
    rc::Rc,
};

use bumpalo::Bump;

use crate::BumpVec;

#[derive(Debug, Clone)]
pub enum CellState {
    Unknown,
    LoopNull,
    WrittenToUnknown(Store),
    WrittenToKnown(Store, u8),
}

/// A change in the known state of the memory caused by a single instruction
#[derive(Debug, Clone)]
pub enum MemoryStateChange {
    /// A cell was changed
    Change { offset: i32, new_state: CellState },
    /// The pointer was moved
    Move(i32),
    /// Forget everything
    Forget,
}

#[derive(Clone)]
pub struct MemoryState<'mir>(Rc<RefCell<MemoryStateInner<'mir>>>);

impl<'mir> MemoryState<'mir> {
    pub fn empty(alloc: &'mir Bump) -> Self {
        Self(Rc::new(RefCell::new(MemoryStateInner {
            prev: None,
            deltas: Vec::new_in(alloc),
        })))
    }

    pub fn state_for_offset(&self, offset: i32) -> CellState {
        self.0.borrow().state_for_offset(offset)
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
pub struct MemoryStateInner<'mir> {
    prev: Option<MemoryState<'mir>>,
    deltas: BumpVec<'mir, MemoryStateChange>,
}

impl<'mir> MemoryStateInner<'mir> {
    pub fn state_for_offset(&self, offset: i32) -> CellState {
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

impl<'mir> MemoryState<'mir> {
    pub fn single(
        alloc: &'mir Bump,
        prev: MemoryState<'mir>,
        delta: MemoryStateChange,
    ) -> MemoryState<'mir> {
        let mut deltas = Vec::new_in(alloc);
        deltas.push(delta);
        Self::new(prev, deltas)
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
        Self::new(prev, deltas)
    }

    pub fn new(
        prev: MemoryState<'mir>,
        deltas: BumpVec<'mir, MemoryStateChange>,
    ) -> MemoryState<'mir> {
        Self(Rc::new(RefCell::new(MemoryStateInner {
            prev: Some(prev),
            deltas,
        })))
    }
}

#[derive(Clone)]
pub struct Store(Rc<RefCell<StoreInner>>);

impl Store {
    pub fn unknown() -> Self {
        StoreInner::Unknown.into()
    }
}

impl Debug for Store {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0
            .try_borrow()
            .map(|s| StoreInner::fmt(&*s, f))
            .unwrap_or_else(|_| f.debug_struct("Store").finish_non_exhaustive())
    }
}

#[derive(Debug, Clone)]
pub enum StoreInner {
    Unknown,
    Used(usize),
    Dead,
}

impl From<StoreInner> for Store {
    fn from(inner: StoreInner) -> Self {
        Self(Rc::new(RefCell::new(inner)))
    }
}
