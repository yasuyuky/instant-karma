use crate::db;
use crate::key::Key;
use std::io::{stdin, Read};
use std::path::{Path, PathBuf};

pub async fn load_stdin_to_dict(k: &Key) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = String::default();
    let mut stdin = stdin();
    stdin.read_to_string(&mut buf)?;
    db::put_content(k, &buf).await?;
    Ok(())
}

pub async fn load_file_to_dict(k: &Key, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = String::default();
    let mut f = std::fs::File::open(path)?;
    f.read_to_string(&mut buf)?;
    db::put_content(k, &buf).await?;
    Ok(())
}

pub async fn load_input_to_dict(
    k: &Key,
    path: &Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    match path {
        Some(p) => load_file_to_dict(k, p).await,
        None => load_stdin_to_dict(k).await,
    }
}
