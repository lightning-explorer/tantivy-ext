use std::marker::PhantomData;

use tantivy::{collector::TopDocs, query::Query, Searcher};

use crate::entity::entity_trait;

pub struct QueryBuilder<'a, Q, M>
where
    M: entity_trait::Index,
    Q: Query,
{
    query: &'a Q,
    searcher: Searcher,
    max_results: usize,
    phantom: PhantomData<M>,
}

impl<'a, Q, M> QueryBuilder<'a, Q, M>
where
    M: entity_trait::Index,
    Q: Query,
{
    pub fn new(query: &'a Q, searcher: Searcher, max_results: usize) -> Self {
        Self {
            query,
            searcher,
            max_results,
            phantom: PhantomData,
        }
    }

    // TODO: add a score tweak feature

    pub fn execute(self) -> tantivy::Result<Vec<M>> {
        let documents = self
            .searcher
            .search(self.query, &TopDocs::with_limit(self.max_results))?;
        let models: Vec<M> = documents
            .into_iter()
            .map(|(score, address)| {
                let doc = self.searcher.doc(address).unwrap();
                M::from_document(doc, score)
            })
            .collect();
        Ok(models)
    }
}
