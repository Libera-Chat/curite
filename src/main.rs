mod config;
mod xmlrpc;

use std::collections::HashMap;
use std::fs::{remove_file, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::Arc;

use clap::Parser;
use closure::closure;
use handlebars::Handlebars;
use serde_yaml::from_reader;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use warp::reply::Reply;
use warp::Filter;

use self::config::Config;
use self::xmlrpc::Xmlrpc;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    config: PathBuf,
}

struct VerifyContext<'a> {
    config: Arc<Config>,
    hb: Handlebars<'a>,
}

impl<'a> VerifyContext<'a> {
    fn new(config: Arc<Config>, template: &PathBuf) -> Self {
        let mut hb = Handlebars::new();
        hb.register_template_file("template", template)
            .expect("couldn't register template");
        Self { config, hb }
    }

    fn get(context: Arc<VerifyContext>, account: String, token: String) -> impl Reply {
        let body = context
            .hb
            .render(
                "template",
                &HashMap::from([("account", account), ("token", token)]),
            )
            .unwrap_or_else(|_e| String::from("couldn't format template"));
        warp::reply::html(body)
    }

    async fn post<'b>(
        context: Arc<VerifyContext<'b>>,
        account: String,
        token: String,
    ) -> impl Reply {
        let xmlrpc = Xmlrpc::new(context.config.outbound.clone());
        let result = xmlrpc.verify(account, token).await;

        warp::redirect::see_other(
            match result {
                Ok(_) => &context.config.verify.outcomes.success,
                Err(e) => {
                    println!("{:?}", e);
                    &context.config.verify.outcomes.failure
                }
            }
            .clone(),
        )
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

    let verify_context = Arc::new(VerifyContext::new(
        Arc::clone(&config),
        &config.verify.template,
    ));

    let get_verify = warp::get()
        // look at this god foresaken appeasement of rustc
        .and(warp::any().map(closure!(clone verify_context, || Arc::clone(&verify_context))))
        .and(warp::path!("verify" / String / String))
        .map(VerifyContext::get);

    let post_verify = warp::post()
        .and(warp::any().map(closure!(clone verify_context, || Arc::clone(&verify_context))))
        .and(warp::path!("verify" / String / String))
        .then(VerifyContext::post);

    let listener = UnixListener::bind(&config.inbound).expect("failed to bind unix domain socket");
    let incoming = UnixListenerStream::new(listener);
    warp::serve(get_verify.or(post_verify))
        .run_incoming(incoming)
        .await;
}
