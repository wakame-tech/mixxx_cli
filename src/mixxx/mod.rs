use std::path::PathBuf;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub mod repo;
mod serde_datetime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackLocation {
    pub id: i32,
    pub location: PathBuf,
    filename: String,
    pub directory: PathBuf,
    filesize: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    id: i32,
    pub artist: Option<String>,
    pub title: String,
    album: Option<String>,
    year: Option<String>,
    genre: Option<String>,
    tracknumber: Option<String>,
    location: f32,
    comment: Option<String>,
    duration: f32,
    bitrate: f32,
    pub bpm: f32,
    key: Option<String>,
    rating: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CueType {
    Invalid,
    HotCue,
    MainCue,
    Beat,
    Loop,
    Jump,
    Intro,
    Outro,
    N60dBSound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cue {
    id: i32,
    track_id: i32,
    r#type: CueType,
    position: f32,
    length: f32,
    hotcue: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    id: i32,
    playlist_id: i32,
    track_id: i32,
    position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    id: i32,
    pub name: String,
    position: usize,
    hidden: i32,
    #[serde(with = "serde_datetime")]
    date_created: NaiveDateTime,
    locked: bool,
}
