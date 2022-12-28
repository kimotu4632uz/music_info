use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use oauth2::{
    basic::{BasicClient, BasicTokenType},
    ureq::http_client,
    {
        AuthUrl, ClientId, ClientSecret, EmptyExtraTokenFields, StandardTokenResponse,
        TokenResponse, TokenUrl,
    },
};

use serde::{Deserialize, Serialize};

pub type Token = StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>;

fn get_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("your system or rust std::time is broken")
}

fn token_expired(token: &Token) -> bool {
    token
        .expires_in()
        .map(|d| get_timestamp() > d)
        .unwrap_or(false)
}

fn tt2str(tt: &BasicTokenType) -> String {
    match tt {
        BasicTokenType::Bearer => "Bearer".into(),
        BasicTokenType::Mac => "Mac".into(),
        BasicTokenType::Extension(s) => s.into(),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientCredential {
    pub client_id: String,
    pub client_secret: String,
}

impl ClientCredential {
    pub fn perform_auth<P: AsRef<Path>>(
        &self,
        auth_url: &str,
        token_store: P,
    ) -> anyhow::Result<Token> {
        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            Some(ClientSecret::new(self.client_secret.clone())),
            AuthUrl::new(auth_url.to_string()).unwrap(),
            Some(TokenUrl::new(auth_url.to_string()).unwrap()),
        );
        let mut token_resp = client.exchange_client_credentials().request(http_client)?;
        let expires_in = token_resp.expires_in().map(|d| get_timestamp() + d);
        token_resp.set_expires_in(expires_in.as_ref());

        std::fs::write(
            token_store,
            serde_json::to_string_pretty(&token_resp).unwrap(),
        )?;

        Ok(token_resp)
    }
}

pub struct Client {
    cred: ClientCredential,
    token: RefCell<Token>,
    auth_url: String,
    token_store: PathBuf,
}

impl Client {
    pub fn new<P: AsRef<Path>>(
        client_cred: P,
        access_token: P,
        auth_url: String,
    ) -> anyhow::Result<Client> {
        let client_cred_path = client_cred.as_ref();
        let access_token_path = access_token.as_ref();

        if !client_cred_path.exists() {
            anyhow::bail!("client credential file not exists.")
        }
        let client_cred_str = std::fs::read_to_string(client_cred_path)?;
        let cred: ClientCredential = serde_json::from_str(&client_cred_str)?;

        let token = if !access_token_path.exists() {
            cred.perform_auth(auth_url.as_str(), access_token_path)?
        } else {
            let token_str = std::fs::read_to_string(access_token_path)?;
            let token: Token = serde_json::from_str(&token_str)?;

            if token_expired(&token) {
                cred.perform_auth(auth_url.as_str(), access_token_path)?
            } else {
                token
            }
        };

        Ok(Client {
            cred,
            token: RefCell::new(token),
            auth_url,
            token_store: access_token_path.into(),
        })
    }

    pub fn get(&self, url: &str) -> anyhow::Result<ureq::Request> {
        let token_need_update = {
            let token = self.token.borrow();
            token_expired(&token)
        };
        if token_need_update {
            let token = self.cred.perform_auth(&self.auth_url, &self.token_store)?;
            self.token.replace(token);
        }
        let token = self.token.borrow();
        let auth = format!(
            "{} {}",
            tt2str(token.token_type()),
            token.access_token().secret()
        );
        Ok(ureq::get(url).set("Authorization", &auth))
    }
}
