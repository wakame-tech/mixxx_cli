use crate::{
    cmds::utils::{cue_at, get_hotcue, get_track},
    ffmpeg::ffmpeg_complex_filter,
    mixxx::{cue::Cue, library::Library},
};
use anyhow::Result;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

#[derive(Debug, clap::Parser)]
pub struct CrossFadeArgs {
    #[arg(long)]
    pub a_id: i32,
    #[arg(long)]
    pub a_hotcue: u8,
    #[arg(long)]
    pub b_id: i32,
    #[arg(long)]
    pub b_hotcue: u8,
    #[arg(long)]
    pub crossfade: u32,
    #[arg(long)]
    pub bpm: f32,
    #[arg(long)]
    pub out: PathBuf,
}

#[derive(Debug)]
pub struct CrossFadeCommand {
    pub a_path: PathBuf,
    pub a: Library,
    pub a_cue: Cue,
    pub b_path: PathBuf,
    pub b: Library,
    pub b_cue: Cue,
    pub crossfade: u32,
    pub bpm: f32,
}

impl CrossFadeCommand {
    pub fn new(
        conn: &Connection,
        a_id: i32,
        a_hotcue: u8,
        b_id: i32,
        b_hotcue: u8,
        crossfade: u32,
        bpm: f32,
    ) -> Result<Self> {
        let (a_path, a) = get_track(conn, a_id)?;
        let a_cue = get_hotcue(conn, a_id, a_hotcue)?;

        let (b_path, b) = get_track(conn, b_id)?;
        let b_cue = get_hotcue(conn, b_id, b_hotcue)?;
        Ok(Self {
            a_path,
            a,
            a_cue,
            b_path,
            b,
            b_cue,
            crossfade,
            bpm,
        })
    }

    pub fn id(&self) -> String {
        format!(
            "crossfade_{}-{}_{}-{}",
            self.a.id, self.a_cue.hotcue, self.b.id, self.b_cue.hotcue
        )
    }

    pub fn complex_filter(&self) -> Vec<String> {
        let curve = "squ";
        let bpm = self.bpm;
        let a_scale = bpm / self.a.bpm;
        let b_scale = bpm / self.b.bpm;

        let a_beat = 60.0 / self.a.bpm;
        let a_cross = a_beat * self.crossfade as f32;
        let b_beat = 60.0 / self.b.bpm;
        let b_cross = b_beat * self.crossfade as f32;

        let a_cue_at = cue_at(&self.a, &self.a_cue);
        let b_cue_at = cue_at(&self.b, &self.b_cue);

        let a_filters = vec![
            format!("[0] atrim=start={}:duration={} [0_1]", a_cue_at, a_cross),
            format!(
                "[0_1] afade=t=out:st={}:duration={}:curve={} [0_2]",
                0.0, a_cross, curve,
            ),
            format!("[0_2] loudnorm [0_3]"),
            format!("[0_3] equalizer=f=300:t=h:width=200:g=-10 [0_4]"),
            format!("[0_4] atempo={} [0_out]", a_scale),
        ];
        let b_filters = vec![
            format!(
                "[1] atrim=start={}:duration={},asetpts=PTS-STARTPTS [1_1]",
                b_cue_at, b_cross,
            ),
            format!(
                "[1_1] afade=t=in:st={}:duration={}:curve={} [1_2]",
                0.0, b_cross, curve,
            ),
            format!("[1_2] loudnorm [1_3]"),
            // format!("[1_3] equalizer=f=300:t=h:width=200:g=-10 [1_4]"),
            format!("[1_3] atempo={} [1_out]", b_scale),
        ];
        println!(
            "A  : @{} {}s + {}s\nB  : @{} {}s + {}s",
            self.a_cue.hotcue, a_cue_at, a_cross, self.b_cue.hotcue, b_cue_at, b_cross,
        );
        println!(
            "a_cross {} / {} = {}, b_cross {} / {} = {}",
            a_cross,
            a_scale,
            a_cross / a_scale,
            b_cross,
            b_scale,
            b_cross / b_scale,
        );
        vec![
            a_filters,
            b_filters,
            vec![format!("[0_out][1_out] amix=duration=longest [out]")],
        ]
        .concat()
    }

    pub fn execute(&self, out: &Path) -> Result<()> {
        let filters = self.complex_filter();
        ffmpeg_complex_filter(vec![&self.a_path, &self.b_path], out, filters)
    }
}
