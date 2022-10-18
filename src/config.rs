use http::uri::Uri;
use regex::Regex;
use serde::Deserialize;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct Verify {
    #[serde(with = "http_serde::uri")]
    pub success: Uri,
    #[serde(with = "http_serde::uri")]
    pub failure: Uri,
}

#[derive(Debug, Deserialize)]
pub struct Validation {
    #[serde(with = "serde_regex")]
    pub account: Regex,
    #[serde(with = "serde_regex")]
    pub token: Regex,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub inbound: PathBuf,
    pub outbound: Url,
    pub verify: Verify,
    pub validation: Validation,
}
