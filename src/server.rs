use rmcp::handler::server::tool::{Parameters, ToolRouter};
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolResult, Content, ListResourceTemplatesResult, ListResourcesResult,
    PaginatedRequestParam, RawContent, RawTextContent, ReadResourceRequestParam,
    ReadResourceResult, ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{schemars, tool, tool_handler, tool_router, ErrorData as McpError};
use std::future::Future;
use std::process::Stdio;
use tokio::process::Command;

use crate::polkadot_sdk_releases;
use serde::Deserialize;

use crate::resources;
use crate::substrate::client::SubstrateClient;
use crate::substrate::metadata::MetadataFilter;
use crate::substrate::events::EventFilter;
use crate::substrate::storage::{StorageQuery, list_pallet_storage};
use crate::substrate::historical::{query_historical_events, HistoricalEventsQuery};
use subxt::OnlineClient;
use subxt::PolkadotConfig;

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct StorageBisectArgs {
    pub start_block: u32,
    pub end_block: u32,
    pub key: String,
    pub rpc_url: Option<String>,
}

#[derive(Clone)]
pub struct SubstrateService {
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct GetPolkadotSdkReleasePrdocsRequest {
    /// polkadot-sdk release (examples: '1.9.0', 'stable2412-1', 'stable2412')
    pub release: String,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct SubxtExecuteArgs {
    /// The subxt command and arguments to execute (e.g., ["metadata", "-f", "json", "--url", "ws://localhost:9944"])
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct MetadataFilterArgs {
    /// The RPC URL to connect to (defaults to public Polkadot endpoint)
    pub rpc_url: Option<String>,
    /// Filter by item type (e.g., "pallet", "storage", "call", "event", "constant", "error")
    pub item_type: Option<String>,
    /// Filter by pallet name (supports partial matching)
    pub pallet: Option<String>,
    /// Filter by item name (supports partial matching)
    pub name: Option<String>,
    /// Include detailed type information
    pub include_details: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct EventFilterArgs {
    /// The RPC URL to connect to (defaults to public Polkadot endpoint)
    pub rpc_url: Option<String>,
    /// Filter by pallet name (supports partial matching)
    pub pallet: Option<String>,
    /// Filter by event variant name (supports partial matching)
    pub variant: Option<String>,
    /// Start block number (inclusive, defaults to 100 blocks ago)
    pub from_block: Option<u32>,
    /// End block number (inclusive, defaults to latest)
    pub to_block: Option<u32>,
    /// Maximum number of events to return
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct StorageQueryArgs {
    /// The RPC URL to connect to (defaults to public Polkadot endpoint)
    pub rpc_url: Option<String>,
    /// The pallet name
    pub pallet: String,
    /// The storage entry name
    pub entry: String,
    /// Optional keys for map-type storage (as JSON array)
    pub keys: Option<Vec<serde_json::Value>>,
    /// Block number to query at (None for latest)
    pub at_block: Option<u32>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct ListPalletStorageArgs {
    /// The RPC URL to connect to (defaults to public Polkadot endpoint)
    pub rpc_url: Option<String>,
    /// The pallet name
    pub pallet: String,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct QueryHistoricalEventsArgs {
    /// The RPC endpoint to connect to. Can be 'polkadot', 'kusama', 'westend', or a custom endpoint without protocol prefix
    pub endpoint: Option<String>,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago)
    pub from_block: i32,
    /// End block number (negative = relative to current, defaults to from_block)
    pub to_block: Option<i32>,
    /// Filter by pallet name (optional)
    pub pallet: Option<String>,
    /// Filter by event name (optional)
    pub event: Option<String>,
}

#[tool_router]
impl SubstrateService {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get all documented changes for a given polkadot-sdk release")]
    pub async fn get_polkadot_sdk_release_prdocs(
        &self,
        Parameters(GetPolkadotSdkReleasePrdocsRequest { release }): Parameters<
            GetPolkadotSdkReleasePrdocsRequest,
        >,
    ) -> Result<CallToolResult, McpError> {
        let response = polkadot_sdk_releases::query_prdocs(&release)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: e.to_string().into(),
                data: None,
            })?;

        Ok(CallToolResult {
            content: vec![Content {
                annotations: None,
                raw: RawContent::Text(RawTextContent { text: response }),
            }],
            is_error: None,
        })
    }

    #[tool(
        description = "Find all storage changes between two blocks on a Substrate chain for a specific key"
    )]
    pub async fn chain_storage_bisect(
        &self,
        Parameters(args): Parameters<StorageBisectArgs>,
    ) -> Result<CallToolResult, McpError> {
        let url = args
            .rpc_url
            .unwrap_or_else(|| crate::public_endpoints::endpoints::DEFAULT.to_string());

        let chain_name = crate::public_endpoints::chain_name_from_endpoint(&url);
        log::info!("Connecting to {} at {}", chain_name, url);
        let client = SubstrateClient::connect(&url).await.map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: e.to_string().into(),
            data: None,
        })?;

        let result = client
            .find_all_storage_changes(args.start_block, args.end_block, args.key)
            .await;

        match result {
            Ok(changes) => {
                let json_result = serde_json::to_string_pretty(&changes).map_err(|e| McpError {
                    code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                    message: format!("Serialization error: {e}").into(),
                    data: None,
                })?;

                Ok(CallToolResult {
                    content: vec![Content {
                        annotations: None,
                        raw: RawContent::Text(RawTextContent { text: json_result }),
                    }],
                    is_error: None,
                })
            }
            Err(e) => Err(McpError {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: format!("Storage changes error: {e}").into(),
                data: None,
            }),
        }
    }

