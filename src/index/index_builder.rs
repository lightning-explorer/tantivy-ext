use std::{cell::RefCell, marker::PhantomData, path::PathBuf};

use crate::entity::entity_trait;

use super::search_index::SearchIndex;

pub struct SearchIndexBuilder<M>
where
    M: entity_trait::Index,
{
    save_path: PathBuf,
    memory_budget_in_bytes: RefCell<usize>,

    phantom: PhantomData<M>,
}

impl<M> SearchIndexBuilder<M>
where
    M: entity_trait::Index,
{
    pub fn new(save_path: PathBuf) -> Self {
        Self {
            save_path,
            memory_budget_in_bytes: RefCell::new(50_000_000),
            phantom: PhantomData,
        }
    }

    pub fn with_memory_budget(self, memory_budget_in_bytes: usize) -> Self {
        *self.memory_budget_in_bytes.borrow_mut() = memory_budget_in_bytes;
        self
    }

    pub fn build(self) -> SearchIndex<M> {
        SearchIndex::new(*self.memory_budget_in_bytes.borrow(), self.save_path)
    }
}
