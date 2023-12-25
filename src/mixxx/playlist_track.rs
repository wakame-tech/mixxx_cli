use super::repo::{AsRepo, Repo};
use anyhow::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub id: i32,
    pub playlist_id: i32,
    pub track_id: i32,
    pub position: i32,
}

impl<'a> AsRepo<'a> for PlaylistTrack {
    fn repo(conn: &'a rusqlite::Connection) -> Repo<'a, Self> {
        Repo::new(conn, "PlaylistTracks")
    }
}

impl<'a> Repo<'a, PlaylistTrack> {
    pub fn find_by_playlist_id(&self, playlist_id: i32) -> Result<Vec<PlaylistTrack>> {
        let mut stmt = self
            .conn
            .prepare(format!("SELECT * FROM {} WHERE playlist_id=?1", self.table).as_str())?;
        self.query(&mut stmt, params![playlist_id])
    }
}
