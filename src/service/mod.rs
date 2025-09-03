use rmcp::handler::server::ServerHandler;
use rmcp::handler::server::router::prompt::PromptRouter;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, GetPromptRequestParam, GetPromptResult, ListPromptsResult,
    ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParam, PromptMessage,
    ReadResourceRequestParam, ReadResourceResult, ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{
    ErrorData as McpError, prompt, prompt_handler, prompt_router, tool, tool_handler, tool_router,
};

pub(crate) mod prompts;
pub(crate) mod resources;
pub(crate) mod tools;
mod utils;

use utils::catch_panic_as_mcp_error;

use crate::service::prompts::{
    analyze_release, automated_analysis, code_security_audit, economic_security,
    incentive_analysis, release_comparison, scaffold_pallet, threat_modeling, weight_analysis,
};

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub(crate) struct SubstrateService {
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

impl Default for SubstrateService {
    fn default() -> Self {
        Self::new()
    }
}

#[prompt_router]
#[tool_router]
impl SubstrateService {
    pub(crate) fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    #[tool(
        description = "Fetches and analyzes a Polkadot SDK release - downloads PRDocs and generates summaries (manifest, crate changes, audience breakdown)"
    )]
    pub(crate) async fn fetch_and_analyze_release(
        &self,
        Parameters(properties): Parameters<tools::FetchAndAnalyzeReleaseProperties>,
    ) -> Result<CallToolResult, McpError> {
        catch_panic_as_mcp_error(tools::handle_fetch_and_analyze_release(properties)).await
    }

    #[tool(
        description = "Use subxt to decode and explore Substrate blockchain data. Useful for: analyzing chain metadata structure, generating type-safe Rust code for chain interactions, exploring available pallets/calls/storage/events, decoding extrinsics and storage values, and understanding runtime APIs. The 'explore' subcommand provides interactive browsing of chain state. Common investigations: System.LastRuntimeUpgrade (find runtime upgrades), Staking.ActiveEra (current era info), Democracy.ReferendumCount (governance activity). Outputs human-readable decoded values by default. Particularly valuable when building applications that need to interact with Substrate chains or when investigating chain functionality. Pass '--help' to any subcommand to learn more."
    )]
    pub(crate) async fn subxt_execute(
        &self,
        Parameters(properties): Parameters<tools::SubxtExecuteProperties>,
    ) -> Result<CallToolResult, McpError> {
        catch_panic_as_mcp_error(tools::handle_subxt_execute(properties)).await
    }

    #[tool(
        description = "Filter and search chain metadata to discover available pallets, storage entries, calls, events, constants, and errors. Supports partial name matching for easy discovery. Use this to understand what functionality is available on a chain before making specific queries."
    )]
    pub(crate) async fn filter_metadata(
        &self,
        Parameters(properties): Parameters<tools::MetadataFilterProperties>,
    ) -> Result<CallToolResult, McpError> {
        catch_panic_as_mcp_error(tools::handle_filter_metadata(properties)).await
    }

    #[tool(
        description = "Query events from blocks. Supports querying by pallet and event name. Returns event details such as pallet name, event index and data. Supports relative block numbers (e.g., -10 for 10 blocks ago). If to_block is left blank, will query only a single block equal to from_block; to query a range it needs both parameters. Maximum block range is 100 blocks."
    )]
    pub(crate) async fn query_events(
        &self,
        Parameters(properties): Parameters<tools::QueryEventsProperties>,
    ) -> Result<CallToolResult, McpError> {
        catch_panic_as_mcp_error(tools::handle_query_events(properties)).await
    }

    #[tool(
        description = "Query chain storage entries by pallet and storage name. Supports querying map-type storage with keys. Use this to read chain state like account balances, staking info, or governance proposals. Supports relative block numbers (e.g., -10 for 10 blocks ago). If to_block is left blank, will query only a single block equal to from_block; to query a range it needs both parameters. Maximum block range is 100 blocks."
    )]
    pub(crate) async fn query_storage(
        &self,
        Parameters(properties): Parameters<tools::QueryStorageProperties>,
    ) -> Result<CallToolResult, McpError> {
        catch_panic_as_mcp_error(tools::handle_query_storage(properties)).await
    }

    #[tool(
        description = "List all storage entries available in a specific pallet. Use this to discover what storage items are available before querying them."
    )]
    pub(crate) async fn list_pallet_storage(
        &self,
        Parameters(properties): Parameters<tools::ListPalletStorageProperties>,
    ) -> Result<CallToolResult, McpError> {
        catch_panic_as_mcp_error(tools::handle_list_pallet_storage(properties)).await
    }

    #[tool(
        description = "Submit a generic extrinsic to a Substrate chain using dev accounts. Supports any pallet call with arbitrary arguments. Use dev account names like 'alice', 'bob', 'charlie', etc. for signing. Recommend using filter_metadata first to understand argument format. Arguments must be in scale_value string format and will be parsed using scale_value::stringify::from_str - consult the 'substrate:scale-value-format' resource for detailed syntax and examples, consulting this is especially important when dealing with calls that involve accounts/addresses."
    )]
    pub(crate) async fn submit_dev_extrinsic(
        &self,
        Parameters(properties): Parameters<tools::SubmitExtrinsicProperties>,
    ) -> Result<CallToolResult, McpError> {
        catch_panic_as_mcp_error(tools::handle_submit_dev_extrinsic(properties)).await
    }

    #[tool(
        description = "Query extrinsics from blocks. Supports filtering by pallet, call name, and signer address. Returns decoded transaction data including signer, call info, and arguments. Supports relative block numbers (e.g., -10 for 10 blocks ago). If to_block is left blank, will query only a single block equal to from_block; to query a range it needs both parameters. Maximum block range is 100 blocks. Filters out setValidationData call from ParachainSystem pallet."
    )]
    pub(crate) async fn query_extrinsics(
        &self,
        Parameters(properties): Parameters<tools::QueryExtrinsicsProperties>,
    ) -> Result<CallToolResult, McpError> {
        catch_panic_as_mcp_error(tools::handle_query_extrinsics(properties)).await
    }

    // Prompt implementations
    /// Compare changes between two Polkadot SDK versions
    #[prompt(
        name = "release_comparison",
        description = "List changes between two polkadot-sdk release versions"
    )]
    async fn release_comparison(
        &self,
        Parameters(args): Parameters<release_comparison::ReleaseComparisonArgs>,
    ) -> Result<GetPromptResult, McpError> {
        prompts::release_comparison::generate_prompt(args).await
    }

    /// Analyze how a Polkadot SDK release impacts your project
    #[prompt(
        name = "analyze_release",
        description = "Analyze how specific release(s) impact your current project"
    )]
    async fn analyze_release(
        &self,
        Parameters(args): Parameters<analyze_release::AnalyzeReleaseArgs>,
    ) -> Vec<PromptMessage> {
        prompts::analyze_release::generate_prompt(args).await
    }

    /// Generate pallet structure and implementation templates
    #[prompt(
        name = "scaffold_pallet",
        description = "Generate pallet structure and implementation templates"
    )]
    async fn scaffold_pallet(
        &self,
        Parameters(args): Parameters<scaffold_pallet::ScaffoldPalletArgs>,
    ) -> Vec<PromptMessage> {
        prompts::scaffold_pallet::generate_prompt(args).await
    }

    /// Comprehensive security and quality analysis
    #[prompt(
        name = "automated_analysis",
        description = "Automated comprehensive security and quality analysis"
    )]
    async fn automated_analysis(
        &self,
        Parameters(args): Parameters<automated_analysis::AutomatedAnalysisArgs>,
    ) -> Vec<PromptMessage> {
        prompts::automated_analysis::generate_prompt(args).await
    }

    /// Security audit for specific components
    #[prompt(
        name = "code_security_audit",
        description = "Security audit for specific components or pallets"
    )]
    async fn code_security_audit(
        &self,
        Parameters(args): Parameters<code_security_audit::CodeSecurityAuditArgs>,
    ) -> Vec<PromptMessage> {
        prompts::code_security_audit::generate_prompt(args).await
    }

    /// Economic security assessment
    #[prompt(
        name = "economic_security",
        description = "Economic security assessment for blockchain systems"
    )]
    async fn economic_security(
        &self,
        Parameters(args): Parameters<economic_security::EconomicSecurityArgs>,
    ) -> Vec<PromptMessage> {
        prompts::economic_security::generate_prompt(args).await
    }

    /// Cryptoeconomic incentive analysis
    #[prompt(
        name = "incentive_analysis",
        description = "Cryptoeconomic incentive analysis for specific pallets"
    )]
    async fn incentive_analysis(
        &self,
        Parameters(args): Parameters<incentive_analysis::IncentiveAnalysisArgs>,
    ) -> Vec<PromptMessage> {
        prompts::incentive_analysis::generate_prompt(args).await
    }

    /// Threat model analysis
    #[prompt(
        name = "threat_modeling",
        description = "Comprehensive threat model analysis"
    )]
    async fn threat_modeling(
        &self,
        Parameters(args): Parameters<threat_modeling::ThreatModelingArgs>,
    ) -> Vec<PromptMessage> {
        prompts::threat_modeling::generate_prompt(args).await
    }

    /// Weight and benchmark analysis
    #[prompt(
        name = "weight_analysis",
        description = "Weight and benchmark analysis for Substrate pallets"
    )]
    async fn weight_analysis(
        &self,
        Parameters(args): Parameters<weight_analysis::WeightAnalysisArgs>,
    ) -> Vec<PromptMessage> {
        prompts::weight_analysis::generate_prompt(args).await
    }
}

#[prompt_handler]
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
