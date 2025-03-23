use crate::ctrlc;
use crate::db;
use crate::key::Key;
use crate::load::load_input_to_dict;
use crate::statics::{CONFIG, LISTENER, RENDER_TEMPLATE};
use crate::watch::{handle_sse_req, watch_path};
use async_std::prelude::*;
use pulldown_cmark::{html, Options, Parser};
use std::path::PathBuf;
use tide::{http::mime, sse, Request, Response};

pub async fn render(path: &Option<PathBuf>) -> tide::Result<()> {
    let k = Key::new();
    load_input_to_dict(&k, path).await.expect("load error");
    print!("{}{}", CONFIG.prefix, &k);
    let app = async {
        let mut app = tide::new();
        app.at("/:id/*path").get(handle_get);
        if let Some(p) = path {
            log::info!("/{}", p.to_str().unwrap());
            watch_path(p);
            app.at("/sse/:id/*path").get(sse::endpoint(handle_sse_req));
        } else {
            log::info!("");
        }
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
            let mut options = Options::empty();
            options.insert(Options::ENABLE_STRIKETHROUGH);
            let parser = Parser::new_ext(&s, options);
            let mut rendered = String::default();
            html::push_html(&mut rendered, parser);
            let resp = RENDER_TEMPLATE
                .replace("{id}", &k.to_string())
                .replace("{path}", path)
                .replace("{}", &rendered);
            Ok(Response::builder(200)
                .body(resp)
                .content_type(mime::HTML)
                .build())
        }
        None => Ok(tide::Response::new(404)),
    }
}
