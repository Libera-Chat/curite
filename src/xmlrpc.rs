use dxr_client::{Client, ClientBuilder, ClientError};
use url::Url;

pub fn is_noverify(error: &ClientError) -> bool {
    // Magic numbers: 4 is fault_nosuch_target, 12 is fault_nochange
    match error {
        ClientError::Fault { fault } => fault.code() == 4 || fault.code() == 12,
        _ => false,
    }
}

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
        service: &str,
        command: &str,
        params: Vec<&str>,
    ) -> Result<String, ClientError> {
        self.client
            .call(
                "atheme.command",
                [vec!["", "", "127.0.0.1", service, command], params].concat(),
            )
            .await
    }

    pub async fn verify(&self, account: &str, token: &str) -> Result<String, ClientError> {
        self.request("NickServ", "VERIFY", vec!["REGISTER", account, token])
            .await
    }
}
