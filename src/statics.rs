use crate::config::Config;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

pub static mut GLOBAL_DATA: Lazy<Mutex<HashMap<u128, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub static COPY_TEMPLATE: &str = include_str!("html/copy.html");

pub static CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let home = std::env::var("HOME").unwrap();
    let mut path = PathBuf::from(home);
    let default = std::env::var("XDG_CONFIG_HOME").unwrap_or(".config".to_owned());
    path.push(default);
    path.push("instant-karma");
    path.push("config.toml");
    path
});

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_path(&CONFIG_PATH));

pub static LISTENER: Lazy<String> = Lazy::new(|| format!("127.0.0.1:{}", CONFIG.port));
