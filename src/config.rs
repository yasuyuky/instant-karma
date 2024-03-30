use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_prefix")]
    pub prefix: String,
    #[serde(default = "default_port")]
    pub port: u32,
    #[serde(default = "default_length")]
    pub length: usize,
}

fn default_prefix() -> String { "http://localhost:4989/".to_owned() }
fn default_port() -> u32 { 4989 }
fn default_length() -> usize { 13 }

impl Config {
    pub fn new() -> Self {
        Self {
            prefix: default_prefix(),
            port: default_port(),
            length: default_length(),
        }
    }

    pub fn from_path(p: &Path) -> Self {
        let mut s = String::default();
        match File::open(p).and_then(|mut f| f.read_to_string(&mut s)) {
            Ok(_) => toml::from_str(&s).unwrap_or_else(|_| Self::new()),
            Err(_) => Self::new(),
        }
    }
}
