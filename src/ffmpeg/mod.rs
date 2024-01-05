use anyhow::Result;
use std::{path::Path, process::Command};

pub mod stepped_tempo_filter;

fn ffmpeg(args: Vec<String>) -> Result<()> {
    let args = vec![
        vec![
            "-loglevel".to_string(),
            // "warning".to_string(),
            "verbose".to_string(),
            "-y".to_string(),
        ],
        args,
    ]
    .concat();
    println!("{:#?}", args);
    let child = Command::new("ffmpeg").args(args).spawn()?;
    let cmd_output = child.wait_with_output()?;
    if !cmd_output.status.success() {
        println!("{}", String::from_utf8_lossy(&cmd_output.stderr));
    }
    Ok(())
}

pub fn ffmpeg_complex_filter(
    inputs: Vec<&Path>,
    output: &Path,
    filters: Vec<String>,
) -> Result<()> {
    let args = vec![
        inputs
            .into_iter()
            .flat_map(|i| vec!["-i".to_string(), i.display().to_string()])
            .collect(),
        vec![
            "-filter_complex".to_string(),
            filters.join(";"),
            "-map".to_string(),
            format!("[{}]", "out"),
            output.display().to_string(),
        ],
    ]
    .concat();
    ffmpeg(args)
}

pub fn concat_cmd(file_list_path: &Path, out: &Path) -> Result<()> {
    let args = vec![
        "-f".to_string(),
        "concat".to_string(),
        "-i".to_string(),
        file_list_path.display().to_string(),
        out.display().to_string(),
    ];
    ffmpeg(args)
}

pub fn slice_cmd(a_path: &Path, filters: &[String], a_range: (f32, f32), out: &Path) -> Result<()> {
    let args = vec![
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
    ffmpeg(args)
}
