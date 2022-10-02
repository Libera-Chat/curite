mod config;
mod xmlrpc;

use std::collections::HashMap;
use std::fs::{remove_file, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;

use clap::Parser;
use handlebars::Handlebars;
use serde_yaml::from_reader;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use warp::Filter;

use self::config::Config;
use self::xmlrpc::Xmlrpc;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    config: PathBuf,
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

    let mut hb = Handlebars::new();

    hb.register_template_file("get_verify", &config.verify.template)
        .expect("couldn't register template");

    let get_verify = warp::get().and(warp::path!("verify" / String / String).map(
        move |account, token| {
            let body = hb.render(
                "get_verify",
                &HashMap::from([("account", account), ("token", token)]),
            )
            .unwrap_or_else(|_e| String::from("couldn't format template"));
            warp::reply::html(body)
        },
    ));

    let config_verify = config.clone();
    let post_verify = warp::post().and(warp::path!("verify" / String / String).then(
        move |account, token| {
            let config = config_verify.clone();
            async move {
                let xmlrpc = Xmlrpc::new(config.outbound.clone());
                let result = xmlrpc.verify(account, token).await;

                warp::redirect::see_other(
                    match result {
                        Ok(_) => &config.verify.outcomes.success,
                        Err(_) => &config.verify.outcomes.failure,
                    }
                    .clone(),
                )
            }
        },
    ));

    let listener = UnixListener::bind(&config.inbound).expect("failed to bind unix domain socket");
    let incoming = UnixListenerStream::new(listener);
    warp::serve(get_verify.or(post_verify))
        .run_incoming(incoming)
        .await;
}
