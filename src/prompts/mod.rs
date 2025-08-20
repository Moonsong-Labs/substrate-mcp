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
pub struct SubstratePromptDefinition {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
    pub template: String,
}

/// Create a new Prompts instance with all available prompts
pub fn prompt_definitions() -> Vec<SubstratePromptDefinition> {
    vec![
        release_comparison::prompt_definition(),
        automated_analysis::prompt_definition(),
        code_security_audit::prompt_definition(),
        economic_security::prompt_definition(),
        incentive_analysis::prompt_definition(),
        scaffold_pallet::prompt_definition(),
        threat_modeling::prompt_definition(),
        weight_analysis::prompt_definition(),
        analyze_release::prompt_definition(),
    ]
}

/// Get the list of all available prompts
fn list_prompts() -> Vec<Prompt> {
    prompt_definitions()
        .iter()
        .map(|p| Prompt {
            name: p.name.clone(),
            description: Some(p.description.clone()),
            arguments: Some(p.arguments.clone()),
        })
        .collect()
}

/// Get a specific prompt by name
fn get_prompt_definition(name: &str) -> Option<SubstratePromptDefinition> {
    prompt_definitions().into_iter().find(|p| p.name == name)
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

pub fn handle_get_prompt(request: GetPromptRequestParam) -> Result<GetPromptResult, McpError> {
    let prompt_def = get_prompt_definition(&request.name).ok_or_else(|| McpError {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::{GetPromptRequestParam, PromptMessageRole};
    use serde_json::json;

    // Helper to test prompts
    fn test_prompt(
        prompt_name: &str,
        arguments: serde_json::Map<String, serde_json::Value>,
        expected_content_fragments: Vec<&str>,
    ) {
        let request = GetPromptRequestParam {
            name: prompt_name.to_string(),
            arguments: Some(arguments),
        };

        let result = handle_get_prompt(request);

        assert!(
            result.is_ok(),
            "Failed to get prompt '{}': {:?}",
            prompt_name,
            result
        );
        let prompt_result = result.unwrap();

        // Check basic structure
        assert_eq!(prompt_result.messages.len(), 1);
        assert_eq!(prompt_result.messages[0].role, PromptMessageRole::User);
        assert!(prompt_result.description.is_some());

        // Extract text content
        let text = match &prompt_result.messages[0].content {
            PromptMessageContent::Text { text } => text,
            _ => panic!("Expected text content for prompt '{}'", prompt_name),
        };

        // Verify expected content fragments are present
        for fragment in expected_content_fragments {
            assert!(
                text.contains(fragment),
                "Prompt '{}' should contain '{}' but got:\n{}",
                prompt_name,
                fragment,
                text
            );
        }
    }

    #[test]
    fn test_release_comparison_prompt() {
        let mut args = serde_json::Map::new();
        args.insert("current_version".to_string(), json!("1.9.0"));
        args.insert("target_version".to_string(), json!("1.10.0"));

        test_prompt(
            "release_comparison",
            args.clone(),
            vec![
                "Compare changes between Polkadot SDK versions 1.9.0 and 1.10.0",
                "fetch_and_analyze_release",
                "Breaking Changes",
                "New Features",
                "Migration Recommendations",
            ],
        );

        // Test with optional specific_changes parameter
        args.insert(
            "specific_changes".to_string(),
            json!("pallet_treasury changes"),
        );
        test_prompt(
            "release_comparison",
            args,
            vec![
                "Compare changes between Polkadot SDK versions 1.9.0 and 1.10.0",
                "Filtered Analysis",
                "Focus only on changes related to: pallet_treasury changes",
            ],
        );
    }

    #[test]
    fn test_automated_analysis_prompt() {
        let mut args = serde_json::Map::new();
        args.insert(
            "change_description".to_string(),
            json!("Added new pallet_balances functionality for multi-asset support"),
        );

        test_prompt(
            "automated_analysis",
            args,
            vec![
                "comprehensive security and quality analysis",
                "Added new pallet_balances functionality for multi-asset support",
                "Code Security Analysis",
                "Substrate-Specific Security",
                "Runtime Safety",
                "Common Vulnerability Patterns",
            ],
        );
    }

    #[test]
    fn test_code_security_audit_prompt() {
        let mut args = serde_json::Map::new();
        args.insert("audit_target".to_string(), json!("pallet_staking"));

        test_prompt(
            "code_security_audit",
            args,
            vec![
                "Security Expert",
                "pallet_staking",
                "Audit Scope",
                "Storage security",
                "Dispatchable function",
                "Weight calculations",
            ],
        );
    }

    #[test]
    fn test_economic_security_prompt() {
        let mut args = serde_json::Map::new();
        args.insert(
            "system_description".to_string(),
            json!("treasury funding mechanism"),
        );

        test_prompt(
            "economic_security",
            args,
            vec![
                "economic security assessment",
                "treasury funding mechanism",
                "Economic Model Analysis",
                "Game Theory Analysis",
                "MEV",
            ],
        );
    }

    #[test]
    fn test_incentive_analysis_prompt() {
        let mut args = serde_json::Map::new();
        args.insert("target_pallets".to_string(), json!("pallet_staking"));
        args.insert(
            "analysis_specifications".to_string(),
            json!("staking rewards distribution"),
        );

        test_prompt(
            "incentive_analysis",
            args,
            vec![
                "expert in Cryptoeconomics",
                "staking rewards distribution",
                "Analysis Specifications",
                "game theory",
            ],
        );
    }

    #[test]
    fn test_scaffold_pallet_prompt() {
        let mut args = serde_json::Map::new();
        args.insert("pallet_description".to_string(), json!("NFT marketplace"));

        test_prompt(
            "scaffold_pallet",
            args,
            vec![
                "NFT marketplace",
                "Implementation Requirements",
                "Storage Design",
                "Error Handling",
                "Events",
                "Weights",
            ],
        );
    }

    #[test]
    fn test_threat_modeling_prompt() {
        let mut args = serde_json::Map::new();
        args.insert("system_description".to_string(), json!("governance system"));

        test_prompt(
            "threat_modeling",
            args,
            vec![
                "threat model analysis",
                "governance system",
                "Asset Analysis",
                "Attack Surface Mapping",
                "trust boundaries",
            ],
        );
    }

    #[test]
    fn test_weight_analysis_prompt() {
        let mut args = serde_json::Map::new();
        args.insert("target_pallet".to_string(), json!("pallet_democracy"));

        test_prompt(
            "weight_analysis",
            args,
            vec![
                "weight analysis",
                "pallet_democracy",
                "Weight Function Analysis",
                "benchmarks",
                "resource usage",
            ],
        );
    }

    #[test]
    fn test_analyze_release_prompt() {
        let mut args = serde_json::Map::new();
        args.insert("release".to_string(), json!("stable2412"));

        test_prompt(
            "analyze_release",
            args.clone(),
            vec![
                "release(s) stable2412 impact",
                "Project Dependency Discovery",
                "construct_runtime!",
                "Migration",
            ],
        );

        // Test with optional focus parameter
        args.insert("focus".to_string(), json!("XCM improvements"));
        test_prompt(
            "analyze_release",
            args,
            vec!["release(s) stable2412 impact", "XCM improvements"],
        );
    }

    #[test]
    fn test_unknown_prompt_error() {
        let request = GetPromptRequestParam {
            name: "non_existent_prompt".to_string(),
            arguments: None,
        };

        let result = handle_get_prompt(request);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, rmcp::model::ErrorCode::INVALID_PARAMS);
        assert!(error.message.contains("Unknown prompt"));
    }
}
