use async_ctrlc::CtrlC;
use std::path::PathBuf;
use structopt::StructOpt;

mod config;
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
