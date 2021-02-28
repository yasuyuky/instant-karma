use crate::config::Config;
use async_std::sync::Mutex as AsyncMutex;
use notify::{watcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::{stdin, Read};
use std::path::{Path, PathBuf};
use std::sync::{mpsc::channel, Mutex};
use std::time::Duration;
use tide::{sse::Sender, Request};
use uuid::Uuid;

pub static mut GLOBAL_DATA: Lazy<Mutex<HashMap<u128, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub static COPY_TEMPLATE: &str = include_str!("html/copy.html");

pub static RENDER_TEMPLATE: &str = include_str!("html/render.html");

pub static INDEX_TEMPLATE: &str = include_str!("html/index.html");

pub static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = match std::env::var("XDG_CONFIG_HOME") {
        Ok(p) => PathBuf::from(p),
        Err(_) => PathBuf::from(std::env::var("HOME").unwrap() + "/.config"),
    };
    path.push("instant-karma");
    path.push("config.toml");
    path
});

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_path(&CONFIG_PATH));

pub static LISTENER: Lazy<String> = Lazy::new(|| format!("127.0.0.1:{}", CONFIG.port));

pub fn put_dict(k: u128, v: &str) {
    if let Ok(d) = unsafe { GLOBAL_DATA.get_mut() } {
        d.insert(k, v.to_owned());
    }
}

pub fn load_stdin_to_dict(k: &Uuid) -> Result<(), std::io::Error> {
    let mut buf = String::new();
    let mut stdin = stdin();
    stdin.read_to_string(&mut buf)?;
    put_dict(k.as_u128(), &buf);
    Ok(())
}

pub fn load_file_to_dict(k: &Uuid, path: &Path) -> Result<(), std::io::Error> {
    let mut buf = String::new();
    let mut f = std::fs::File::open(path)?;
    f.read_to_string(&mut buf)?;
    put_dict(k.as_u128(), &buf);
    Ok(())
}

pub fn load_input_to_dict(k: &Uuid, path: &Option<PathBuf>) -> Result<(), std::io::Error> {
    match path {
        Some(p) => load_file_to_dict(k, &p),
        None => load_stdin_to_dict(k),
    }
}

static MODIFIED: Lazy<AsyncMutex<bool>> = Lazy::new(|| AsyncMutex::new(false));

pub fn watch_path(path: &Path) {
    let p = PathBuf::from(path);
    std::thread::spawn(move || {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
        watcher.watch(p, RecursiveMode::Recursive).unwrap();
        loop {
            match rx.recv().unwrap() {
                _ => {
                    let mut b = async_std::task::block_on(MODIFIED.lock());
                    *b = true;
                }
            }
        }
    });
}

pub fn async_watch_modified() -> async_std::channel::Receiver<bool> {
    let (atx, arx) = async_channel::unbounded();
    async_std::task::spawn(async move {
        loop {
            let mut b = MODIFIED.lock().await;
            if *b {
                *b = false;
                atx.send(true).await.unwrap_or_default();
            } else {
                drop(b);
            }
            async_std::task::sleep(Duration::from_secs(1)).await;
        }
    });
    arx
}

pub static KEY: Lazy<Uuid> = Lazy::new(|| Uuid::new_v4());
static PATH: Lazy<AsyncMutex<PathBuf>> = Lazy::new(|| AsyncMutex::new(PathBuf::new()));

pub async fn handle_sse_req<State>(req: Request<State>, sender: Sender) -> Result<(), tide::Error>
where
    State: Clone + Send + Sync + 'static,
{
    if *KEY != Uuid::parse_str(req.param("id")?)? {
        return Err(tide::Error::new(
            403,
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid key"),
        ));
    }
    let arx = async_watch_modified();
    loop {
        match arx.recv().await? {
            _ => {
                load_file_to_dict(&KEY, &PATH.lock().await)?;
                sender.send("", "", None).await?;
            }
        };
    }
}

pub fn load_path(p: &Path) {
    let mut mgp = async_std::task::block_on(PATH.lock());
    *mgp = PathBuf::from(&p);
}
