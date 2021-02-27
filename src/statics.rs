use crate::config::Config;
use notify::{watcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::{stdin, Read};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::time::Duration;
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

static MODIFIED: Lazy<async_std::sync::Mutex<bool>> =
    Lazy::new(|| async_std::sync::Mutex::new(false));

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
            println!("modified");
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
                atx.send(true).await.unwrap();
            } else {
                drop(b);
                async_std::task::sleep(Duration::from_secs(1)).await;
            }
        }
    });
    arx
}
