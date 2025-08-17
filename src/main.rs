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
mod handlers;
mod router;
mod xmlrpc;

use std::fs::{remove_file, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, RwLock};

use clap::Parser;
use serde_yaml::from_reader;
use tera::Tera;
use tokio::net::UnixListener;
use tokio::signal::unix::{signal, SignalKind};

use self::config::Config;
use self::error::Error;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    config: PathBuf,
}

async fn sighup(templates: Arc<RwLock<Tera>>) -> Result<(), Error> {
    let mut stream = signal(SignalKind::hangup())?;
    loop {
        stream.recv().await;
        templates.write()?.full_reload()?;
    }
}

type Templates = Arc<RwLock<Tera>>;

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    let file = match File::open(opts.config) {
        Ok(reader) => reader,
        Err(e) => {
            eprintln!("can't read config: {e:?}");
            exit(1);
        }
    };

    let config = match from_reader::<_, Config>(BufReader::new(file)) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("can't load config: {e:?}");
            exit(2);
        }
    };

    if Path::exists(&config.listen) && remove_file(&config.listen).is_err() {
        eprintln!("can't remove old unix socket");
        exit(3);
    }

    let listener = UnixListener::bind(&config.listen).unwrap_or_else(|e| {
        eprintln!("failed to bind unix domain socket: {e}");
        exit(4);
    });

    let templates = Tera::new(&config.templates).unwrap_or_else(|e| {
        eprintln!("cannot load templates: {e}");
        exit(5);
    });
    let templates = Arc::new(RwLock::new(templates));

    let router = router::init(config, Arc::clone(&templates));

    let result = tokio::try_join!(
        async { axum::serve(listener, router).await.map_err(Error::Std) },
        sighup(templates),
    );
    if let Err(e) = result {
        eprintln!("errored during runtime: {e:?}");
        exit(6);
    }
}
