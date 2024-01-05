use self::{
    converter::{convert_track_locations, ConvertArgs},
    cross_fade::{CrossFadeArgs, CrossFadeCommand},
    mix::{CreateMixArgs, MixList, MixTrack},
    playlist::{list_playlist_tracks, PlaylistArgs},
    slice::{SliceArgs, SliceCommand},
    tag::list_mp3_tag,
};
use anyhow::Result;
use clap::Parser;
use rusqlite::Connection;
use std::path::PathBuf;

pub mod converter;
pub mod cross_fade;
pub mod mix;
pub mod playlist;
pub mod slice;
pub mod tag;
pub mod utils;

#[derive(Debug, clap::Parser)]
enum MixxxCli {
    Playlist(PlaylistArgs),
    Convert(ConvertArgs),
    CrossFade(CrossFadeArgs),
    Slice(SliceArgs),
    CreateMix(CreateMixArgs),
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
        MixxxCli::CrossFade(args) => {
            let cmd = CrossFadeCommand::new(
                &conn,
                args.a_id,
                args.a_hotcue,
                args.b_id,
                args.b_hotcue,
                args.crossfade,
                args.bpm,
            )?;
            cmd.execute(&args.out)
        }
        MixxxCli::Slice(args) => {
            let cmd = SliceCommand::new(
                &conn,
                args.id,
                args.from_hotcue,
                args.from_offset,
                args.to_hotcue,
                args.to_offset,
                args.bpm,
                args.to_bpm,
            )?;
            cmd.execute(&args.out)
        }
        MixxxCli::CreateMix(args) => {
            let tracks = csv::Reader::from_path(&args.input)?
                .deserialize()
                .collect::<Result<Vec<MixTrack>, _>>()?;
            let mix = MixList::from_tracks(&conn, &tracks)?;
            mix.execute(&args.out)
        }
        MixxxCli::Tag => list_mp3_tag(&conn),
    }
}
