use rmcp::model::{
    GetPromptRequestParam, GetPromptResult, ListPromptsResult, PaginatedRequestParam, Prompt,
    PromptArgument, PromptMessage, PromptMessageContent, PromptMessageRole,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ErrorData as McpError;

mod templates;

use templates::{get_security_disclaimer, TEMPLATE_REGISTRY};

/// Metadata for a single prompt
pub struct SubstratePrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

/// Create a new Prompts instance with all available prompts
pub fn prompts() -> Vec<SubstratePrompt> {
    vec![
        SubstratePrompt {
            name: "release_comparison".to_string(),
            description: "List changes between two polkadot-sdk release versions".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "current_version".to_string(),
                    description: Some("Version currently being used".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "target_version".to_string(),
                    description: Some("Version dev wants to compare with (must be greater than current)".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "specific_changes".to_string(),
                    description: Some("What specific changes to look for (e.g: was there any change in `pallet_treasury` ?)".to_string()),
                    required: Some(false),
                },
            ],
        },
        SubstratePrompt {
            name: "automated_analysis".to_string(),
            description: "Template for automated code and runtime analysis".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "change_description".to_string(),
                    description: Some("Description of the changes made to the code that trigger this analysis (PR description, new release, etc)".to_string()),
                    required: Some(true),
                },
            ],
        },
        SubstratePrompt {
            name: "code_security_audit".to_string(),
            description: "Audit specific component for common code-related vulnerabilities".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "audit_type".to_string(),
                    description: Some("pallet/runtime/node/general".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "audit_target".to_string(),
                    description: Some("Describe the target of the audit".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "specific_checks".to_string(),
                    description: Some("Specific things to look for".to_string()),
                    required: Some(false),
                },
            ],
        },
        SubstratePrompt {
            name: "economic_security".to_string(),
            description: "Do an economic security analysis on a specific subsystem".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "system_description".to_string(),
                    description: Some("Description of the system to make the analysis for (all pallets, a specific group/flow, etc)".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "extra_context".to_string(),
                    description: Some("Extra context to provide for analysis".to_string()),
                    required: Some(true),
                },
            ],
        },
        SubstratePrompt {
            name: "incentive_analysis".to_string(),
            description: "Analyze economic viability of incentives".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "target_pallets".to_string(),
                    description: Some("List of pallets that make the scope of the analysis".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "analysis_specifications".to_string(),
                    description: Some("Specific things to look out for during the analysis".to_string()),
                    required: Some(true),
                },
            ],
        },
        SubstratePrompt {
            name: "scaffold_pallet".to_string(),
            description: "Generate pallet structure and implementation templates".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "pallet_description".to_string(),
                    description: Some("Description for the pallet".to_string()),
                    required: Some(true),
                },
            ],
        },
        SubstratePrompt {
            name: "threat_modeling".to_string(),
            description: "Do threat modeling of a specific part of the system".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "system_description".to_string(),
                    description: Some("Description of the system to make the analysis for (all pallets, a specific group/flow, node, etc)".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "extra_context".to_string(),
                    description: Some("Extra context to provide for analysis".to_string()),
                    required: Some(true),
                },
            ],
        },
        SubstratePrompt {
            name: "weight_analysis".to_string(),
            description: "Weight-based system breakdown analysis under extreme conditions".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "target_pallet".to_string(),
                    description: Some("Pallet to make the analysis for".to_string()),
                    required: Some(true),
                },
            ],
        },
        SubstratePrompt {
            name: "analyze_release".to_string(),
            description: "Analyzes how Polkadot SDK release changes impact your project using parallel processing".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "release".to_string(),
                    description: Some("The release version(s) to analyze. Examples: 'stable2503-7' for single release, 'stable2502,stable2503' for comparison".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "focus".to_string(),
                    description: Some("Optional: Specific aspect to focus on (e.g., 'breaking changes', 'migrations', 'security'). Leave empty for comprehensive analysis".to_string()),
                    required: Some(false),
                }
            ],
        },
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
fn get_prompt(name: &str) -> Option<SubstratePrompt> {
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
    let prompt_def = get_prompt(&request.name).ok_or_else(|| McpError {
        code: rmcp::model::ErrorCode::INVALID_PARAMS,
        message: format!("Unknown prompt: {}", request.name).into(),
        data: None,
    })?;

    let empty_map = serde_json::Map::new();
    let args = request.arguments.as_ref().unwrap_or(&empty_map);

    // Add security disclaimer to args for templates that need it
    let mut enriched_args = args.clone();

    // For security-related prompts, inject the security disclaimer
    if matches!(
        request.name.as_str(),
        "automated_analysis"
            | "code_security_audit"
            | "economic_security"
            | "pallet_incentive_analysis"
            | "threat_modeling"
            | "weight_analysis"
    ) {
        enriched_args.insert("security_disclaimer".to_string(), get_security_disclaimer());
    }

    // For code_security_audit, validate audit_type if provided and no specific_checks
    if request.name == "code_security_audit" {
        if let Some(audit_type) = enriched_args.get("audit_type") {
            if enriched_args.get("specific_checks").is_none() {
                let valid_types = ["pallet", "runtime", "node", "general"];
                if let Some(audit_type_str) = audit_type.as_str() {
                    if !valid_types.contains(&audit_type_str) {
                        return Err(McpError {
                            code: rmcp::model::ErrorCode::INVALID_PARAMS,
                            message: format!(
                                "Invalid audit_type '{}'. Must be one of: pallet, runtime, node, general",
                                audit_type_str
                            ).into(),
                            data: None,
                        });
                    }
                }
            }
        }
    }

    // Render the template
    let prompt = TEMPLATE_REGISTRY
        .render(&request.name, &enriched_args)
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Failed to render {} template: {}", request.name, e).into(),
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

