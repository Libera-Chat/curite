use http::uri::Uri;
use serde::Deserialize;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct VerifyOutcomes {
    #[serde(with = "http_serde::uri")]
    pub success: Uri,
    #[serde(with = "http_serde::uri")]
    pub failure: Uri,
}

#[derive(Debug, Deserialize)]
pub struct Verify {
    pub template: String,
    pub outcomes: VerifyOutcomes,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub inbound: PathBuf,
    pub outbound: Url,
    pub verify: Verify,
}
