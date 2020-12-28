use async_ctrlc::CtrlC;
use async_std::prelude::*;
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use tide::{http::mime, Request, Response};
use uuid::Uuid;

mod statics;
use statics::*;

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
        Command::Copy => copy().await?,
        Command::View { path } => view(&path).await?,
    }
    Ok(())
}

async fn ctrlc() -> Result<(), std::io::Error> {
    CtrlC::new().expect("Cannot use CTRL-C handler").await;
    println!("termination signal received, stopping server...");
    Ok(())
}

pub async fn copy() -> tide::Result<()> {
    let mut buf = String::new();
    let mut stdin = std::io::stdin();
    stdin.read_to_string(&mut buf)?;
    let k = Uuid::new_v4();
    put_dict(k.as_u128(), &buf);
    println!("{}{}", CONFIG.prefix, k);
    let app = async {
        let mut app = tide::new();
        app.at("/:id").get(get_copy);
        app.listen(LISTENER.to_owned()).await
    };
    app.race(ctrlc()).await?;
    Ok(())
}

fn put_dict(k: u128, v: &str) {
    unsafe {
        match GLOBAL_DATA.get_mut() {
            Ok(d) => {
                d.insert(k, v.to_owned());
            }
            Err(_) => (),
        }
    }
}

async fn get_copy(req: Request<()>) -> tide::Result {
    let k = Uuid::parse_str(req.param("id")?)?;
    unsafe {
        match GLOBAL_DATA.get_mut()?.get(&k.as_u128()) {
            Some(s) => {
                let resp = COPY_TEMPLATE.replace("{}", &html_escape::encode_text(s));
                Ok(Response::builder(200)
                    .body(resp)
                    .content_type(mime::HTML)
                    .build())
            }
            None => Ok(tide::Response::new(404)),
        }
    }
}

async fn view(path: &Path) -> tide::Result<()> {
    let k = Uuid::new_v4();
    println!("{}{}", CONFIG.prefix, k);
    for entry in path.read_dir().expect("read dir") {
        if let Ok(e) = entry {
            let f = e.path().file_name().unwrap_or_default().to_owned();
            println!("{}{}/{}", CONFIG.prefix, k, f.to_str().unwrap_or_default());
        }
    }
    let app = async {
        let mut app = tide::new();
        app.at(&format!("/{}", k)).serve_dir(&path)?;
        app.listen(LISTENER.to_owned()).await
    };
    app.race(ctrlc()).await?;
    Ok(())
}
