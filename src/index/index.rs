use std::{fs, marker::PhantomData, path::PathBuf, sync::Arc, time::Duration};

use tantivy::{collector::TopDocs, query::Query, schema::Schema, Index, IndexReader, IndexWriter};
use tokio::sync::Mutex;

use crate::{entity::entity_trait, util::async_retry};

pub struct SearchIndex<M>
where
    M: entity_trait::Index,
{
    writer: Arc<Mutex<IndexWriter>>,
    reader: IndexReader,

    phantom: PhantomData<M>,
}

impl<M> SearchIndex<M>
where
    M: entity_trait::Index,
{
    pub fn new(buffer_size: usize, index_path: PathBuf) -> Self {
        let schema = M::schema();
        // Create the Tantivy index
        let index = if index_path.exists() {
            // If the index directory exists, open the existing index
            println!("Opening existing index at {:?}", index_path);
            Index::open_in_dir(index_path)
        } else {
            // If the index directory doesn't exist, create a new index
            println!("Creating a new index at {:?}", index_path);
            fs::create_dir_all(index_path.clone()).expect("could not create output directory");
            Index::create_in_dir(index_path, schema.clone())
        };
        let index = index.unwrap();
        let writer: IndexWriter = index.writer(buffer_size).unwrap();

        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .unwrap();

        Self {
            writer: Arc::new(Mutex::new(writer)),
            reader,
            phantom: PhantomData,
        }
    }

    /// Adds the provided models to the search index without committing.
    pub async fn add<'a, T>(&self, models: T) -> tantivy::Result<()>
    where
        T: IntoIterator<Item = &'a M>,
        M: 'a,
    {
        let writer_lock = self.writer.lock().await;
        for model in models {
            // Delete by the primary key
            let primary_key_term = model.get_primary_key()?;
            writer_lock.delete_term(primary_key_term);

            writer_lock.add_document(model.as_document())?;
        }

        Ok(())
    }

    /// Removes the provided models from the search index without committing.
    pub async fn remove<'a, T>(&self, models: T) -> tantivy::Result<()>
    where
        T: IntoIterator<Item = &'a M>,
        M: 'a,
    {
        let writer_lock = self.writer.lock().await;
        for model in models {
            let primary_key_term = model.get_primary_key()?; // Assuming `get_primary_key` is implemented for `M`
            writer_lock.delete_term(primary_key_term);
        }
        Ok(())
    }

    /// Attempt to commit all pending changes.
    ///
    /// This function will retry up to 3 times in case of errors.
    pub async fn commit(&self) -> tantivy::Result<()> {
        let mut writer_lock = self.writer.lock().await;
        async_retry::retry_with_backoff(|_| writer_lock.commit(), 3, Duration::from_millis(100))
            .await?;

        Ok(())
    }

    pub fn query<Q>(&self, query: &Q, max_results: usize) -> tantivy::Result<Vec<M>>
    where
        Q: Query + Sized,
    {
        let searcher = self.reader.searcher();
        let documents = searcher.search(query, &TopDocs::with_limit(max_results))?;
        let models:Vec<M> = documents.into_iter().map(|(score,address)|{
            let doc = searcher.doc(address).unwrap();
            M::from_document(doc,score)
        }).collect();

        Ok(models)
    }

    /// Get the schema that this index uses
    pub fn schema() -> Schema {
        M::schema()
    }
}
