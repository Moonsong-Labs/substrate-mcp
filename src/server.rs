use rmcp::handler::server::tool::{Parameters, ToolRouter};
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolResult, Content, GetPromptRequestParam, GetPromptResult, ListPromptsResult,
    ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParam, RawContent,
    RawTextContent, ReadResourceRequestParam, ReadResourceResult, ResourceContents,
    ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{schemars, tool, tool_handler, tool_router, ErrorData as McpError};
use std::future::Future;
use std::process::Stdio;
use tokio::process::Command;

use crate::polkadot_sdk_releases;
use crate::prompts;
use crate::tools;
use serde::Deserialize;

use crate::resources;
use crate::substrate::events::{query_historical_events, EventFilter, HistoricalEventsQuery};
use crate::substrate::extrinsic::{query_extrinsics, ExtrinsicsQuery};
use crate::substrate::metadata::MetadataFilter;
use crate::substrate::runtime::list_runtime_changes;
use crate::substrate::storage::{list_pallet_storage, query_storage, StorageQuery};

use subxt::OnlineClient;
use subxt::PolkadotConfig;

#[derive(Clone)]
pub struct SubstrateService {
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct FetchAndAnalyzeReleaseRequest {
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
    /// The RPC URL to connect to
    pub rpc_url: String,
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
    /// The RPC URL to connect to
    pub rpc_url: String,
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
pub struct QueryStorageProperties {
    /// The RPC URL to connect to
    pub rpc_url: String,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago. 0 returns current)
    pub from_block: i32,
    /// End block number (negative = relative to current, defaults to from_block. Leaving this blank will return a single block equal to from_block)
    pub to_block: Option<i32>,
    /// The pallet name
    pub pallet: String,
    /// The storage entry name
    pub entry: String,
    /// Optional keys for map-type storage (as JSON array). Supports SS58 addresses which will be automatically decoded to AccountId32
    pub keys: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct ListPalletStorageArgs {
    /// The RPC URL to connect to
    pub rpc_url: String,
    /// The pallet name
    pub pallet: String,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct QueryHistoricalEventsArgs {
    /// The RPC endpoint to connect to
    pub rpc_url: String,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago. 0 returns current)
    pub from_block: i32,
    /// End block number (negative = relative to current, defaults to from_block. Leaving this blank will return a single block equal to from_block)
    pub to_block: Option<i32>,
    /// Filter by pallet name (optional)
    pub pallet: Option<String>,
    /// Filter by event name (optional)
    pub event: Option<String>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct QueryExtrinsicsProperties {
    /// The RPC endpoint to connect to
    pub rpc_url: String,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago. 0 returns current)
    pub from_block: i32,
    /// End block number (negative = relative to current, defaults to from_block. Leaving this blank will return a single block equal to from_block)
    pub to_block: Option<i32>,
    /// Filter by pallet name (optional)
    pub pallet: Option<String>,
    /// Filter by call name (optional)
    pub call: Option<String>,
    /// Filter by signer address (optional)
    pub signer: Option<String>,
}

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct ListRuntimeChangesProperties {
    /// The RPC endpoint to connect to
    pub rpc_url: String,
    /// Start block number (negative = relative to current, e.g. -10 = 10 blocks ago. 0 returns current)
    pub from_block: i32,
    /// End block number (negative = relative to current, defaults to current block. Leaving this blank will return a single block equal to from_block)
    pub to_block: Option<i32>,
}

impl Default for SubstrateService {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl SubstrateService {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Fetches and analyzes a Polkadot SDK release - downloads PRDocs and generates summaries (manifest, crate changes, audience breakdown)")]
    pub async fn fetch_and_analyze_release(
        &self,
        Parameters(FetchAndAnalyzeReleaseRequest { release }): Parameters<
            FetchAndAnalyzeReleaseRequest,
        >,
    ) -> Result<CallToolResult, McpError> {
        let response = polkadot_sdk_releases::fetch_and_analyze_release(&release)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: e.to_string().into(),
                data: None,
            })?;

        // Format the response as JSON string
        let response_text = serde_json::to_string_pretty(&response)
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to serialize response: {}", e).into(),
                data: None,
            })?;

