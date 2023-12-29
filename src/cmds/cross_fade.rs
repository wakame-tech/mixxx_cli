use crate::mixxx::{cue::Cue, library::Library, repo::AsRepo, track_location::TrackLocation};
use anyhow::Result;
use rusqlite::Connection;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    process::Command,
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
    margin_beats: u32,
    #[arg(long)]
    cross_fade_beats: u32,
    #[arg(long)]
    bpm: Option<f32>,
    #[arg(long)]
    out: PathBuf,
}

#[derive(Debug)]
struct TrackClip {
    id: i32,
    path: PathBuf,
    bpm: f32,
    duration: f32,
    beat: f32,
    cue_at: f32,
}

impl TrackClip {
    pub fn new(path: &Path, library: &Library, cue: &Cue) -> Self {
        let cue_at = cue.position / library.samplerate as f32 / 2.0;
        let beat = 60.0 / library.bpm as f32;
        Self {
            id: library.id,
            path: path.to_path_buf(),
            bpm: library.bpm,
            duration: library.duration,
            beat,
            cue_at,
        }
    }

    pub fn at(&self, scale: f32, offset: i32) -> f32 {
        let cue_at = self.cue_at / scale;
        let beat = self.beat / scale;
        cue_at + beat * offset as f32
    }
}

impl Display for TrackClip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id={},bpm={:.0} cue={:3.2}s path={}, dur={}s",
            self.id,
            self.bpm,
            self.cue_at,
            self.path.display(),
            self.duration,
        )
    }
}

fn cross_fade_cmd(
    a: &TrackClip,
    b: &TrackClip,
    a_range: (i32, i32),
    b_range: (i32, i32),
    cross_fade_beats: u32,
    bpm: f32,
    out: &Path,
) -> Result<()> {
    // <https://stackoverflow.com/questions/47437050/crossfading-between-two-audio-files-with-ffmpeg>
    // ffmpeg -i a.mp3 -i b.mp3 -filter_complex "[0]atrim=0:185.0[a]; [1]atrim=80.0[b]; [a][b]acrossfade=d=5.0" out.mp3
    let a_scale = bpm / a.bpm;
    let b_scale = bpm / b.bpm;
    let (mut a_from, a_to) = (a.at(a_scale, a_range.0), a.at(a_scale, a_range.1));
    let (mut b_from, b_to) = (b.at(b_scale, b_range.0), b.at(b_scale, b_range.1));
    let beat = b.beat / b_scale;
    let mut cross = beat * cross_fade_beats as f32;

    println!("beat={}s, cross={}b={}s", beat, cross_fade_beats, cross);
    println!("A: {} .. {} .. {}", a_from, a.cue_at / a_scale, a_to);
    println!("B: {} .. {} .. {}", b_from, b_from + cross, b_to);

    if b_from < 0.0 {
        a_from += b_from.abs();
        cross -= b_from.abs();
        b_from = 0.0;
    }

    // let curve = "cbr";
    let curve = "tri";
    let filters = vec![
        format!("[0]atempo={}[0_1]", a_scale),
        format!("[0_1]atrim={}:{}[0_2]", a_from, a_to),
        format!("[0_2]loudnorm[0_3]"),
        format!("[1]atempo={}[1_1]", b_scale),
        format!("[1_1]atrim={}:{}[1_2]", b_from, b_to),
        format!("[1_2]loudnorm[1_3]"),
        format!("[0_3][1_3]acrossfade=d={}:c1={}:c2={}", cross, curve, curve),
    ];
    let filter_complex = filters.join(";");
    println!("{}", filter_complex);
    let output = Command::new("ffmpeg")
        .args([
            "-y".to_string(),
            "-i".to_string(),
            a.path.display().to_string(),
            "-i".to_string(),
            b.path.display().to_string(),
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

fn get_track_clip<'a>(conn: &'a Connection, track_id: i32, hotcue: i32) -> Result<TrackClip> {
    let lib_repo = Library::repo(conn);
    let track_location_repo = TrackLocation::repo(conn);
    let cue_repo = Cue::repo(conn);

    let library = lib_repo
        .select(track_id)?
        .ok_or(anyhow::anyhow!("track not found"))?;
    let track_location = track_location_repo
        .select(track_id)?
        .ok_or(anyhow::anyhow!("track location not found"))?
        .location;
    let cue = cue_repo
        .hot_cues_by_track_id(track_id)?
        .into_iter()
        .find(|cue| cue.hotcue == hotcue)
        .ok_or(anyhow::anyhow!("hotcue not found"))?;
    Ok(TrackClip::new(&track_location, &library, &cue))
}

pub fn cross_fade<'a>(conn: &'a Connection, args: &CrossFadeArgs) -> Result<()> {
    let clip_a = get_track_clip(conn, args.track_a_id, args.track_a_hotcue)?;
    let clip_b = get_track_clip(conn, args.track_b_id, args.track_b_hotcue)?;
    println!("A:{}\nB:{}", clip_a, clip_b);
    let a_range = (-(args.margin_beats as i32), args.cross_fade_beats as i32);
    let b_range = (0, args.cross_fade_beats as i32 + args.margin_beats as i32);
    cross_fade_cmd(
        &clip_a,
        &clip_b,
        a_range,
        b_range,
        args.cross_fade_beats,
        args.bpm.unwrap_or(clip_b.bpm),
        &args.out,
    )?;
    Ok(())
}
