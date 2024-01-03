use crate::{
    cmds::utils::{cue_at, get_hotcue, get_track},
    ffmpeg::cross_fade::cross_fade_cmd,
};
use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;

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
    pub bpm: Option<f32>,
    #[arg(long)]
    pub out: PathBuf,
}

pub fn cross_fade(conn: &Connection, args: &CrossFadeArgs) -> Result<()> {
    let (a_path, a) = get_track(conn, args.a_id)?;
    let a_cue = get_hotcue(conn, args.a_id, args.a_hotcue)?;

    let (b_path, b) = get_track(conn, args.b_id)?;
    let b_cue = get_hotcue(conn, args.b_id, args.b_hotcue)?;

    let bpm = args.bpm.unwrap_or(b.bpm);
    let a_scale = bpm / a.bpm;
    let b_scale = bpm / b.bpm;

    let beat_a = 60.0 / a.bpm;
    let beat_b = 60.0 / b.bpm;

    let a_cue_at = cue_at(&a, &a_cue);
    let b_cue_at = cue_at(&b, &b_cue);

    let a_filters = vec![
        format!(
            "[0] atrim=start={}:duration={},asetpts=PTS-STARTPTS [0_1]",
            a_cue_at,
            beat_a * args.crossfade as f32,
        ),
        format!(
            "[0_1] afade=t=out:st={}:duration={}:curve=tri [0_2]",
            beat_a,
            beat_a * args.crossfade as f32,
        ),
        format!("[0_2] loudnorm [0_3]"),
    ];
    let a_out = format!("[0_{}]", a_filters.len());
    let b_filters = vec![
        format!(
            "[1] atrim=start={}:duration={},asetpts=PTS-STARTPTS [1_1]",
            b_cue_at,
            beat_b * (args.crossfade) as f32,
        ),
        format!(
            "[1_1] afade=t=in:st={}:duration={}:curve=tri,asetpts=PTS-STARTPTS [1_2]",
            0.0,
            beat_b * args.crossfade as f32,
        ),
        format!("[1_2] loudnorm [1_3]"),
    ];
    let b_out = format!("[1_{}]", b_filters.len());

    println!(
        "A  : @{}\nA+B: ({}b)\nB  : @{}b",
        args.a_hotcue, args.crossfade, args.b_hotcue,
    );
    let filters = vec![
        a_filters,
        b_filters,
        vec![
            format!("{} atempo={} [1a]", a_out, a_scale),
            format!("{} atempo={} [1b]", b_out, b_scale),
            format!("[1a][1b] amix [out]"),
        ],
    ]
    .concat();

    println!("{:#?}", filters);
    cross_fade_cmd(&a_path, &b_path, &filters, &args.out)?;
    Ok(())
}
