use anyhow::Result;
use serde::{Deserialize, Serialize};
use subxt::Metadata;
use scale_info::{TypeDef, TypeDefPrimitive};

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

/// Represents detailed call argument information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallArgumentInfo {
    /// Argument name
    pub name: String,
    /// Type information as a string (simplified)
    pub type_name: String,
    /// Whether the argument is optional
    pub optional: bool,
    /// Documentation for the argument
    pub docs: Vec<String>,
}

/// Represents detailed information about a specific call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallMetadataDetail {
    /// Pallet name
    pub pallet: String,
    /// Call name  
    pub call: String,
    /// Call documentation
    pub docs: Vec<String>,
    /// Call index
    pub index: u8,
    /// Arguments for this call
    pub arguments: Vec<CallArgumentInfo>,
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

/// Extract detailed metadata for a specific call
pub fn get_call_metadata(metadata: &Metadata, pallet_name: &str, call_name: &str) -> Result<CallMetadataDetail> {
    // Find the pallet
    let pallet = metadata.pallets()
        .find(|p| p.name().eq_ignore_ascii_case(pallet_name))
        .ok_or_else(|| anyhow::anyhow!("Pallet '{}' not found", pallet_name))?;
    
    // Get call variants
    let call_variants = pallet.call_variants()
        .ok_or_else(|| anyhow::anyhow!("Pallet '{}' has no calls", pallet_name))?;
    
    // Find the specific call
    let call_variant = call_variants
        .iter()
        .find(|v| v.name.eq_ignore_ascii_case(call_name))
        .ok_or_else(|| anyhow::anyhow!("Call '{}' not found in pallet '{}'", call_name, pallet_name))?;
    
    // Extract argument information
    let mut arguments = Vec::new();
    for field in &call_variant.fields {
        arguments.push(CallArgumentInfo {
            name: field.name.clone().unwrap_or_else(|| format!("arg_{}", arguments.len())),
            type_name: format_type_name(field.ty.id, metadata),
            optional: false, // TODO: Determine if optional based on type analysis
            docs: field.docs.clone(),
        });
    }
    
    Ok(CallMetadataDetail {
        pallet: pallet.name().to_string(),
        call: call_variant.name.clone(),
        docs: call_variant.docs.clone(),
        index: call_variant.index,
        arguments,
    })
}

/// Format a type name for human readability
fn format_type_name(type_id: u32, metadata: &Metadata) -> String {
    // Get the type from the registry
    if let Some(ty) = metadata.types().resolve(type_id) {
        match &ty.type_def {
            TypeDef::Composite(composite) => {
                if composite.fields.is_empty() {
                    "()".to_string()
                } else if composite.fields.len() == 1 {
                    format_type_name(composite.fields[0].ty.id, metadata)
                } else {
                    let field_types: Vec<String> = composite.fields
                        .iter()
                        .map(|f| {
                            if let Some(name) = &f.name {
                                format!("{}: {}", name, format_type_name(f.ty.id, metadata))
                            } else {
                                format_type_name(f.ty.id, metadata)
                            }
                        })
                        .collect();
                    format!("{{ {} }}", field_types.join(", "))
                }
            },
            TypeDef::Variant(_variant) => {
                ty.path.ident()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            },
            TypeDef::Sequence(seq) => {
                format!("Vec<{}>", format_type_name(seq.type_param.id, metadata))
            },
            TypeDef::Array(arr) => {
                format!("[{}; {}]", format_type_name(arr.type_param.id, metadata), arr.len)
            },
            TypeDef::Tuple(tuple) => {
                if tuple.fields.is_empty() {
                    "()".to_string()
                } else {
                    let field_types: Vec<String> = tuple.fields
                        .iter()
                        .map(|f| format_type_name(f.id, metadata))
                        .collect();
                    format!("({})", field_types.join(", "))
                }
            },
            TypeDef::Primitive(primitive) => {
                match primitive {
                    TypeDefPrimitive::Bool => "bool".to_string(),
                    TypeDefPrimitive::Char => "char".to_string(),
                    TypeDefPrimitive::Str => "String".to_string(),
                    TypeDefPrimitive::U8 => "u8".to_string(),
                    TypeDefPrimitive::U16 => "u16".to_string(),
                    TypeDefPrimitive::U32 => "u32".to_string(),
                    TypeDefPrimitive::U64 => "u64".to_string(),
                    TypeDefPrimitive::U128 => "u128".to_string(),
                    TypeDefPrimitive::U256 => "U256".to_string(),
                    TypeDefPrimitive::I8 => "i8".to_string(),
                    TypeDefPrimitive::I16 => "i16".to_string(),
                    TypeDefPrimitive::I32 => "i32".to_string(),
                    TypeDefPrimitive::I64 => "i64".to_string(),
                    TypeDefPrimitive::I128 => "i128".to_string(),
                    TypeDefPrimitive::I256 => "I256".to_string(),
                }
            },
            TypeDef::Compact(compact) => {
                format!("Compact<{}>", format_type_name(compact.type_param.id, metadata))
            },
            TypeDef::BitSequence(_) => "BitVec".to_string(),
        }
    } else {
        format!("Type({type_id})")
    }
}
