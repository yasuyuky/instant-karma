use async_ctrlc::CtrlC;
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

mod copy;
mod statics;
mod view;

#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Command {
    Copy,
    View { path: PathBuf },
}

#[derive(Debug, Deserialize)]
pub struct Config {
    prefix: String,
    #[serde(default = "default_port")]
    port: u32,
}

fn default_port() -> u32 { 4989 }

impl Config {
    fn new() -> Self {
        Self {
            prefix: String::new(),
            port: default_port(),
        }
    }

    fn from_path(p: &Path) -> Self {
        let mut s = String::new();
        match File::open(p).and_then(|mut f| f.read_to_string(&mut s)) {
            Ok(_) => toml::from_str(&s).unwrap_or(Self::new()),
            Err(_) => Self::new(),
        }
    }
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let opt = Opt::from_args();
    match opt.cmd {
        Command::Copy => copy::copy().await?,
        Command::View { path } => view::view(&path).await?,
    }
    Ok(())
}

async fn ctrlc() -> Result<(), std::io::Error> {
    CtrlC::new().expect("Cannot use CTRL-C handler").await;
    println!("termination signal received, stopping server...");
    Ok(())
}
