use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Raest", about = "A toy ray tracer written in Rust")]
pub struct Config {
    #[structopt(short, long)]
    pub output: Option<PathBuf>,

    #[structopt(short = "n", long, default_value = "50")]
    pub samples: u32,

    #[structopt(short = "j", long, default_value = "4")]
    pub threads: usize,

    #[structopt(long, default_value = "640")]
    pub width: usize,

    #[structopt(long, default_value = "360")]
    pub height: usize,
}
