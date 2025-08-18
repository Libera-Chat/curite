use http::uri::Uri;
use regex::Regex;
use serde::Deserialize;
use std::path::PathBuf;
use url::Url;

#[derive(Clone, Debug, Deserialize)]
pub struct Verify {
    pub target: String,
    #[serde(with = "http_serde::uri")]
    pub success: Uri,
    #[serde(with = "http_serde::uri")]
    pub nochange: Uri,
    #[serde(with = "http_serde::uri")]
    pub failure: Uri,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Validation {
    #[serde(with = "serde_regex")]
    pub account: Regex,
    #[serde(with = "serde_regex")]
    pub token: Regex,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub listen: PathBuf,
    pub xmlrpc: Url,
    pub verify: Verify,
    pub validation: Validation,
    pub templates: String,
}
