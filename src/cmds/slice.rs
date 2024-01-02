use crate::cmds::utils::{cue_at, get_hotcue, get_track};
use anyhow::Result;
use rusqlite::Connection;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug)]
struct SteppedTempoFilter {
    spans: Vec<(f32, f32, f32)>,
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

impl SteppedTempoFilter {
    fn new(from: (f32, f32), to: (f32, f32), steps: usize) -> Self {
        let mut spans = vec![];
        for i in 1..=steps {
            let begin = lerp(from.0, to.0, (i - 1) as f32 / steps as f32);
            let end = lerp(from.0, to.0, i as f32 / steps as f32);
            let v = lerp(from.1, to.1, (i - 1) as f32 / steps as f32);
            spans.push((begin, end, v));
        }
        Self { spans }
    }

    fn to_filters(&self, index: usize) -> Vec<String> {
        let mut i = index;
        let mut filters = vec![];
        let mut src_labels = vec![];
        let mut dst_labels = vec![];

        for (begin, end, scale) in self.spans.iter() {
            filters.push(format!(
                "[0_{}] atrim={}:{} [0_{}]",
                i + 1,
                begin,
                end,
                i + 2
            ));
            src_labels.push(format!("[0_{}]", i + 1));
            filters.push(format!("[0_{}] atempo={} [0_{}]", i + 2, scale, i + 3));
            dst_labels.push(format!("[0_{}]", i + 3));
            i += 3;
        }
        filters.insert(
            0,
            format!(
                "[0_{}] asplit={} {}",
                index,
                self.spans.len(),
                src_labels.join("")
            ),
        );
        filters.push(format!(
            "{} concat=n={}:v=0:a=1",
            dst_labels.join(""),
            dst_labels.len()
        ));
        filters
    }
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
    to_bpm: Option<f32>,
    #[arg(long)]
    out: PathBuf,
}

fn slice_cmd(
    a_path: &Path,
    a_scale: &SteppedTempoFilter,
    a_range: (f32, f32),
    out: &Path,
) -> Result<()> {
    let mut filters = vec![
        format!("[0]loudnorm[0_1]"),
        format!("[0_1]atrim={}:{}[0_2]", a_range.0, a_range.1),
    ];
    filters.extend(a_scale.to_filters(2));
    println!("{:#?}", filters);
    let filter_complex = filters.join(";");
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

#[cfg(test)]
mod tests {
    use super::SteppedTempoFilter;

    #[test]
    fn test_pts_filter() {
        let filter = SteppedTempoFilter::new((0.0, 1.0), (20.0, 1.8), 4);
        dbg!(&filter);
        dbg!(filter.to_filters(2));
    }
}
