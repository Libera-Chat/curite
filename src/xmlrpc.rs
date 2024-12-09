use dxr_client::{Client, ClientBuilder, ClientError};
use url::Url;

pub struct Xmlrpc {
    client: Client,
}
impl Xmlrpc {
    pub fn new(url: Url) -> Self {
        Self {
            client: ClientBuilder::new(url).user_agent("curite").build(),
        }
    }

    async fn request<'a>(
        &self,
        service: &'static str,
        command: &'static str,
        params: Vec<&'a str>,
    ) -> Result<(), ClientError> {
        self.client
            .call(
                "atheme.command",
                [vec!["", "", "127.0.0.1", service, command], params].concat(),
            )
            .await
    }

    pub async fn verify(&self, account: &str, token: &str) -> Result<(), String> {
        self.request("NickServ", "VERIFY", vec!["REGISTER", account, token])
            .await
            .map_err(|e| format!("{e:?}"))
    }
}
