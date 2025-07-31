use rmcp::handler::server::tool::{Parameters, ToolRouter};
use rmcp::handler::server::ServerHandler;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::model::{
    CallToolResult, Content, RawContent, RawTextContent, ServerCapabilities, ServerInfo,
    // Prompt-related imports
    GetPromptRequestParam, GetPromptResult, ListPromptsResult, PaginatedRequestParam,
    Prompt,
};
use rmcp::{schemars, tool, tool_handler, tool_router, ErrorData as McpError};
use std::future::Future;

use crate::polkadot_sdk_releases;
use crate::prompts::{self, ReleaseComparisonArgs};
use serde::Deserialize;

use crate::substrate::client::SubstrateClient;

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
            .unwrap_or_else(|| "http://127.0.0.1:9944".to_string());

        log::info!("Connecting to Substrate node at {url}...");
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
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
            ..Default::default()
        }
    }

    async fn list_prompts(
        &self,
        _cursor: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        let prompts = vec![
            Prompt {
                name: "release_comparison".to_string(),
                description: Some("List changes between two polkadot-sdk release versions".to_string()),
                arguments: Some(prompts::get_release_comparison_prompt_arguments()),
            },
        ];

        Ok(ListPromptsResult {
            prompts,
            next_cursor: None,
        })
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match request.name.as_str() {
            "release_comparison" => {
                let args = request.arguments.as_ref();
                let current_version = args
                    .and_then(|a| a.get("current_version"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError {
                        code: rmcp::model::ErrorCode::INVALID_PARAMS,
                        message: "current_version is required".to_string().into(),
                        data: None,
                    })?
                    .to_string();
                    
                let target_version = args
                    .and_then(|a| a.get("target_version"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError {
                        code: rmcp::model::ErrorCode::INVALID_PARAMS,
                        message: "target_version is required".to_string().into(),
                        data: None,
                    })?
                    .to_string();
                    
                let specific_changes = args
                    .and_then(|a| a.get("specific_changes"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let messages = prompts::release_comparison_prompt(ReleaseComparisonArgs {
                    current_version,
                    target_version,
                    specific_changes,
                })?;

                Ok(GetPromptResult {
                    messages,
                    description: Some("List changes between two polkadot-sdk release versions".to_string()),
                })
            },
            _ => Err(McpError {
                code: rmcp::model::ErrorCode::INVALID_PARAMS,
                message: format!("Unknown prompt: {}", request.name).into(),
                data: None,
            }),
        }
    }
}
