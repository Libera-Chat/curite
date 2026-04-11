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

    async fn request(
        &self,
        ip: &str,
        service: &str,
        command: &str,
        params: Vec<&str>,
    ) -> Result<String, ClientError> {
        self.client
            .call(
                "atheme.command",
                [vec!["", "", ip, service, command], params].concat(),
            )
            .await
    }

    pub async fn verify(&self, ip: &str, account: &str, token: &str) -> Result<String, ClientError> {
        self.request(ip, "NickServ", "VERIFY", vec!["REGISTER", account, token])
            .await
    }
}
