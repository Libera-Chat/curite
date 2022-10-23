mod config;
mod contexts;
mod error;
mod xmlrpc;

use std::fs::{remove_file, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, RwLock};

use clap::Parser;
use closure::closure;
use http::response::Response;
use http::StatusCode;
use hyper::body::Body;
use serde_yaml::from_reader;
use tera::Tera;
use tokio::net::UnixListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio_stream::wrappers::UnixListenerStream;
use warp::{Filter, Reply};

use self::config::Config;
use self::contexts::verify::VerifyContext;
use self::error::Error;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    config: PathBuf,
}

fn display<T: Reply>(res: Result<T, Error>) -> Response<Body> {
    match res {
        Ok(res) => res.into_response(),
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
        Ok(data) => data,
        Err(e) => {
            eprintln!("can't read config: {:?}", e);
            exit(1);
        }
    };

    let reader = BufReader::new(file);
    let config: Arc<Config> = Arc::new(match from_reader(reader) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("can't load config: {:?}", e);
            exit(2);
        }
    });

    if Path::exists(&config.inbound) && remove_file(&config.inbound).is_err() {
        eprintln!("can't remove old unix socket");
        exit(3);
    }

    let templates = Arc::new(RwLock::new(Tera::new(&config.templates).unwrap()));

    let verify_context = Arc::new(VerifyContext::new(
        Arc::clone(&config),
        Arc::clone(&templates),
    ));

    let verify_get = warp::get()
        // look at this god foresaken appeasement of rustc
        .and(warp::any().map(closure!(clone verify_context, || Arc::clone(&verify_context))))
        .and(warp::path!("verify" / String / String))
        .map(VerifyContext::get)
        .map(display);

    let verify_post = warp::post()
        .and(warp::any().map(closure!(clone verify_context, || Arc::clone(&verify_context))))
        .and(warp::path!("verify" / String / String))
        .then(VerifyContext::post)
        .map(display);

    let listener = UnixListener::bind(&config.inbound).expect("failed to bind unix domain socket");
    let incoming = UnixListenerStream::new(listener);

    tokio::try_join!(
        async {
            Result::<(), Error>::Ok(
                warp::serve(verify_get.or(verify_post))
                    .run_incoming(incoming)
                    .await,
            )
        },
        sighup(Arc::clone(&templates)),
    )
    .unwrap();
}
