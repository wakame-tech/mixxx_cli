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
    id: i32,
    #[arg(long)]
    from_hotcue: i32,
    #[arg(long, allow_hyphen_values = true)]
    from_offset: i32,
    #[arg(long)]
    to_hotcue: i32,
    #[arg(long, allow_hyphen_values = true)]
    to_offset: i32,
    #[arg(long)]
    bpm: Option<f32>,
    #[arg(long)]
    to_bpm: Option<f32>,
    #[arg(long)]
    out: PathBuf,
}

pub fn slice(conn: &Connection, args: &SliceArgs) -> Result<()> {
    let (a_path, a) = get_track(conn, args.id)?;
    let from_cue = get_hotcue(conn, args.id, args.from_hotcue)?;
    let to_cue = get_hotcue(conn, args.id, args.to_hotcue)?;
    println!(
        "@{}+{} .. @{}+{}",
        args.from_hotcue, args.from_offset, args.to_hotcue, args.to_offset
    );

    let a_range = (
        cue_at(&a, &from_cue, args.from_offset),
        cue_at(&a, &to_cue, args.to_offset),
    );
    let duration = a_range.1 - a_range.0;
    let a_scale = if let Some(to_bpm) = args.to_bpm {
        let from_scale = args.bpm.unwrap_or(a.bpm) / a.bpm;
        let to_scale = to_bpm / a.bpm;
        SteppedTempoFilter::new((0.0, from_scale), (duration, to_scale), 4)
    } else {
        let scale = args.bpm.unwrap_or(a.bpm) / a.bpm;
        SteppedTempoFilter::new((0.0, scale), (duration, scale), 1)
    };
    slice_cmd(&a_path, &a_scale, a_range, &args.out)?;
    Ok(())
}
