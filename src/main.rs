use std::path::PathBuf;
use entity::{
    entity_trait::Index,
    field::{self, Field},
};
use index::index_builder::SearchIndexBuilder;
use index_macro::Index;
use tantivy::{doc, time::OffsetDateTime};
mod entity;
mod index;
mod util;

#[derive(Index)]
struct MyModel {
    #[tantivy_ext("primary_key")]
    id: field::FastU64,
    name: field::Str,
    path: field::Tokenized,
    date: field::Date,
    score: field::Score,
}

#[tokio::main]
async fn main() {
    let model = MyModel {
        id: 32.into(),
        name: "file.txt".into(),
        path: "C:/directory/file.txt".into(),
        date: tantivy::DateTime::from_utc(OffsetDateTime::now_utc()).into(),
        score: 3.2.into()
    };

    let save_path = PathBuf::from(r"C:\Users\grays\OneDrive\Desktop\ExtTest");

    let index = MyModel::index_builder(save_path)
        .with_memory_budget(50_000_000)
        .build();

    if let Err(err) = index.add(vec![&model]).await {
        println!("{}", err);
    }
}