        Ok(CallToolResult {
            content: vec![Content {
                annotations: None,
                raw: RawContent::Text(RawTextContent { text: response_text }),
            }],
            is_error: None,
        })
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
                message: format!("Failed to execute subxt: {e}. Make sure subxt is installed.")
                    .into(),
                data: None,
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("subxt command failed: {stderr}").into(),
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
        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to connect to chain: {e}").into(),
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
            message: format!("Failed to filter metadata: {e}").into(),
            data: None,
        })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&results).map_err(|e| McpError {
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

    #[tool(
        description = "Query and filter blockchain events within a specified block range. Supports filtering by pallet and event name with partial matching. Use this to find specific events like transfers, staking rewards, or governance actions."
    )]
    pub async fn query_events(
        &self,
        Parameters(args): Parameters<EventFilterArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to connect to chain: {e}").into(),
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
            message: format!("Failed to query events: {e}").into(),
            data: None,
        })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&results).map_err(|e| McpError {
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

    #[tool(
        description = "Query chain storage entries by pallet and storage name. Supports querying map-type storage with keys. Use this to read chain state like account balances, staking info, or governance proposals. Supports relative block numbers (e.g., -10 for 10 blocks ago). If to_block is left blank, will query only a single block equal to from_block; to query a range it needs both parameters. Maximum block range is 100 blocks."
    )]
    pub async fn query_storage(
        &self,
        Parameters(args): Parameters<QueryStorageProperties>,
    ) -> Result<CallToolResult, McpError> {
        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to connect to chain: {e}").into(),
                data: None,
            })?;

        // Create query
        let query = StorageQuery {
            from_block: args.from_block,
            to_block: args.to_block,
            pallet: args.pallet,
            entry: args.entry,
            keys: args.keys,
        };

        // Query historical storage
        let result = query_storage(query, &client, &args.rpc_url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to query historical storage: {e}").into(),
                data: None,
            })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&result).map_err(|e| McpError {
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

    #[tool(
        description = "List all storage entries available in a specific pallet. Use this to discover what storage items are available before querying them."
    )]
    pub async fn list_pallet_storage(
        &self,
        Parameters(args): Parameters<ListPalletStorageArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to connect to chain: {e}").into(),
                data: None,
            })?;

        // List storage entries
        let entries = list_pallet_storage(&client, &args.pallet)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to list storage: {e}").into(),
                data: None,
            })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&entries).map_err(|e| McpError {
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

    #[tool(
        description = "Query events from historical blocks. Supports relative block numbers (e.g., -10 for 10 blocks ago). If to_block is left blank, will query only a single block equal to from_block; to query a range it needs both parameters. Maximum block range is 100 blocks."
    )]
    pub async fn query_historical_events(
        &self,
        Parameters(args): Parameters<QueryHistoricalEventsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!(
                    "Failed to connect to chain with URL '{}': {e}",
                    args.rpc_url
                )
                .into(),
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
        let result = query_historical_events(query, &client, &args.rpc_url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to query historical events: {e}").into(),
                data: None,
            })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&result).map_err(|e| McpError {
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

    #[tool(
        description = "Submit a generic extrinsic to a Substrate chain using dev accounts. Supports any pallet call with arbitrary arguments. Use dev account names like 'alice', 'bob', 'charlie', etc. for signing. Recommend using filter_metadata first to understand argument format. Arguments must be in scale_value string format and will be parsed using scale_value::stringify::from_str - consult the 'substrate:scale-value-format' resource for detailed syntax and examples."
    )]
    pub async fn submit_dev_extrinsic(
        &self,
        Parameters(properties): Parameters<tools::SubmitExtrinsicProperties>,
    ) -> Result<CallToolResult, McpError> {
        tools::handle_submit_dev_extrinsic(properties).await
    }

    #[tool(
        description = "Query extrinsics from blocks. Supports filtering by pallet, call name, and signer address. Returns decoded transaction data including signer, call info, and arguments. Supports relative block numbers (e.g., -10 for 10 blocks ago). If to_block is left blank, will query only a single block equal to from_block; to query a range it needs both parameters. Maximum block range is 100 blocks."
    )]
    pub async fn query_extrinsics(
        &self,
        Parameters(args): Parameters<QueryExtrinsicsProperties>,
    ) -> Result<CallToolResult, McpError> {
        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!(
                    "Failed to connect to chain with URL '{}': {e}",
                    args.rpc_url
                )
                .into(),
                data: None,
            })?;

        // Create query
        let query = ExtrinsicsQuery {
            from_block: args.from_block,
            to_block: args.to_block,
            pallet: args.pallet,
            call: args.call,
            signer: args.signer,
        };

        // Query extrinsics
        let result = query_extrinsics(query, &client, &args.rpc_url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to query historical transactions: {e}").into(),
                data: None,
            })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&result).map_err(|e| McpError {
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

    #[tool(
        description = "List all runtime changes (upgrades) in a block range with detailed information including events, storage changes, and transactions for each upgrade block. Supports relative block numbers (e.g., -10 for 10 blocks ago). If to_block is left blank, will query only a single block equal to from_block; to query a range it needs both parameters. Maximum block range is 100 blocks."
    )]
    pub async fn list_runtime_changes(
        &self,
        Parameters(args): Parameters<ListRuntimeChangesProperties>,
    ) -> Result<CallToolResult, McpError> {
        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&args.rpc_url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!(
                    "Failed to connect to chain with URL '{}': {e}",
                    args.rpc_url
                )
                .into(),
                data: None,
            })?;

        // List runtime changes
        let result = list_runtime_changes(args.from_block, args.to_block, &client, &args.rpc_url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to list runtime changes: {e}").into(),
                data: None,
            })?;

        // Convert to JSON
        let json_result = serde_json::to_string_pretty(&result).map_err(|e| McpError {
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
                .enable_prompts()
                .enable_resources()
                .build(),
            ..Default::default()
        }
    }

    async fn list_prompts(
        &self,
        cursor: Option<PaginatedRequestParam>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        prompts::handle_list_prompts(cursor, context).await
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        prompts::handle_get_prompt(request, context).await
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

