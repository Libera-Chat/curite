use std::sync::{Arc, RwLock};

use tera::{Context, Tera};
use warp::Reply;

use crate::config::Config;
use crate::error::Error;
use crate::xmlrpc::Xmlrpc;

pub(crate) struct VerifyContext {
    config: Arc<Config>,
    templates: Arc<RwLock<Tera>>,
}

impl VerifyContext {
    pub fn new(config: Arc<Config>, templates: Arc<RwLock<Tera>>) -> Self {
        Self { config, templates }
    }

    pub fn get(
        context: Arc<VerifyContext>,
        account: String,
        token: String,
    ) -> Result<impl Reply, Error> {
        let mut tera_context = Context::new();
        tera_context.insert("account", &account);
        tera_context.insert("token", &token);

        Ok(warp::reply::html(
            context.templates.read()?.render("verify", &tera_context)?,
        ))
    }

    pub async fn post(
        context: Arc<VerifyContext>,
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
