use crate::ctrlc;
use crate::statics::*;
use async_std::prelude::*;
use tide::{http::mime, Request, Response};
use uuid::Uuid;

pub async fn copy() -> tide::Result<()> {
    let k = load_stdin_to_dict()?;
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
            let resp = COPY_TEMPLATE.replace("{}", &html_escape::encode_text(s));
            Ok(Response::builder(200)
                .body(resp)
                .content_type(mime::HTML)
                .build())
        }
        None => Ok(tide::Response::new(404)),
    }
}
