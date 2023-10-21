use crate::mixxx::repo::{Repo, TrackLocationRepo};
use anyhow::Result;
use kdam::BarExt;
use rusqlite::Connection;
use std::path::PathBuf;

pub struct TrackLocationsConverter {
    conn: Connection,
}

impl TrackLocationsConverter {
    pub fn new(db_path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    pub fn convert_track_locations(&self, directory: &PathBuf) -> Result<()> {
        let track_location_repo = TrackLocationRepo::new(&self.conn);

        let mut track_locations = track_location_repo.read_all()?;
        let mut pb = kdam::tqdm!(total = track_locations.len());
        for track_location in track_locations.iter_mut() {
            let file_name = track_location.location.file_name().unwrap();
            let new_location = directory.clone().join(file_name);
            track_location.location = new_location.clone();
            track_location.directory = directory.clone();
            if let Err(e) = track_location_repo.update(track_location) {
                log::debug!("{}", e);
                log::debug!("location={}", new_location.display());
            }
            pb.update(1)?;
        }
        log::debug!("{} tracks converted", track_locations.len());
        Ok(())
    }
}
