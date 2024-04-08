use clap::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path of storage disk dump.
    #[arg(short, long)]
    pub path: Option<String>,

    /// Time interval (seconds) of storage disk dump.
    #[arg(short, long, default_value_t = 3600)]
    pub interval: u16,
}
