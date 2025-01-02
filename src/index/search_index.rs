use std::{fs, marker::PhantomData, path::PathBuf, sync::Arc, time::Duration};

use tantivy::{query::Query, schema::Schema, Index, IndexReader, IndexWriter, Searcher, Term};
use tokio::sync::Mutex;

use crate::{entity::entity_trait, util::async_retry};

use super::query::builder::QueryBuilder;

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

    /// Adds the provided models to the search index and then commits the changes.
    pub async fn add<'a, T>(&self, models: T) -> tantivy::Result<()>
    where
        T: IntoIterator<Item = &'a M>,
        M: 'a,
    {
        let writer_lock = self.writer.lock().await;
        for model in models {
            // Delete by the primary key
            let primary_key_term = model.get_primary_key();
            writer_lock.delete_term(primary_key_term);

            writer_lock.add_document(model.as_document())?;
        }
        self.commit(writer_lock).await?;
        Ok(())
    }

    /// Removes the provided models from the search index and then commits the changes.
    pub async fn remove<'a, T>(&self, models: T) -> tantivy::Result<()>
    where
        T: IntoIterator<Item = &'a M>,
        M: 'a,
    {
        let writer_lock = self.writer.lock().await;
        for model in models {
            let primary_key_term = model.get_primary_key();
            writer_lock.delete_term(primary_key_term);
        }
        self.commit(writer_lock).await?;
        Ok(())
    }

    /// Removes all models from the search index with the provided term 
    /// 
    /// Example:
    /// ```rust
    /// let term = MyModel::name_field().term(String::from("Joe"));
    /// index.remove_by_terms(vec![term]).await;
    /// ```
    pub async fn remove_by_terms(&self,terms:Vec<Term>) -> tantivy::Result<()>{
        let writer_lock = self.writer.lock().await;
        for term in terms{
            writer_lock.delete_term(term);

        }
        self.commit(writer_lock).await
    }

    /// Attempt to commit all pending changes.
    ///
    /// This function will retry up to 3 times in case of errors.
    async fn commit(
        &self,
        mut writer_lock: tokio::sync::MutexGuard<'_, IndexWriter>,
    ) -> tantivy::Result<()> {
        async_retry::retry_with_backoff(|_| writer_lock.commit(), 3, Duration::from_millis(100))
            .await?;

        Ok(())
    }

    pub fn query<'a, Q>(&self, query: &'a Q, max_results: usize) -> QueryBuilder<'a, Q, M>
    where
        Q: Query + Sized,
    {
        let searcher = self.reader.searcher();
        QueryBuilder::new(query, searcher, max_results)
    }

    pub fn scored_docs_to_models(&self, docs:Vec<(f64, tantivy::DocAddress)>)->Vec<M>{
        let mut res = Vec::new();
        let searcher = self.reader.searcher();
        for (score,address) in docs{
            let doc = searcher.doc(address).unwrap();
            res.push(M::from_document(doc, score as f32));
        }
        res
    }

    /// Get the schema that this index uses
    pub fn schema() -> Schema {
        M::schema()
    }

    pub fn searcher(&self) -> Searcher {
        self.reader.searcher()
    }
}
