use std::sync::Arc;

use askama::Template;
use warp::Reply;

use crate::config::Config;
use crate::error::Error;
use crate::xmlrpc::Xmlrpc;

pub(crate) struct VerifyContext {
    config: Arc<Config>,
}

#[derive(Template)]
#[template(path = "verify.html")]
struct VerifyHtml {
    account: String,
    token: String,
}

impl VerifyContext {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub fn get(
        context: Arc<VerifyContext>,
        account: String,
        token: String,
    ) -> Result<impl Reply, Error> {
        Ok(warp::reply::html(VerifyHtml { account, token }.render()?))
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
                Ok(_) => &context.config.verify.outcomes.success,
                Err(e) => {
                    println!("{:?}", e);
                    &context.config.verify.outcomes.failure
                }
            }
            .clone(),
        ))
    }
}
