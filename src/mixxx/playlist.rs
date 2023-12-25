use super::{
    repo::{AsRepo, Repo},
    serde_datetime,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

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

impl<'a> AsRepo<'a> for Playlist {
    fn repo(conn: &'a rusqlite::Connection) -> super::repo::Repo<'a, Self> {
        Repo::new(conn, "Playlists")
    }
}
