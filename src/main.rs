use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::prelude::*;
use std::sync::Mutex;
use structopt::StructOpt;
use tide::Request;
use uuid::Uuid;

static mut GLOBAL_DATA: Lazy<Mutex<HashMap<u128, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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

async fn copy() -> tide::Result<()> {
    let mut app = tide::new();
    let mut buf = String::new();
    let mut stdin = std::io::stdin();
    stdin.read_to_string(&mut buf)?;
    println!("{}", create(&buf));
    app.at("/:id").get(get);
    app.listen("127.0.0.1:4989").await?;
    Ok(())
}

fn create(v: &str) -> String {
    let k = Uuid::new_v4();
    unsafe {
        match GLOBAL_DATA.get_mut() {
            Ok(d) => {
                d.insert(k.as_u128(), String::from(v));
            }
            Err(_) => (),
        }
    }
    format!("{}", k)
}

async fn get(req: Request<()>) -> tide::Result {
    let k = Uuid::parse_str(req.param("id")?)?;
    unsafe {
        match GLOBAL_DATA.get_mut()?.get(&k.as_u128()) {
            Some(s) => Ok(format!("{}", s).into()),
            None => Ok(tide::Response::new(404)),
        }
    }
}
