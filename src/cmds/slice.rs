use crate::{
    cmds::utils::{cue_at, get_hotcue, get_track},
    ffmpeg::{slice::slice_cmd, stepped_tempo_filter::SteppedTempoFilter},
};
use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

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
    pub bpm: Option<f32>,
    #[arg(long)]
    pub to_bpm: Option<f32>,
    #[arg(long)]
    pub out: PathBuf,
}

pub fn slice(conn: &Connection, args: &SliceArgs) -> Result<()> {
    let (a_path, a) = get_track(conn, args.id)?;
    let from_cue = get_hotcue(conn, args.id, args.from_hotcue)?;
    let to_cue = get_hotcue(conn, args.id, args.to_hotcue)?;
    println!(
        "@{}+{} .. @{}+{}",
        args.from_hotcue, args.from_offset, args.to_hotcue, args.to_offset
    );

    let bpm = args.bpm.unwrap_or(a.bpm);
    let beat = 60.0 / bpm;
    let a_range = (
        cue_at(&a, &from_cue) + beat * args.from_offset as f32,
        cue_at(&a, &to_cue) + beat * args.to_offset as f32,
    );
    let (f, t) = (0.0, a_range.1 - a_range.0);
    // let (f, t) = a_range;
    let a_scale = if let Some(to_bpm) = args.to_bpm {
        let from_scale = args.bpm.unwrap_or(a.bpm) / a.bpm;
        let to_scale = to_bpm / a.bpm;
        SteppedTempoFilter::new((f, from_scale), (t, to_scale), 4)
    } else {
        let scale = args.bpm.unwrap_or(a.bpm) / a.bpm;
        SteppedTempoFilter::new((f, scale), (t, scale), 1)
    };
    println!("bpm={} target_bpm={} tempo={:?}", a.bpm, bpm, a_scale);
    let filters = vec![a_scale.to_filters("0", "a"), vec![format!("[a] loudnorm")]].concat();
    slice_cmd(&a_path, &filters, a_range, &args.out)?;
    Ok(())
}
