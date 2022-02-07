use clap::Parser;
use handlebars::Handlebars;
use serde_json::json;
use warp::Filter;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    port: u16,
    path: String,
}

#[tokio::main]
async fn main() {
    let args = Opts::parse();

    let mut hb = Handlebars::new();

    hb.register_template_file("page", args.path)
        .expect("couldn't register template");

    let page = warp::path!(String / String).map(move |account, token| {
        hb.render("page", &json!({"account": account, "token": token}))
            .expect("couldn't format template")
    });

    warp::serve(page).run(([0, 0, 0, 0], args.port)).await;
}
