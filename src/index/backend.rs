use tantivy::{schema::Schema, IndexReader};

#[derive(Clone)]
pub struct TantivyBackend<'a>{
    pub reader: &'a IndexReader,
    pub index: &'a tantivy::Index,
    pub schema: &'static Schema
}