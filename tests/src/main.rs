use rand::Rng;
use std::{path::PathBuf, time::Duration};
use tantivy::time::OffsetDateTime;
use tantivy_ext::TantivySearchIndex;

#[derive(TantivySearchIndex, Debug)]
struct MyModel {
    #[tantivy_ext("primary_key")]
    name: tantivy_ext::FastStr,
    path: tantivy_ext::Tokenized,
    date: tantivy_ext::Date,
    popularity: tantivy_ext::FastF64,
    not_score: tantivy_ext::Score,
}

#[tokio::main]
async fn main() {
    let mut rng = rand::thread_rng();
    let mut models = Vec::new();

    for _ in 0..100_000 {
        let random_int: i32 = rng.gen_range(1..=100_000);
        let model = MyModel {
            name: random_int.to_string().into(),
            path: "tokenized".into(),
            date: tantivy::DateTime::from_utc(OffsetDateTime::now_utc()).into(),
            popularity: 10.0.into(),
            not_score: 0.0.into(),
        };
        models.push(model);
    }

    let save_path = PathBuf::from(r"./TestIndex");

    let index = MyModel::index_builder(save_path)
        .with_memory_budget(50_000_000)
        .build();

    index.add(&models).await.expect("failed to add item");

    index.recycle_writer().await.expect("Failed to recycle writer");

    loop {
        println!("Indexing finished");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
