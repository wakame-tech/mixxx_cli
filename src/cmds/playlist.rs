use crate::mixxx::{
    cue::Cue, library::Library, playlist::Playlist, playlist_track::PlaylistTrack, repo::AsRepo,
};
use anyhow::Result;
use comfy_table::Table;
use rusqlite::Connection;
use std::{collections::BTreeMap, time::Duration};

#[derive(Debug, clap::Parser)]
pub struct PlaylistArgs {
    #[arg(long)]
    playlist_id: i32,
}

#[derive(Debug)]
pub struct TrackModel {
    pub position: i32,
    pub title: String,
    pub artist: Option<String>,
    pub bpm: u8,
    pub cues: BTreeMap<u8, Duration>,
}

#[derive(Debug)]
pub struct PlaylistModel {
    pub title: String,
    pub tracks: Vec<TrackModel>,
}

fn fetch_track(conn: &Connection, playlist_track: &PlaylistTrack) -> Result<TrackModel> {
    let lib_repo = Library::repo(conn);
    let cue_repo = Cue::repo(conn);

    let library = lib_repo
        .select(playlist_track.track_id)?
        .ok_or(anyhow::anyhow!("track not found"))?;
    let cues = cue_repo
        .hot_cues_by_track_id(playlist_track.track_id)?
        .iter()
        .map(|cue| {
            (
                cue.hotcue as u8,
                Duration::from_secs_f32(cue.position.max(0.) / library.samplerate as f32 / 2.0),
            )
        })
        .collect::<BTreeMap<_, _>>();
    Ok(TrackModel {
        position: playlist_track.position,
        title: library.title,
        artist: library.artist,
        bpm: library.bpm as u8,
        cues,
    })
}

fn fetch_playlist(conn: &Connection, id: i32) -> Result<PlaylistModel> {
    let playlist_repo = Playlist::repo(conn);
    let playlist_track_repo = PlaylistTrack::repo(conn);

    let playlist = playlist_repo
        .select(id)?
        .ok_or(anyhow::anyhow!("playlist id={} not found", id))?;
    let tracks = playlist_track_repo
        .find_by_playlist_id(id)?
        .iter()
        .map(|playlist_track| fetch_track(conn, playlist_track))
        .collect::<Result<Vec<_>>>()?;
    Ok(PlaylistModel {
        title: playlist.name,
        tracks,
    })
}

pub fn list_playlist_tracks(conn: &Connection, args: &PlaylistArgs) -> Result<()> {
    let playlist = fetch_playlist(conn, args.playlist_id)?;

    let mut table = Table::new();
    table.set_header(vec!["bpm", "title", "artist", "cues"]);
    for track in playlist.tracks.iter() {
        let artists = track
            .artist
            .clone()
            .unwrap_or("---".to_string())
            .to_string();
        table.add_row(vec![
            track.bpm.to_string(),
            track.title.to_string(),
            artists.chars().take(20).collect(),
            track
                .cues
                .iter()
                .map(|(n, dur)| format!("{}: {:.1}s", n + 1, dur.as_secs_f32()))
                .collect::<Vec<_>>()
                .join(" "),
        ]);
    }
    println!("{}", table);
    Ok(())
}
