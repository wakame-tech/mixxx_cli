use anyhow::Result;
use std::{path::Path, process::Command};

pub fn slice_cmd(a_path: &Path, filters: &[String], a_range: (f32, f32), out: &Path) -> Result<()> {
    let args = [
        "-loglevel".to_string(),
        "info".to_string(),
        "-y".to_string(),
        "-ss".to_string(),
        a_range.0.to_string(),
        "-to".to_string(),
        a_range.1.to_string(),
        "-i".to_string(),
        a_path.display().to_string(),
        "-filter_complex".to_string(),
        filters.join(";"),
        out.display().to_string(),
    ];
    println!("{:#?}", args);
    let child = Command::new("ffmpeg").args(args).spawn()?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
    println!("O: {}\n", out.display());
    Ok(())
}
