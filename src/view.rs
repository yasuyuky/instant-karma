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

    fn pathstr(&self) -> String {
        match self {
            Self::File { path } => path.to_str().unwrap_or_default().to_owned(),
            Self::Dir { path, children: _ } => path.to_str().unwrap_or_default().to_owned() + "/",
        }
    }

    fn name(&self) -> String {
        match self {
            Self::File { path } => {
                let name = path.file_name().unwrap();
                name.to_str().unwrap_or_default().to_owned()
            }
            Self::Dir { path, children: _ } => {
                let name = path.file_name().unwrap();
                name.to_str().unwrap_or_default().to_owned() + "/"
            }
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
    let root = list_items(&path, &path)?;
    print_recursively(k, &root);
    let app = async {
        let mut app = tide::new();
        index_dirs(&mut app, &k, &root);
        app.at(&format!("/{}", k)).serve_dir(&path)?;
        app.listen(LISTENER.to_owned()).await
    };
    app.race(ctrlc()).await?;
    Ok(())
}

fn list_items(base: &Path, path: &Path) -> Result<Entry, std::io::Error> {
    let mut result = BTreeSet::new();
    for entry in path.read_dir().expect("read dir") {
        if let Ok(e) = entry {
            if e.metadata()?.is_file() {
                let ep = e.path();
                let rp = ep.strip_prefix(base).unwrap();
                result.insert(Entry::File {
                    path: PathBuf::from(rp),
                });
            } else if e.metadata()?.is_dir() {
                result.insert(list_items(&base, &e.path())?);
            }
        }
    }
    let rp = path.strip_prefix(base).unwrap();
    Ok(Entry::Dir {
        path: PathBuf::from(rp),
        children: result,
    })
}

fn print_recursively(k: Uuid, root: &Entry) {
    match root {
        Entry::File { path: p } => {
            println!("{}{}/{}", CONFIG.prefix, k, p.to_str().unwrap_or_default());
        }
        Entry::Dir { path: _, children } => {
            for e in children {
                print_recursively(k, e)
            }
        }
    }
}

fn create_list_string(children: &BTreeSet<Entry>) -> String {
    let mut list = vec![];
    for e in children {
        let ps = e.path().to_str().unwrap_or_default();
        let name = e.path().file_name().unwrap().to_str().unwrap_or_default();
        list.push(format!("<li><a href={}>{}</a></li>", ps, name))
    }
    list.join("\n")
}

fn index_dirs(app: &mut tide::Server<()>, k: &Uuid, entry: &Entry) {
    if let Entry::Dir { path: p, children } = entry {
        let list = create_list_string(children);
        let p = format!("/{}/{}", k, p.to_str().unwrap_or_default());
        app.at(&p).get(move |r| index(list.clone(), r));
        for e in children {
            index_dirs(app, k, e)
        }
    }
}

async fn index(list: String, _: Request<()>) -> tide::Result {
    Ok(Response::builder(200)
        .body(INDEX_TEMPLATE.replace("{}", &list))
        .content_type(mime::HTML)
        .build())
}
