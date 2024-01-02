use anyhow::Result;
use std::{path::Path, process::Command};

pub fn cross_fade_cmd(
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
