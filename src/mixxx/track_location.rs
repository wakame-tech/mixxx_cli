use super::repo::{AsRepo, Repo};
use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackLocation {
    pub id: i32,
    pub location: PathBuf,
    filename: String,
    pub directory: PathBuf,
    filesize: usize,
}

impl<'a> AsRepo<'a> for TrackLocation {
    fn repo(conn: &'a Connection) -> Repo<'a, Self> {
        Repo::new(conn, "track_locations")
    }
}

impl<'a> Repo<'a, TrackLocation> {
    pub fn update(&self, track_location: &TrackLocation) -> Result<()> {
        self.conn.execute(
            format!(
                "UPDATE {} SET location=?1, directory=?2 WHERE id=?3",
                self.table
            )
            .as_str(),
            params![
                track_location.location.display().to_string(),
                track_location.directory.display().to_string(),
                track_location.id,
            ],
        )?;
        Ok(())
    }
}
