use crate::cmds::utils::{cue_at, get_hotcue, get_track};
use anyhow::Result;
use rusqlite::Connection;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, clap::Parser)]
pub struct CrossFadeArgs {
    #[arg(long)]
    a_id: i32,
    #[arg(long)]
    a_hotcue: i32,
    #[arg(long)]
    b_id: i32,
    #[arg(long)]
    b_hotcue: i32,
    #[arg(long)]
    margin: u32,
    #[arg(long)]
    crossfade: u32,
    #[arg(long)]
    bpm: Option<f32>,
    #[arg(long)]
    out: PathBuf,
}

fn cross_fade_cmd(
    (a_path, a_scale, a_range): (&Path, f32, (f32, f32)),
    (b_path, b_scale, b_range): (&Path, f32, (f32, f32)),
    duration: f32,
    out: &Path,
) -> Result<()> {
    // let curve = "cbr";
    let curve = "tri";
    let filters = vec![
        format!("[0]atempo={}[0_1]", a_scale),
        format!("[0_1]atrim={}:{}[0_2]", a_range.0, a_range.1),
        format!("[0_2]loudnorm[0_3]"),
        format!("[1]atempo={}[1_1]", b_scale),
        format!("[1_1]atrim={}:{}[1_2]", b_range.0, b_range.1),
        format!("[1_2]loudnorm[1_3]"),
        format!(
            "[0_3][1_3]acrossfade=d={}:c1={}:c2={}",
            duration, curve, curve
        ),
    ];
    let filter_complex = filters.join(";");
    println!("{}", filter_complex);
    let output = Command::new("ffmpeg")
        .args([
            "-y".to_string(),
            "-i".to_string(),
            a_path.display().to_string(),
            "-i".to_string(),
            b_path.display().to_string(),
            "-filter_complex".to_string(),
            filter_complex,
            out.display().to_string(),
        ])
        .output()?;
    if !output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
    println!("O: {}\n", out.display());
    Ok(())
}

pub fn cross_fade(conn: &Connection, args: &CrossFadeArgs) -> Result<()> {
    let (a_path, a) = get_track(conn, args.a_id)?;
    let a_cue = get_hotcue(conn, args.a_id, args.a_hotcue)?;
    let (b_path, b) = get_track(conn, args.b_id)?;
    let b_cue = get_hotcue(conn, args.b_id, args.b_hotcue)?;
    let a_range = (-(args.margin as i32), args.crossfade as i32);
    let b_range = (0, args.crossfade as i32 + args.margin as i32);
    println!(
        "@{}+{} ..({}).. @{}+{}",
        args.a_hotcue,
        -(args.margin as i32),
        args.crossfade,
        args.b_hotcue,
        args.margin
    );
    let bpm = args.bpm.unwrap_or(b.bpm);
    let a_scale = bpm / a.bpm;
    let b_scale = bpm / b.bpm;
    let beat = 60.0 / bpm;
    let mut a_range = (
        cue_at(&a, &a_cue, a_range.0) / a_scale,
        cue_at(&a, &a_cue, a_range.1) / a_scale,
    );
    let mut b_range = (
        cue_at(&b, &b_cue, b_range.0) / b_scale,
        cue_at(&b, &b_cue, b_range.1) / b_scale,
    );
    let mut cross = beat * args.crossfade as f32;

    println!("beat={}s, cross={}b={}s", beat, args.crossfade, cross);

    println!(
        "A: BPM={}->{}(scale={}) {} .. {} ({}s)",
        a.bpm,
        bpm,
        a_scale,
        a_range.0,
        a_range.1,
        a_range.1 - a_range.0
    );
    println!(
        "B:BPM={}->{}(scale={}) {} .. {} ({}s)",
        b.bpm,
        bpm,
        b_scale,
        b_range.0,
        b_range.1,
        b_range.1 - b_range.0
    );

    if b_range.0 < 0.0 {
        a_range.0 += b_range.0.abs();
        cross -= b_range.0.abs();
        b_range.0 = 0.0;
    }
    cross_fade_cmd(
        (&a_path, a_scale, a_range),
        (&b_path, b_scale, b_range),
        cross,
        &args.out,
    )?;
    Ok(())
}
