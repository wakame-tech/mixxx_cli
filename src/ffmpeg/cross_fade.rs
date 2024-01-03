use anyhow::Result;
use std::{path::Path, process::Command};

pub fn cross_fade_cmd(a_path: &Path, b_path: &Path, filters: &[String], out: &Path) -> Result<()> {
    let args = [
        "-loglevel".to_string(),
        "warning".to_string(),
        "-y".to_string(),
        "-i".to_string(),
        a_path.display().to_string(),
        "-i".to_string(),
        b_path.display().to_string(),
        "-filter_complex".to_string(),
        filters.join(";"),
        "-map".to_string(),
        format!("[{}]", "out"),
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
