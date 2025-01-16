use std::{fs, marker::PhantomData, path::PathBuf, time::Duration};

use tantivy::{
    query::{Query, QueryParser},
    schema::{Field, Schema},
    Index, IndexReader, IndexWriter, Term,
};

use crate::{entity::entity_trait, util::async_retry};

use super::{backend::TantivyBackend, query::builder::QueryBuilder};

/// A tantivy search index over instances of the provided struct.
///
/// Because all of the underlying data wraps an `Arc`, this can be negligibly cloned
#[derive(Clone)]
pub struct SearchIndex<M>
where
    M: entity_trait::Index,
{
    buffer_size: usize,

    reader: IndexReader,
    index: tantivy::Index,

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

        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .unwrap();

        Self {
            buffer_size,
            reader,
            index,
            phantom: PhantomData,
        }
    }

    /// Adds the provided models to the search index and then commits the changes.
    pub async fn add<'a, T>(&self, models: T) -> tantivy::Result<()>
    where
        T: IntoIterator<Item = &'a M>,
        M: 'a,
    {
        let mut writer = self.create_writer();
        for model in models {
            // Delete by the primary key
            let primary_key_term = model.get_primary_key();
            writer.delete_term(primary_key_term);

            writer.add_document(model.as_document())?;
        }
        self.commit(&mut writer).await?;
        writer.wait_merging_threads()?;
        Ok(())
    }

    /// Removes the provided models from the search index and then commits the changes.
    pub async fn remove<'a, T>(&self, models: T) -> tantivy::Result<()>
    where
        T: IntoIterator<Item = &'a M>,
        M: 'a,
    {
        let mut writer = self.create_writer();
        for model in models {
            let primary_key_term = model.get_primary_key();
            writer.delete_term(primary_key_term);
        }
        self.commit(&mut writer).await?;
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
        let mut writer = self.create_writer();
        for term in terms {
            writer.delete_term(term);
        }
        self.commit(&mut writer).await
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

    pub fn create_writer(&self) -> IndexWriter {
        self.index.writer(self.buffer_size).unwrap()
    }

    /// Get the schema that this index uses
    pub fn schema() -> Schema {
        M::schema()
    }

    pub fn get_tantivy_backend(&self) -> TantivyBackend {
        TantivyBackend {
            reader: &self.reader,
            index: &self.index,
            schema: M::schema(),
        }
    }
}
