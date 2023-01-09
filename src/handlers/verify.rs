use std::sync::{Arc, RwLock};

use tera::{Context, Tera};

use super::Handled;
use crate::config::Config;
use crate::error::Error;
use crate::xmlrpc::Xmlrpc;

pub struct Get {
    config: Config,
    templates: Arc<RwLock<Tera>>,
}

impl Get {
    pub fn new(config: Config, templates: Arc<RwLock<Tera>>) -> Self {
        Self { config, templates }
    }

    pub fn handle(&self, account: &str, token: &str) -> Result<Handled, Error> {
        let mut tera_context = Context::new();
        tera_context.insert("account", &account);
        tera_context.insert("token", &token);
        let target = self
            .templates
            .write()?
            .render_str(&self.config.verify.target, &tera_context)?;
        tera_context.insert("target", &target);

        Ok(Handled::Html(
            self.templates
                .read()?
                .render("verify.html", &tera_context)?,
        ))
    }
}

pub struct Post {
    config: Config,
}

impl Post {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn handle(&self, account: &str, token: &str) -> Result<Handled, Error> {
        self.config
            .validation
            .account
            .find(account)
            .ok_or(Error::BadArgument("account"))?;
        self.config
            .validation
            .token
            .find(token)
            .ok_or(Error::BadArgument("token"))?;

        let xmlrpc = Xmlrpc::new(self.config.xmlrpc.clone());
        let result = xmlrpc.verify(account, token).await;

        Ok(Handled::Redirect(
            match result {
                Ok(_) => &self.config.verify.success,
                Err(e) => {
                    println!("{e:?}");
                    &self.config.verify.failure
                }
            }
            .clone(),
        ))
    }
}
