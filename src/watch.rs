use crate::key::Key;
use crate::load::load_file_to_dict;
use crate::statics::*;
use async_std::sync::Mutex as AsyncMutex;
use notify::{watcher, RecursiveMode, Watcher};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;
use tide::{sse::Sender, Request};

static MODIFIED: Lazy<AsyncMutex<HashMap<PathBuf, bool>>> =
    Lazy::new(|| AsyncMutex::new(HashMap::new()));

pub fn watch_path(path: &Path) {
    let p = PathBuf::from(path);
    std::thread::spawn(move || {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
        watcher.watch(p.clone(), RecursiveMode::Recursive).unwrap();
        loop {
            rx.recv().unwrap();
            let mut b = async_std::task::block_on(MODIFIED.lock());
            (*b).insert(p.clone(), true);
        }
    });
}

pub fn async_watch_modified(path: &Path) -> async_std::channel::Receiver<bool> {
    let (atx, arx) = async_channel::unbounded();
    let p = PathBuf::from(path);
    async_std::task::spawn(async move {
        loop {
            let mut b = MODIFIED.lock().await;
            if *(*b).get(&p).unwrap_or(&false) {
                (*b).insert(p.clone(), false);
                atx.send(true).await.unwrap_or_default();
            }
            drop(b);
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
    if let Ok(d) = unsafe { GLOBAL_DATA.lock() } {
        if !d.contains_key(&key) {
            return Err(tide::Error::new(
                403,
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid key"),
            ));
        }
    }
    let arx = async_watch_modified(&path);
    loop {
        match arx.recv().await? {
            _ => {
                load_file_to_dict(&key, &path)?;
                sender.send("", "", None).await?;
            }
        };
    }
}
