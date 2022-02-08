use clap::Parser;
use handlebars::Handlebars;
use serde_json::json;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use warp::Filter;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    sock: String,
    path: String,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    let mut hb = Handlebars::new();

    hb.register_template_file("page", opts.path)
        .expect("couldn't register template");

    let page = warp::path!(String / String).map(move |account, token| {
        hb.render("page", &json!({"account": account, "token": token}))
            .expect("couldn't format template")
    });

    let listener = UnixListener::bind(opts.sock)
        .expect("failed to bind unix domain socket");
    let incoming = UnixListenerStream::new(listener);
    warp::serve(page).run_incoming(incoming).await;
}
