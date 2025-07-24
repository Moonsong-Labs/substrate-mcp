use crate::client::{hex_utils, DefaultSubstrateClient, SubstrateRpcClient};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Represents a single storage change at a specific block.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageChange {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub block_number: u32,
}

/// Storage-specific client that extends the base Substrate client functionality
pub struct StorageBisectClient<T: SubstrateRpcClient> {
    inner: T,
}

impl StorageBisectClient<DefaultSubstrateClient> {
    /// Create a new storage client with the default implementation
    pub async fn new(url: &str) -> Result<Self> {
        let inner = DefaultSubstrateClient::connect(url).await?;
        Ok(Self { inner })
    }
}

impl<T: SubstrateRpcClient> StorageBisectClient<T> {
    /// Paginates through all storage keys at a given block hash
    pub async fn get_all_keys_paged(&self, block_hash: &str) -> Result<Vec<String>> {
        let mut all_keys = Vec::new();
        let mut start_key: Option<String> = None;
        const PAGE_SIZE: u32 = 1000;

        loop {
            let keys_page = self
                .inner
                .get_keys_paged(
                    None, // No prefix, fetch all keys
                    PAGE_SIZE,
                    start_key.as_deref(),
                    Some(block_hash),
                )
                .await?;

            let page_len = keys_page.len();
            all_keys.extend(keys_page);

            if page_len < PAGE_SIZE as usize {
                break; // Last page
            } else {
                start_key = all_keys.last().cloned();
            }
        }
        Ok(all_keys)
    }

    /// Find all storage changes between two blocks
    pub async fn find_all_storage_changes(
        &self,
        start_block: u32,
        end_block: u32,
    ) -> Result<Vec<StorageChange>> {
        // Step 1: Get block hashes
        let start_hash = self
            .inner
            .get_block_hash(start_block)
            .await?
            .ok_or_else(|| anyhow!("Start block {} not found", start_block))?;

        let end_hash = self
            .inner
            .get_block_hash(end_block)
            .await?
            .ok_or_else(|| anyhow!("End block {} not found", end_block))?;

        // Step 2: Get all keys at both blocks
        log::debug!("Fetching keys at block {}...", start_block);
        let start_keys = self.get_all_keys_paged(&start_hash).await?;

        log::debug!("Fetching keys at block {}...", end_block);
        let end_keys = self.get_all_keys_paged(&end_hash).await?;

        // Create union of all keys
        let mut all_keys: HashSet<String> = HashSet::new();
        all_keys.extend(start_keys.iter().cloned());
        all_keys.extend(end_keys.iter().cloned());

        log::debug!("Total unique keys to check: {}", all_keys.len());

        // Step 3: Find changed keys
        let mut changed_keys = Vec::new();

        for key in all_keys {
            // Get values at both blocks
            let start_value = self.get_storage_value(&key, &start_hash).await?;
            let end_value = self.get_storage_value(&key, &end_hash).await?;

            if start_value != end_value {
                changed_keys.push(key);
            }
        }

        log::info!("Found {} keys that changed", changed_keys.len());

        // Step 4: Binary search for exact change blocks
        let mut changes = Vec::new();

        for (idx, key) in changed_keys.iter().enumerate() {
            log::debug!(
                "Finding change point for key {}/{}...",
                idx + 1,
                changed_keys.len()
            );

            if let Some(change) = self
                .find_exact_change_block(key, start_block, end_block)
                .await?
            {
                changes.push(change);
            }
        }

        Ok(changes)
    }

    /// Get storage value for a key at a specific block
    async fn get_storage_value(&self, key: &str, block_hash: &str) -> Result<Option<Vec<u8>>> {
        let result = self.inner.get_storage(key, Some(block_hash)).await?;

        match result {
            Some(hex_value) => Ok(Some(hex_utils::decode_hex(&hex_value)?)),
            None => Ok(None),
        }
    }

    /// Binary search to find the exact block where a storage key changed
    async fn find_exact_change_block(
        &self,
        key: &str,
        start_block: u32,
        end_block: u32,
    ) -> Result<Option<StorageChange>> {
        let start_hash = self
            .inner
            .get_block_hash(start_block)
            .await?
            .ok_or_else(|| anyhow!("Block {} not found", start_block))?;

        let initial_value = self.get_storage_value(key, &start_hash).await?;

        let mut low = start_block;
        let mut high = end_block;

        while low <= high {
            let mid = low + (high - low) / 2;

            let mid_hash = self
                .inner
                .get_block_hash(mid)
                .await?
                .ok_or_else(|| anyhow!("Block {} not found", mid))?;

            let mid_value = self.get_storage_value(key, &mid_hash).await?;

            if mid_value != initial_value {
                // Change occurred at or before mid
                high = mid - 1;
            } else {
                // Change occurred after mid
                low = mid + 1;
            }
        }

        // low now points to the first block where the change occurred
        if low <= end_block {
            let change_hash = self
                .inner
                .get_block_hash(low)
                .await?
                .ok_or_else(|| anyhow!("Block {} not found", low))?;

            let new_value = self.get_storage_value(key, &change_hash).await?;

            // Convert key and value to bytes
            let key_bytes = hex_utils::decode_hex(key)?;
            let value_bytes = new_value.unwrap_or_default();

            Ok(Some(StorageChange {
                key: key_bytes,
                value: value_bytes,
                block_number: low,
            }))
        } else {
            Ok(None)
        }
    }
}
