use clap::Parser;
use handlebars::Handlebars;
use serde_json::json;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use warp::Filter;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    socket: String,
    template: String,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    let mut hb = Handlebars::new();

    hb.register_template_file("page", opts.template)
        .expect("couldn't register template");

    let page = warp::path!(String / String).map(move |account, token| {
        hb.render("page", &json!({"account": account, "token": token}))
            .unwrap_or_else(|_e| String::from("couldn't format template"))
    });

    let listener = UnixListener::bind(opts.socket)
        .expect("failed to bind unix domain socket");
    let incoming = UnixListenerStream::new(listener);
    warp::serve(page).run_incoming(incoming).await;
}
