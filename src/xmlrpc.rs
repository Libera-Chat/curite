use dxr::client::{Call, Client, ClientBuilder};
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

    fn request<'a>(
        service: &'static str,
        command: &'static str,
        params: Vec<&'a str>,
    ) -> Call<'a, Vec<&'a str>, String> {
        Call::new(
            "atheme.command",
            [vec!["", "", "127.0.0.1", service, command], params].concat(),
        )
    }

    pub async fn verify(&self, account: &str, token: &str) -> Result<(), String> {
        let request = Xmlrpc::request("NickServ", "VERIFY", vec!["REGISTER", account, token]);
        self.client
            .call(request)
            .await
            .map(|_| ())
            .map_err(|e| format!("{e:?}"))
    }
}
