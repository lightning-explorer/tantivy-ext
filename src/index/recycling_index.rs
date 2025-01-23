use std::{
    fs,
    marker::PhantomData,
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use tantivy::{
    query::{Query, QueryParser},
    schema::{Field, Schema},
    Index, IndexReader, IndexWriter, Term,
};
use tokio::sync::RwLock;

use crate::{entity::entity_trait, util::async_retry};

use super::query::builder::QueryBuilder;

/// A tantivy search index over instances of the provided struct.
///
/// Because all of the underlying data wraps an `Arc`, this can be negligibly cloned
#[derive(Clone)]
pub struct RecyclingSearchIndex<M>
where
    M: entity_trait::Index,
{
    inner: Arc<RwLock<Option<TantivyBackend>>>,

    index_path: PathBuf,
    buffer_size: usize,
    entries_processed: Arc<AtomicUsize>,
    entries_before_recycle: usize,
    phantom: PhantomData<M>,
}
pub struct TantivyBackend {
    writer: IndexWriter,
    reader: IndexReader,
    index: tantivy::Index,
}

impl<M> RecyclingSearchIndex<M>
where
    M: entity_trait::Index,
{
    pub fn new(index_path: PathBuf, buffer_size: usize, entries_before_recycle: usize) -> Self {
        let schema = M::schema();
        // Create the Tantivy index
        let index = if index_path.exists() {
            // If the index directory exists, open the existing index
            println!("Opening existing index at {:?}", index_path);
            Index::open_in_dir(index_path.clone())
        } else {
            // If the index directory doesn't exist, create a new index
            println!("Creating a new index at {:?}", index_path);
            fs::create_dir_all(index_path.clone()).expect("could not create output directory");
            Index::create_in_dir(index_path.clone(), schema.clone())
        };
        let index = index.unwrap();

        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .unwrap();

        let writer = index.writer(buffer_size).unwrap();

        Self {
            inner: Arc::new(RwLock::new(Some(TantivyBackend {
                writer,
                reader,
                index,
            }))),
            index_path,
            buffer_size,
            entries_processed: Arc::new(AtomicUsize::new(0)),
            entries_before_recycle,
            phantom: PhantomData,
        }
    }

    /// Adds the provided models to the search index and then commits the changes.
    pub async fn add(&self, models: &[M]) -> tantivy::Result<()> {
        let models_len = models.len();
        {
            let mut inner_lock = self.inner.write().await;
            let inner = inner_lock.as_mut().unwrap();
            for model in models {
                // Delete by the primary key
                let primary_key_term = model.get_primary_key();
                inner.writer.delete_term(primary_key_term);

                inner.writer.add_document(model.as_document())?;
            }
            self.commit(&mut inner.writer).await?;
        }
        // Writer lock must be dropped so this function can use it
        self.register_entries_processed(models_len).await?;
        Ok(())
    }

    /// Removes the provided models from the search index and then commits the changes.
    pub async fn remove<'a, T>(&self, models: &[M]) -> tantivy::Result<()> {
        let models_len = models.len();
        let mut inner_lock = self.inner.write().await;
        let inner = inner_lock.as_mut().unwrap();
        {
            for model in models {
                let primary_key_term = model.get_primary_key();
                inner.writer.delete_term(primary_key_term);
            }
            self.commit(&mut inner.writer).await?;
        }
        // Writer lock must be dropped so this function can use it
        self.register_entries_processed(models_len).await?;
        Ok(())
    }

    /// Removes all models from the search index with the provided term
    ///
    /// Example:
    /// ```rust
    /// let term = MyModel::name_field().term(String::from("Joe"));
    /// index.remove_by_terms(vec![term]).await;
    /// ```
    pub async fn remove_by_terms(&self, terms: Vec<Term>) -> tantivy::Result<()> {
        let mut inner_lock = self.inner.write().await;
        let inner = inner_lock.as_mut().unwrap();
        let terms_len = terms.len();
        {
            for term in terms {
                inner.writer.delete_term(term);
            }
            self.commit(&mut inner.writer).await?;
        }
        self.register_entries_processed(terms_len).await?;
        Ok(())
    }

    /// Attempt to commit all pending changes.
    ///
    /// This function will retry up to 3 times in case of errors.
    async fn commit(&self, writer: &mut IndexWriter) -> tantivy::Result<()> {
        async_retry::retry_with_backoff(|_| writer.commit(), 3, Duration::from_millis(100)).await?;

        Ok(())
    }

    /// Get the query parser for this search index
    pub async fn query_parser(&self, default_fields: Vec<Field>) -> QueryParser {
        QueryParser::for_index(
            &self.inner.read().await.as_ref().unwrap().index,
            default_fields,
        )
    }

    pub async fn query<'a, Q>(&self, query: &'a Q, max_results: usize) -> QueryBuilder<'a, Q, M>
    where
        Q: Query + Sized,
    {
        let searcher = self.inner.read().await.as_ref().unwrap().reader.searcher();
        QueryBuilder::new(query, searcher, max_results)
    }

    pub async fn scored_doc_to_model(&self, doc: (f64, tantivy::DocAddress)) -> tantivy::Result<M> {
        let searcher = self.inner.read().await.as_ref().unwrap().reader.searcher();
        let (score, address) = doc;

        let doc = searcher.doc(address)?;

        Ok(M::from_document(doc, score as f32))
    }

    /// Get the schema that this index uses
    pub fn schema() -> &'static Schema {
        M::schema()
    }

    async fn register_entries_processed(&self, entries: usize) -> tantivy::Result<()> {
        self.entries_processed.fetch_add(entries, Ordering::Relaxed);
        if self.entries_processed.load(Ordering::Relaxed) >= self.entries_before_recycle {
            self.recycle_self().await?;
            self.entries_processed.store(0, Ordering::Relaxed);
        }
        Ok(())
    }

    pub async fn recycle_self(&self) -> tantivy::Result<()> {
        let mut inner_guard = self.inner.write().await;
        // Take ownership of the old TantivyBackend and drop it
        if let Some(old_inner) = inner_guard.take() {
            old_inner.writer.wait_merging_threads()?;
        }

        let schema = M::schema();
        let index_path = self.index_path.clone();
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

        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .unwrap();

        let writer = index.writer(self.buffer_size).unwrap();

        *inner_guard = Some(TantivyBackend {
            writer,
            reader,
            index,
        });
        Ok(())
    }

    /// Can safely be unwrapped under normal circumstances
    pub async fn get_tantivy_backend(&self) -> Arc<RwLock<Option<TantivyBackend>>> {
        self.inner.clone()
    }
}
