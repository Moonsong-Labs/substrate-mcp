use rmcp::handler::server::tool::{Parameters, ToolRouter};
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolResult, Content, RawContent, RawTextContent, ServerCapabilities, ServerInfo,
};
use rmcp::{schemars, tool, tool_handler, tool_router, ErrorData as McpError};
use std::future::Future;

use crate::polkadot_sdk_releases;
use serde::Deserialize;

use crate::tools::{StorageBisectClient, StorageChange};

#[derive(Debug, Deserialize, Clone, schemars::JsonSchema)]
pub struct StorageChangesArgs {
    pub start_block: u32,
    pub end_block: u32,
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

    #[tool(description = "Find all storage changes between two blocks on a Substrate chain")]
    pub fn storage_changes(
        &self,
        Parameters(args): Parameters<StorageChangesArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Bridge sync tool to async implementation
        let runtime = tokio::runtime::Handle::current();
        let result = runtime.block_on(async {
            self.storage_changes_async(args.start_block, args.end_block, args.rpc_url)
                .await
        });

        match result {
            Ok(changes) => {
                let json_result = serde_json::to_string_pretty(&changes).map_err(|e| McpError {
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
            Err(e) => Err(McpError {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: format!("Storage changes error: {}", e).into(),
                data: None,
            }),
        }
    }

    async fn storage_changes_async(
        &self,
        start_block: u32,
        end_block: u32,
        rpc_url: Option<String>,
    ) -> Result<Vec<StorageChange>, anyhow::Error> {
        let url = rpc_url.unwrap_or_else(|| "ws://127.0.0.1:9944".to_string());

        log::info!("Connecting to Substrate node at {}...", url);
        let client = StorageBisectClient::new(&url).await?;

        log::info!(
            "Finding storage changes between blocks {} and {}...",
            start_block,
            end_block
        );
        let mut changes = client
            .find_all_storage_changes(start_block, end_block)
            .await?;

        // Sort by block number as required
        changes.sort_by_key(|c| c.block_number);

        log::info!("Found {} storage changes", changes.len());
        Ok(changes)
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
            instructions: Some("Tools and Prompts to work with Substrate based blockchains".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
