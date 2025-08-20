use rmcp::model::PromptArgument;

use super::SubstratePromptDefinition;

pub fn prompt_definition() -> SubstratePromptDefinition {
    SubstratePromptDefinition {
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
        template: TEMPLATE.to_string(),
    }
}

/// Release comparison prompt template
const TEMPLATE: &str = r#"Compare changes between Polkadot SDK versions {{current_version}} and {{target_version}}.

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
```"#;
