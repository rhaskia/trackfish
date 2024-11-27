struct LastFMClient {
    key: String,
    client: Client,
}

impl LastFMClient {
    pub fn get_tags(&self, artist: &str, track: &str) -> Result<Wrapper, Error> {
        let params = [
            ("method", "track.gettoptags"),
            ("artist", artist),
            ("track", track),
            ("api_key", &self.key),
            ("format", "json"),
            ("autocorrect", "1"),
        ];

        let res = self.client
            .post(URL)
            .header(reqwest::header::USER_AGENT, "rhaskia - music sorter") // Change if you are using this application
            .form(&params)
            .send()?;

        Ok(res.json()?)
    }

    pub fn get_correction(&self, artist: &str, track: &str) -> Result<Track, Error> {
        let params = [
            ("method", "track.getcorrection"),
            ("artist", artist),
            ("track", track),
            ("api_key", &self.key),
            ("format", "json"),
            ("autocorrect", "1"),
        ];

        let res = self.client
            .post(URL)
            .header(reqwest::header::USER_AGENT, "rhaskia - music sorter") // Change if you are using this application
            .form(&params)
            .send()?;

        Ok(res.json()?)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wrapper {
    pub toptags: Toptags,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Toptags {
    pub tag: Vec<Tag>,
    #[serde(rename = "@attr")]
    pub attr: Attr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attr {
    pub artist: Option<String>,
    pub track: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub count: i64,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Search {
    pub results: Results,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Results {
    #[serde(rename = "opensearch:Query")]
    pub opensearch_query: OpensearchQuery,
    #[serde(rename = "opensearch:totalResults")]
    pub opensearch_total_results: String,
    #[serde(rename = "opensearch:startIndex")]
    pub opensearch_start_index: String,
    #[serde(rename = "opensearch:itemsPerPage")]
    pub opensearch_items_per_page: String,
    pub trackmatches: Trackmatches,
    #[serde(rename = "@attr")]
    pub attr: Attr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpensearchQuery {
    #[serde(rename = "#text")]
    pub text: String,
    pub role: String,
    pub start_page: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trackmatches {
    pub track: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub artist: String,
    pub url: String,
    pub streamable: String,
    pub listeners: String,
    pub image: Vec<Image>,
    pub mbid: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "#text")]
    pub text: String,
    pub size: String,
}
