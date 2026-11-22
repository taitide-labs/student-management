use crate::contract::{GatewayError, GatewayFuture};

pub trait RemoteTransport {
    fn send<'a>(
        &'a self,
        method: &'a str,
        url: &'a str,
        bearer_token: Option<&'a str>,
        body: Option<Vec<u8>>,
    ) -> GatewayFuture<'a, Result<(u16, Vec<u8>), GatewayError>>;
}

#[cfg(all(feature = "remote", not(target_arch = "wasm32")))]
#[derive(Clone)]
pub struct UreqTransport {
    agent: ureq::Agent,
}

#[cfg(all(feature = "remote", not(target_arch = "wasm32")))]
impl Default for UreqTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(feature = "remote", not(target_arch = "wasm32")))]
impl UreqTransport {
    #[must_use]
    pub fn new() -> Self {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .new_agent();
        Self { agent }
    }

    fn call_blocking(
        &self,
        method: &str,
        url: &str,
        bearer_token: Option<&str>,
        body: Option<&[u8]>,
    ) -> Result<(u16, Vec<u8>), GatewayError> {
        let auth = bearer_token.map(|tok| format!("Bearer {tok}"));
        let payload = body.unwrap_or(&[]);
        let mut resp = match method {
            "GET" => {
                let mut req = self.agent.get(url);
                if let Some(h) = auth.as_deref() {
                    req = req.header("authorization", h);
                }
                req.call()
            }
            "POST" => {
                let mut req = self
                    .agent
                    .post(url)
                    .header("content-type", "application/json");
                if let Some(h) = auth.as_deref() {
                    req = req.header("authorization", h);
                }
                req.send(payload)
            }
            "PUT" => {
                let mut req = self
                    .agent
                    .put(url)
                    .header("content-type", "application/json");
                if let Some(h) = auth.as_deref() {
                    req = req.header("authorization", h);
                }
                req.send(payload)
            }
            "PATCH" => {
                let mut req = self
                    .agent
                    .patch(url)
                    .header("content-type", "application/json");
                if let Some(h) = auth.as_deref() {
                    req = req.header("authorization", h);
                }
                req.send(payload)
            }
            _ => {
                return Err(GatewayError::Transport(format!(
                    "不支持的 HTTP 方法：{method}"
                )));
            }
        }
        .map_err(|err| GatewayError::Transport(err.to_string()))?;

        let status = resp.status().as_u16();
        let raw = resp
            .body_mut()
            .read_to_vec()
            .map_err(|err| GatewayError::Transport(err.to_string()))?;
        Ok((status, raw))
    }
}

#[cfg(all(feature = "remote", not(target_arch = "wasm32")))]
impl RemoteTransport for UreqTransport {
    fn send<'a>(
        &'a self,
        method: &'a str,
        url: &'a str,
        bearer_token: Option<&'a str>,
        body: Option<Vec<u8>>,
    ) -> GatewayFuture<'a, Result<(u16, Vec<u8>), GatewayError>> {
        let result = self.call_blocking(method, url, bearer_token, body.as_deref());
        Box::pin(std::future::ready(result))
    }
}

#[cfg(feature = "nsh")]
#[derive(Clone, Default)]
pub struct NshTransport;

#[cfg(feature = "nsh")]
impl RemoteTransport for NshTransport {
    fn send<'a>(
        &'a self,
        method: &'a str,
        url: &'a str,
        bearer_token: Option<&'a str>,
        body: Option<Vec<u8>>,
    ) -> GatewayFuture<'a, Result<(u16, Vec<u8>), GatewayError>> {
        let mut req = nsh_core::platform::HttpRequest::new(method, url);
        if let Some(tok) = bearer_token {
            req = req.header("authorization", format!("Bearer {tok}"));
        }
        if let Some(bytes) = body {
            req = req.header("content-type", "application/json").body(bytes);
        }
        Box::pin(async move {
            let resp = nsh_core::prelude::fetch(req)
                .await
                .map_err(GatewayError::Transport)?;
            Ok((resp.status(), resp.body))
        })
    }
}
