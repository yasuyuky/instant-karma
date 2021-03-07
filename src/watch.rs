use crate::key::Key;
use crate::load::load_file_to_dict;
use crate::statics::*;
use async_std::sync::Mutex as AsyncMutex;
use notify::{watcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use tide::{sse::Sender, Request};

static MODIFIED: Lazy<AsyncMutex<bool>> = Lazy::new(|| AsyncMutex::new(false));

pub fn watch_path(path: &Path) {
    let p = PathBuf::from(path);
    std::thread::spawn(move || {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
        watcher.watch(p, RecursiveMode::Recursive).unwrap();
        loop {
            match rx.recv().unwrap() {
                _ => {
                    let mut b = async_std::task::block_on(MODIFIED.lock());
                    *b = true;
                }
            }
        }
    });
}

pub fn async_watch_modified() -> async_std::channel::Receiver<bool> {
    let (atx, arx) = async_channel::unbounded();
    async_std::task::spawn(async move {
        loop {
            let mut b = MODIFIED.lock().await;
            if *b {
                *b = false;
                atx.send(true).await.unwrap_or_default();
            } else {
                drop(b);
            }
            async_std::task::sleep(Duration::from_secs(1)).await;
        }
    });
    arx
}

pub async fn handle_sse_req<State>(req: Request<State>, sender: Sender) -> Result<(), tide::Error>
where
    State: Clone + Send + Sync + 'static,
{
    let key = Key::from(req.param("id")?);
    let path = PathBuf::from(req.param("path")?);
    if *KEY != key {
        return Err(tide::Error::new(
            403,
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid key"),
        ));
    }
    let arx = async_watch_modified();
    loop {
        match arx.recv().await? {
            _ => {
                load_file_to_dict(&key, &path)?;
                sender.send("", "", None).await?;
            }
        };
    }
}

static PATH: Lazy<AsyncMutex<PathBuf>> = Lazy::new(|| AsyncMutex::new(PathBuf::new()));

pub fn load_path(p: &Path) {
    let mut mgp = async_std::task::block_on(PATH.lock());
    *mgp = PathBuf::from(&p);
}
