use anyhow::Result;
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
                if !pallet_name.to_lowercase().contains(&filter_pallet.to_lowercase()) {
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