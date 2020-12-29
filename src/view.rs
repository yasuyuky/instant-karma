use async_std::prelude::*;
use statics::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::ctrlc;
use crate::statics;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Entry {
    File {
        path: PathBuf,
    },
    Dir {
        path: PathBuf,
        children: BTreeSet<Entry>,
    },
}

pub async fn view(path: &Path) -> tide::Result<()> {
    let k = Uuid::new_v4();
    println!("{}{}", CONFIG.prefix, k);
    let items = list_items(&path).unwrap_or_default();
    for e in items {
        if let Entry::File { path: p } = e {
            let rp = p.strip_prefix(path).unwrap();
            println!("{}{}/{}", CONFIG.prefix, k, rp.to_str().unwrap_or_default());
        }
    }
    let app = async {
        let mut app = tide::new();
        app.at(&format!("/{}", k)).serve_dir(&path)?;
        app.listen(LISTENER.to_owned()).await
    };
    app.race(ctrlc()).await?;
    Ok(())
}

fn list_items(path: &Path) -> Result<BTreeSet<Entry>, std::io::Error> {
    let mut result = BTreeSet::new();
    for entry in path.read_dir().expect("read dir") {
        if let Ok(e) = entry {
            if e.metadata()?.is_file() {
                result.insert(Entry::File {
                    path: PathBuf::from(e.path()),
                });
            } else if e.metadata()?.is_dir() {
                let children = list_items(&e.path())?;
                result.insert(Entry::Dir {
                    path: PathBuf::from(e.path()),
                    children: children.clone(),
                });
                result.extend(children);
            }
        }
    }
    Ok(result)
}
