use anyhow::Result;
use cmds::handle_commands;

mod cmds;
mod ffmpeg;
mod mixxx;

fn main() -> Result<()> {
    simplelog::SimpleLogger::init(log::LevelFilter::Debug, Default::default())?;
    handle_commands()
}
