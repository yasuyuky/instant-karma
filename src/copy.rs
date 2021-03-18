use crate::ctrlc;
use crate::key::Key;
use crate::load::load_input_to_dict;
use crate::statics::*;
use async_std::prelude::*;
use std::path::PathBuf;
use tide::{http::mime, Request, Response};

pub async fn copy(path: &Option<PathBuf>) -> tide::Result<()> {
    load_input_to_dict(&KEY, path)?;
    println!("{}{}", CONFIG.prefix, *KEY);
    let app = async {
        let mut app = tide::new();
        app.at("/:id/*path").get(handle_get);
        app.listen(LISTENER.to_owned()).await
    };
    app.race(ctrlc()).await?;
    Ok(())
}

async fn handle_get(req: Request<()>) -> tide::Result {
    let k = Key::from(req.param("id")?);
    match unsafe { GLOBAL_DATA.get_mut() }?.get(&k) {
        Some(s) => {
            let resp = COPY_TEMPLATE.replace("{}", &html_escape::encode_text(s));
            Ok(Response::builder(200)
                .body(resp)
                .content_type(mime::HTML)
                .build())
        }
        None => Ok(tide::Response::new(404)),
    }
}
