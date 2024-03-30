use crate::key::Key;
use crate::statics::GLOBAL_DATA;
use std::io::{stdin, Read};
use std::path::{Path, PathBuf};

pub fn put_dict(k: &Key, v: &str) {
    if let Ok(d) = unsafe { GLOBAL_DATA.get_mut() } {
        d.insert(k.clone(), v.to_owned());
    }
}

pub fn load_stdin_to_dict(k: &Key) -> Result<(), std::io::Error> {
    let mut buf = String::default();
    let mut stdin = stdin();
    stdin.read_to_string(&mut buf)?;
    put_dict(k, &buf);
    Ok(())
}

pub fn load_file_to_dict(k: &Key, path: &Path) -> Result<(), std::io::Error> {
    let mut buf = String::default();
    let mut f = std::fs::File::open(path)?;
    f.read_to_string(&mut buf)?;
    put_dict(k, &buf);
    Ok(())
}

pub fn load_input_to_dict(k: &Key, path: &Option<PathBuf>) -> Result<(), std::io::Error> {
    match path {
        Some(p) => load_file_to_dict(k, p),
        None => load_stdin_to_dict(k),
    }
}
