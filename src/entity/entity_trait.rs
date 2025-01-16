use std::path::PathBuf;

use tantivy::{schema::Schema, TantivyDocument};

use crate::index::index_builder::SearchIndexBuilder;

pub trait Index {
    fn schema() -> &'static Schema;

    fn get_primary_key(&self) -> tantivy::Term;

    fn as_document(&self) -> TantivyDocument;

    fn from_document(doc: TantivyDocument, score: f32)->Self;

    fn index_builder(path: PathBuf) -> SearchIndexBuilder<Self>
    where
        Self: std::marker::Sized;
}
