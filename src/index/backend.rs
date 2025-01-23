use tantivy::{schema::Schema, IndexReader};

use super::writer_recycler::IndexWriterRecycler;

#[derive(Clone)]
pub struct TantivyBackend<'a>{
    pub reader: &'a IndexReader,
    pub writer: &'a IndexWriterRecycler,
    pub index: &'a tantivy::Index,
    pub schema: &'static Schema
}