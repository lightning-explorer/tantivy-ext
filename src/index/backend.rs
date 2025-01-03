use std::sync::Arc;

use tantivy::{schema::Schema, IndexReader, IndexWriter};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct TantivyBackend<'a>{
    pub writer: &'a Arc<Mutex<IndexWriter>>,
    pub reader: &'a IndexReader,
    pub index: &'a tantivy::Index,
    pub schema: Schema
}