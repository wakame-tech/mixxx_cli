use crate::{
    cmds::utils::{cue_at, get_hotcue, get_track},
    ffmpeg::{slice_cmd, stepped_tempo_filter::SteppedTempoFilter},
    mixxx::{cue::Cue, library::Library},
};
use anyhow::Result;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

#[derive(Debug, clap::Parser)]
pub struct SliceArgs {
    #[arg(long)]
    pub id: i32,
    #[arg(long)]
    pub from_hotcue: u8,
    #[arg(long, allow_hyphen_values = true)]
    pub from_offset: i32,
    #[arg(long)]
    pub to_hotcue: u8,
    #[arg(long, allow_hyphen_values = true)]
    pub to_offset: i32,
    #[arg(long)]
    pub bpm: f32,
    #[arg(long)]
    pub to_bpm: Option<f32>,
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug)]
pub struct SliceCommand {
    a_path: PathBuf,
    a: Library,
    from: (Cue, i32),
    to: (Cue, i32),
    bpm: f32,
    to_bpm: Option<f32>,
}

impl SliceCommand {
    pub fn new(
        conn: &Connection,
        track_id: i32,
        from_hotcue: u8,
        from_offset: i32,
        to_hotcue: u8,
        to_offset: i32,
        bpm: f32,
        to_bpm: Option<f32>,
    ) -> Result<Self> {
        let (a_path, a) = get_track(conn, track_id)?;
        let from_cue = get_hotcue(conn, track_id, from_hotcue)?;
        let to_cue = get_hotcue(conn, track_id, to_hotcue)?;
        let from = (from_cue, from_offset);
        let to = (to_cue, to_offset);
        Ok(Self {
            a_path,
            a,
            from,
            to,
            bpm,
            to_bpm,
        })
    }

    pub fn id(&self) -> String {
        format!(
            "slice_{}_{}-{}",
            self.a.id, self.from.0.hotcue, self.to.0.hotcue
        )
    }

    pub fn execute(&self, out: &Path) -> Result<()> {
        let (from_hotcue, from_offset) = &self.from;
        let (to_hotcue, to_offset) = &self.to;
        println!(
            "@{}+{} .. @{}+{}",
            from_hotcue.hotcue, from_offset, to_hotcue.hotcue, to_offset
        );

        let beat = 60.0 / self.a.bpm;
        let a_range = (
            cue_at(&self.a, from_hotcue) + beat * *from_offset as f32,
            cue_at(&self.a, to_hotcue) + beat * *to_offset as f32,
        );
        let (f, t) = (0.0, a_range.1 - a_range.0);
        // let (f, t) = a_range;
        let a_scale = if let Some(to_bpm) = self.to_bpm {
            let from_scale = self.bpm / self.a.bpm;
            let to_scale = to_bpm / self.a.bpm;
            SteppedTempoFilter::new((f, from_scale), (t, to_scale), 4)
        } else {
            let scale = self.bpm / self.a.bpm;
            SteppedTempoFilter::new((f, scale), (t, scale), 1)
        };
        println!(
            "bpm={} target_bpm={} tempo={:?}",
            self.a.bpm, self.bpm, a_scale
        );
        let filters = vec![a_scale.to_filters("0", "a"), vec![format!("[a] loudnorm")]].concat();
        slice_cmd(&self.a_path, &filters, a_range, out)?;
        Ok(())
    }
}
