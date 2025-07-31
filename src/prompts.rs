use rmcp::model::{
    GetPromptRequestParam, GetPromptResult, ListPromptsResult, PaginatedRequestParam, Prompt,
    PromptArgument, PromptMessage, PromptMessageContent, PromptMessageRole,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ErrorData as McpError;

/// Metadata and handler for a single prompt
pub struct SubstratePrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
    pub handler: Box<
        dyn Fn(&serde_json::Map<String, serde_json::Value>) -> Result<Vec<PromptMessage>, McpError>
            + Send
            + Sync,
    >,
}


/// Create a new Prompts instance with all available prompts
pub fn prompts() -> Vec<SubstratePrompt> {
        vec![SubstratePrompt {
            name: "release_comparison".to_string(),
            description: "List changes between two polkadot-sdk release versions".to_string(),
            arguments: 
                vec![
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
            handler: Box::new(|args| {
                let current_version = args
                    .get("current_version")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError {
                        code: rmcp::model::ErrorCode::INVALID_PARAMS,
                        message: "current_version is required".to_string().into(),
                        data: None,
                    })?
                    .to_string();

                let target_version = args
                    .get("target_version")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError {
                        code: rmcp::model::ErrorCode::INVALID_PARAMS,
                        message: "target_version is required".to_string().into(),
                        data: None,
                    })?
                    .to_string();

                let specific_changes = args
                    .get("specific_changes")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                release_comparison_prompt(
                    current_version,
                    target_version,
                    specific_changes,
                )
            }),
        }]
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
    let messages = (prompt_def.handler)(args)?;

    Ok(GetPromptResult {
        messages,
        description: Some(prompt_def.description),
    })
}

fn release_comparison_prompt(current_version: String, target_version: String, specific_changes: Option<String>) -> Result<Vec<PromptMessage>, McpError> {
    let mut prompt = format!(
        r#"Compare changes between Polkadot SDK versions {current_version} and {target_version}.

## Tools and Resources
- Use substrate_mcp's `get_polkadot_sdk_release_prdocs` tool to fetch release documentation
- Reference the polkadot-sdk repository: https://github.com/paritytech/polkadot-sdk
- PRDocs contain release notes with breaking changes, new features, and important updates

## Version Naming Conventions
1. **Semantic versions**: Follow standard semver (e.g., 1.2.3)
2. **Stable releases**: Format `stableYYMM[-patch]`
   - YYMM represents year and month (e.g., 2503 = March 2025)
   - Optional -X suffix for patches (e.g., stable2503-1)

Stable releases were implemented later than semantic versions so oldest stable
release comes after newer semantic versioned release.

## Version Range Logic

### Same Base Version (Different Patches)
If comparing patches of the same release (e.g., stable2503 → stable2503-4):
- Include ALL intermediate patches in sequence
- Example: stable2503 → stable2503-4 requires:
  - stable2503-1
  - stable2503-2
  - stable2503-3
  - stable2503-4

### Different Base Versions
If comparing different releases (e.g., stable2502 → stable2503-2):
1. Include ALL patches from the newer base version up to target
2. Include the base release notes for intermediate versions
3. Example: stable2502 → stable2503-2 requires:
   - stable2503 (base release)
   - stable2503-1
   - stable2503-2"#,
    );

    if let Some(specific_changes) = &specific_changes {
        prompt.push_str(&format!(
            r#"

## Filtered Analysis
Focus only on changes related to: {specific_changes}
Filter PRDocs and code changes to match these criteria."#,
        ));
    }

    prompt.push_str(
        r#"

## Output Format

### Changes by Category

#### 🚨 Breaking Changes
Changes requiring code updates:
- **[Component]**: Description of breaking change
  - Migration guide: Steps to update
  - Affected versions: [version list]

#### ✨ New Features
New functionality added:
- **[Component]**: Feature description
  - First available in: [version]

#### 🐛 Bug Fixes
Issues resolved:
- **[Component]**: Fix description
  - Fixed in: [version]

#### 🔧 Improvements
Performance and quality improvements:
- **[Component]**: Improvement description

### Detailed Version Progression
For each version in sequence:

**[Version Number]**
- Release date: [if available]
- Key changes:
  - [Change 1]
  - [Change 2]
- Full PRDoc reference: [link or doc ID]

### Migration Recommendations
Based on the changes between versions:
1. **High Priority**: [Critical updates needed]
2. **Medium Priority**: [Recommended updates]
3. **Low Priority**: [Optional improvements]"#,
    );

    if specific_changes.is_none() {
        prompt.push_str(
            r#"

### Additional Notes
- Changes not covered by PRDocs may exist in the codebase
- Review CHANGELOG.md files for complete details"#,
        );
    }

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text { text: prompt },
    }])
}
