use async_std::prelude::*;
use statics::*;
use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};
use tide::{http::mime, Request, Response};
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

impl Entry {
    fn path(&self) -> &PathBuf {
        match self {
            Self::File { path } => path,
            Self::Dir { path, children: _ } => path,
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path().to_str().unwrap_or_default())
    }
}

pub async fn view(path: &Path) -> tide::Result<()> {
    let k = Uuid::new_v4();
    println!("{}{}", CONFIG.prefix, k);
    let root = list_items(&path)?;
    print_recursively(path, k, &root);
    let app = async {
        let mut app = tide::new();
        index_dirs(&mut app, &k, path, &root);
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

fn print_recursively(base: &Path, k: Uuid, root: &Entry) {
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

fn create_list_string(base: &Path, children: &BTreeSet<Entry>) -> String {
    let mut list = vec![];
    for e in children {
        let rp = e.path().strip_prefix(base).unwrap();
        let rps = rp.to_str().unwrap_or_default();
        let name = rp.file_name().unwrap().to_str().unwrap_or_default();
        list.push(format!("<li><a href={}>{}</a></li>", rps, name))
    }
    list.join("\n")
}

fn index_dirs(app: &mut tide::Server<()>, k: &Uuid, base: &Path, entry: &Entry) {
    if let Entry::Dir { path: p, children } = entry {
        let rp = p.strip_prefix(base).unwrap();
        let list = create_list_string(base, children);
        let p = format!("/{}/{}", k, rp.to_str().unwrap_or_default());
        app.at(&p).get(move |r| index(list.clone(), r));
        for e in children {
            index_dirs(app, k, base, e)
        }
    }
}

async fn index(list: String, _: Request<()>) -> tide::Result {
    Ok(Response::builder(200)
        .body(INDEX_TEMPLATE.replace("{}", &list))
        .content_type(mime::HTML)
        .build())
}
