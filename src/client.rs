use anyhow::Result;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::ws_client::{WsClient, WsClientBuilder};
use std::time::Duration;

/// Common trait for Substrate RPC client operations
///
/// This trait provides the core functionality needed by various tools
/// to interact with Substrate nodes via JSON-RPC.
#[async_trait::async_trait]
pub trait SubstrateRpcClient: Send + Sync {
    /// Get the underlying WebSocket client
    fn client(&self) -> &WsClient;

    /// Connect to a Substrate node
    async fn connect(url: &str) -> Result<Self>
    where
        Self: Sized;

    /// Get block hash for a given block number
    async fn get_block_hash(&self, block_number: u32) -> Result<Option<String>> {
        Ok(self
            .client()
            .request("chain_getBlockHash", (block_number,))
            .await?)
    }

    /// Get storage value at a specific block
    async fn get_storage(&self, key: &str, at_block: Option<&str>) -> Result<Option<String>> {
        let params = match at_block {
            Some(hash) => (key, Some(hash)),
            None => (key, None::<&str>),
        };
        Ok(self.client().request("state_getStorage", params).await?)
    }

    /// Get storage keys with pagination
    async fn get_keys_paged(
        &self,
        prefix: Option<&str>,
        count: u32,
        start_key: Option<&str>,
        at_block: Option<&str>,
    ) -> Result<Vec<String>> {
        let params = (prefix, count, start_key, at_block);
        Ok(self.client().request("state_getKeysPaged", params).await?)
    }
}

/// Default implementation of SubstrateRpcClient
pub struct DefaultSubstrateClient {
    client: WsClient,
}

#[async_trait::async_trait]
impl SubstrateRpcClient for DefaultSubstrateClient {
    fn client(&self) -> &WsClient {
        &self.client
    }

    async fn connect(url: &str) -> Result<Self> {
        log::debug!("Connecting to Substrate node at {}", url);

        let client = WsClientBuilder::default()
            .connection_timeout(Duration::from_secs(10))
            .build(url)
            .await?;

        Ok(Self { client })
    }
}

/// Helper functions for working with hex data
pub mod hex_utils {
    use hex::FromHexError;

    /// Decode a hex string (with or without "0x" prefix) into bytes
    pub fn decode_hex(hex_str: &str) -> Result<Vec<u8>, FromHexError> {
        hex::decode(hex_str.trim_start_matches("0x"))
    }
}