    #[tool(
        description = "Use subxt to decode and explore Substrate blockchain data. Useful for: analyzing chain metadata structure, generating type-safe Rust code for chain interactions, exploring available pallets/calls/storage/events, decoding extrinsics and storage values, and understanding runtime APIs. The 'explore' subcommand provides interactive browsing of chain state. Common investigations: System.LastRuntimeUpgrade (find runtime upgrades), Staking.ActiveEra (current era info), Democracy.ReferendumCount (governance activity). Outputs human-readable decoded values by default. Particularly valuable when building applications that need to interact with Substrate chains or when investigating chain functionality. Pass '--help' to any subcommand to learn more."
    )]
    pub async fn subxt_execute(
        &self,
        Parameters(args): Parameters<SubxtExecuteArgs>,
    ) -> Result<CallToolResult, McpError> {
        if args.args.is_empty() {
            return Err(McpError {
                code: rmcp::model::ErrorCode(-32602),
                message: "No arguments provided for subxt command".into(),
                data: None,
            });
        }

        log::info!("Executing subxt with args: {:?}", args.args);

        let output = Command::new("subxt")
            .args(&args.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to execute subxt: {e}. Make sure subxt is installed.").into(),
                data: None,
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("subxt command failed: {}", stderr).into(),
                data: None,
            });
        }

        let result = if !stdout.is_empty() {
            stdout.to_string()
        } else if !stderr.is_empty() {
            // Some subxt commands output to stderr even on success
            stderr.to_string()
        } else {
            "Command completed successfully with no output".to_string()
        };

        Ok(CallToolResult {
            content: vec![Content {
                annotations: None,
                raw: RawContent::Text(RawTextContent { text: result }),
            }],
            is_error: None,
        })
    }

    #[tool(
        description = "Filter and search chain metadata to discover available pallets, storage entries, calls, events, constants, and errors. Supports partial name matching for easy discovery. Use this to understand what functionality is available on a chain before making specific queries."
    )]
    pub async fn filter_metadata(
        &self,
        Parameters(args): Parameters<MetadataFilterArgs>,
    ) -> Result<CallToolResult, McpError> {
        let url = args
            .rpc_url
            .unwrap_or_else(|| crate::public_endpoints::endpoints::DEFAULT.to_string());

        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to connect to chain: {}", e).into(),
                data: None,
            })?;

        // Get metadata
        let metadata = client.metadata();

        // Create filter
        let filter = MetadataFilter {
            item_type: args.item_type,
            pallet: args.pallet,
            name: args.name,
            include_details: args.include_details.unwrap_or(false),
        };

        // Apply filter
        let results = filter.apply(&metadata).map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to filter metadata: {}", e).into(),
            data: None,
        })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&results).map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Serialization error: {}", e).into(),
            data: None,
        })?;

        Ok(CallToolResult {
            content: vec![Content {
                annotations: None,
                raw: RawContent::Text(RawTextContent { text: json_result }),
            }],
            is_error: None,
        })
    }

    #[tool(
        description = "Query and filter blockchain events within a specified block range. Supports filtering by pallet and event name with partial matching. Use this to find specific events like transfers, staking rewards, or governance actions."
    )]
    pub async fn query_events(
        &self,
        Parameters(args): Parameters<EventFilterArgs>,
    ) -> Result<CallToolResult, McpError> {
        let url = args
            .rpc_url
            .unwrap_or_else(|| crate::public_endpoints::endpoints::DEFAULT.to_string());

        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to connect to chain: {}", e).into(),
                data: None,
            })?;

        // Create filter
        let filter = EventFilter {
            pallet: args.pallet,
            variant: args.variant,
            from_block: args.from_block,
            to_block: args.to_block,
            limit: args.limit,
        };

        // Query events
        let results = filter.query_events(&client).await.map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to query events: {}", e).into(),
            data: None,
        })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&results).map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Serialization error: {}", e).into(),
            data: None,
        })?;

        Ok(CallToolResult {
            content: vec![Content {
                annotations: None,
                raw: RawContent::Text(RawTextContent { text: json_result }),
            }],
            is_error: None,
        })
    }

    #[tool(
        description = "Query chain storage entries by pallet and storage name. Supports querying map-type storage with keys. Use this to read chain state like account balances, staking info, or governance proposals."
    )]
    pub async fn query_storage(
        &self,
        Parameters(args): Parameters<StorageQueryArgs>,
    ) -> Result<CallToolResult, McpError> {
        let url = args
            .rpc_url
            .unwrap_or_else(|| crate::public_endpoints::endpoints::DEFAULT.to_string());

        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to connect to chain: {}", e).into(),
                data: None,
            })?;

        // Create query
        let query = StorageQuery {
            pallet: args.pallet,
            entry: args.entry,
            keys: args.keys,
            at_block: args.at_block,
        };

        // Execute query
        let result = query.execute(&client).await.map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to query storage: {}", e).into(),
            data: None,
        })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&result).map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Serialization error: {}", e).into(),
            data: None,
        })?;

        Ok(CallToolResult {
            content: vec![Content {
                annotations: None,
                raw: RawContent::Text(RawTextContent { text: json_result }),
            }],
            is_error: None,
        })
    }

    #[tool(
        description = "List all storage entries available in a specific pallet. Use this to discover what storage items are available before querying them."
    )]
    pub async fn list_pallet_storage(
        &self,
        Parameters(args): Parameters<ListPalletStorageArgs>,
    ) -> Result<CallToolResult, McpError> {
        let url = args
            .rpc_url
            .unwrap_or_else(|| crate::public_endpoints::endpoints::DEFAULT.to_string());

        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to connect to chain: {}", e).into(),
                data: None,
            })?;

        // List storage entries
        let entries = list_pallet_storage(&client, &args.pallet)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to list storage: {}", e).into(),
                data: None,
            })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&entries).map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Serialization error: {}", e).into(),
            data: None,
        })?;

        Ok(CallToolResult {
            content: vec![Content {
                annotations: None,
                raw: RawContent::Text(RawTextContent { text: json_result }),
            }],
            is_error: None,
        })
    }

    #[tool(
        description = "Query events from historical blocks. Supports relative block numbers (e.g., -10 for 10 blocks ago). Uses hybrid approach: RPC for historical access, subxt for decoding."
    )]
    pub async fn query_historical_events(
        &self,
        Parameters(args): Parameters<QueryHistoricalEventsArgs>,
    ) -> Result<CallToolResult, McpError> {
        
        // Handle endpoint parameter - if it's a known network name, use the full URL
        // Otherwise, prepend wss:// to the endpoint
        let url = match args.endpoint.as_deref() {
            Some("polkadot") => crate::public_endpoints::endpoints::POLKADOT.to_string(),
            Some("kusama") => crate::public_endpoints::endpoints::KUSAMA.to_string(),
            Some("westend") => crate::public_endpoints::endpoints::WESTEND.to_string(),
            Some("rococo") => crate::public_endpoints::endpoints::ROCOCO.to_string(),
            Some("paseo") => crate::public_endpoints::endpoints::PASEO.to_string(),
            Some(endpoint) => {
                // Handle different protocol schemes
                if endpoint.starts_with("ws://") || endpoint.starts_with("wss://") {
                    // WebSocket URLs are used as-is
                    endpoint.to_string()
                } else if endpoint.starts_with("http://") {
                    // Convert HTTP to WSS
                    endpoint.replace("http://", "wss://")
                } else if endpoint.starts_with("https://") {
                    // Convert HTTPS to WSS  
                    endpoint.replace("https://", "wss://")
                } else {
                    // Assume it's a hostname and prepend wss://
                    format!("wss://{}", endpoint)
                }
            },
            None => crate::public_endpoints::endpoints::DEFAULT.to_string(),
        };


        // Connect to the chain using subxt for metadata
        let client = OnlineClient::<PolkadotConfig>::from_url(&url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to connect to chain with URL '{}': {}", url, e).into(),
                data: None,
            })?;

        // Create query
        let query = HistoricalEventsQuery {
            from_block: args.from_block,
            to_block: args.to_block,
            pallet: args.pallet,
            event: args.event,
        };

        // Query historical events
        let result = query_historical_events(query, &client, &url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to query historical events: {}", e).into(),
                data: None,
            })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&result).map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Serialization error: {}", e).into(),
            data: None,
        })?;

        Ok(CallToolResult {
            content: vec![Content {
                annotations: None,
                raw: RawContent::Text(RawTextContent { text: json_result }),
            }],
            is_error: None,
        })
    }
}

#[tool_handler]
impl ServerHandler for SubstrateService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: rmcp::model::Implementation {
                name: "substrate-mcp".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: Some("Primary source for Substrate/Polkadot SDK development. Provides authoritative tools for chain interaction, release documentation, prompt templates and comprehensive Substrate knowledge resources.".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            ..Default::default()
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult::with_all_items(
            resources::get_all_resources(),
        ))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult::with_all_items(vec![]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match resources::get_resource_content(&request.uri) {
            Some(content) => Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, request.uri.clone())],
            }),
            None => Err(McpError::resource_not_found(request.uri, None)),
        }
    }
}
