use std::{fs, marker::PhantomData, path::PathBuf, sync::Arc, time::Duration};

use tantivy::{
    query::{Query, QueryParser},
    schema::{Field, Schema},
    Index, IndexReader, IndexWriter, Term,
};
use tokio::sync::RwLock;

use crate::{entity::entity_trait, util::async_retry};

use super::{
    backend::RecyclingTantivyBackend, query::builder::QueryBuilder,
    writer_recycler::IndexWriterRecycler,
};

/// A tantivy search index over instances of the provided struct.
///
/// Because all of the underlying data wraps an `Arc`, this can be negligibly cloned
#[derive(Clone)]
pub struct SearchIndex<M>
where
    M: entity_trait::Index,
{
    writer_recycler: IndexWriterRecycler,
    reader: IndexReader,
    index: Arc<tantivy::Index>,

    phantom: PhantomData<M>,
}

impl<M> SearchIndex<M>
where
    M: entity_trait::Index,
{
    pub fn new(index_path: PathBuf, buffer_size: usize, entries_before_recycle: usize) -> Self {
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
        let index = Arc::new(index.unwrap());

        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .unwrap();

        let writer_recycler =
            IndexWriterRecycler::new(Arc::clone(&index), buffer_size, entries_before_recycle);

        Self {
            writer_recycler,
            reader,
            index,
            phantom: PhantomData,
        }
    }

    /// Adds the provided models to the search index and then commits the changes.
    pub async fn add(&self, models: &[M]) -> tantivy::Result<()> {
        let writer = self.writer_recycler.get_writer();
        let models_len = models.len();
        {
            let mut writer_lock = writer.write().await;
            let writer_lock = writer_lock.as_mut().unwrap();
            for model in models {
                // Delete by the primary key
                let primary_key_term = model.get_primary_key();
                writer_lock.delete_term(primary_key_term);

                writer_lock.add_document(model.as_document())?;
            }
            self.commit(writer_lock).await?;
        }
        // Writer lock must be dropped so this function can use it
        self.writer_recycler
            .register_entries_processed(models_len)
            .await?;
        Ok(())
    }

    /// Removes the provided models from the search index and then commits the changes.
    pub async fn remove<'a, T>(&self, models: &[M]) -> tantivy::Result<()> {
        let models_len = models.len();
        let writer = self.get_writer();
        {
            let mut writer_lock = writer.write().await;
            let writer_lock = writer_lock.as_mut().unwrap();
            for model in models {
                let primary_key_term = model.get_primary_key();
                writer_lock.delete_term(primary_key_term);
            }
            self.commit(writer_lock).await?;
        }
        // Writer lock must be dropped so this function can use it
        self.writer_recycler
            .register_entries_processed(models_len)
            .await?;
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
        let writer = self.get_writer();
        let terms_len = terms.len();
        {
            let mut writer_lock = writer.write().await;
            let writer_lock = writer_lock.as_mut().unwrap();
            for term in terms {
                writer_lock.delete_term(term);
            }
            self.commit(writer_lock).await?;
        }
        self.writer_recycler
            .register_entries_processed(terms_len)
            .await
    }

    /// Attempt to commit all pending changes.
    ///
    /// This function will retry up to 3 times in case of errors.
    async fn commit(&self, writer: &mut IndexWriter) -> tantivy::Result<()> {
        async_retry::retry_with_backoff(|_| writer.commit(), 3, Duration::from_millis(100)).await?;

        Ok(())
    }

    /// Get the query parser for this search index
    pub fn query_parser(&self, default_fields: Vec<Field>) -> QueryParser {
        QueryParser::for_index(&self.index, default_fields)
    }

    pub fn query<'a, Q>(&self, query: &'a Q, max_results: usize) -> QueryBuilder<'a, Q, M>
    where
        Q: Query + Sized,
    {
        let searcher = self.reader.searcher();
        QueryBuilder::new(query, searcher, max_results)
    }

    pub fn scored_doc_to_model(&self, doc: (f64, tantivy::DocAddress)) -> tantivy::Result<M> {
        let searcher = self.reader.searcher();
        let (score, address) = doc;

        let doc = searcher.doc(address)?;

        Ok(M::from_document(doc, score as f32))
    }

    pub async fn recycle_writer(&self) -> tantivy::Result<()> {
        self.writer_recycler.replace_writer().await
    }

    pub fn get_writer(&self) -> Arc<RwLock<Option<IndexWriter>>> {
        self.writer_recycler.get_writer()
    }

    /// Get the schema that this index uses
    pub fn schema() -> &'static Schema {
        M::schema()
    }

    pub fn get_tantivy_backend(&self) -> RecyclingTantivyBackend {
        RecyclingTantivyBackend {
            reader: &self.reader,
            writer: &self.writer_recycler,
            index: &self.index,
            schema: M::schema(),
        }
    }
}
