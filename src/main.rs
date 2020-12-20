use async_ctrlc::CtrlC;
use async_std::prelude::*;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use structopt::StructOpt;
use tide::{http::mime, Request, Response};
use uuid::Uuid;

static mut GLOBAL_DATA: Lazy<Mutex<HashMap<u128, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static COPY_TEMPLATE: &str = include_str!("html/copy.html");

static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let home = std::env::var("HOME").unwrap();
    let mut path = PathBuf::from(home);
    let default = std::env::var("XDG_CONFIG_HOME").unwrap_or(".config".to_owned());
    path.push(default);
    path.push("instant-karma");
    path.push("config.toml");
    path
});

static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_path(&CONFIG_PATH));

#[derive(StructOpt)]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Command {
    Copy,
}

#[derive(Debug, Deserialize)]
struct Config {
    prefix: String,
}

impl Config {
    fn new() -> Self {
        Self {
            prefix: String::new(),
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
    }
    Ok(())
}

async fn ctrlc() -> Result<(), std::io::Error> {
    CtrlC::new().expect("Cannot use CTRL-C handler").await;
    println!("termination signal received, stopping server...");
    Ok(())
}

async fn copy() -> tide::Result<()> {
    let mut buf = String::new();
    let mut stdin = std::io::stdin();
    stdin.read_to_string(&mut buf)?;
    let k = Uuid::new_v4();
    put_dict(k.as_u128(), &buf);
    println!("{}{}", CONFIG.prefix, k);
    let app = async {
        let mut app = tide::new();
        app.at("/:id").get(get_copy);
        app.listen("127.0.0.1:4989").await
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
    let pathstr = path.to_str().unwrap_or_default().to_owned();
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
        app.at(&format!("/{}", k)).serve_dir(pathstr + "/")?;
        app.listen("127.0.0.1:4989").await
    };
    app.race(ctrlc()).await?;
    Ok(())
}
