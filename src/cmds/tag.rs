use crate::mixxx::{repo::AsRepo, track_location::TrackLocation};
use anyhow::Result;
use id3::{Tag, TagLike};
use rusqlite::Connection;

pub fn list_mp3_tag(conn: &Connection) -> Result<()> {
    let track_location_repo = TrackLocation::repo(conn);
    let locations = track_location_repo.select_all()?;
    for track_location in locations.iter() {
        // TIT2: title
        // TPE1: artist
        // TPOS
        // TDOR: year
        // TCON: genre
        // TBPM: bpm
        // TKEY: key
        if let Ok(tag) = Tag::read_from_path(&track_location.location) {
            let title = tag.get("TIT2").map(|f| f.content().text().unwrap());
            let artist = tag.get("TPE1").map(|f| f.content().text().unwrap());
            if let (Some(title), Some(artist)) = (title, artist) {
                let file_name = track_location
                    .location
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap();
                let file_name_parts = file_name.rsplitn(2, " - ").collect::<Vec<_>>();
                if file_name_parts.len() != 2 {
                    continue;
                }
                let (file_name_artist, file_name_title) = (file_name_parts[1], file_name_parts[0]);
                if !file_name_title.contains(title) && !file_name_artist.contains(artist) {
                    println!(
                        "filename={}\ntitle={} artist={}\n",
                        track_location.location.display(),
                        title,
                        artist
                    );
                }
            }
        }
    }
    Ok(())
}
