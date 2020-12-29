use async_std::prelude::*;
use statics::*;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::ctrlc;
use crate::statics;

pub async fn view(path: &Path) -> tide::Result<()> {
    let k = Uuid::new_v4();
    println!("{}{}", CONFIG.prefix, k);
    for p in list_items(&path).unwrap_or_default() {
        let rp = p.strip_prefix(path).unwrap();
        println!("{}{}/{}", CONFIG.prefix, k, rp.to_str().unwrap_or_default());
    }
    let app = async {
        let mut app = tide::new();
        app.at(&format!("/{}", k)).serve_dir(&path)?;
        app.listen(LISTENER.to_owned()).await
    };
    app.race(ctrlc()).await?;
    Ok(())
}

fn list_items(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut result = vec![];
    for entry in path.read_dir().expect("read dir") {
        if let Ok(e) = entry {
            if e.metadata()?.is_file() {
                result.push(PathBuf::from(e.path()))
            } else if e.metadata()?.is_dir() {
                result.append(&mut list_items(&e.path())?)
            }
        }
    }
    Ok(result)
}
