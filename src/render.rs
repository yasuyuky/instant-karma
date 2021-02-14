use crate::ctrlc;
use crate::statics::*;
use async_std::prelude::*;
use pulldown_cmark::{html, Options, Parser};
use std::io::prelude::*;
use tide::{http::mime, Request, Response};
use uuid::Uuid;

pub async fn render() -> tide::Result<()> {
    let mut buf = String::new();
    let mut stdin = std::io::stdin();
    stdin.read_to_string(&mut buf)?;
    let k = Uuid::new_v4();
    put_dict(k.as_u128(), &buf);
    println!("{}{}", CONFIG.prefix, k);
    let app = async {
        let mut app = tide::new();
        app.at("/:id").get(handle_get);
        app.listen(LISTENER.to_owned()).await
    };
    app.race(ctrlc()).await?;
    Ok(())
}

async fn handle_get(req: Request<()>) -> tide::Result {
    let k = Uuid::parse_str(req.param("id")?)?;
    match unsafe { GLOBAL_DATA.get_mut() }?.get(&k.as_u128()) {
        Some(s) => {
            let mut options = Options::empty();
            options.insert(Options::ENABLE_STRIKETHROUGH);
            let parser = Parser::new_ext(s, options);
            let mut rendered = String::new();
            html::push_html(&mut rendered, parser);
            let resp = RENDER_TEMPLATE.replace("{}", &rendered);
            Ok(Response::builder(200)
                .body(resp)
                .content_type(mime::HTML)
                .build())
        }
        None => Ok(tide::Response::new(404)),
    }
}
