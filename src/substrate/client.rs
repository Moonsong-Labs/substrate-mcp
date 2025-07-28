use anyhow::{anyhow, Result};
use hex::FromHexError;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Represents a single storage change at a specific block.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageChange {
    pub value: String,
    pub block_number: u32,
}

/// The primary, public-facing client for interacting with a Substrate node.
pub struct SubstrateClient {
    client: HttpClient,
}

impl SubstrateClient {
    /// Get block hash for a given block number
    async fn get_block_hash(&self, block_number: u32) -> Result<Option<String>> {
        Ok(self
            .client
            .request("chain_getBlockHash", (block_number,))
            .await?)
    }

    /// Get storage value at a specific block
    async fn get_storage(&self, key: &str, at_block: Option<&str>) -> Result<Option<String>> {
        let params = match at_block {
            Some(hash) => (key, Some(hash)),
            None => (key, None::<&str>),
        };
        Ok(self.client.request("state_getStorage", params).await?)
    }

    /// Get storage value for a key at a specific block
    async fn get_storage_value(&self, key: &str, block_hash: &str) -> Result<Option<Vec<u8>>> {
        let result = self.get_storage(key, Some(block_hash)).await?;

        match result {
            Some(hex_value) => Ok(Some(self.decode_hex(&hex_value)?)),
            None => Ok(None),
        }
    }

    /// Decode a hex string (with or without "0x" prefix) into bytes
    fn decode_hex(&self, hex_str: &str) -> Result<Vec<u8>, FromHexError> {
        hex::decode(hex_str.trim_start_matches("0x"))
    }

    /// Connect to a Substrate node
    pub async fn connect(url: &str) -> Result<Self> {
        log::debug!("Connecting to Substrate node at {url}");

        let client = HttpClientBuilder::default()
            .request_timeout(Duration::from_secs(10))
            .build(url)?;

        Ok(Self { client })
    }

    /// Find all storage changes between two blocks for a single key
    pub async fn find_all_storage_changes(
        &self,
        start_block: u32,
        end_block: u32,
        key: String,
    ) -> Result<Vec<StorageChange>> {
        let mut changes = Vec::new();

        log::debug!(
            "Finding all changes for key {key} between blocks {start_block} and {end_block}..."
        );

        // Get the initial value at start_block
        let start_hash = self
            .get_block_hash(start_block)
            .await?
            .ok_or_else(|| anyhow!("Block {start_block} not found"))?;

        let mut previous_value = self.get_storage_value(&key, &start_hash).await?;

        // Iterate through all blocks in the range
        for block_num in (start_block + 1)..=end_block {
            let block_hash = self
                .get_block_hash(block_num)
                .await?
                .ok_or_else(|| anyhow!("Block {block_num} not found"))?;

            let current_value = self.get_storage_value(&key, &block_hash).await?;

            // Check if the value changed
            if current_value != previous_value {
                log::debug!("Found storage change at block {block_num}");

                let value = format!(
                    "0x{}",
                    hex::encode(current_value.clone().unwrap_or_default())
                );

                changes.push(StorageChange {
                    value,
                    block_number: block_num,
                });

                previous_value = current_value;
            }
        }

        Ok(changes)
    }
}
