use super::{Cue, Library, Playlist, PlaylistTrack, TrackLocation};
use anyhow::Result;
use rusqlite::{params, Connection};
use serde::Deserialize;

fn select_all<T: for<'de> Deserialize<'de>>(conn: &Connection, table: &str) -> Result<Vec<T>> {
    let mut stmt = conn.prepare(format!("SELECT * FROM {}", table).as_str())?;
    let items = stmt
        .query_and_then([], |row| serde_rusqlite::from_row(row))?
        .map(|r| r.map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;
    Ok(items)
}

fn select<T: for<'de> Deserialize<'de> + Clone>(
    conn: &Connection,
    table: &str,
    id: i32,
) -> Result<Option<T>> {
    let mut stmt = conn.prepare(format!("SELECT * FROM {} WHERE id={}", table, id).as_str())?;
    let items = stmt
        .query_and_then([], |row| serde_rusqlite::from_row::<T>(row))?
        .map(|r| r.map_err(Into::into))
        .collect::<Result<Vec<_>>>()?;
    Ok(items.get(0).cloned())
}

pub trait Repo<T: for<'de> Deserialize<'de>> {
    fn read_all(&self) -> Result<Vec<T>>;
    fn read(&self, id: i32) -> Result<Option<T>>;
}

pub struct CueRepo<'a> {
    conn: &'a Connection,
}

impl<'a> CueRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl Repo<Cue> for CueRepo<'_> {
    fn read_all(&self) -> Result<Vec<Cue>> {
        select_all(self.conn, "cues")
    }

    fn read(&self, id: i32) -> Result<Option<Cue>> {
        select(self.conn, "cues", id)
    }
}

pub struct TrackLocationRepo<'a> {
    table: &'static str,
    conn: &'a Connection,
}

impl<'a> TrackLocationRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            table: "track_locations",
            conn,
        }
    }

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

impl Repo<TrackLocation> for TrackLocationRepo<'_> {
    fn read_all(&self) -> Result<Vec<TrackLocation>> {
        select_all(self.conn, self.table)
    }

    fn read(&self, id: i32) -> Result<Option<TrackLocation>> {
        select(self.conn, self.table, id)
    }
}

pub struct PlaylistRepo<'a> {
    conn: &'a Connection,
}

impl<'a> PlaylistRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl Repo<Playlist> for PlaylistRepo<'_> {
    fn read_all(&self) -> Result<Vec<Playlist>> {
        select_all(self.conn, "Playlists")
    }

    fn read(&self, id: i32) -> Result<Option<Playlist>> {
        select(self.conn, "Playlists", id)
    }
}

pub struct PlaylistTrackRepo<'a> {
    conn: &'a Connection,
}

impl<'a> PlaylistTrackRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn find_by_playlist_id(&self, playlist_id: i32) -> Result<Vec<i32>> {
        let table = "PlaylistTracks";
        let mut stmt = self.conn.prepare(
            format!(
                "SELECT track_id FROM {} WHERE playlist_id={}",
                table, playlist_id
            )
            .as_str(),
        )?;
        let items = stmt
            .query_map([], |row| row.get::<_, i32>(0))?
            .map(|r| r.map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;
        Ok(items)
    }
}

impl Repo<PlaylistTrack> for PlaylistTrackRepo<'_> {
    fn read_all(&self) -> Result<Vec<PlaylistTrack>> {
        select_all(self.conn, "PlaylistTracks")
    }

    fn read(&self, id: i32) -> Result<Option<PlaylistTrack>> {
        select(self.conn, "PlaylistTracks", id)
    }
}

pub struct LibraryRepo<'a> {
    conn: &'a Connection,
}

impl<'a> LibraryRepo<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl Repo<Library> for LibraryRepo<'_> {
    fn read_all(&self) -> Result<Vec<Library>> {
        select_all(self.conn, "library")
    }

    fn read(&self, id: i32) -> Result<Option<Library>> {
        select(self.conn, "library", id)
    }
}
