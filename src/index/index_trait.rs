use tantivy::{query::{Query, QueryParser}, schema::{Field, Schema}, IndexWriter, Term};

use crate::entity::entity_trait;

use super::query::builder::QueryBuilder;

pub trait SearchIndexTrait<M>: Send + Sync + 'static
where
    M: entity_trait::Index,
{
    /// Adds the provided models to the search index and then commits the changes.
    async fn add(&self, models: &[M]) -> tantivy::Result<()>;

    /// Removes the provided models from the search index and then commits the changes.
    async fn remove<'a, T>(&self, models: &[M]) -> tantivy::Result<()>;

    /// Removes all models from the search index with the provided term
    ///
    /// Example:
    /// ```rust
    /// let term = MyModel::name_field().term(String::from("Joe"));
    /// index.remove_by_terms(vec![term]).await;
    /// ```
    async fn remove_by_terms(&self, terms: Vec<Term>) -> tantivy::Result<()>;

    /// Attempt to commit all pending changes.
    ///
    /// This function will retry up to 3 times in case of errors.
    async fn commit(&self, writer: &mut IndexWriter) -> tantivy::Result<()>;

    /// Get the query parser for this search index
    async fn query_parser(&self, default_fields: Vec<Field>) -> QueryParser;

    async fn query<'a, Q>(&self, query: &'a Q, max_results: usize) -> QueryBuilder<'a, Q, M>
    where
        Q: Query + Sized;

    async fn scored_doc_to_model(&self, doc: (f64, tantivy::DocAddress)) -> tantivy::Result<M>;

    /// Get the schema that this index uses
    fn schema() -> &'static Schema;
}
