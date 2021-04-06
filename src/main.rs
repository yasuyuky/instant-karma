use async_ctrlc::CtrlC;
use std::path::PathBuf;
use structopt::StructOpt;

mod config;
mod copy;
mod key;
mod load;
mod logger;
mod render;
mod statics;
mod view;
mod watch;

#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Command {
    Copy { path: Option<PathBuf> },
    Render { path: Option<PathBuf> },
    View { path: PathBuf },
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    log::set_logger(&logger::CONSOLE_LOGGER).unwrap_or_default();
    log::set_max_level(log::LevelFilter::Info);
    let opt = Opt::from_args();
    match opt.cmd {
        Command::Copy { path } => copy::copy(&path).await?,
        Command::Render { path } => render::render(&path).await?,
        Command::View { path } => view::view(&path).await?,
    }
    Ok(())
}

async fn ctrlc() -> Result<(), std::io::Error> {
    CtrlC::new().expect("Cannot use CTRL-C handler").await;
    log::info!("termination signal received, stopping server...");
    Ok(())
}
