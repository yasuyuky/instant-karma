use async_std::prelude::*;
use statics::*;
use std::path::Path;
use uuid::Uuid;

use crate::ctrlc;
use crate::statics;

pub async fn view(path: &Path) -> tide::Result<()> {
    let k = Uuid::new_v4();
    println!("{}{}", CONFIG.prefix, k);
    for entry in path.read_dir().expect("read dir") {
        if let Ok(e) = entry {
            let f = e.path().file_name().unwrap_or_default().to_owned();
            println!("{}{}/{}", CONFIG.prefix, k, f.to_str().unwrap_or_default());
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
