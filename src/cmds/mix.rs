use super::{
    cross_fade::{cross_fade, CrossFadeArgs},
    playlist::PlaylistModel,
    slice::{slice, SliceArgs},
};
use crate::ffmpeg::concat::concat_cmd;
use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use std::{fs::OpenOptions, path::PathBuf};

#[derive(Debug, clap::Parser)]
pub struct CreateMixArgs {
    input: PathBuf,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Slice {
    id: i32,
    begin_hotcue: u8,
    begin_offset: i32,
    end_hotcue: u8,
    bpm: f32,
    to_bpm: Option<f32>,
}

impl Slice {
    pub fn new(
        id: i32,
        begin_hotcue: u8,
        begin_offset: i32,
        end_hotcue: u8,
        bpm: f32,
        to_bpm: Option<f32>,
    ) -> Self {
        Self {
            id,
            begin_hotcue,
            begin_offset,
            end_hotcue,
            bpm,
            to_bpm,
        }
    }

    pub fn into_args(&self, out: &Path) -> SliceArgs {
        SliceArgs {
            id: self.id,
            from_hotcue: self.begin_hotcue,
            from_offset: self.begin_offset,
            to_hotcue: self.end_hotcue,
            to_offset: 0,
            bpm: Some(self.bpm),
            to_bpm: self.to_bpm,
            out: out.to_path_buf(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossFade {
    a_id: i32,
    a_hotcue: u8,
    b_id: i32,
    b_hotcue: u8,
    crossfade: u32,
    bpm: Option<f32>,
}

impl CrossFade {
    pub fn new(
        a_id: i32,
        a_hotcue: u8,
        b_id: i32,
        b_hotcue: u8,
        crossfade: u32,
        bpm: Option<f32>,
    ) -> Self {
        Self {
            a_id,
            a_hotcue,
            b_id,
            b_hotcue,
            crossfade,
            bpm,
        }
    }

    pub fn into_args(&self, out: &Path) -> CrossFadeArgs {
        CrossFadeArgs {
            a_id: self.a_id,
            a_hotcue: self.a_hotcue,
            b_id: self.b_id,
            b_hotcue: self.b_hotcue,
            crossfade: self.crossfade,
            bpm: self.bpm,
            out: out.to_path_buf(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Mix {
    Slice(Slice),
    CrossFade(CrossFade),
}

impl Mix {
    fn file_name(&self) -> PathBuf {
        match self {
            Mix::Slice(slice) => PathBuf::from(format!(
                "{}_{}_{}.mp3",
                slice.id, slice.begin_hotcue, slice.end_hotcue
            )),
            Mix::CrossFade(cross_fade) => PathBuf::from(format!(
                "{}_{}_{}_{}.mp3",
                cross_fade.a_id, cross_fade.a_hotcue, cross_fade.b_id, cross_fade.b_hotcue
            )),
        }
    }
}

fn generate_mixes(conn: &Connection, mixes: &[Mix], out: &Path) -> Result<()> {
    let mut file_paths = vec![];
    for mix in mixes.iter() {
        let out = mix.file_name();
        match mix {
            Mix::Slice(s) => {
                let args = s.into_args(&out);
                slice(conn, &args)
            }
            Mix::CrossFade(c) => {
                let args = c.into_args(&out);
                cross_fade(conn, &args)
            }
        }?;
        file_paths.push(format!("file \'{}\'", out.display()));
    }
    let file_list_path = PathBuf::from("./filelist.txt");
    let mut f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&file_list_path)?;
    write!(f, "{}", file_paths.join("\n"))?;
    concat_cmd(&file_list_path, out)?;
    Ok(())
}

pub fn create_mix(conn: &Connection, args: &CreateMixArgs) -> Result<()> {
    let mut buf = String::new();
    let mut f = File::open(&args.input)?;
    f.read_to_string(&mut buf)?;
    let mixes: Vec<Mix> = ron::from_str(&buf)?;
    generate_mixes(conn, &mixes, &args.out)
}

pub fn export_playlist_mixes(playlist: &PlaylistModel, path: &PathBuf) -> Result<()> {
    let mut mixes = vec![];
    for (i, ab) in playlist.tracks.windows(2).enumerate() {
        let (a, b) = (&ab[0], &ab[1]);
        let begin_offset = if i == 0 { 0 } else { 32 };
        let (a_first_hotcue, a_last_hotcue) =
            (a.first_hotcue().unwrap_or(0), a.last_hotcue().unwrap_or(4));
        let (b_first_hotcue, b_last_hotcue) =
            (b.first_hotcue().unwrap_or(0), b.last_hotcue().unwrap_or(4));
        mixes.push(Mix::Slice(Slice::new(
            a.track_id,
            a_first_hotcue,
            begin_offset,
            a_last_hotcue,
            a.bpm,
            Some(b.bpm),
        )));
        mixes.push(Mix::CrossFade(CrossFade::new(
            a.track_id,
            a_last_hotcue,
            b.track_id,
            b_first_hotcue,
            32,
            Some(b.bpm),
        )));
        if i == playlist.tracks.len() - 1 {
            mixes.push(Mix::Slice(Slice::new(
                b.track_id,
                b_first_hotcue,
                1,
                b_last_hotcue,
                b.bpm,
                None,
            )));
        }
    }
    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    write!(
        f,
        "{}",
        ron::ser::to_string_pretty(&mixes, Default::default())?
    )?;
    Ok(())
}
