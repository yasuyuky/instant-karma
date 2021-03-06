use crate::config::Config;
use crate::key::Key;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::{stdin, Read};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub static mut GLOBAL_DATA: Lazy<Mutex<HashMap<Key, String>>> =
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

pub fn put_dict(k: &Key, v: &str) {
    if let Ok(d) = unsafe { GLOBAL_DATA.get_mut() } {
        d.insert(k.clone(), v.to_owned());
    }
}

pub fn load_stdin_to_dict(k: &Key) -> Result<(), std::io::Error> {
    let mut buf = String::new();
    let mut stdin = stdin();
    stdin.read_to_string(&mut buf)?;
    put_dict(k, &buf);
    Ok(())
}

pub fn load_file_to_dict(k: &Key, path: &Path) -> Result<(), std::io::Error> {
    let mut buf = String::new();
    let mut f = std::fs::File::open(path)?;
    f.read_to_string(&mut buf)?;
    put_dict(k, &buf);
    Ok(())
}

pub fn load_input_to_dict(k: &Key, path: &Option<PathBuf>) -> Result<(), std::io::Error> {
    match path {
        Some(p) => load_file_to_dict(k, &p),
        None => load_stdin_to_dict(k),
    }
}

pub static KEY: Lazy<Key> = Lazy::new(|| Key::new());
