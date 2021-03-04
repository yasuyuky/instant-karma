use crate::ctrlc;
use crate::key::Key;
use crate::statics::*;
use async_std::prelude::*;
use pulldown_cmark::{html, Options, Parser};
use std::path::PathBuf;
use tide::{http::mime, sse, Request, Response};
use uuid::Uuid;

pub async fn render(path: &Option<PathBuf>) -> tide::Result<()> {
    load_input_to_dict(&KEY, &path)?;
    println!("{}{}", CONFIG.prefix, *KEY);
    println!("{}", CONFIG.length);
    let app = async {
        let mut app = tide::new();
        app.at("/:id").get(handle_get);
        if let Some(p) = path {
            load_path(p);
            watch_path(p);
            app.at("/:id/sse").get(sse::endpoint(handle_sse_req));
        }
        app.listen(LISTENER.to_owned()).await
    };
    app.race(ctrlc()).await?;
    Ok(())
}

async fn handle_get(req: Request<()>) -> tide::Result {
    let k = Key::from(req.param("id")?);
    match unsafe { GLOBAL_DATA.get_mut() }?.get(&k) {
        Some(s) => {
            let mut options = Options::empty();
            options.insert(Options::ENABLE_STRIKETHROUGH);
            let parser = Parser::new_ext(s, options);
            let mut rendered = String::new();
            html::push_html(&mut rendered, parser);
            let resp = RENDER_TEMPLATE
                .replace("{id}", &k.to_string())
                .replace("{}", &rendered);
            Ok(Response::builder(200)
                .body(resp)
                .content_type(mime::HTML)
                .build())
        }
        None => Ok(tide::Response::new(404)),
    }
}
