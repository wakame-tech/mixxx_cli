use crate::mixxx::{cue::Cue, library::Library, repo::AsRepo, track_location::TrackLocation};
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
    out: PathBuf,
}

fn get_track<'a>(conn: &'a Connection, track_id: i32) -> Result<(PathBuf, Library)> {
    let lib_repo = Library::repo(conn);
    let track_location_repo = TrackLocation::repo(conn);
    let library = lib_repo
        .select(track_id)?
        .ok_or(anyhow::anyhow!("track not found"))?;
    let track_location = track_location_repo
        .select(track_id)?
        .ok_or(anyhow::anyhow!("track location not found"))?
        .location;
    Ok((track_location, library))
}

fn get_hotcue<'a>(conn: &'a Connection, track_id: i32, hotcue: i32) -> Result<Cue> {
    let cue_repo = Cue::repo(conn);
    let cues = cue_repo
        .hot_cues_by_track_id(track_id)?
        .into_iter()
        .collect::<Vec<_>>();
    let cue = cues
        .into_iter()
        .find(|cue| cue.hotcue == hotcue)
        .ok_or(anyhow::anyhow!("hotcue not found"))?;
    Ok(cue)
}

fn cue_at(library: &Library, cue: &Cue, scale: f32, offset_beats: i32) -> f32 {
    let cue_at = cue.position / library.samplerate as f32 / 2.0;
    let beat = 60.0 / library.bpm;
    (cue_at + beat * offset_beats as f32) / scale
}

fn slice_cmd(a_path: &Path, a_scale: f32, a_range: (f32, f32), out: &Path) -> Result<()> {
    let filters = vec![
        format!("[0]atempo={}[0_1]", a_scale),
        format!("[0_1]atrim={}:{}[0_2]", a_range.0, a_range.1),
        format!("[0_2]loudnorm"),
    ];
    let filter_complex = filters.join(";");
    println!("{}", filter_complex);
    let output = Command::new("ffmpeg")
        .args([
            "-y".to_string(),
            "-i".to_string(),
            a_path.display().to_string(),
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

pub fn cross_fade<'a>(conn: &'a Connection, args: &CrossFadeArgs) -> Result<()> {
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
        cue_at(&a, &a_cue, a_scale, a_range.0),
        cue_at(&a, &a_cue, a_scale, a_range.1),
    );
    let mut b_range = (
        cue_at(&b, &b_cue, b_scale, b_range.0),
        cue_at(&b, &b_cue, b_scale, b_range.1),
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

pub fn slice<'a>(conn: &'a Connection, args: &SliceArgs) -> Result<()> {
    let (a_path, a) = get_track(conn, args.id)?;
    let from_cue = get_hotcue(conn, args.id, args.from_hotcue)?;
    let to_cue = get_hotcue(conn, args.id, args.to_hotcue)?;
    println!(
        "@{}+{} .. @{}+{}",
        args.from_hotcue, args.from_offset, args.to_hotcue, args.to_offset
    );

    let a_scale = args.bpm.unwrap_or(a.bpm) / a.bpm;
    let a_range = (
        cue_at(&a, &from_cue, a_scale, args.from_offset),
        cue_at(&a, &to_cue, a_scale, args.to_offset),
    );
    slice_cmd(&a_path, a_scale, a_range, &args.out)?;
    Ok(())
}
