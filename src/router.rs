use std::sync::Arc;

use axum::{
    body::Body,
    extract::Path,
    response::{Html, IntoResponse, Redirect, Response},
};
use http::StatusCode;

use crate::{
    config::Config,
    error::Error,
    handlers::{
        verify::{Get, Post},
        Handled,
    },
};

pub fn init(config: Config, templates: crate::Templates) -> axum::Router {
    let handler_get = Arc::new(Get::new(config.clone(), templates));
    let handler_post = Arc::new(Post::new(config));
    axum::Router::new()
        .route(
            "/verify/{account}/{token}",
            axum::routing::get(
                async move |Path((account, token)): Path<(String, String)>| {
                    display(handler_get.handle(&account, &token))
                },
            ),
        )
        .route(
            "/verify/{account}/{token}",
            axum::routing::post(
                async move |Path((account, token)): Path<(String, String)>| {
                    display(handler_post.handle(&account, &token).await)
                },
            ),
        )
}

fn display(res: Result<Handled, Error>) -> Response {
    match res {
        Ok(res) => match res {
            Handled::Html(body) => Html(Body::from(body)).into_response(),
            Handled::Redirect(uri) => Redirect::to(uri.path()).into_response(),
        },
        Err(e) => match e {
            Error::BadTemplate(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("bad template: {e:?}"),
            )
                .into_response(),
            Error::Lock => (StatusCode::INTERNAL_SERVER_ERROR, "lock failure").into_response(),
            Error::BadArgument(name) => {
                (StatusCode::BAD_REQUEST, format!("bad argument: {name}")).into_response()
            }
            Error::Std(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("unknown error: {e:?}"),
            )
                .into_response(),
        },
    }
}
