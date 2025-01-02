

// #[derive(Index, Debug)]
// struct MyModel {
//     #[tantivy_ext("primary_key")]
//     name: field::FastStr,
//     path: field::Tokenized,
//     date: field::Date,
//     popularity: field::FastF64,
//     score: field::Score,
// }

// #[tokio::main]
// async fn main() {
//     let model = MyModel {
//         name: "file.txt".into(),
//         path: "C:/directory/file.txt".into(),
//         date: tantivy::DateTime::from_utc(OffsetDateTime::now_utc()).into(),
//         popularity: 10.0.into(),
//         score: 0.0.into(),
//     };

//     let save_path = PathBuf::from(r"C:\Users\grays\OneDrive\Desktop\ExtTest");

//     let index = MyModel::index_builder(save_path)
//         .with_memory_budget(50_000_000)
//         .build();

//     index.add(vec![&model]).await.expect("failed to add item");
//     // Query
//     let query_parser =
//         QueryParser::for_index(index.searcher().index(), vec![MyModel::path_field().into()]);
//     let q = query_parser.parse_query("txt").unwrap();
//     let query = BooleanQuery::new(vec![(Occur::Should, q)]);

//     let results = index
//         .query(&query, 50)
//         .execute()
//         .expect("Failed to execute query");
//     println!("got {} results", results.len());
//     for result in results {
//         println!("{:#?}", result);
//     }
// }
