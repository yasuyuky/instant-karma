use async_std::prelude::*;
use statics::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::ctrlc;
use crate::statics;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Entry {
    Dir {
        path: PathBuf,
        children: BTreeSet<Entry>,
    },
    File {
        path: PathBuf,
    },
}

pub async fn view(path: &Path) -> tide::Result<()> {
    let k = Uuid::new_v4();
    println!("{}{}", CONFIG.prefix, k);
    let root = list_items(&path)?;
    print_recursively(path, k, root);
    let app = async {
        let mut app = tide::new();
        app.at(&format!("/{}", k)).serve_dir(&path)?;
        app.listen(LISTENER.to_owned()).await
    };
    app.race(ctrlc()).await?;
    Ok(())
}

fn list_items(path: &Path) -> Result<Entry, std::io::Error> {
    let mut result = BTreeSet::new();
    for entry in path.read_dir().expect("read dir") {
        if let Ok(e) = entry {
            if e.metadata()?.is_file() {
                result.insert(Entry::File {
                    path: PathBuf::from(e.path()),
                });
            } else if e.metadata()?.is_dir() {
                result.insert(list_items(&e.path())?);
            }
        }
    }
    Ok(Entry::Dir {
        path: PathBuf::from(path),
        children: result,
    })
}

fn print_recursively(base: &Path, k: Uuid, root: Entry) {
    match root {
        Entry::File { path: p } => {
            let rp = p.strip_prefix(base).unwrap();
            println!("{}{}/{}", CONFIG.prefix, k, rp.to_str().unwrap_or_default());
        }
        Entry::Dir { path: _, children } => {
            for e in children {
                print_recursively(base, k, e)
            }
        }
    }
}
