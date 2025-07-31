use rmcp::handler::server::tool::{Parameters, ToolRouter};
use rmcp::handler::server::ServerHandler;
use rmcp::service::{RequestContext, RoleServer};
use rmcp::model::{
    CallToolResult, Content, RawContent, RawTextContent, ServerCapabilities, ServerInfo,
    // Prompt-related imports
    GetPromptRequestParam, GetPromptResult, ListPromptsResult, PaginatedRequestParam,
    Prompt, PromptMessage, PromptArgument, PromptMessageRole, PromptMessageContent,
};
use rmcp::{schemars, tool, tool_handler, tool_router, ErrorData as McpError};
use std::future::Future;

use crate::polkadot_sdk_releases;
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
                name: "analyze_pallet".to_string(),
                description: Some("Analyze a Substrate pallet's structure and functionality".to_string()),
                arguments: Some(vec![
                    PromptArgument {
                        name: "pallet_name".to_string(),
                        description: Some("Name of the pallet to analyze".to_string()),
                        required: Some(true),
                    }
                ]),
            },
            Prompt {
                name: "explain_extrinsic".to_string(),
                description: Some("Explain a Substrate extrinsic and its effects".to_string()),
                arguments: Some(vec![
                    PromptArgument {
                        name: "extrinsic_hash".to_string(),
                        description: Some("Hash of the extrinsic to explain".to_string()),
                        required: Some(true),
                    },
                    PromptArgument {
                        name: "block_number".to_string(),
                        description: Some("Block number where the extrinsic was included".to_string()),
                        required: Some(false),
                    }
                ]),
            },
            Prompt {
                name: "substrate_upgrade_guide".to_string(),
                description: Some("Generate a migration guide for upgrading between Substrate versions".to_string()),
                arguments: Some(vec![
                    PromptArgument {
                        name: "from_version".to_string(),
                        description: Some("Starting Substrate/polkadot-sdk version".to_string()),
                        required: Some(true),
                    },
                    PromptArgument {
                        name: "to_version".to_string(),
                        description: Some("Target Substrate/polkadot-sdk version".to_string()),
                        required: Some(true),
                    }
                ]),
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
            "analyze_pallet" => {
                let pallet_name = request.arguments
                    .as_ref()
                    .and_then(|args| args.get("pallet_name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let messages = vec![
                    PromptMessage {
                        role: PromptMessageRole::Assistant,
                        content: PromptMessageContent::Text {
                            text: "You are a Substrate blockchain expert. Analyze the provided pallet structure, its storage items, events, errors, and dispatchable functions.".to_string()
                        },
                    },
                    PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::Text {
                            text: format!("Please provide a comprehensive analysis of the {} pallet in Substrate.", pallet_name)
                        },
                    },
                ];

                Ok(GetPromptResult {
                    messages,
                    description: Some("Analyze a Substrate pallet's structure and functionality".to_string()),
                })
            },
            "explain_extrinsic" => {
                let args = request.arguments.as_ref();
                let extrinsic_hash = args
                    .and_then(|a| a.get("extrinsic_hash"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let block_number = args
                    .and_then(|a| a.get("block_number"))
                    .and_then(|v| v.as_str());

                let mut user_message = format!("Please explain the extrinsic with hash: {}", extrinsic_hash);
                if let Some(block) = block_number {
                    user_message.push_str(&format!(" at block {}", block));
                }

                let messages = vec![
                    PromptMessage {
                        role: PromptMessageRole::Assistant,
                        content: PromptMessageContent::Text {
                            text: "You are a Substrate blockchain expert. Explain the given extrinsic, its parameters, effects on chain state, and any events it emitted.".to_string()
                        },
                    },
                    PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::Text {
                            text: user_message
                        },
                    },
                ];

                Ok(GetPromptResult {
                    messages,
                    description: Some("Explain a Substrate extrinsic and its effects".to_string()),
                })
            },
            "substrate_upgrade_guide" => {
                let args = request.arguments.as_ref();
                let from_version = args
                    .and_then(|a| a.get("from_version"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let to_version = args
                    .and_then(|a| a.get("to_version"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let messages = vec![
                    PromptMessage {
                        role: PromptMessageRole::Assistant,
                        content: PromptMessageContent::Text {
                            text: "You are a Substrate migration expert. Generate a detailed upgrade guide covering breaking changes, new features, and migration steps.".to_string()
                        },
                    },
                    PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::Text {
                            text: format!("Generate a migration guide for upgrading from Substrate/polkadot-sdk {} to {}", from_version, to_version)
                        },
                    },
                ];

                Ok(GetPromptResult {
                    messages,
                    description: Some("Generate a migration guide for upgrading between Substrate versions".to_string()),
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
