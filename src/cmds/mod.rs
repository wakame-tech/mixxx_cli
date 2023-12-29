use self::{
    converter::{convert_track_locations, ConvertArgs},
    cross_fade::{cross_fade, slice, CrossFadeArgs, SliceArgs},
    playlist::{list_playlist_tracks, PlaylistArgs},
    tag::list_mp3_tag,
};
use anyhow::Result;
use clap::Parser;
use rusqlite::Connection;
use std::path::PathBuf;

pub mod converter;
pub mod cross_fade;
pub mod playlist;
pub mod tag;

#[derive(Debug, clap::Parser)]
enum MixxxCli {
    Playlist(PlaylistArgs),
    Convert(ConvertArgs),
    CrossFade(CrossFadeArgs),
    Slice(SliceArgs),
    Tag,
}

pub fn handle_commands() -> Result<()> {
    let args = MixxxCli::try_parse()?;

    let db_path: String = std::env::var("MIXXX_DB_PATH")?;
    let db_path = PathBuf::from(db_path);
    let conn = Connection::open(db_path)?;

    match args {
        MixxxCli::Playlist(args) => list_playlist_tracks(&conn, &args),
        MixxxCli::Convert(args) => convert_track_locations(&conn, &args),
        MixxxCli::CrossFade(args) => cross_fade(&conn, &args),
        MixxxCli::Slice(args) => slice(&conn, &args),
        MixxxCli::Tag => list_mp3_tag(&conn),
    }
}
