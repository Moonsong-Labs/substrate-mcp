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
use serde::Deserialize;

use crate::config::Config;
use crate::resources;
use crate::substrate::client::SubstrateClient;
use crate::substrate::events::EventFilter;
use crate::substrate::historical::{query_historical_events, HistoricalEventsQuery};
use crate::substrate::metadata::MetadataFilter;
use crate::substrate::storage::{list_pallet_storage, StorageQuery};
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
    config: Config,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct GetPolkadotSdkReleasePrdocsRequest {
    /// polkadot-sdk release (examples: '1.9.0', 'stable2412-1', 'stable2412').
    /// Can also be a range using '>': 'stable2502>stable2503-2' to get all releases between them.
    pub release: String,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize)]
pub struct AnalyzeReleaseRequest {
    /// polkadot-sdk release(s) to analyze (must have PR analysis data available).
    /// Can be a single release or comma-separated list for cross-release analysis.
    /// Example: "stable2412" or "stable2412,stable2412-1,stable2412-2"
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

impl Default for SubstrateService {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_router]
impl SubstrateService {
    pub fn new() -> Self {
        let config = Config::load_from_file("rpc_endpoints.json").unwrap_or_else(|e| {
            log::error!("Failed to load config file 'rpc_endpoints.json': {e}");
            log::error!("Please ensure rpc_endpoints.json exists in the current directory");
            log::error!("You can find an example configuration in the repository");
            std::process::exit(1);
        });

        Self {
            tool_router: Self::tool_router(),
            config,
        }
    }

    fn get_rpc_url(&self, url_option: Option<String>) -> String {
        url_option.unwrap_or_else(|| {
            self.config
                .get_default_url()
                .unwrap_or("wss://westend-rpc.polkadot.io")
                .to_string()
        })
    }

