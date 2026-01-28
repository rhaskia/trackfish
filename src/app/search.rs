use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter, ReloadPolicy, IndexReader};
use crate::app::track::Track;

pub struct SearchManager {
    tracks: SearchGroup,
    artists: SearchGroup,
    albums: SearchGroup,
    genres: SearchGroup
}

pub struct SearchGroup {
    schema: Schema,
    index: Index,
    index_writer: IndexWriter,
    reader: Option<IndexReader>,
}

impl SearchGroup {
    pub fn new(fields: Vec<&str>) -> Self {
        let mut schema_builder = Schema::builder();

        for field in fields {
            schema_builder.add_text_field(field, TEXT | STORED);
        }
        schema_builder.add_u64_field("id", STORED | FAST);

        let schema = schema_builder.build();
        let index = Index::create_from_tempdir(schema.clone()).unwrap();

        let index_writer: IndexWriter = index.writer(15_000_000).unwrap();

        Self { 
            schema, 
            index,
            index_writer,
            reader: None,
        }
    }

    pub fn search(&self, query_string: &str, fields: Vec<&str>) -> Vec<usize> {
        let searcher = self.reader.as_ref().unwrap().searcher();
        let fields = fields.iter().map(|f| self.schema.get_field(f).unwrap()).collect();
        let id = self.schema.get_field("id").unwrap();

        let query_parser = QueryParser::for_index(&self.index, fields);

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

    pub fn search_for_strings(&self, query_string: &str, fields: Vec<&str>) -> Vec<String> {
        let searcher = self.reader.as_ref().unwrap().searcher();
        let search_field = self.schema.get_field(fields[0]).unwrap();
        let fields = fields.iter().map(|f| self.schema.get_field(f).unwrap()).collect();

        let query_parser = QueryParser::for_index(&self.index, fields);

        let query = query_parser.parse_query(query_string).unwrap();

        let top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

        let mut results = Vec::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address).unwrap();
            let title = retrieved_doc.get_first(search_field).unwrap().as_str().unwrap();
            results.push(title.to_string());
        }

        results
    }
}

impl SearchManager {
    pub fn new() -> Self {
        Self {
            tracks: SearchGroup::new(vec!["title", "artist"]),
            artists: SearchGroup::new(vec!["artist"]),
            albums: SearchGroup::new(vec!["album", "artist"]),
            genres: SearchGroup::new(vec!["genre"]),
        }
    }

    pub fn fill_track_information(&mut self, tracks: &[Track]) {
        let searcher = &mut self.tracks;
        let title = searcher.schema.get_field("title").unwrap();
        let artist = searcher.schema.get_field("artist").unwrap();
        let id = searcher.schema.get_field("id").unwrap();

        for i in 0..tracks.len() {
            let mut doc = TantivyDocument::default();
            doc.add_text(title, tracks[i].title.clone());
            doc.add_text(artist, tracks[i].artists.join(", "));
            doc.add_u64(id, i as u64);

            searcher.index_writer.add_document(doc);
        }

        searcher.index_writer.commit().unwrap();

        let reader = searcher.index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into().unwrap();

        searcher.reader = Some(reader);
    } 

    pub fn fill_artist_information(&mut self, artists: &Vec<(String, (String, usize))>) {
        let searcher = &mut self.artists;
        let artist = searcher.schema.get_field("artist").unwrap();
        let id = searcher.schema.get_field("id").unwrap();

        for i in 0..artists.len() {
            let mut doc = TantivyDocument::default();
            doc.add_text(artist, artists[i].1.0.clone());
            doc.add_u64(id, i as u64);

            searcher.index_writer.add_document(doc);
        }

        searcher.index_writer.commit().unwrap();

        let reader = searcher.index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into().unwrap();

        searcher.reader = Some(reader);
    } 

    pub fn fill_album_information(&mut self, albums: &Vec<(String, usize)>) {
        let searcher = &mut self.albums;
        let album = searcher.schema.get_field("album").unwrap();
        let id = searcher.schema.get_field("id").unwrap();

        for i in 0..albums.len() {
            let mut doc = TantivyDocument::default();
            doc.add_text(album, albums[i].0.clone());
            doc.add_u64(id, i as u64);

            searcher.index_writer.add_document(doc).unwrap();
        }

        searcher.index_writer.commit().unwrap();

        let reader = searcher.index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into().unwrap();

        searcher.reader = Some(reader);
    } 

    pub fn fill_genre_information(&mut self, genres: &Vec<(String, usize)>) {
        let searcher = &mut self.genres;
        let genre = searcher.schema.get_field("genre").unwrap();
        let id = searcher.schema.get_field("id").unwrap();

        for i in 0..genres.len() {
            let mut doc = TantivyDocument::default();
            doc.add_text(genre, genres[i].0.clone());
            doc.add_u64(id, i as u64);

            searcher.index_writer.add_document(doc).unwrap();
        }

        searcher.index_writer.commit().unwrap();

        let reader = searcher.index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into().unwrap();

        searcher.reader = Some(reader);
    } 

    pub fn search_tracks(&mut self, query: &str) -> Vec<usize> {
        self.tracks.search(query, vec!["title", "artist"])
    }

    pub fn search_artists(&mut self, query: &str) -> Vec<String> {
        self.artists.search_for_strings(query, vec!["artist"])
    }

    pub fn search_albums(&mut self, query: &str) -> Vec<String> {
        self.albums.search_for_strings(query, vec!["album", "artist"])
    }

    pub fn search_genres(&mut self, query: &str) -> Vec<String> {
        self.genres.search_for_strings(query, vec!["genre"])
    }
}