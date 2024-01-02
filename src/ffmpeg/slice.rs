use super::stepped_tempo_filter::SteppedTempoFilter;
use anyhow::Result;
use std::{path::Path, process::Command};

pub fn slice_cmd(
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