    #[tool(
        description = "Get all documented changes for a given polkadot-sdk release. Downloads PRDoc files to ./polkadot-release-analysis/releases/{release}/pr-docs/ directory in your current working directory.

Files are saved to: ./polkadot-release-analysis/releases/{release}/pr-docs/
- Individual PRDocs: pr_XXXX.prdoc
- manifest.json: Basic metadata (release, total PRDocs, PR numbers from filenames, download date)
- crate_summary.json: Changes grouped by crate with bump levels (major/minor/patch/none) and counts
- audience_summary.json: Changes grouped by target audience (Runtime Dev, Node Dev, Runtime User, Node Operator)

These manifests enable efficient analysis without parsing all PRDocs individually. After downloading, use standard file tools (Read, Grep, etc.) to explore the PRDocs."
    )]
    pub async fn get_polkadot_sdk_release_prdocs(
        &self,
        Parameters(GetPolkadotSdkReleasePrdocsRequest { release }): Parameters<
            GetPolkadotSdkReleasePrdocsRequest,
        >,
    ) -> Result<CallToolResult, McpError> {
        // Check if this is a version range
        if release.contains('>') {
            // Handle version range: current>target
            let parts: Vec<&str> = release.split('>').collect();
            if parts.len() != 2 {
                return Err(McpError {
                    code: rmcp::model::ErrorCode::INVALID_PARAMS,
                    message: "Invalid version range format. Use: 'current>target' (e.g., 'stable2502>stable2503-2')".into(),
                    data: None,
                });
            }

            let current_version = parts[0].trim();
            let target_version = parts[1].trim();

            // Get all releases in the range
            let releases =
                polkadot_sdk_releases::get_releases_between(current_version, target_version)
                    .await
                    .map_err(|e| McpError {
                        code: rmcp::model::ErrorCode(-32603),
                        message: e.to_string().into(),
                        data: None,
                    })?;

            if releases.is_empty() {
                return Ok(CallToolResult {
                    content: vec![Content {
                        annotations: None,
                        raw: RawContent::Text(RawTextContent {
                            text: format!(
                                "No releases found between {current_version} and {target_version}"
                            ),
                        }),
                    }],
                    is_error: None,
                });
            }

            // Download PRDocs for each release
            let mut total_files = 0;
            let mut total_size = 0;
            let mut downloaded_releases = Vec::new();

            for release_version in &releases {
                match polkadot_sdk_releases::query_prdocs(release_version).await {
                    Ok(result) => {
                        if result.success {
                            total_files += result.file_count;
                            total_size += result.total_size;
                            downloaded_releases.push(release_version.clone());
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to download PRDocs for {release_version}: {e}");
                    }
                }
            }

            let response_text = format!(
                "Downloaded PRDocs for {} releases between {} and {}:\n\nReleases processed: {}\n\nTotal files: {}\nTotal size: {} bytes\n\n📁 PRDocs saved to: ./polkadot-release-analysis/releases/\nEach release has its own subdirectory with pr-docs/\n\nYou can now use standard file operations (LS, Read, Glob, Grep) to explore the PRDocs.",
                downloaded_releases.len(),
                current_version,
                target_version,
                downloaded_releases.join(", "),
                total_files,
                total_size
            );

            Ok(CallToolResult {
                content: vec![Content {
                    annotations: None,
                    raw: RawContent::Text(RawTextContent {
                        text: response_text,
                    }),
                }],
                is_error: None,
            })
        } else {
            // Single release
            let result = polkadot_sdk_releases::query_prdocs(&release)
                .await
                .map_err(|e| McpError {
                    code: rmcp::model::ErrorCode(-32603),
                    message: e.to_string().into(),
                    data: None,
                })?;

            let response_text = if result.success {
                format!(
                    "Successfully downloaded {} PRDoc files for release '{}' to:\n{}\n\nTotal size: {} bytes\n\nYou can now use standard file operations (LS, Read, Glob, Grep) to explore the PRDocs.",
                    result.file_count,
                    result.release,
                    result.output_dir.display(),
                    result.total_size
                )
            } else {
                format!(
                    "No PRDoc files found for release '{}'. The directory {} was created but is empty.",
                    result.release,
                    result.output_dir.display()
                )
            };

            Ok(CallToolResult {
                content: vec![Content {
                    annotations: None,
                    raw: RawContent::Text(RawTextContent {
                        text: response_text,
                    }),
                }],
                is_error: None,
            })
        }
    }

    #[tool(
        description = "Analyze Polkadot SDK release(s) and return data for generating a comprehensive markdown report with migration guides, breaking changes, security analysis, and PR-by-PR breakdown. IMPORTANT: After using this tool, YOU (the LLM) MUST create a markdown file with the analysis results - this is mandatory unless explicitly told not to. Save the markdown file to ./polkadot-release-analysis/releases/{release}/reports/. This tool expects PRDoc data to be in ./polkadot-release-analysis/releases/{release}/pr-docs/ (use get_polkadot_sdk_release_prdocs first to download). The tool itself returns analysis prompts and data; YOU must execute the analysis and create the report file. Supports single or multiple releases (comma-separated)."
    )]
    pub async fn analyze_release(
        &self,
        Parameters(AnalyzeReleaseRequest { release }): Parameters<AnalyzeReleaseRequest>,
    ) -> Result<CallToolResult, McpError> {
        let base_path = std::env::current_dir().map_err(|e| McpError {
            code: rmcp::model::ErrorCode(-32603),
            message: format!("Failed to get current directory: {e}").into(),
            data: None,
        })?;

        let analysis_path = crate::release_analysis::analyze_polkadot_release(&release, base_path)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: e.to_string().into(),
                data: None,
            })?;

        // Check if multiple releases were analyzed
        let releases: Vec<&str> = release.split(',').map(|s| s.trim()).collect();
        let response_text = if releases.len() > 1 {
            format!(
                "Successfully analyzed {} releases: {}. Combined analysis saved to:\n{}\n\nThe analysis includes data for each release:\n- Release summaries with PR counts by category\n- Comprehensive index of all PRs across releases\n- Impact analysis for breaking changes and migrations\n- Cross-release dependency tracking\n- Cumulative change analysis\n\nUse Read to view the full analysis JSON.",
                releases.len(),
                releases.join(", "),
                analysis_path
            )
        } else {
            format!(
                "Successfully analyzed release '{release}'. Analysis saved to:\n{analysis_path}\n\nThe analysis includes:\n- Release summary with PR counts by category\n- Comprehensive index of all PRs\n- Impact analysis for breaking changes and migrations\n- Categorization by subsystem and audience\n- Change relationships and dependencies\n\nUse Read to view the full analysis JSON."
            )
        };

        Ok(CallToolResult {
            content: vec![Content {
                annotations: None,
                raw: RawContent::Text(RawTextContent {
                    text: response_text,
                }),
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
        let url = self.get_rpc_url(args.rpc_url);
        let chain_name = crate::config::chain_name_from_endpoint(&url, &self.config);
        log::info!("Connecting to {chain_name} at {url}");
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
        let url = self.get_rpc_url(args.rpc_url);

        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&url)
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
        let url = self.get_rpc_url(args.rpc_url);

        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&url)
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
        description = "Query chain storage entries by pallet and storage name. Supports querying map-type storage with keys. Use this to read chain state like account balances, staking info, or governance proposals."
    )]
    pub async fn query_storage(
        &self,
        Parameters(args): Parameters<StorageQueryArgs>,
    ) -> Result<CallToolResult, McpError> {
        let url = self.get_rpc_url(args.rpc_url);

        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to connect to chain: {e}").into(),
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
            message: format!("Failed to query storage: {e}").into(),
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
        let url = self.get_rpc_url(args.rpc_url);

        // Connect to the chain using subxt
        let client = OnlineClient::<PolkadotConfig>::from_url(&url)
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
        description = "Query events from historical blocks. Supports relative block numbers (e.g., -10 for 10 blocks ago). Uses hybrid approach: RPC for historical access, subxt for decoding."
    )]
    pub async fn query_historical_events(
        &self,
        Parameters(args): Parameters<QueryHistoricalEventsArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Handle endpoint parameter - if it's a known network name, use the full URL from config
        // Otherwise, prepend wss:// to the endpoint
        let url = match args.endpoint.as_deref() {
            Some(name) => {
                // Check if it's a known endpoint name in config
                if let Some(endpoint_url) = self.config.get_endpoint_url(name) {
                    endpoint_url.to_string()
                } else {
                    // Handle different protocol schemes for custom endpoints
                    if name.starts_with("ws://") || name.starts_with("wss://") {
                        // WebSocket URLs are used as-is
                        name.to_string()
                    } else if name.starts_with("http://") {
                        // Convert HTTP to WSS
                        name.replace("http://", "wss://")
                    } else if name.starts_with("https://") {
                        // Convert HTTPS to WSS
                        name.replace("https://", "wss://")
                    } else {
                        // Assume it's a hostname and prepend wss://
                        format!("wss://{name}")
                    }
                }
            }
            None => self.get_rpc_url(None),
        };

        // Connect to the chain using subxt for metadata
        let client = OnlineClient::<PolkadotConfig>::from_url(&url)
            .await
            .map_err(|e| McpError {
                code: rmcp::model::ErrorCode(-32603),
                message: format!("Failed to connect to chain with URL '{url}': {e}").into(),
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
