use crate::config::Config;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::{stdin, Read};
use std::path::PathBuf;
use std::sync::Mutex;
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
    match unsafe { GLOBAL_DATA.get_mut() } {
        Ok(d) => {
            d.insert(k, v.to_owned());
        }
        Err(_) => (),
    }
}

pub fn load_stdin_to_dict() -> Result<Uuid, std::io::Error> {
    let mut buf = String::new();
    let mut stdin = stdin();
    stdin.read_to_string(&mut buf)?;
    let k = Uuid::new_v4();
    put_dict(k.as_u128(), &buf);
    Ok(k)
}
