use anyhow::Result;
use clap::Parser;
use cmds::playlist::PlaylistViewer;
use std::path::PathBuf;

use crate::cmds::converter::TrackLocationsConverter;

mod cmds;
mod mixxx;

#[derive(Debug, clap::Parser)]
enum MixxxCli {
    Playlist(PlaylistArgs),
    Convert(ConvertArgs),
}

#[derive(Debug, clap::Parser)]
struct PlaylistArgs {
    #[arg(long)]
    id: i32,
}

#[derive(Debug, clap::Parser)]
struct ConvertArgs {
    #[arg(long)]
    r#in: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(short, long)]
    directory: PathBuf,
}

fn main() -> Result<()> {
    simplelog::SimpleLogger::init(log::LevelFilter::Debug, Default::default())?;

    let args = MixxxCli::try_parse()?;
    match args {
        MixxxCli::Playlist(args) => {
            let db_path = std::env::var("MIXXX_DB_PATH")?;
            let db_path = PathBuf::from(db_path);
            let viewer = PlaylistViewer::new(&db_path)?;
            viewer.list_playlist_tracks(args.id)?;
        }
        MixxxCli::Convert(args) => {
            std::fs::copy(&args.r#in, &args.out)?;
            log::debug!("copied {} to {}", &args.r#in.display(), args.out.display());

            let converter = TrackLocationsConverter::new(&args.out)?;
            converter.convert_track_locations(&args.directory)?;
        }
    }

    Ok(())
}
