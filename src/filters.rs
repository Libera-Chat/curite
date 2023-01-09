use std::sync::{Arc, RwLock};

use tera::Tera;
use warp::Filter;

use crate::config::Config;
use crate::error::Error;
use crate::handlers::Handled;

pub fn verify_get(
    config: Config,
    templates: Arc<RwLock<Tera>>,
) -> impl Filter<Extract = (Result<Handled, Error>,), Error = warp::Rejection> + Clone {
    let context = Arc::new(crate::handlers::verify::Get::new(config, templates));
    warp::get()
        .and(warp::path("verify"))
        .and(warp::path::param())
        .and(warp::path::param())
        .map(move |account: String, token: String| {
            let context = Arc::clone(&context);
            context.handle(&account, &token)
        })
}

pub fn verify_post(
    config: Config,
) -> impl Filter<Extract = (Result<Handled, Error>,), Error = warp::Rejection> + Clone {
    let context = Arc::new(crate::handlers::verify::Post::new(config));
    warp::post()
        .and(warp::path("verify"))
        .and(warp::path::param())
        .and(warp::path::param())
        .then(move |account: String, token: String| {
            let c = Arc::clone(&context);
            async move { c.handle(&account, &token).await }
        })
}
