use anyhow::Result;
use std::{path::Path, process::Command};

pub fn concat_cmd(file_list_path: &Path, out: &Path) -> Result<()> {
    let child = Command::new("ffmpeg")
        .args(vec![
            "-loglevel".to_string(),
            "warning".to_string(),
            "-f".to_string(),
            "concat".to_string(),
            "-y".to_string(),
            "-i".to_string(),
            file_list_path.display().to_string(),
            out.display().to_string(),
        ])
        .spawn()?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
    println!("O: {}\n", out.display());
    Ok(())
}
