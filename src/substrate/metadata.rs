use anyhow::Result;
use codec::Decode;
use serde::{Deserialize, Serialize};
use subxt::Metadata;

/// Represents a filtered metadata item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataItem {
    /// The type of item (pallet, storage, call, event, etc.)
    pub item_type: String,
    /// The pallet name
    pub pallet: String,
    /// The item name (if applicable)
    pub name: Option<String>,
    /// Additional metadata about the item
    pub details: serde_json::Value,
}

/// Filter criteria for metadata queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFilter {
    /// Filter by item type (e.g., "pallet", "storage", "call", "event", "constant", "error")
    pub item_type: Option<String>,
    /// Filter by pallet name (supports partial matching)
    pub pallet: Option<String>,
    /// Filter by item name (supports partial matching)
    pub name: Option<String>,
    /// Include detailed type information
    pub include_details: bool,
}

impl MetadataFilter {
    /// Apply the filter to metadata and return matching items
    pub fn apply(&self, metadata: &Metadata) -> Result<Vec<MetadataItem>> {
        let mut results = Vec::new();

        // Iterate through all pallets
        for pallet in metadata.pallets() {
            let pallet_name = pallet.name();

            // Check if pallet matches filter
            if let Some(ref filter_pallet) = self.pallet {
                if !pallet_name
                    .to_lowercase()
                    .contains(&filter_pallet.to_lowercase())
                {
                    continue;
                }
            }

            // If filtering for pallets specifically
            if self.item_type.as_deref() == Some("pallet") {
                results.push(MetadataItem {
                    item_type: "pallet".to_string(),
                    pallet: pallet_name.to_string(),
                    name: None,
                    details: serde_json::json!({
                        "index": pallet.index(),
                        "docs": pallet.docs(),
                    }),
                });
                continue;
            }

            // Process storage entries
            if self.item_type.is_none() || self.item_type.as_deref() == Some("storage") {
                if let Some(storage) = pallet.storage() {
                    for entry in storage.entries() {
                        if self.matches_name(entry.name()) {
                            let details = if self.include_details {
                                serde_json::json!({
                                    "docs": entry.docs(),
                                    "modifier": format!("{:?}", entry.modifier()),
                                    "default": format!("{:?}", entry.default_bytes()),
                                })
                            } else {
                                serde_json::json!({
                                    "docs": entry.docs(),
                                })
                            };

                            results.push(MetadataItem {
                                item_type: "storage".to_string(),
                                pallet: pallet_name.to_string(),
                                name: Some(entry.name().to_string()),
                                details,
                            });
                        }
                    }
                }
            }

            // Process calls
            if self.item_type.is_none() || self.item_type.as_deref() == Some("call") {
                if let Some(calls) = pallet.call_variants() {
                    for variant in calls {
                        if self.matches_name(&variant.name) {
                            let details = if self.include_details {
                                serde_json::json!({
                                    "docs": &variant.docs,
                                    "index": variant.index,
                                })
                            } else {
                                serde_json::json!({
                                    "docs": &variant.docs,
                                })
                            };

                            results.push(MetadataItem {
                                item_type: "call".to_string(),
                                pallet: pallet_name.to_string(),
                                name: Some(variant.name.to_string()),
                                details,
                            });
                        }
                    }
                }
            }

            // Process events
            if self.item_type.is_none() || self.item_type.as_deref() == Some("event") {
                if let Some(events) = pallet.event_variants() {
                    for variant in events {
                        if self.matches_name(&variant.name) {
                            let details = if self.include_details {
                                serde_json::json!({
                                    "docs": &variant.docs,
                                    "index": variant.index,
                                })
                            } else {
                                serde_json::json!({
                                    "docs": &variant.docs,
                                })
                            };

                            results.push(MetadataItem {
                                item_type: "event".to_string(),
                                pallet: pallet_name.to_string(),
                                name: Some(variant.name.to_string()),
                                details,
                            });
                        }
                    }
                }
            }

            // Process constants
            if self.item_type.is_none() || self.item_type.as_deref() == Some("constant") {
                for constant in pallet.constants() {
                    if self.matches_name(constant.name()) {
                        let details = if self.include_details {
                            serde_json::json!({
                                "docs": constant.docs(),
                                "value": format!("{:?}", constant.value()),
                            })
                        } else {
                            serde_json::json!({
                                "docs": constant.docs(),
                            })
                        };

                        results.push(MetadataItem {
                            item_type: "constant".to_string(),
                            pallet: pallet_name.to_string(),
                            name: Some(constant.name().to_string()),
                            details,
                        });
                    }
                }
            }

            // Process errors
            if self.item_type.is_none() || self.item_type.as_deref() == Some("error") {
                if let Some(errors) = pallet.error_variants() {
                    for variant in errors {
                        if self.matches_name(&variant.name) {
                            let details = if self.include_details {
                                serde_json::json!({
                                    "docs": &variant.docs,
                                    "index": variant.index,
                                })
                            } else {
                                serde_json::json!({
                                    "docs": &variant.docs,
                                })
                            };

                            results.push(MetadataItem {
                                item_type: "error".to_string(),
                                pallet: pallet_name.to_string(),
                                name: Some(variant.name.to_string()),
                                details,
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    fn matches_name(&self, name: &str) -> bool {
        if let Some(ref filter_name) = self.name {
            name.to_lowercase().contains(&filter_name.to_lowercase())
        } else {
            true
        }
    }
}

/// Summary of metadata contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSummary {
    /// List of pallets with their details
    pub pallets: Vec<PalletSummary>,
    /// Number of types in the type registry
    pub type_registry_size: u32,
}

/// Summary of a single pallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PalletSummary {
    /// Pallet name
    pub name: String,
    /// Pallet index
    pub index: u8,
    /// Pallet documentation
    pub docs: Vec<String>,
    /// Number of storage entries
    pub storage_count: u32,
    /// Number of calls/extrinsics
    pub calls_count: u32,
    /// Number of events
    pub events_count: u32,
    /// Number of constants
    pub constants_count: u32,
    /// Number of errors
    pub errors_count: u32,
}

/// Decode hex-encoded metadata into a Metadata object
pub fn decode_metadata_from_hex(hex_string: &str) -> Result<Metadata> {
    // Strip the '0x' prefix if it exists
    let hex_string = hex_string.strip_prefix("0x").unwrap_or(hex_string);

    // Decode the hex string into raw bytes
    let bytes =
        hex::decode(hex_string).map_err(|e| anyhow::anyhow!("Failed to decode hex: {}", e))?;

    // Use the Decode trait to parse the SCALE-encoded bytes
    let metadata = Metadata::decode(&mut &bytes[..])
        .map_err(|e| anyhow::anyhow!("Failed to decode metadata: {}", e))?;

    Ok(metadata)
}

/// Extract a summary of metadata contents
pub fn extract_metadata_summary(metadata: &Metadata) -> MetadataSummary {
    let mut pallets = Vec::new();

    // Iterate through all pallets and collect statistics
    for pallet in metadata.pallets() {
        let storage_count = pallet
            .storage()
            .map(|s| s.entries().len() as u32)
            .unwrap_or(0);

        let calls_count = pallet
            .call_variants()
            .map(|calls| calls.len() as u32)
            .unwrap_or(0);

        let events_count = pallet
            .event_variants()
            .map(|events| events.len() as u32)
            .unwrap_or(0);

        let constants_count = pallet.constants().count() as u32;

        let errors_count = pallet
            .error_variants()
            .map(|errors| errors.len() as u32)
            .unwrap_or(0);

        pallets.push(PalletSummary {
            name: pallet.name().to_string(),
            index: pallet.index(),
            docs: pallet.docs().to_vec(),
            storage_count,
            calls_count,
            events_count,
            constants_count,
            errors_count,
        });
    }

    // Sort pallets by index for consistent ordering
    pallets.sort_by_key(|p| p.index);

    MetadataSummary {
        pallets,
        type_registry_size: metadata.types().types.len() as u32,
    }
}
