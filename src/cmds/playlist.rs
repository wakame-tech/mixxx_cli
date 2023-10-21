use crate::mixxx::repo::{LibraryRepo, PlaylistRepo, PlaylistTrackRepo, Repo};
use anyhow::Result;
use comfy_table::Table;
use rusqlite::Connection;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Track {
    pub title: String,
    pub artist: Option<String>,
    pub bpm: u8,
}

#[derive(Debug)]
pub struct Playlist {
    pub title: String,
    pub tracks: Vec<Track>,
}

pub struct PlaylistViewer {
    conn: Connection,
}

impl PlaylistViewer {
    pub fn new(db_path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    fn fetch_playlist(&self, id: i32) -> Result<Playlist> {
        let repo = PlaylistRepo::new(&self.conn);
        let playlist = repo
            .read(id)?
            .ok_or(anyhow::anyhow!("playlist id={} not found", id))?;

        let repo = PlaylistTrackRepo::new(&self.conn);
        let lib_repo = LibraryRepo::new(&self.conn);
        let tracks = repo
            .find_by_playlist_id(id)?
            .iter()
            .map(|id| lib_repo.read(*id))
            .collect::<Result<Option<Vec<_>>>>()?
            .ok_or(anyhow::anyhow!("track not found"))?
            .into_iter()
            .map(|lib| Track {
                title: lib.title,
                artist: lib.artist,
                bpm: lib.bpm as u8,
            })
            .collect();

        Ok(Playlist {
            title: playlist.name,
            tracks,
        })
    }

    pub fn list_playlist_tracks(&self, id: i32) -> Result<()> {
        let playlist = self.fetch_playlist(id)?;

        let mut table = Table::new();
        table.set_header(vec!["bpm", "title", "artist"]);
        for track in playlist.tracks.iter() {
            table.add_row(vec![
                track.bpm.to_string(),
                track.title.to_string(),
                track
                    .artist
                    .clone()
                    .unwrap_or("---".to_string())
                    .to_string()
                    .chars()
                    .take(20)
                    .collect(),
            ]);
        }
        println!("{}", table);
        Ok(())
    }
}
