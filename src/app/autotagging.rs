use crate::app::track::Track;
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::Value;

pub async fn get_possible_track_recordings(track: Track) -> anyhow::Result<Vec<Recording>> {
    let artist = track.artists.join(", ").replace("&", "%26");
    let title = &track.title.replace("#", "%23");

    let client = Client::builder()
        .user_agent("Trackfish/1.0 { https://github.com/rhaskia/trackfish }")
        .build()?;

    let url = format!("https://musicbrainz.org/ws/2/recording/?query=artist:{artist} AND recording={title} &limit=10&fmt=json");

    log::info!("requesting possible tracks at url {url}");

    let response = client.get(url).send()
        .await?;

    if response.status() != 200 {
        log::info!("{response:?}");
    }

    let body = response.text()
        .await?;

    let v: RecordingLookUp = serde_json::from_str(&body)?;

    Ok(v.recordings)
}

pub async fn get_lastfm_genres(track: &str, artist: &str, api_key: &str) -> anyhow::Result<Vec<String>> {
    let genres = Vec::new();

    let client = Client::builder()
        .user_agent("Trackfish/1.0 { https://github.com/rhaskia/trackfish }")
        .build()?;

    let track = track.replace("&", "%26").replace(" ", "+").to_ascii_lowercase();
    let artist = artist.replace("&", "%26").replace(" ", "+").to_ascii_lowercase();
    let url = format!("https://ws.audioscrobbler.com/2.0/?method=track.getTopTags&track={track}&artist={artist}&api_key={api_key}&format=json&autocorrect=1");

    log::info!("requesting possible genres at url {url}");

    let response = client.get(url).send()
        .await?;

    let body = response.text()
        .await?;

    Ok(genres)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingLookUp {
    pub created: String,
    pub count: i64,
    pub offset: i64,
    pub recordings: Vec<Recording>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recording {
    pub id: String,
    pub score: i64,
    #[serde(rename = "artist-credit-id")]
    pub artist_credit_id: String,
    pub title: String,
    pub length: Option<i64>,
    pub video: Value,
    #[serde(rename = "artist-credit")]
    pub artist_credit: Vec<ArtistCredit>,
    #[serde(rename = "first-release-date")]
    pub first_release_date: Option<String>,
    pub releases: Option<Vec<Release>>,
    #[serde(default)]
    pub isrcs: Vec<String>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    pub disambiguation: Option<String>,
}

impl Recording {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn artists(&self) -> Vec<String> {
        self.artist_credit.iter().map(|a| a.name.clone()).collect()
    }

    pub fn artist_str(&self) -> String {
        let mut result = Vec::new();

        for i in 0..self.artist_credit.len() {
            result.push(self.artist_credit[i].name.clone());

            if i == self.artist_credit.len() - 1 {
                continue;
            }

            if let Some(join_phrase) = &self.artist_credit[i].joinphrase {
                result.push(join_phrase.clone());
            } else {
                result.push(", ".to_string());
            }
        }

        result.join("")
    }

    pub fn album(&self) -> &str {
        if let Some(ref releases) = self.releases {
            &releases[0].title
        } else {
            &self.title
        }
    }

    pub fn genres(&self) -> Vec<String> {
        self.tags.iter().map(|t| t.name.clone()).collect()
    }

    pub fn mbid(&self) -> &str {
        &self.id
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistCredit {
    pub name: String,
    pub artist: Artist,
    pub joinphrase: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: String,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: String,
    pub disambiguation: Option<String>,
    #[serde(default)]
    pub aliases: Option<Vec<Alias>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Alias {
    #[serde(rename = "sort-name")]
    pub sort_name: String,
    #[serde(rename = "type-id")]
    pub type_id: Option<String>,
    pub name: String,
    pub locale: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub primary: Option<bool>,
    #[serde(rename = "begin-date")]
    pub begin_date: Value,
    #[serde(rename = "end-date")]
    pub end_date: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub id: String,
    #[serde(rename = "status-id")]
    pub status_id: Option<String>,
    pub count: i64,
    pub title: String,
    pub status: Option<String>,
    #[serde(rename = "release-group")]
    pub release_group: ReleaseGroup,
    pub date: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "release-events")]
    #[serde(default)]
    pub release_events: Vec<Event>,
    #[serde(rename = "track-count")]
    pub track_count: i64,
    pub media: Vec<Medum>,
    pub disambiguation: Option<String>,
    #[serde(rename = "artist-credit-id")]
    pub artist_credit_id: Option<String>,
    #[serde(rename = "artist-credit")]
    #[serde(default)]
    pub artist_credit: Vec<ArtistCredit>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseGroup {
    pub id: String,
    #[serde(rename = "type-id")]
    pub type_id: Option<String>,
    #[serde(rename = "primary-type-id")]
    pub primary_type_id: Option<String>,
    pub title: String,
    #[serde(rename = "primary-type")]
    pub primary_type: Option<String>,
    #[serde(rename = "secondary-types")]
    #[serde(default)]
    pub secondary_types: Vec<String>,
    #[serde(rename = "secondary-type-ids")]
    #[serde(default)]
    pub secondary_type_ids: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub date: String,
    pub area: Area,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Area {
    pub id: String,
    pub name: String,
    #[serde(rename = "sort-name")]
    pub sort_name: String,
    #[serde(rename = "iso-3166-1-codes")]
    pub iso_3166_1_codes: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Medum {
    pub id: String,
    pub position: i64,
    pub format: Option<String>,
    pub track: Vec<Track2>,
    #[serde(rename = "track-count")]
    pub track_count: i64,
    #[serde(rename = "track-offset")]
    pub track_offset: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Track2 {
    pub id: String,
    pub number: String,
    pub title: String,
    pub length: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub count: i64,
    pub name: String,
}
