use crate::mixxx::{cue::Cue, library::Library, repo::AsRepo, track_location::TrackLocation};
use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

pub fn get_track(conn: &Connection, track_id: i32) -> Result<(PathBuf, Library)> {
    let lib_repo = Library::repo(conn);
    let track_location_repo = TrackLocation::repo(conn);
    let library = lib_repo
        .select(track_id)?
        .ok_or(anyhow::anyhow!("track not found"))?;
    let track_location = track_location_repo
        .select(track_id)?
        .ok_or(anyhow::anyhow!("track location not found"))?
        .location;
    Ok((track_location, library))
}

pub fn get_hotcue(conn: &Connection, track_id: i32, hotcue: i32) -> Result<Cue> {
    let cue_repo = Cue::repo(conn);
    let cues = cue_repo
        .hot_cues_by_track_id(track_id)?
        .into_iter()
        .collect::<Vec<_>>();
    let cue = cues
        .into_iter()
        .find(|cue| cue.hotcue == hotcue)
        .ok_or(anyhow::anyhow!("hotcue not found"))?;
    Ok(cue)
}

/// returns seconds at cue in original bpm
pub fn cue_at(library: &Library, cue: &Cue, offset_beats: i32) -> f32 {
    let cue_at = cue.position / library.samplerate as f32 / 2.0;
    let beat = 60.0 / library.bpm;
    cue_at + beat * offset_beats as f32
}
