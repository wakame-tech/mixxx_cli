use super::repo::{AsRepo, Repo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub id: i32,
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

impl<'a> AsRepo<'a> for Library {
    fn repo(conn: &'a rusqlite::Connection) -> Repo<'a, Self> {
        Repo::new(conn, "library")
    }
}
