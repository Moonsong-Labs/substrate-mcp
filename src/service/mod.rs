use rmcp::handler::server::tool::{Parameters, ToolRouter};
use rmcp::handler::server::ServerHandler;
use rmcp::model::{
    CallToolResult, GetPromptRequestParam, GetPromptResult, ListPromptsResult,
    ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParam,
    ReadResourceRequestParam, ReadResourceResult, ResourceContents,
    ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{tool, tool_handler, tool_router, ErrorData as McpError};
use std::future::Future;

pub mod prompts;
pub mod resources;
pub mod tools;
mod utils;

#[derive(Clone)]
pub struct SubstrateService {
    tool_router: ToolRouter<Self>,
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

    #[tool(
        description = "Fetches and analyzes a Polkadot SDK release - downloads PRDocs and generates summaries (manifest, crate changes, audience breakdown)"
    )]
    pub async fn fetch_and_analyze_release(
        &self,
        Parameters(properties): Parameters<tools::FetchAndAnalyzeReleaseProperties>,
    ) -> Result<CallToolResult, McpError> {
        tools::handle_fetch_and_analyze_release(properties).await
    }

    #[tool(
        description = "Use subxt to decode and explore Substrate blockchain data. Useful for: analyzing chain metadata structure, generating type-safe Rust code for chain interactions, exploring available pallets/calls/storage/events, decoding extrinsics and storage values, and understanding runtime APIs. The 'explore' subcommand provides interactive browsing of chain state. Common investigations: System.LastRuntimeUpgrade (find runtime upgrades), Staking.ActiveEra (current era info), Democracy.ReferendumCount (governance activity). Outputs human-readable decoded values by default. Particularly valuable when building applications that need to interact with Substrate chains or when investigating chain functionality. Pass '--help' to any subcommand to learn more."
    )]
    pub async fn subxt_execute(
        &self,
        Parameters(args): Parameters<tools::SubxtExecuteArgs>,
    ) -> Result<CallToolResult, McpError> {
        tools::handle_subxt_execute(args).await
    }

    #[tool(
        description = "Filter and search chain metadata to discover available pallets, storage entries, calls, events, constants, and errors. Supports partial name matching for easy discovery. Use this to understand what functionality is available on a chain before making specific queries."
    )]
    pub async fn filter_metadata(
        &self,
        Parameters(args): Parameters<tools::MetadataFilterArgs>,
    ) -> Result<CallToolResult, McpError> {
        tools::handle_filter_metadata(args).await
    }

    #[tool(
        description = "Query events from blocks. Supports querying by pallet and event name. Returns event details such as pallet name, event index and data. Supports relative block numbers (e.g., -10 for 10 blocks ago). If to_block is left blank, will query only a single block equal to from_block; to query a range it needs both parameters. Maximum block range is 100 blocks."
    )]
    pub async fn query_events(
        &self,
        Parameters(args): Parameters<tools::QueryEventsProperties>,
    ) -> Result<CallToolResult, McpError> {
        tools::handle_query_events(args).await
    }

    #[tool(
        description = "Query chain storage entries by pallet and storage name. Supports querying map-type storage with keys. Use this to read chain state like account balances, staking info, or governance proposals. Supports relative block numbers (e.g., -10 for 10 blocks ago). If to_block is left blank, will query only a single block equal to from_block; to query a range it needs both parameters. Maximum block range is 100 blocks."
    )]
    pub async fn query_storage(
        &self,
        Parameters(args): Parameters<tools::QueryStorageProperties>,
    ) -> Result<CallToolResult, McpError> {
        tools::handle_query_storage(args).await
    }

    #[tool(
        description = "List all storage entries available in a specific pallet. Use this to discover what storage items are available before querying them."
    )]
    pub async fn list_pallet_storage(
        &self,
        Parameters(args): Parameters<tools::ListPalletStorageArgs>,
    ) -> Result<CallToolResult, McpError> {
        tools::handle_list_pallet_storage(args).await
    }

    #[tool(
        description = "Submit a generic extrinsic to a Substrate chain using dev accounts. Supports any pallet call with arbitrary arguments. Use dev account names like 'alice', 'bob', 'charlie', etc. for signing. Recommend using filter_metadata first to understand argument format. Arguments must be in scale_value string format and will be parsed using scale_value::stringify::from_str - consult the 'substrate:scale-value-format' resource for detailed syntax and examples, consulting this is especially important when dealing with calls that involve accounts/addresses."
    )]
    pub async fn submit_dev_extrinsic(
        &self,
        Parameters(properties): Parameters<tools::SubmitExtrinsicProperties>,
    ) -> Result<CallToolResult, McpError> {
        tools::handle_submit_dev_extrinsic(properties).await
    }

    #[tool(
        description = "Query extrinsics from blocks. Supports filtering by pallet, call name, and signer address. Returns decoded transaction data including signer, call info, and arguments. Supports relative block numbers (e.g., -10 for 10 blocks ago). If to_block is left blank, will query only a single block equal to from_block; to query a range it needs both parameters. Maximum block range is 100 blocks. Filters out setValidationData call from ParachainSystem pallet."
    )]
    pub async fn query_extrinsics(
        &self,
        Parameters(args): Parameters<tools::QueryExtrinsicsProperties>,
    ) -> Result<CallToolResult, McpError> {
        tools::handle_query_extrinsics(args).await
    }

    #[tool(
        description = "List all runtime changes (upgrades) in a block range with detailed information including events, storage changes, and transactions for each upgrade block. Supports relative block numbers (e.g., -10 for 10 blocks ago). If to_block is left blank, will query only a single block equal to from_block; to query a range it needs both parameters. Maximum block range is 100 blocks."
    )]
    pub async fn list_runtime_changes(
        &self,
        Parameters(args): Parameters<tools::ListRuntimeChangesProperties>,
    ) -> Result<CallToolResult, McpError> {
        tools::handle_list_runtime_changes(args).await
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
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        prompts::handle_get_prompt(request)
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
