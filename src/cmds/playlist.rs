use super::mix::MixTrack;
use crate::mixxx::{
    cue::Cue, library::Library, playlist::Playlist, playlist_track::PlaylistTrack, repo::AsRepo,
};
use anyhow::Result;
use comfy_table::Table;
use rusqlite::Connection;
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

#[derive(Debug, clap::Parser)]
pub struct PlaylistArgs {
    #[arg(long)]
    playlist_id: i32,
    #[arg(long)]
    out: Option<PathBuf>,
}

#[derive(Debug)]
pub struct TrackModel {
    pub track_id: i32,
    pub position: i32,
    pub title: String,
    pub artist: Option<String>,
    pub bpm: f32,
    pub cues: BTreeMap<u8, Duration>,
}

impl TrackModel {
    fn first_cue(&self) -> Option<u8> {
        self.cues.iter().min_by_key(|(_, d)| *d).map(|(c, _)| *c)
    }

    fn last_cue(&self) -> Option<u8> {
        self.cues.iter().max_by_key(|(_, d)| *d).map(|(c, _)| *c)
    }
}

#[derive(Debug)]
pub struct PlaylistModel {
    pub title: String,
    pub tracks: Vec<TrackModel>,
}

impl PlaylistModel {
    fn into_mix_tracks(&self) -> Vec<MixTrack> {
        let mut mix_tracks: Vec<MixTrack> = vec![];
        let mut current_bpm: f32 = 0.;
        for (i, track) in self.tracks.iter().enumerate() {
            let mix_track = MixTrack::new(
                track.position as usize,
                track.track_id,
                track.title.clone(),
                track.first_cue().unwrap_or(0),
                if i == 0 { 0 } else { 32 + 1 },
                track.last_cue().unwrap_or(0),
                if i == 0 { Some(track.bpm) } else { None },
                None,
                32,
            );
            if i != 0 && track.bpm > current_bpm {
                mix_tracks[i - 1].to_bpm = Some(track.bpm);
            }
            current_bpm = current_bpm.max(track.bpm);
            mix_tracks.push(mix_track);
        }
        mix_tracks
    }
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
                cue.hotcue,
                Duration::from_secs_f32(cue.position.max(0.) / library.samplerate as f32 / 2.0),
            )
        })
        .collect::<BTreeMap<_, _>>();
    Ok(TrackModel {
        track_id: library.id,
        position: playlist_track.position,
        title: library.title,
        artist: library.artist,
        bpm: library.bpm,
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
    table.set_header(vec!["#", "track_id", "bpm", "title", "artist", "cues"]);
    for track in playlist.tracks.iter() {
        let artists = track
            .artist
            .clone()
            .unwrap_or("---".to_string())
            .to_string();
        table.add_row(vec![
            track.position.to_string(),
            track.track_id.to_string(),
            track.bpm.to_string(),
            track.title.chars().take(25).collect(),
            artists.chars().take(15).collect(),
            track
                .cues
                .iter()
                .map(|(n, dur)| format!("{}: {:.1}s", n + 1, dur.as_secs_f32()))
                .collect::<Vec<_>>()
                .join(" "),
        ]);
    }
    println!("{}", table);

    if let Some(out) = &args.out {
        let mix_tracks = playlist.into_mix_tracks();
        let mut writer = csv::Writer::from_writer(vec![]);
        for track in mix_tracks.iter() {
            writer.serialize(track)?;
        }
        std::fs::write(out, writer.into_inner()?)?;
    }
    Ok(())
}
