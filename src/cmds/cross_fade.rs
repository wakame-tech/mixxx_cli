use crate::mixxx::{cue::Cue, library::Library, repo::AsRepo, track_location::TrackLocation};
use anyhow::Result;
use rusqlite::Connection;
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

#[derive(Debug, clap::Parser)]
pub struct CrossFadeArgs {
    #[arg(long)]
    track_a_id: i32,
    #[arg(long)]
    track_a_hotcue: i32,
    #[arg(long)]
    track_b_id: i32,
    #[arg(long)]
    track_b_hotcue: i32,
    #[arg(long)]
    out: PathBuf,
}

fn cross_fade_cmd(
    a_path: &Path,
    a_begin: Duration,
    a_end: Duration,
    b_path: &Path,
    b_begin: Duration,
    b_end: Duration,
    cross: Duration,
    out: &Path,
) -> Result<()> {
    // <https://stackoverflow.com/questions/47437050/crossfading-between-two-audio-files-with-ffmpeg>
    // ffmpeg -i a.mp3 -i b.mp3 -filter_complex "[0]atrim=0:185.0[a]; [1]atrim=80.0[b]; [a][b]acrossfade=d=5.0" out.mp3
    let filter_complex = format!(
        r#"
        [0]loudnorm[0l];
        [1]loudnorm[1l];
        [0l]atrim={:.2}:{:.2}[a];
        [1l]atrim={:.2}:{:.2}[b]; 
        [a][b]acrossfade=d={:.2}"#,
        a_begin.as_secs_f32(),
        a_end.as_secs_f32(),
        b_begin.as_secs_f32(),
        b_end.as_secs_f32(),
        cross.as_secs_f32()
    );
    println!("{}", filter_complex);
    let output = Command::new("ffmpeg")
        .args([
            "-i".to_string(),
            a_path.display().to_string(),
            "-i".to_string(),
            b_path.display().to_string(),
            "-filter_complex".to_string(),
            filter_complex,
            out.display().to_string(),
        ])
        .output()?;
    println!("{}", output.status);
    // println!("{}", String::from_utf8_lossy(&output.stdout));
    // println!("{}", String::from_utf8_lossy(&output.stderr));
    Ok(())
}

pub fn cross_fade<'a>(conn: &'a Connection, args: &CrossFadeArgs) -> Result<()> {
    let lib_repo = Library::repo(conn);
    let track_location_repo = TrackLocation::repo(conn);
    let cue_repo = Cue::repo(conn);

    let library_a = lib_repo
        .select(args.track_a_id)?
        .ok_or(anyhow::anyhow!("track not found"))?;
    let track_location_a = track_location_repo
        .select(library_a.id)?
        .ok_or(anyhow::anyhow!("track not found"))?
        .location;

    let cue_a = dbg!(cue_repo.hot_cues_by_track_id(args.track_a_id)?)
        .into_iter()
        .find(|cue| cue.hotcue == args.track_a_hotcue)
        .ok_or(anyhow::anyhow!("hotcue not found"))?;
    let cue_a_at = Duration::from_secs_f32(cue_a.position / library_a.samplerate as f32 / 2.0);

    let library_b = lib_repo
        .select(args.track_b_id)?
        .ok_or(anyhow::anyhow!("track not found"))?;
    let track_location_b = track_location_repo
        .select(library_b.id)?
        .ok_or(anyhow::anyhow!("track not found"))?
        .location;
    let cue_b = dbg!(cue_repo.hot_cues_by_track_id(args.track_b_id)?)
        .into_iter()
        .find(|cue| cue.hotcue == args.track_b_hotcue)
        .ok_or(anyhow::anyhow!("hotcue not found"))?;
    let cue_b_at = Duration::from_secs_f32(cue_b.position / library_b.samplerate as f32 / 2.0);

    // 16 beats
    let beat = Duration::from_secs_f32(60.0 / library_a.bpm as f32);
    println!("{}s/beat", beat.as_secs_f32());
    println!(
        "A: {}(d={}s) BPM={}\ncue-16={} cue={}",
        track_location_a.display(),
        library_a.duration,
        library_a.bpm,
        (cue_a_at - beat * 16).as_secs_f32(),
        cue_a_at.as_secs_f32()
    );
    println!(
        "B: {}(d={}s) BPM={}\ncue={} cue+16={}",
        track_location_b.display(),
        library_b.duration,
        library_b.bpm,
        cue_b_at.as_secs_f32(),
        (cue_b_at + beat * 16).as_secs_f32()
    );
    // |-- A(16) --|a@-- A(16) --|
    //             |b@-- B(16) --|-- B(16) --|
    // cross_fade_cmd(
    //     &track_location_a,
    //     cue_a_at - dur_16beats,
    //     cue_a_at + dur_16beats,
    //     &track_location_b,
    //     cue_b_at,
    //     cue_b_at + dur_16beats + dur_16beats,
    //     dur_16beats,
    //     &args.out,
    // )?;
    // |-- A(16) --|a@
    //             |b@-- B(16) --|
    cross_fade_cmd(
        &track_location_a,
        cue_a_at - beat * 16,
        cue_a_at + beat,
        &track_location_b,
        cue_b_at,
        cue_b_at + beat * 16,
        beat,
        &args.out,
    )?;
    Ok(())
}
