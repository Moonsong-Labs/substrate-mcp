use handlebars::Handlebars;
use rmcp::model::{
    GetPromptRequestParam, GetPromptResult, ListPromptsResult, PaginatedRequestParam, Prompt,
    PromptArgument, PromptMessage, PromptMessageContent, PromptMessageRole,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ErrorData as McpError;
use serde_json::json;

// Import all prompt modules
mod analyze_release;
mod automated_analysis;
mod code_security_audit;
mod economic_security;
mod incentive_analysis;
mod release_comparison;
mod scaffold_pallet;
mod security_disclaimer;
mod threat_modeling;
mod weight_analysis;

/// Metadata for a single prompt with its template
pub struct SubstratePrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
    pub template: String,
}

/// Create a new Prompts instance with all available prompts
pub fn prompts() -> Vec<SubstratePrompt> {
    vec![
        release_comparison::prompt(),
        automated_analysis::prompt(),
        code_security_audit::prompt(),
        economic_security::prompt(),
        incentive_analysis::prompt(),
        scaffold_pallet::prompt(),
        threat_modeling::prompt(),
        weight_analysis::prompt(),
        analyze_release::prompt(),
    ]
}

/// Get the list of all available prompts
fn list_prompts() -> Vec<Prompt> {
    prompts()
        .iter()
        .map(|p| Prompt {
            name: p.name.clone(),
            description: Some(p.description.clone()),
            arguments: Some(p.arguments.clone()),
        })
        .collect()
}

/// Get a specific prompt by name
fn get_substrate_prompt(name: &str) -> Option<SubstratePrompt> {
    prompts().into_iter().find(|p| p.name == name)
}

/// Handle list_prompts request
pub async fn handle_list_prompts(
    _cursor: Option<PaginatedRequestParam>,
    _context: RequestContext<RoleServer>,
) -> Result<ListPromptsResult, McpError> {
    Ok(ListPromptsResult {
        prompts: list_prompts(),
        next_cursor: None,
    })
}

/// Handle get_prompt request
pub async fn handle_get_prompt(
    request: GetPromptRequestParam,
    _context: RequestContext<RoleServer>,
) -> Result<GetPromptResult, McpError> {
    let prompt_def = get_substrate_prompt(&request.name).ok_or_else(|| McpError {
        code: rmcp::model::ErrorCode::INVALID_PARAMS,
        message: format!("Unknown prompt: {}", request.name).into(),
        data: None,
    })?;

    let empty_map = serde_json::Map::new();
    let args = request.arguments.as_ref().unwrap_or(&empty_map);

    // Add security disclaimer to args for templates that need it
    let mut args_with_disclaimer = args.clone();
    args_with_disclaimer.insert(
        "security_disclaimer".to_string(),
        json!(security_disclaimer::SECURITY_DISCLAIMER),
    );

    let mut registry = Handlebars::new();
    registry.set_strict_mode(true);

    let prompt = registry
        .render_template(&prompt_def.template, &args_with_disclaimer)
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Failed to render template: {e}").into(),
            data: None,
        })?;

    Ok(GetPromptResult {
        messages: vec![PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text { text: prompt },
        }],
        description: Some(prompt_def.description),
    })
}
