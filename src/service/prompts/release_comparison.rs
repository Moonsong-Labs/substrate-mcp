//! Release comparison prompt implementation

use handlebars::Handlebars;
use rmcp::model::{GetPromptResult, PromptMessage, PromptMessageRole};
use rmcp::ErrorData as McpError;
use serde_json::json;

use super::common::SECURITY_DISCLAIMER;
use super::types::ReleaseComparisonArgs;

/// Generate release comparison prompt content
pub async fn generate_prompt(args: ReleaseComparisonArgs) -> Result<GetPromptResult, McpError> {
    let handlebars = Handlebars::new();

    let context = json!({
        "current_version": args.current_version,
        "target_version": args.target_version,
        "specific_changes": args.specific_changes,
        "security_disclaimer": SECURITY_DISCLAIMER
    });

    let content = handlebars
        .render_template(TEMPLATE, &context)
        .map_err(|e| McpError::internal_error(format!("Template rendering failed: {}", e), None))?;

    let description = handlebars
      .render_template(
          "Compare changes between Polkadot SDK versions {{current_version}} and {{target_version}}", 
          &context
      )
      .map_err(|e| McpError::internal_error(format!("Description template rendering failed: {}", e), None))?;

    Ok(GetPromptResult {
        description: Some(description),
        messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
    })
}

/// Release comparison prompt template
const TEMPLATE: &str = r#"{{security_disclaimer}}

Compare changes between Polkadot SDK versions {{current_version}} and {{target_version}}.

## Getting Release Data

### Fetching and Analyzing Releases
The `fetch_and_analyze_release` tool downloads and analyzes Pull Request Documentation (PRDocs) from the polkadot-sdk repository.

**For single release:**
```
fetch_and_analyze_release with release: "stable2503-1"
```

**For version range (fetches all intermediate releases):**
```
fetch_and_analyze_release with release: "{{current_version}}>{{target_version}}"
```
Example: `stable2502>stable2503-2` fetches stable2503, stable2503-1, and stable2503-2

The tool will:
1. Download all PRDocs from GitHub for the specified release(s)
2. Generate analysis summaries (manifest.json, crate_summary.json, audience_summary.json)
3. Organize files in `~/.substrate-mcp/[project]/releases/[release]/pr-docs/`

### Version Format Guide
- **Semantic versions**: Standard format like `1.9.0`, `1.10.2`
- **Stable releases**: Format `stableYYMM[-patch]` where:
  - YYMM = year and month (e.g., 2503 = March 2025)
  - Optional patch suffix (e.g., stable2503-1, stable2503-2)
  
Note: Stable releases began after semantic versioning, so v1.x.x releases predate stable releases.

### Resources
- Repository: https://github.com/paritytech/polkadot-sdk
- PRDocs contain: breaking changes, new features, migrations, and bug fixes

{{#if specific_changes}}

## Filtered Analysis
Focus only on changes related to: {{specific_changes}}
Filter PRDocs and code changes to match these criteria.
{{/if}}

## Output Format

```markdown
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
3. **Low Priority**: [Optional improvements]

{{#unless specific_changes}}
### Additional Notes
- Changes not covered by PRDocs may exist in the codebase
- Review CHANGELOG.md files for complete details
{{/unless}}
```

{{security_disclaimer}}"#;
