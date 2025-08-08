use anyhow::Result;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::traits::ToRpcParams;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use serde::de::DeserializeOwned;

/// Calculate actual block number from a relative or absolute block identifier
///
/// # Arguments
/// * `block_identifier` - Negative for relative to current, 0 for current, positive for absolute
/// * `current_block` - The current block number
///
/// # Returns
/// The calculated absolute block number
pub fn calculate_block_number(block_identifier: i32, current_block: u32) -> u32 {
    if block_identifier < 0 {
        (current_block as i32 + block_identifier) as u32
    } else if block_identifier == 0 {
        current_block
    } else {
        block_identifier as u32
    }
}

/// RPC client that can be either HTTP or WebSocket
pub enum RpcClient {
    Http(Box<HttpClient>),
    Ws(WsClient),
}

impl RpcClient {
    /// Create an RPC client based on the URL scheme
    ///
    /// # Arguments
    /// * `rpc_url` - The RPC URL (http://, https://, ws://, or wss://)
    ///
    /// # Returns
    /// An RPC client appropriate for the URL scheme
    pub async fn new(rpc_url: &str) -> Result<Self> {
        if rpc_url.starts_with("ws://") || rpc_url.starts_with("wss://") {
            let client = WsClientBuilder::default().build(rpc_url).await?;
            Ok(RpcClient::Ws(client))
        } else if rpc_url.starts_with("http://") || rpc_url.starts_with("https://") {
            let client = HttpClientBuilder::default().build(rpc_url)?;
            Ok(RpcClient::Http(Box::new(client)))
        } else {
            Err(anyhow::anyhow!("Unsupported RPC URL scheme: {}", rpc_url))
        }
    }

    /// Make an RPC request
    pub async fn request<R, Params>(&self, method: &str, params: Params) -> Result<R>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        match self {
            RpcClient::Http(client) => client.request(method, params).await.map_err(Into::into),
            RpcClient::Ws(client) => client.request(method, params).await.map_err(Into::into),
        }
    }
}
