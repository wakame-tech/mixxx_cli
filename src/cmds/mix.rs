use super::{cross_fade::CrossFadeCommand, slice::SliceCommand};
use crate::ffmpeg::concat_cmd;
use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, path::PathBuf};
use std::{io::Write, path::Path};

#[derive(Debug, clap::Parser)]
pub struct CreateMixArgs {
    pub input: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MixTrack {
    position: usize,
    id: i32,
    title: String,
    begin_hotcue: u8,
    begin_offset: i32,
    end_hotcue: u8,
    bpm: Option<f32>,
    pub to_bpm: Option<f32>,
    crossfade: usize,
}

impl MixTrack {
    pub fn new(
        position: usize,
        id: i32,
        title: String,
        begin_hotcue: u8,
        begin_offset: i32,
        end_hotcue: u8,
        bpm: Option<f32>,
        to_bpm: Option<f32>,
        crossfade: usize,
    ) -> Self {
        Self {
            position,
            id,
            title,
            begin_hotcue,
            begin_offset,
            end_hotcue,
            bpm,
            to_bpm,
            crossfade,
        }
    }
}

#[derive(Debug)]
pub enum MixOp {
    Slice(Box<SliceCommand>),
    CrossFade(Box<CrossFadeCommand>),
}

#[derive(Debug)]
pub struct MixList {
    pub ops: Vec<MixOp>,
}

impl MixList {
    pub fn from_tracks(conn: &Connection, tracks: &[MixTrack]) -> Result<Self> {
        assert!(tracks.len() > 1);
        assert!(tracks[0].bpm.is_some());

        let mut ops = vec![];
        let mut current_bpm = tracks[0].bpm.unwrap();
        for i in 0..tracks.len() - 1 {
            let (a, b) = (&tracks[i], &tracks[i + 1]);
            let begin_offset = if i == 0 {
                0
            } else {
                tracks[i - 1].crossfade as i32
            };
            let a_slice = SliceCommand::new(
                conn,
                a.id,
                a.begin_hotcue,
                begin_offset,
                a.end_hotcue,
                0,
                current_bpm,
                a.to_bpm,
            )?;
            if let Some(to_bpm) = a.to_bpm {
                current_bpm = to_bpm;
            }
            ops.push(MixOp::Slice(Box::new(a_slice)));

            if a.crossfade == 0 {
                continue;
            }
            let cross_fade = CrossFadeCommand::new(
                conn,
                a.id,
                a.end_hotcue,
                b.id,
                b.begin_hotcue,
                a.crossfade as u32,
                current_bpm,
            )?;
            ops.push(MixOp::CrossFade(Box::new(cross_fade)));
            if i == tracks.len() - 2 {
                let b_slice = SliceCommand::new(
                    conn,
                    b.id,
                    b.begin_hotcue,
                    0,
                    b.end_hotcue,
                    0,
                    current_bpm,
                    None,
                )?;
                ops.push(MixOp::Slice(Box::new(b_slice)));
            }
        }
        Ok(Self { ops })
    }

    pub fn execute(&self, out: &Path) -> Result<()> {
        let mut file_paths = vec![];
        for op in self.ops.iter() {
            let out = match op {
                MixOp::Slice(slice) => PathBuf::from(format!("{}.mp3", slice.id())),
                MixOp::CrossFade(cross_fade) => PathBuf::from(format!("{}.mp3", cross_fade.id())),
            };
            file_paths.push(format!("file \'{}\'", out.display()));
            if out.exists() {
                continue;
            }
            match op {
                MixOp::Slice(slice) => slice.execute(&out)?,
                MixOp::CrossFade(cross_fade) => cross_fade.execute(&out)?,
            };
        }
        let file_list_path = PathBuf::from("./filelist.txt");
        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&file_list_path)?;
        write!(f, "{}", file_paths.join("\n"))?;
        if !out.exists() {
            concat_cmd(&file_list_path, out)?;
        }
        Ok(())
    }
}
