use crate::mixxx::{repo::AsRepo, track_location::TrackLocation};
use anyhow::Result;
use kdam::BarExt;
use rusqlite::Connection;
use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub struct ConvertArgs {
    #[arg(long)]
    r#in: PathBuf,
    #[arg(long)]
    out: PathBuf,
    #[arg(short, long)]
    directory: PathBuf,
}

pub fn convert_track_locations(conn: &Connection, args: &ConvertArgs) -> Result<()> {
    std::fs::copy(&args.r#in, &args.out)?;
    log::debug!("copied {} to {}", args.r#in.display(), args.out.display());

    let track_location_repo = TrackLocation::repo(conn);

    let mut track_locations = track_location_repo.select_all()?;
    let mut pb = kdam::tqdm!(total = track_locations.len());
    for track_location in track_locations.iter_mut() {
        let file_name = track_location.location.file_name().unwrap();
        let new_location = args.directory.clone().join(file_name);
        track_location.location = new_location.clone();
        track_location.directory = args.directory.to_path_buf();
        if let Err(e) = track_location_repo.update(track_location) {
            log::debug!("{}", e);
            log::debug!("location={}", new_location.display());
        }
        pb.update(1)?;
    }
    log::debug!("{} tracks converted", track_locations.len());
    Ok(())
}
