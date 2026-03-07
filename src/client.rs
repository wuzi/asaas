use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderValue, USER_AGENT};
use reqwest::{Client as HttpClient, Method};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::environment::{Endpoints, Environment};
use crate::error::Error;

pub struct Client {
    pub(crate) api_key: String,
    pub(crate) user_agent: String,
    pub(crate) environment: Environment,
    pub(crate) http: HttpClient,
}

pub struct ClientBuilder {
    api_key: Option<String>,
    user_agent: Option<String>,
    environment: Environment,
    http: Option<HttpClient>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            api_key: None,
            user_agent: None,
            environment: Environment::Sandbox,
            http: None,
        }
    }
}

impl ClientBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn api_key(mut self, value: impl Into<String>) -> Self {
        self.api_key = Some(value.into());
        self
    }

    #[must_use]
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    #[must_use]
    pub const fn environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    #[must_use]
    pub fn http_client(mut self, http_client: HttpClient) -> Self {
        self.http = Some(http_client);
        self
    }

    pub fn build(self) -> Result<Client, Error> {
        let api_key = self.api_key.ok_or(Error::BuilderMissingField("api_key"))?;
        let user_agent = self
            .user_agent
            .ok_or(Error::BuilderMissingField("user_agent"))?;

        Ok(Client {
            api_key,
            user_agent,
            environment: self.environment,
            http: self.http.unwrap_or_default(),
        })
    }
}

impl Client {
    #[must_use]
    pub const fn endpoints(&self) -> Endpoints {
        self.environment.endpoints()
    }

    pub(crate) async fn send_typed<Req, Res>(
        &self,
        method: Method,
        path: &str,
        payload: Option<&Req>,
    ) -> Result<Res, Error>
    where
        Req: Serialize + Sync,
        Res: DeserializeOwned,
    {
        self.send_typed_with_accept(method, path, "application/json", payload)
            .await
    }

    pub(crate) async fn send_typed_with_accept<Req, Res>(
        &self,
        method: Method,
        path: &str,
        accept: &str,
        payload: Option<&Req>,
    ) -> Result<Res, Error>
    where
        Req: Serialize + Sync,
        Res: DeserializeOwned,
    {
        let url = format!("{}{path}", self.endpoints().api_base_url);

        let mut request = self
            .http
            .request(method, &url)
            .header(ACCEPT, accept)
            .header(USER_AGENT, &self.user_agent)
            .header("access_token", &self.api_key);

        if let Some(json_payload) = payload {
            request = request
                .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                .json(json_payload);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::RequestFailed { status, body });
        }

        let body = response.text().await?;
        if body.trim().is_empty() {
            return Err(Error::EmptyResponse);
        }

        Ok(serde_json::from_str(&body)?)
    }

    pub(crate) async fn send_bytes(
        &self,
        method: Method,
        path: &str,
        accept: &str,
    ) -> Result<Vec<u8>, Error> {
        let url = format!("{}{path}", self.endpoints().api_base_url);

        let response = self
            .http
            .request(method, &url)
            .header(ACCEPT, accept)
            .header(USER_AGENT, &self.user_agent)
            .header("access_token", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::RequestFailed { status, body });
        }

        let bytes = response.bytes().await?;
        if bytes.is_empty() {
            return Err(Error::EmptyResponse);
        }

        Ok(bytes.to_vec())
    }
}
