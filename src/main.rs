#![deny(clippy::pedantic)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::debug_assert_with_mut_call)]
#![deny(clippy::equatable_if_let)]
#![deny(clippy::if_then_some_else_none)]
#![deny(clippy::same_name_method)]
#![deny(clippy::try_err)]
#![deny(clippy::undocumented_unsafe_blocks)]

mod config;
mod error;
mod filters;
mod handlers;
mod xmlrpc;

use std::fs::{remove_file, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, RwLock};

use clap::Parser;
use http::response::Response;
use http::StatusCode;
use hyper::body::Body;
use serde_yaml::from_reader;
use tera::Tera;
use tokio::net::UnixListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio_stream::wrappers::UnixListenerStream;
use warp::{Filter as _, Reply};

use self::config::Config;
use self::error::Error;
use self::handlers::Handled;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    config: PathBuf,
}

fn display(res: Result<Handled, Error>) -> Response<Body> {
    match res {
        Ok(res) => match res {
            Handled::Html(body) => warp::reply::html(body).into_response(),
            Handled::Redirect(uri) => warp::redirect::see_other(uri).into_response(),
        },
        Err(e) => match e {
            Error::BadTemplate(e) => warp::reply::with_status(
                format!("bad template: {:?}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response(),
            Error::Lock => {
                warp::reply::with_status("lock failure", StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
            Error::BadArgument(name) => {
                warp::reply::with_status(format!("bad argument: {}", name), StatusCode::BAD_REQUEST)
                    .into_response()
            }
            Error::Std(e) => warp::reply::with_status(
                format!("unknown error: {:?}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response(),
        },
    }
}

async fn sighup(templates: Arc<RwLock<Tera>>) -> Result<(), Error> {
    let mut stream = signal(SignalKind::hangup())?;
    loop {
        stream.recv().await;
        templates.write()?.full_reload()?;
    }
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    let file = match File::open(opts.config) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("can't read config: {:?}", e);
            exit(1);
        }
    };

    let config = match from_reader::<_, Config>(BufReader::new(file)) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("can't load config: {:?}", e);
            exit(2);
        }
    };

    if Path::exists(&config.listen) && remove_file(&config.listen).is_err() {
        eprintln!("can't remove old unix socket");
        exit(3);
    }

    let listener = UnixListener::bind(&config.listen).expect("failed to bind unix domain socket");
    let incoming = UnixListenerStream::new(listener);

    let templates = Arc::new(RwLock::new(Tera::new(&config.templates).unwrap()));

    let verify_get = self::filters::verify_get(config.clone(), Arc::clone(&templates)).map(display);
    let verify_post = self::filters::verify_post(config.clone()).map(display);

    tokio::try_join!(
        async {
            warp::serve(verify_get.or(verify_post))
                .run_incoming(incoming)
                .await;
            Result::<(), Error>::Ok(())
        },
        sighup(Arc::clone(&templates)),
    )
    .unwrap();
}
