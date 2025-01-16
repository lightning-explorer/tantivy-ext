use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use tantivy::{Index, IndexWriter};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct IndexWriterRecycler {
    index: Arc<Index>,
    writer: Arc<RwLock<Option<IndexWriter>>>, // Use Option for handling writer state
    mem_budget: usize,

    entries_processed: Arc<AtomicUsize>,
    entries_before_recycle: usize,
}

impl IndexWriterRecycler {
    pub fn new(index: Arc<Index>, mem_budget: usize, entries_before_recycle: usize) -> Self {
        let writer = Arc::new(RwLock::new(Some(index.writer(mem_budget).unwrap())));
        Self {
            index,
            writer,
            mem_budget,
            entries_processed: Arc::new(AtomicUsize::new(0)),
            entries_before_recycle,
        }
    }

    pub fn get_writer(&self) -> Arc<RwLock<Option<IndexWriter>>> {
        Arc::clone(&self.writer)
    }

    /// Returns an error if the function tries to replace the `IndexWriter` but fails
    pub async fn register_entries_processed(&self, num: usize) -> tantivy::Result<()> {
        let entries = self.entries_processed.load(Ordering::Relaxed);
        let new_num = entries + num;
        if new_num > self.entries_before_recycle {
            self.entries_processed.store(0, Ordering::Relaxed);
            self.replace_writer().await?;
        } else {
            self.entries_processed
                .store(entries + num, Ordering::Relaxed);
        }
        Ok(())
    }

    async fn replace_writer(&self) -> tantivy::Result<()> {
        let mut writer_lock = self.writer.write().await;

        if let Some(old_writer) = writer_lock.take() {
            // Wait for the old writer to clean up
            println!("Recycling writer");
            old_writer.wait_merging_threads()?;
        }

        // Now it is safe to create a new writer
        let new_writer = self.index.writer(self.mem_budget)?;
        *writer_lock = Some(new_writer);

        Ok(())
    }
}