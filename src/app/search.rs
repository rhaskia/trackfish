use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy, IndexReader};
use super::settings::Settings;
use crate::app::track::Track;
use log::info;

pub struct SearchManager {
    schema: Schema,
    index: Index,
    index_writer: IndexWriter,
    reader: Option<IndexReader>,
}

impl SearchManager {
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();

        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("artist", TEXT);
        schema_builder.add_u64_field("id", STORED | FAST);

        let schema = schema_builder.build();
        let path = Settings::dir().join("tantivy");
        let index = Index::create_from_tempdir(schema.clone()).unwrap();

        let mut index_writer: IndexWriter = index.writer(20_000_000).unwrap();

        Self { 
            schema, 
            index,
            index_writer,
            reader: None,
        }
    }

    pub fn fill_track_information(&mut self, tracks: &[Track]) {
        let title = self.schema.get_field("title").unwrap();
        let artist = self.schema.get_field("artist").unwrap();
        let id = self.schema.get_field("id").unwrap();

        for i in 0..tracks.len() {
            let mut doc = TantivyDocument::default();
            doc.add_text(title, tracks[i].title.clone());
            doc.add_text(artist, tracks[i].artists.join(", "));
            doc.add_u64(id, i as u64);

            self.index_writer.add_document(doc);
        }

        self.index_writer.commit().unwrap();

        let reader = self.index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into().unwrap();

        self.reader = Some(reader);
    } 

    pub fn search(&self, query_string: &str) -> Vec<usize> {
        let searcher = self.reader.as_ref().unwrap().searcher();
        let title = self.schema.get_field("title").unwrap();
        let artist = self.schema.get_field("artist").unwrap();
        let id = self.schema.get_field("id").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![title, artist]);

        let query = query_parser.parse_query(query_string).unwrap();

        let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

        let mut results = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address).unwrap();
            let id = retrieved_doc.get_first(id).unwrap().as_u64().unwrap() as usize;
            results.push(id);
        }

        results
    }
}