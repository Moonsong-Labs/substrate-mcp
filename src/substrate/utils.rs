use anyhow::{anyhow, Result};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::traits::ToRpcParams;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use serde::de::DeserializeOwned;
use subxt::OnlineClient;
use subxt::PolkadotConfig;

/// Converts relative or absolute block numbers into an absolute block range.
///
/// # Arguments
///
/// * `from_block` - Starting block number. Negative values are relative to the current block (e.g., -10 means 10 blocks ago). 0 means the current block.
/// * `to_block` - Optional ending block number. Negative values are relative to the current block. If `None`, defaults to `from_block` (single block range).
/// * `subxt_client` - Reference to the Substrate client for fetching the current block.
///
/// # Returns
///
/// A tuple `(from, to)` containing the absolute block numbers for the range.
pub async fn get_block_range(
    from_block: i32,
    to_block: Option<i32>,
    subxt_client: &OnlineClient<PolkadotConfig>,
) -> Result<(u32, u32)> {
    let (from, to);
    if from_block <= 0 {
        // Get current block number
        let latest_block = subxt_client.blocks().at_latest().await?;
        let current_block = latest_block.header().number;

        // Calculate actual block range
        from = if from_block < 0 {
            (current_block as i32 + from_block) as u32
        } else {
            current_block
        };

        to = match to_block {
            Some(b) => {
                if b < 0 {
                    (current_block as i32 + b) as u32
                } else {
                    current_block
                }
            }
            None => from, // Default to single block if not specified
        };
    } else {
        from = from_block as u32;
        to = match to_block {
            Some(b) => b as u32,
            None => from,
        };
    }

    if to - from > 100 {
        return Err(anyhow!(
            "Maximum block range is 100. Requested range: {}",
            to - from
        ));
    }

    Ok((from, to))
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
