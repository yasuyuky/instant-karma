use async_std::prelude::*;
use statics::*;
use std::io::prelude::*;
use tide::{http::mime, Request, Response};
use uuid::Uuid;

use crate::ctrlc;
use crate::statics;

pub async fn copy() -> tide::Result<()> {
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

fn put_dict(k: u128, v: &str) {
    match unsafe { GLOBAL_DATA.get_mut() } {
        Ok(d) => {
            d.insert(k, v.to_owned());
        }
        Err(_) => (),
    }
}

async fn handle_get(req: Request<()>) -> tide::Result {
    let k = Uuid::parse_str(req.param("id")?)?;
    match unsafe { GLOBAL_DATA.get_mut() }?.get(&k.as_u128()) {
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
