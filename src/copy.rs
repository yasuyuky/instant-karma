use crate::ctrlc;
use crate::db;
use crate::key::Key;
use crate::load::load_input_to_dict;
use crate::statics::{CONFIG, COPY_TEMPLATE, KEY, LISTENER};
use async_std::prelude::*;
use std::path::PathBuf;
use tide::{http::mime, Request, Response};

pub async fn copy(path: &Option<PathBuf>) -> tide::Result<()> {
    load_input_to_dict(&KEY, path).await.expect("load error");
    log::info!(
        "{}{}/{}",
        CONFIG.prefix,
        *KEY,
        path.clone().unwrap_or_default().to_str().unwrap()
    );
    let app = async {
        let mut app = tide::new();
        app.at("/:id/*path").get(handle_get);
        app.listen(&*LISTENER).await
    };
    app.race(ctrlc()).await?;
    Ok(())
}

async fn handle_get(req: Request<()>) -> tide::Result {
    let k = Key::from(req.param("id")?);
    let path = req.param("path").unwrap_or("");
    match db::get_content(&k).await.expect("db error") {
        Some(s) => {
            let resp = COPY_TEMPLATE
                .replace("{path}", path)
                .replace("{}", &html_escape::encode_text(&s));
            Ok(Response::builder(200)
                .body(resp)
                .content_type(mime::HTML)
                .build())
        }
        None => Ok(tide::Response::new(404)),
    }
}
