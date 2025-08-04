use indoc::indoc;
use rmcp::model::PromptArgument;

/// Custom prompt structure that includes instructions
pub struct ReleasePrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
    pub instructions: String,
}

/// Prompts specifically for release analysis
pub fn get_release_prompts() -> Vec<ReleasePrompt> {
    vec![
        ReleasePrompt {
            name: "analyze-release-for-runtime-dev".to_string(),
            description: "Analyze a Polkadot SDK release from a runtime developer's perspective".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "release".to_string(),
                    description: Some("The release version (e.g., stable2412-1)".to_string()),
                    required: Some(true),
                }
            ],
            instructions: indoc! {r#"
                You are analyzing Polkadot SDK release {{release}} for a runtime developer.
                
                First, use the get_polkadot_sdk_release_prdocs tool to download the PRDocs for this release.
                Then use Read to examine the manifest files (manifest.json, crate_summary.json, audience_summary.json).
                
                Focus on:
                1. **Breaking Changes**: Identify all breaking changes that affect runtime development
                2. **Pallet Updates**: List all pallet changes with their bump levels (major/minor/patch)
                3. **Migration Requirements**: Find all PRs that require runtime migrations
                4. **New Features**: Highlight new features available for runtime developers
                5. **Deprecations**: Note any deprecated functionality
                
                For each significant change:
                - Provide the PR number
                - Explain what changed
                - Show migration path if applicable
                - Include code examples where helpful
                
                Organize your response by priority:
                - Critical (breaking changes, required migrations)
                - Important (major version bumps, new features)
                - Minor (patches, improvements)
            "#}.to_string(),
        },
        
        ReleasePrompt {
            name: "generate-migration-guide".to_string(),
            description: "Generate a migration guide for upgrading between releases".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "from_release".to_string(),
                    description: Some("The release upgrading from".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "to_release".to_string(),
                    description: Some("The release upgrading to".to_string()),
                    required: Some(true),
                }
            ],
            instructions: indoc! {r#"
                Generate a comprehensive migration guide for upgrading from {{from_release}} to {{to_release}}.
                
                Use the analyze_release tool to analyze {{to_release}} comprehensively.
                
                Structure the guide as follows:
                
                ## Overview
                - Summary of major changes
                - Risk assessment
                - Estimated upgrade complexity
                
                ## Pre-upgrade Checklist
                - [ ] Backup current state
                - [ ] Review breaking changes
                - [ ] Test migrations on testnet
                - [ ] Update dependencies
                
                ## Breaking Changes
                For each breaking change:
                - What broke
                - Why it changed
                - How to fix it
                - Code examples (before/after)
                
                ## Runtime Migrations
                For each pallet requiring migration:
                - Pallet name
                - Migration steps
                - Code snippets
                - Testing approach
                
                ## Step-by-Step Upgrade Process
                1. Update dependencies in Cargo.toml
                2. Apply code changes
                3. Run migrations
                4. Test thoroughly
                
                ## Post-upgrade Validation
                - Tests to run
                - Metrics to monitor
                - Rollback procedure
                
                Include specific code examples and commands throughout.
            "#}.to_string(),
        },
        
        ReleasePrompt {
            name: "security-audit-release".to_string(),
            description: "Perform a security-focused analysis of a release".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "release".to_string(),
                    description: Some("The release to audit".to_string()),
                    required: Some(true),
                }
            ],
            instructions: indoc! {r#"
                Perform a security audit of Polkadot SDK release {{release}}.
                
                Use both get_polkadot_sdk_release_prdocs and analyze_release tools.
                
                Focus on:
                
                ## Critical Security Areas
                1. **Consensus Changes**: Any modifications to consensus mechanisms
                2. **Cryptography**: Changes to crypto primitives or signatures
                3. **Weight/Fee Changes**: Modifications that could enable DoS
                4. **Permission/Authorization**: Changes to access control
                5. **Storage Security**: New storage items or access patterns
                
                ## Analysis Approach
                - Search for PRs with keywords: security, vulnerability, CVE, fix, patch
                - Review all consensus-related changes
                - Check for emergency patches or hotfixes
                - Identify changes to critical pallets (staking, balances, democracy)
                
                ## Output Format
                For each security-relevant change:
                - Severity: Critical/High/Medium/Low
                - PR Number and Title
                - Security Impact
                - Recommended Actions
                - Testing Requirements
                
                Conclude with:
                - Overall security assessment
                - High-priority items requiring immediate attention
                - Recommended security testing plan
            "#}.to_string(),
        },
        
        ReleasePrompt {
            name: "find-specific-changes".to_string(),
            description: "Find specific types of changes in a release".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "release".to_string(),
                    description: Some("The release to search".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "search_type".to_string(),
                    description: Some("Type of changes to find (e.g., 'storage', 'rpc', 'weights', 'benchmarks')".to_string()),
                    required: Some(true),
                }
            ],
            instructions: indoc! {r#"
                Search for {{search_type}} related changes in release {{release}}.
                
                First download PRDocs, then use the analysis data to find relevant changes.
                
                Search strategies by type:
                - **storage**: Look for StorageMap, StorageValue, storage migrations
                - **rpc**: Search for RPC method changes, JSON-RPC updates
                - **weights**: Find weight updates, benchmarking changes
                - **benchmarks**: Look for new or updated benchmarks
                - **api**: Find API changes, trait modifications
                
                For each found change:
                1. PR number and title
                2. Affected components
                3. Nature of change (added/modified/removed)
                4. Impact assessment
                5. Example usage if applicable
                
                Group results by:
                - Pallet/Component
                - Severity of change
                - Action required (none/monitor/update code)
            "#}.to_string(),
        },
        
        ReleasePrompt {
            name: "parachain-upgrade-impact".to_string(),
            description: "Analyze release impact on parachain development".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "release".to_string(),
                    description: Some("The release to analyze".to_string()),
                    required: Some(true),
                }
            ],
            instructions: indoc! {r#"
                Analyze the impact of release {{release}} on parachain development.
                
                Focus on:
                
                ## Cumulus Changes
                - Updates to cumulus pallets
                - XCM version changes
                - Parachain consensus updates
                
                ## XCMP/XCM Updates
                - New XCM instructions
                - Breaking changes in cross-chain messaging
                - Fee changes
                
                ## Parachain Runtime
                - Required pallet updates
                - New parachain features
                - Validation function changes
                
                ## Collator Updates
                - Collator software changes
                - Network protocol updates
                - Performance improvements
                
                For each area:
                1. List all relevant PRs
                2. Assess upgrade urgency (critical/high/medium/low)
                3. Provide upgrade path
                4. Note any coordination required with relay chain
                
                Create a prioritized action plan for parachain teams.
            "#}.to_string(),
        }
    ]
}

/// Get a specific release prompt by name
pub fn get_release_prompt(name: &str) -> Option<ReleasePrompt> {
    get_release_prompts()
        .into_iter()
        .find(|p| p.name == name)
}

/// Get all release prompt names
pub fn list_release_prompt_names() -> Vec<String> {
    get_release_prompts()
        .into_iter()
        .map(|p| p.name)
        .collect()
}