use rmcp::handler::server::tool::{Parameters, ToolRouter};
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    Annotations, CallToolResult, Content, ListResourceTemplatesResult, ListResourcesResult,
    PaginatedRequestParam, RawContent, RawResource, RawTextContent, ReadResourceRequestParam,
    ReadResourceResult, Resource, ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{schemars, tool, tool_handler, tool_router, ErrorData as McpError};
use std::future::Future;
use std::path::PathBuf;
use tokio::fs;

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
        let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("resources");

        // Helper function to get file size
        async fn get_file_size(filename: &str) -> Option<u32> {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("resources")
                .join(filename);

            tokio::fs::metadata(&path)
                .await
                .ok()
                .and_then(|metadata| u32::try_from(metadata.len()).ok())
        }

        let resources = vec![
                Resource::new(
                    RawResource {
                        uri: format!("file://{}/getting-started.md", base_path.display()),
                        name: "Getting Started with Substrate".to_string(),
                        description: Some("Comprehensive guide to Substrate development with examples and best practices".to_string()),
                        mime_type: Some("text/markdown".to_string()),
                        size: get_file_size("getting-started.md").await,
                    },
                    Some(Annotations {
                        audience: None,
                        priority: Some(1.0), // High priority for beginners
                        timestamp: None,
                    }),
                ),
                Resource::new(
                    RawResource {
                        uri: format!("file://{}/polkadot-sdk.md", base_path.display()),
                        name: "Polkadot SDK Reference".to_string(),
                        description: Some("Extensive documentation about Polkadot SDK architecture, components, and usage patterns".to_string()),
                        mime_type: Some("text/markdown".to_string()),
                        size: get_file_size("polkadot-sdk.md").await,
                    },
                    Some(Annotations {
                        audience: None,
                        priority: Some(0.95), // Very high priority
                        timestamp: None,
                    }),
                ),
                Resource::new(
                    RawResource {
                        uri: format!("file://{}/xcm-caching.md", base_path.display()),
                        name: "XCM Caching Strategies".to_string(),
                        description: Some("Comprehensive guide to XCM caching patterns, optimization techniques, and implementation".to_string()),
                        mime_type: Some("text/markdown".to_string()),
                        size: get_file_size("xcm-caching.md").await,
                    },
                    Some(Annotations {
                        audience: None,
                        priority: Some(0.85), // High priority for XCM developers
                        timestamp: None,
                    }),
                ),
                Resource::new(
                    RawResource {
                        uri: format!("file://{}/chain-spec.md", base_path.display()),
                        name: "Chain Specifications Guide".to_string(),
                        description: Some("Comprehensive guide to Substrate chain specifications with examples, patterns, and best practices".to_string()),
                        mime_type: Some("text/markdown".to_string()),
                        size: get_file_size("chain-spec.md").await,
                    },
                    Some(Annotations {
                        audience: None,
                        priority: Some(0.9), // High priority - essential for network setup
                        timestamp: None,
                    }),
                ),
            ];

        Ok(ListResourcesResult::with_all_items(resources))
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
        // Parse file:// URI to get the actual file path
        let file_path = if let Some(path) = request.uri.strip_prefix("file://") {
            PathBuf::from(path)
        } else {
            return Err(McpError::resource_not_found(request.uri, None));
        };

        // Verify the file is within our resources directory for security
        let resources_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("resources");

        if !file_path.starts_with(&resources_dir) {
            return Err(McpError {
                code: rmcp::model::ErrorCode(-32002), // Invalid params
                message: "Access denied: Resource must be within the resources directory".into(),
                data: None,
            });
        }

        // Read the file content
        match fs::read_to_string(&file_path).await {
            Ok(content) => Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, request.uri.clone())],
            }),
            Err(e) => Err(McpError {
                code: rmcp::model::ErrorCode::INTERNAL_ERROR,
                message: format!("Failed to read resource file: {e}").into(),
                data: None,
            }),
        }
    }
}
