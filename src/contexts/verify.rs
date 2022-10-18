use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use askama::Template;
use tinytemplate::TinyTemplate;
use warp::Reply;

use crate::config::Config;
use crate::error::Error;
use crate::xmlrpc::Xmlrpc;

pub(crate) struct VerifyContext<'a> {
    config: Arc<Config>,
    templates: Arc<RwLock<TinyTemplate<'a>>>,
}

#[derive(Template)]
#[template(path = "verify.html")]
struct VerifyHtml {
    account: String,
    token: String,
}

impl<'a> VerifyContext<'a> {
    pub fn new(config: Arc<Config>, templates: Arc<RwLock<TinyTemplate<'a>>>) -> Self {
        Self { config, templates }
    }

    pub fn get(
        context: Arc<VerifyContext<'a>>,
        account: String,
        token: String,
    ) -> Result<impl Reply, Error> {
        Ok(warp::reply::html(context.templates.read()?.render(
            "verify",
            &HashMap::from([("account", account), ("token", token)]),
        )?))
    }

    pub async fn post(
        context: Arc<VerifyContext<'a>>,
        account: String,
        token: String,
    ) -> Result<impl Reply, Error> {
        context
            .config
            .validation
            .account
            .find(&account)
            .ok_or(Error::BadArgument("account"))?;
        context
            .config
            .validation
            .token
            .find(&token)
            .ok_or(Error::BadArgument("token"))?;

        let xmlrpc = Xmlrpc::new(context.config.outbound.clone());
        let result = xmlrpc.verify(account, token).await;

        Ok(warp::redirect::see_other(
            match result {
                Ok(_) => &context.config.verify.success,
                Err(e) => {
                    println!("{:?}", e);
                    &context.config.verify.failure
                }
            }
            .clone(),
        ))
    }
}
