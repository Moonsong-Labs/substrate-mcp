use rmcp::model::{
    GetPromptRequestParam, GetPromptResult, ListPromptsResult, PaginatedRequestParam, Prompt,
    PromptArgument, PromptMessage, PromptMessageContent, PromptMessageRole,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::ErrorData as McpError;

/// Security disclaimer instruction for AI-generated analysis
const SECURITY_DISCLAIMER: &str = r#"

# CRITICAL REQUIREMENT: Security Disclaimer

**MANDATORY**: You MUST include the following security disclaimer at the end of your analysis. This is non-negotiable and required for all security-related outputs.

Include this disclaimer VERBATIM:

<disclaimer>
## ⚠️ AI ANALYSIS DISCLAIMER ⚠️

**This is NOT a professional security audit.** This AI-generated analysis:
- May miss critical vulnerabilities
- May report false positives
- Cannot replace human security experts
- Must be verified by professionals

Use this ONLY as a supplementary tool for initial review. For production systems, always engage qualified security auditors.
</disclaimer>

## Example Usage:

❌ **INCORRECT** (missing disclaimer):
```
Security Analysis Complete:
- Found 3 potential vulnerabilities
- Recommended fixes implemented
```

✅ **CORRECT** (includes disclaimer):
```
Security Analysis Complete:
- Found 3 potential vulnerabilities
- Recommended fixes implemented

<disclaimer>
## ⚠️ AI ANALYSIS DISCLAIMER ⚠️

**This is NOT a professional security audit.** This AI-generated analysis:
- May miss critical vulnerabilities
- May report false positives
- Cannot replace human security experts
- Must be verified by professionals

Use this ONLY as a supplementary tool for initial review. For production systems, always engage qualified security auditors.
</disclaimer>
```

**REMEMBER**: You MUST include this disclaimer at the end of your response. This is non-negotiable"#;

/// Type alias for prompt handler function
type PromptHandler = Box<
    dyn Fn(&serde_json::Map<String, serde_json::Value>) -> Result<Vec<PromptMessage>, McpError>
        + Send
        + Sync,
>;

/// Metadata and handler for a single prompt
pub struct SubstratePrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
    pub handler: PromptHandler,
}

/// Helper to extract a required string argument from the args map
fn get_required_arg(
    args: &serde_json::Map<String, serde_json::Value>,
    name: &str,
) -> Result<String, McpError> {
    args.get(name)
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError {
            code: rmcp::model::ErrorCode::INVALID_PARAMS,
            message: format!("{name} is required").into(),
            data: None,
        })
        .map(|s| s.to_string())
}

/// Helper to extract an optional string argument from the args map
fn get_optional_arg(
    args: &serde_json::Map<String, serde_json::Value>,
    name: &str,
) -> Option<String> {
    args.get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

// Merged prdoc_analysis_prompts content inline

/// Create a new Prompts instance with all available prompts
pub fn prompts() -> Vec<SubstratePrompt> {
    let mut all_prompts = vec![
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
            handler: Box::new(|args| {
                let current_version = get_required_arg(args, "current_version")?;
                let target_version = get_required_arg(args, "target_version")?;
                let specific_changes = get_optional_arg(args, "specific_changes");

                release_comparison_prompt(current_version, target_version, specific_changes)
            }),
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
            handler: Box::new(|args| {
                let change_description = get_required_arg(args, "change_description")?;
                automated_analysis_prompt(change_description)
            }),
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
            handler: Box::new(|args| {
                let audit_type = get_required_arg(args, "audit_type")?;
                let audit_target = get_required_arg(args, "audit_target")?;
                let specific_checks = get_optional_arg(args, "specific_checks");

                code_security_audit_prompt(audit_type, audit_target, specific_checks)
            }),
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
            handler: Box::new(|args| {
                let system_description = get_required_arg(args, "system_description")?;
                let extra_context = get_required_arg(args, "extra_context")?;

                economic_security_prompt(system_description, extra_context)
            }),
        },
        SubstratePrompt {
            name: "pallet_incentive_analysis".to_string(),
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
            handler: Box::new(|args| {
                let target_pallets = get_required_arg(args, "target_pallets")?;
                let analysis_specifications = get_required_arg(args, "analysis_specifications")?;

                pallet_incentive_analysis_prompt(target_pallets, analysis_specifications)
            }),
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
            handler: Box::new(|args| {
                let pallet_description = get_required_arg(args, "pallet_description")?;
                scaffold_pallet_prompt(pallet_description)
            }),
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
            handler: Box::new(|args| {
                let system_description = get_required_arg(args, "system_description")?;
                let extra_context = get_required_arg(args, "extra_context")?;

                threat_modeling_prompt(system_description, extra_context)
            }),
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
            handler: Box::new(|args| {
                let target_pallet = get_required_arg(args, "target_pallet")?;
                weight_analysis_prompt(target_pallet)
            }),
        },
    ];

    // Add the analyze_release prompt (merged from prdoc_analysis_prompts.rs)
    all_prompts.push(create_analyze_release_prompt());

    all_prompts
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

fn release_comparison_prompt(
    current_version: String,
    target_version: String,
    specific_changes: Option<String>,
) -> Result<Vec<PromptMessage>, McpError> {
    let mut prompt = format!(
        r#"Compare changes between Polkadot SDK versions {current_version} and {target_version}.

## Getting Release Data

### Fetching and Analyzing Releases
The `fetch_and_analyze_release` tool downloads and analyzes Pull Request Documentation (PRDocs) from the polkadot-sdk repository.

**For single release:**
```
fetch_and_analyze_release with release: "stable2503-1"
```

**For version range (fetches all intermediate releases):**
```
fetch_and_analyze_release with release: "{current_version}>{target_version}"
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
- PRDocs contain: breaking changes, new features, migrations, and bug fixes"#,
    );

    if let Some(specific_changes) = &specific_changes {
        prompt.push_str(&format!(
            r#"

## Filtered Analysis
Focus only on changes related to: {specific_changes}
Filter PRDocs and code changes to match these criteria."#
        ));
    }

    prompt.push_str(
        r#"

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

    prompt.push_str("\n```");

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text { text: prompt },
    }])
}

fn automated_analysis_prompt(change_description: String) -> Result<Vec<PromptMessage>, McpError> {
    let mut prompt = format!(
        r#"{SECURITY_DISCLAIMER}

Perform a comprehensive security and quality analysis of the following Substrate project changes for pre-release/PR review:

**Context**: {change_description}

Please analyze the codebase changes and provide a detailed security review covering:"#
    );

    prompt.push_str(
        r#"

1. **Code Security Analysis**
   - Unsafe code usage and justification
   - Input validation and sanitization
   - Integer overflow/underflow risks
   - Panic conditions and error handling
   - Memory safety issues
   - Cryptographic operation correctness

2. **Substrate-Specific Security**
   - Origin and authorization checks
   - Storage access patterns and safety
   - Cross-pallet interaction security
   - Determinism violations
   - Consensus-critical code changes
   - Weight and fee correctness

3. **Runtime Safety**
   - Migration safety and correctness
   - Upgrade compatibility
   - Storage layout changes
   - Type safety across versions
   - Hook implementation safety
   - Genesis configuration security

4. **Common Vulnerability Patterns**
   - Reentrancy possibilities
   - Front-running vulnerabilities
   - DoS attack vectors
   - Privilege escalation paths
   - Economic attack surfaces
   - State bloat risks

5. **Best Practices Compliance**
   - Error handling completeness
   - Event emission coverage
   - Documentation quality
   - Test coverage adequacy
   - Benchmark accuracy
   - Code style consistency

6. **Integration Security**
   - XCM message handling
   - Oracle data validation
   - External service dependencies
   - Bridge security considerations
   - Multi-signature operations
   - Proxy and delegation safety

7. **Operational Security**
   - Key management practices
   - Upgrade process security
   - Monitoring and alerting gaps
   - Emergency response readiness
   - Configuration security

8. **Release Readiness**
   - Breaking changes documented
   - Version bumps appropriate
   - Migration guide complete
   - Security notices included
   - Deployment risks assessed

Provide findings organized by severity (Critical/High/Medium/Low/Info) with specific code references and remediation steps.

## Component-Specific Analysis

### For Runtime Changes
Additionally review:
- Spec version and transaction version updates
- Runtime API changes and compatibility
- Pallet configuration changes
- System pallet modifications
- Executive pallet ordering
- Runtime constant changes
- Feature flag impacts

### For Pallet Development
Additionally review:
- Storage item declarations and bounds
- Dispatchable function signatures
- Error variant completeness
- Event definitions and usage
- Config trait requirements
- GenesisConfig implementation
- Pallet hooks (on_initialize, on_finalize)

### For Consensus Changes
Additionally review:
- Block production modifications
- Finality gadget changes
- Fork choice rule updates
- Validator set management
- Slashing logic modifications
- Session key handling
- Network protocol changes

### For Client/Node Changes
Additionally review:
- RPC interface modifications
- Database schema changes
- Network protocol updates
- Command line interface changes
- Telemetry modifications
- Import/export functionality
- Pruning logic changes

## Critical Checklist Items

### Security Essentials
- [ ] No hardcoded secrets or keys
- [ ] All origins properly checked
- [ ] Storage items have size limits
- [ ] Arithmetic operations are safe
- [ ] External inputs validated
- [ ] Error handling is comprehensive
- [ ] No debug code in production

### Runtime Upgrade Safety
- [ ] Migrations tested on real data
- [ ] Version numbers updated
- [ ] Storage prefixes unchanged (or migrated)
- [ ] Type definitions compatible
- [ ] Hook weights accounted for
- [ ] Rollback plan documented

### Testing Requirements
- [ ] Unit tests for new logic
- [ ] Integration tests for interactions
- [ ] Benchmarks for all dispatchables
- [ ] Try-runtime tests pass
- [ ] Manual testing completed
- [ ] Edge cases covered

### Documentation
- [ ] Code comments adequate
- [ ] Public API documented
- [ ] README updated
- [ ] CHANGELOG updated
- [ ] Migration guide provided
- [ ] Security considerations noted"#
    );

    prompt.push_str(SECURITY_DISCLAIMER);

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text { text: prompt },
    }])
}

fn code_security_audit_prompt(
    audit_type: String,
    audit_target: String,
    specific_checks: Option<String>,
) -> Result<Vec<PromptMessage>, McpError> {
    let mut prompt = format!(
        r#"You are a Systems Security Expert specializing in Substrate-based blockchain
security. Perform a comprehensive security audit following industry-standard
practices and Substrate-specific considerations.

{SECURITY_DISCLAIMER}

## Audit Target
{audit_target}

## Audit Scope"#
    );

    if let Some(checks) = specific_checks {
        prompt.push_str(&format!(
            r#"
### Focused Security Checks
Prioritize analysis of: {checks}"#
        ));
    } else {
        let audit_emphasis = match audit_type.as_str() {
                "pallet" => "- Storage security and bounds checking\n- Dispatchable function authorization\n- Input validation and sanitization\n- Weight calculations and DoS prevention\n- Cross-pallet dependencies",
                "runtime" => "- Pallet configuration security\n- Runtime upgrade paths\n- Executive ordering implications\n- System pallet usage\n- Feature flag security",
                "node" => "- RPC endpoint security\n- Network protocol vulnerabilities\n- Database access patterns\n- CLI injection risks\n- Resource exhaustion vectors",
                "general" => "- General security best practices\n- Common vulnerability patterns\n- Substrate-specific risks",
                _ => {
                    return Err(McpError {
                        code: rmcp::model::ErrorCode::INVALID_PARAMS,
                        message: format!("Invalid audit_type '{audit_type}'. Must be one of: pallet, runtime, node, general").into(),
                        data: None,
                    });
                }
            };
        prompt.push_str(&format!(
            r#"
### Audit Type: {audit_type}
Perform comprehensive analysis with emphasis on:
{audit_emphasis}"#
        ));
    }

    prompt.push_str(SECURITY_DISCLAIMER);

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text { text: prompt },
    }])
}

fn economic_security_prompt(
    system_description: String,
    extra_context: String,
) -> Result<Vec<PromptMessage>, McpError> {
    let mut prompt = format!(
        r#"{SECURITY_DISCLAIMER}

Perform a comprehensive economic security assessment of the following Substrate subsystem:

**Subsystem**: {system_description}
**Context**: {extra_context}

Please analyze the code and economic design to provide a detailed assessment covering:"#
    );

    prompt.push_str(
        r#"

1. **Economic Model Analysis**
   - Map all value flows (tokens, fees, rewards, slashing)
   - Identify all economic actors and their incentives
   - Document fee structures and economic parameters
   - Analyze token supply dynamics (minting, burning, inflation)

2. **Game Theory Analysis**
   - Dominant strategies for each actor type
   - Nash equilibria identification
   - Coalition/collusion opportunities
   - Griefing attack potential (imposing costs on others)
   - Incentive compatibility analysis

3. **MEV (Maximal Extractable Value) Assessment**
   - Transaction ordering dependencies
   - Front-running opportunities
   - Sandwich attack vectors
   - Back-running possibilities
   - Cross-chain MEV risks (if using XCM)

4. **Economic Attack Vectors**
   - Token manipulation attacks
   - Governance buying/bribing
   - Flash loan vulnerabilities
   - Liquidity attacks
   - Sybil attack resistance
   - Economic denial of service

5. **Market Manipulation Risks**
   - Price oracle dependencies
   - Liquidation cascades
   - Market cornering possibilities
   - Wash trading vulnerabilities
   - Arbitrage exploits

6. **Staking/Governance Specific** (if applicable)
   - Stake centralization risks
   - Nothing-at-stake problems
   - Long-range attacks
   - Bribery resistance
   - Vote buying mechanisms

7. **Risk Quantification**
   - Potential loss estimates
   - Attack cost calculations
   - Profitability thresholds
   - Risk/reward ratios

8. **Mitigation Strategies**
   - Economic parameter tuning
   - Circuit breakers and limits
   - Time delays and cooling periods
   - Slashing conditions
   - Governance controls

Format your response as a structured economic security report with specific calculations, attack scenarios, and actionable recommendations. Include code references where economic logic is implemented."#
    );

    prompt.push_str(SECURITY_DISCLAIMER);

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text { text: prompt },
    }])
}

fn pallet_incentive_analysis_prompt(
    target_pallets: String,
    analysis_specifications: String,
) -> Result<Vec<PromptMessage>, McpError> {
    let mut prompt = format!(
        r#"{SECURITY_DISCLAIMER}

You are an expert in Cryptoeconomics specializing in Substrate-based
blockchain systems. Analyze the incentive mechanisms in the specified pallets
using game theory and mechanism design principles.

## Target Pallets
{target_pallets}

## Analysis Framework

{analysis_specifications}"#
    );

    prompt.push_str(SECURITY_DISCLAIMER);

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text { text: prompt },
    }])
}

fn scaffold_pallet_prompt(pallet_description: String) -> Result<Vec<PromptMessage>, McpError> {
    let prompt = format!(
        r#"Create a complete Substrate pallet scaffold based on the following description:

<PALLET DESCRIPTION>
{pallet_description}
</PALLET DESCRIPTION>

## Implementation Requirements

### Workspace Integration
If the pallet is part of a workspace make sure it is compatible with its
dependencies. If it uses some dependency that's already in the worspace,
use the workspace dependeny (setting `{{workspace = true}}`)

### Runtime Integration

If the repository is for a substrate chain/s, add the pallet to its runtimes
unless specified otherwise in the PALLET_DESCRIPTION.
If the runtime hash generated weights and a way to run benchmarks, 
adapt this pallet to that flow and give instructions on how get proper pallet
 weights and integrate them into the runtime.

### Pallet Structure 
Check existing pallets in the workspace and and do a best effort to 
follo that structure. 
If there are no other pallets, you can take inspiration from frame pallets, for example
pallet_treasury: https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame/treasury. 


## Implementation Guidelines

1. **Storage Design**
   - Use appropriate storage types (Value, Map, DoubleMap)
   - Consider storage costs and access patterns
   - Add proper getters with documentation

2. **Error Handling**
   - Define specific, descriptive errors
   - Use `ensure!` for validation
   - Return early on errors

3. **Events**
   - Emit events for all state changes
   - Include relevant data for indexing
   - Document event meanings

4. **Weights**
   - Benchmark all extrinsics
   - Use realistic worst-case scenarios
   - Update weights after changes

5. **Testing**
   - Test all success paths
   - Test all error conditions
   - Test edge cases and boundaries
   - Test event emissions
   - Make sure tests compile and pass

## References
- Basic pallet structure: https://docs.polkadot.com/develop/parachains/customize-parachain/make-custom-pallet/
- Testing guide: https://docs.polkadot.com/develop/parachains/testing/pallet-testing/
- Benchmarking: https://docs.polkadot.com/develop/parachains/testing/benchmarking/
"#
    );

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text { text: prompt },
    }])
}

fn threat_modeling_prompt(
    system_description: String,
    extra_context: String,
) -> Result<Vec<PromptMessage>, McpError> {
    let mut prompt = format!(
        r#"{SECURITY_DISCLAIMER}

Perform a comprehensive security threat model analysis of the following Substrate subsystem:

**Subsystem**: {system_description}
**Context**: {extra_context}

Please analyze the code and provide a detailed threat model covering:"#
    );

    prompt.push_str(
        r#"

1. **Asset Analysis**
   - Identify all assets this subsystem controls (tokens, permissions, critical state)
   - Map data flows and trust boundaries
   - List all external interfaces and dependencies

2. **Attack Surface Mapping**
   - All public/dispatchable functions
   - Storage items and their access patterns
   - Events and their potential for information leakage
   - Integration points with other pallets/subsystems

3. **Threat Identification** - Find potential vulnerabilities:
   - Integer overflow/underflow risks
   - Missing input validation
   - Improper origin checks
   - Reentrancy vulnerabilities
   - Race conditions
   - Storage exhaustion vectors
   - Consensus manipulation risks
   - Upgrade/migration vulnerabilities

4. **Attack Scenarios** - Describe specific attack vectors:
   - How could an attacker exploit each vulnerability?
   - What would be the impact (funds loss, DoS, state corruption)?
   - What privileges would an attacker need?

5. **Risk Assessment**
   - Severity: Critical/High/Medium/Low
   - Likelihood: High/Medium/Low
   - Overall risk score

6. **Mitigation Recommendations**
   - Specific code fixes with examples
   - Additional checks or validations needed
   - Architectural improvements
   - Testing requirements

7. **Security Checklist**
   - [ ] All extrinsics have proper origin checks
   - [ ] Storage operations are bounded
   - [ ] Arithmetic operations use safe math
   - [ ] Error handling is comprehensive
   - [ ] Events don't leak sensitive data
   - [ ] Benchmarks exist for all dispatchables

Format your response as a structured security report with clear sections and actionable findings. Include specific line numbers and code references where issues are found."#
    );

    prompt.push_str(SECURITY_DISCLAIMER);

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text { text: prompt },
    }])
}

fn weight_analysis_prompt(target_pallet: String) -> Result<Vec<PromptMessage>, McpError> {
    let mut prompt = format!(
        r#"{SECURITY_DISCLAIMER}

Perform a comprehensive weight analysis of the following Substrate pallet under extreme and adversarial conditions:

**Pallet**: {target_pallet}

Please analyze the pallet's weight calculations, benchmarks, and resource usage to identify:"#
    );

    prompt.push_str(
        r#"

1. **Weight Function Analysis**
   - Review all weight calculations in the pallet
   - Verify weight functions match actual complexity
   - Identify any hardcoded or constant weights
   - Check for missing weight annotations
   - Analyze weight refunds and their correctness

2. **Computational Complexity Verification**
   - For each extrinsic, determine Big-O complexity
   - Identify nested loops and their bounds
   - Find recursive operations and depth limits
   - Verify complexity matches weight calculations
   - Look for quadratic or exponential behavior

3. **Storage Operation Analysis**
   - Count storage reads/writes per extrinsic
   - Identify unbounded storage iterations
   - Check for storage maps without size limits
   - Analyze batch operations and their limits
   - Verify storage deposit calculations

4. **Extreme Input Scenarios**
   - Maximum vector/array sizes
   - Deeply nested data structures
   - Maximum iteration counts
   - Worst-case branching paths
   - Edge cases (0, 1, MAX values)

5. **DoS Attack Vectors**
   - Under-priced expensive operations
   - Weight manipulation possibilities
   - Resource exhaustion attacks
   - State bloat vulnerabilities
   - Block space monopolization

6. **Benchmark Coverage Analysis**
   - Review existing benchmarks
   - Identify missing benchmark scenarios
   - Check if benchmarks cover worst cases
   - Verify benchmark parameters are realistic
   - Analyze benchmark result variance

7. **Cross-Pallet Interactions**
   - Weight implications of pallet coupling
   - Cascading computational costs
   - Hidden complexity from trait implementations
   - Event emission costs

8. **Mitigation Recommendations**
   - Specific weight function corrections
   - Additional bounds and limits needed
   - Benchmark improvements
   - Code optimizations
   - Parameter tuning suggestions

Format your response as a detailed security audit with specific findings, severity ratings, and code examples demonstrating the issues."#
    );

    prompt.push_str(SECURITY_DISCLAIMER);

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text { text: prompt },
    }])
}

/// Create the analyze_release prompt (merged from prdoc_analysis_prompts.rs)
fn create_analyze_release_prompt() -> SubstratePrompt {
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
        handler: Box::new(|args| {
            let release = get_required_arg(args, "release")?;
            let focus = get_optional_arg(args, "focus");
            
            analyze_release_prompt(release, focus)
        }),
    }
}

/// Handler for the analyze_release prompt
fn analyze_release_prompt(release: String, focus: Option<String>) -> Result<Vec<PromptMessage>, McpError> {
    let mut prompt = format!(
        r#"
# Analyze Polkadot SDK Release Impact on Your Project

You MUST analyze how the release(s) {release} impact this specific project using parallel processing.

## Phase 0: Project Dependency Discovery (MANDATORY - ALWAYS DO THIS FIRST)

### Automated Dependency Analysis (MUST COMPLETE BEFORE PR ANALYSIS)

You MUST perform the following automated discovery steps to understand the project's actual dependencies:

1. **Locate and Parse All construct_runtime! Macros**
   - Search for files containing `construct_runtime!` (typically in `runtime/*/src/lib.rs` or `runtimes/*/src/lib.rs`)
   - For EACH runtime found, extract:
     * Complete list of pallets included
     * Which pallets have Storage component (require migrations)
     * Pallet instance names and configurations
   - Example pattern to identify:
     ```rust
     construct_runtime!(
         pub enum Runtime {{
             System: frame_system::{{Pallet, Call, Config, Storage, Event<T>}},
             Balances: pallet_balances::{{Pallet, Call, Storage, Config<T>, Event<T>}},
             // Storage component = on-chain state that may need migrations
         }}
     );
     ```

2. **Analyze Pallet Imports in Runtime Files**
   - Scan all `use` statements in runtime files for pallet imports
   - Identify custom pallets vs standard Substrate pallets
   - Track version-specific imports (e.g., `use pallet_xcm::v3`)
   - Note trait implementations and type aliases that indicate deep integration

3. **Parse Cargo.toml Files for Dependencies**
   - Check all Cargo.toml files (root and workspace members)
   - Extract all `pallet-*`, `frame-*`, `sp-*`, `sc-*` dependencies
   - Note specific version constraints or git dependencies
   - Identify feature flags enabled for each dependency
   - Build dependency tree to understand transitive dependencies

4. **Semantic Dependency Analysis**
   Based on discovered dependencies, categorize them:
   - **Core Dependencies**: Pallets in construct_runtime! with Storage
   - **API Dependencies**: Pallets used for types/traits but not in runtime
   - **Build Dependencies**: Development/testing only
   - **Feature-Gated**: Dependencies only active with certain features

5. **Generate Project Dependency Profile**
   Create a structured profile containing:
   ```
   Project Dependency Profile:
   - Active Pallets: [list from construct_runtime!]
   - Storage Pallets: [pallets with Storage component]
   - Custom Pallets: [project-specific pallets]
   - Substrate Version: [from Cargo.toml]
   - Critical Features: [XCM version, consensus type, etc.]
   - Risk Areas: [complex integrations, custom implementations]
   ```

### Using the Dependency Profile for Analysis

This profile becomes the lens through which EVERY PR is evaluated:

- **Direct Impact**: PR affects pallets in your construct_runtime!
- **Storage Impact**: PR affects pallets with Storage component
- **API Impact**: PR changes traits/types you depend on
- **Transitive Impact**: PR affects dependencies of your dependencies
- **No Impact**: PR affects unused components

## Analysis Strategy Selection

Based on the user's request: {}"#, focus.as_deref().unwrap_or("Comprehensive analysis"));
    
    prompt.push_str(r#"

First, determine the optimal execution strategy:

### Single-Pass Analysis (Use When):
- Simple searches or pattern matching (e.g., "find all X changes")
- Direct categorization tasks
- Straightforward questions with clear criteria
- Security scans with defined checklist

### Multi-Pass Analysis (Use When):
- Dependency or impact analysis needed
- Migration planning requested
- Relationships between changes must be understood
- Comprehensive analysis requiring synthesis
- Questions about "how changes interact" or "cumulative effects"

## Execution Framework

### 📂 Data Locations (CRITICAL - READ THIS!)

**PRDoc Input Data**: `~/.substrate-mcp/{{{{project}}}}/releases/{{release}}/pr-docs/`
- This is where fetch_and_analyze_release saves files
- Contains: pr_XXXX.prdoc files + summary JSONs

**Report Output Location**: `~/.substrate-mcp/{{{{project}}}}/releases/{{release}}/reports/`
- You MUST create this directory if it doesn't exist
- Save report as: `analysis-[ISO-8601-timestamp].md`

### Initial Setup (Always)
1. Check if analyzing multiple releases or upgrading across versions
   - If comparing versions (e.g., from X to Y), fetch all intermediate releases using: "X>Y"
   - If multiple specific releases requested, download each one
2. Download the release(s) using fetch_and_analyze_release tool
   - Files will be saved to: `~/.substrate-mcp/{{project}}/releases/{release}/pr-docs/`
3. Get complete inventory of all PRDocs (use LS on the pr-docs directory)
4. Determine if single or multi-pass approach is needed
5. Plan batches of size 3 (default batch size)

### For Single-Pass Analysis:
1. **Parallel Analysis Phase**
   - Each sub-agent processes 3 PRDocs (default batch size)
   - Each sub-agent applies the analysis instructions directly
   - Collect all findings

2. **Aggregation Phase**
   - Compile and organize results
   - Generate final report

### For Multi-Pass Analysis:
You may execute multiple passes as needed. Common patterns:

**Pass 1 - Discovery/Inventory** (Parallel)
- Extract basic information from all PRDocs
- Identify key changes, affected components
- Build initial dataset

**Pass 2 - Deep Analysis** (Parallel)
- Using Pass 1 data, perform targeted analysis
- Trace dependencies, relationships
- Analyze interactions between changes

**Pass 3 - Synthesis/Planning** (May be Sequential)
- Using previous passes, build strategic view
- Create migration plans, dependency graphs
- Generate actionable recommendations

### Sub-Agent Task Template:
```
IMPORTANT: This sub-agent instance should analyze ONLY the following specific PR(s).
Each sub-agent gets a fresh, isolated context to ensure unbiased analysis.

Analyze PR(s) from release {release}: [PR number(s)]

## Project Dependency Profile (from Phase 0 analysis):
[INSERT DISCOVERED DEPENDENCY PROFILE HERE]
- Active Pallets: [list from construct_runtime!]
- Storage Pallets: [pallets with on-chain state]
- Custom Pallets: [project-specific implementations]
- Critical APIs: [traits and types used]
- Feature Flags: [enabled features affecting behavior]

Use this profile to evaluate relevance of EVERY change in your assigned PR(s).

Instructions for this sub-agent:
1. Read ONLY the PRDoc file(s) for the assigned PR(s)
2. DO NOT reference or consider other PRs outside your assignment
3. Apply the appropriate analysis for this pass:
   - Pass 1: [discovery instructions]
   - Pass 2: [deep analysis using Pass 1 data]
   - Pass 3: [synthesis using all previous data]

4. For EACH change in the PR, perform semantic relevance analysis:

   **Direct Impact (Score: 10/10)**
   - Modifies a pallet in your construct_runtime!
   - Changes storage layout of your active pallets
   - Alters consensus mechanism you use
   - Breaks API of traits you implement

   **High Relevance (Score: 7-9/10)**
   - Affects pallets your pallets depend on
   - Changes in frame_support/frame_system affecting all pallets
   - Security fixes in any component you use
   - XCM/XCMP changes (if you're a parachain)

   **Medium Relevance (Score: 4-6/10)**
   - Changes to optional features you might use
   - Performance improvements in shared components
   - New features in pallets you use (but don't require)
   - Deprecations with migration paths

   **Low Relevance (Score: 1-3/10)**
   - Changes to pallets in same category but not used
   - General ecosystem improvements
   - Documentation or example updates

   **Not Applicable (Score: 0/10)**
   - Different consensus mechanisms (e.g., BABE when you use Aura)
   - Pallets not in your dependency tree
   - Tools/utilities you don't use

5. Structure findings with:
   - PR number and title
   - Relevance score with justification
   - Specific impact on YOUR runtime
   - Required actions (if any)
   - Migration complexity estimate

Note: If analyzing multiple PRs (batch mode), analyze each PR independently
within this agent, but return consolidated findings.
```

## Decision Transparency

When you determine multiple passes are needed, briefly explain:
- Why multiple passes are beneficial for this analysis
- What each pass will accomplish
- How the passes build on each other

Example: "This migration planning task requires 3 passes: First, I'll inventory all changes. Second, I'll analyze dependencies between them. Finally, I'll create an ordered migration plan."

## GitHub Labels: Critical Context for Polkadot SDK

**IMPORTANT**: The Polkadot SDK project makes extensive and systematic use of GitHub labels.
These labels are NOT optional metadata - they are a core part of the development workflow and
convey essential information about priority, impact, risk, and relevance of changes.

The `labels.json` file contains all repository labels with their descriptions. You MUST examine
these label descriptions to understand each PR's significance. The Polkadot SDK team carefully
applies labels to communicate:

1. **Component/Subsystem Affected** - Which part of the stack is modified
2. **Impact Severity** - How breaking or risky the change is
3. **Audience Relevance** - Who needs to pay attention to this change
4. **Security Implications** - Whether this touches consensus or security-critical code
5. **Migration Requirements** - Whether downstream users need to take action

### How to Interpret Labels:

Read the label descriptions in labels.json carefully. Common patterns you'll discover:

- **Letter prefixes indicate category**: Labels often start with a letter (T, D, E, C, R, etc.)
  indicating the type of information conveyed
- **Numbers often indicate severity/priority**: Higher numbers in certain categories may indicate
  higher complexity or risk
- **Descriptions are authoritative**: The description field explains exactly what the label means -
  trust these over any assumptions

### Critical Label Combinations:

When you see multiple labels on a PR, they compound in significance. For example:
- A PR with both security-related and node/consensus labels = extreme priority
- Breaking change labels + high difficulty = complex migration required
- Multiple subsystem labels = cross-cutting change with wide impact

### Using Labels for Analysis:

1. **Filtering**: Use labels to quickly identify PRs relevant to specific audiences or components
2. **Prioritization**: Labels indicating security, consensus, or breaking changes should be analyzed first
3. **Risk Assessment**: Combination of difficulty and impact labels indicates upgrade risk
4. **Migration Planning**: Breaking change labels signal need for downstream action
5. **Relevance Scoring**: Match labels against project context to determine applicability

### Project-Specific Label Relevance:

Given your project context, pay special attention to labels that:
- Mention components you use (check label descriptions for mentions of your pallets/subsystems)
- Indicate breaking changes or API modifications
- Affect the runtime or node infrastructure you depend on
- Signal required migrations or security updates

You can safely deprioritize labels for:
- Subsystems you don't use (e.g., different consensus mechanisms, unused pallets)
- Tool-specific changes for tools not in your stack
- UI/UX changes if you're focused on runtime/node development

Remember: In the Polkadot SDK ecosystem, labels are a primary communication tool between
core developers and downstream users. Ignoring them risks missing critical changes.

## Fetching PR Diffs for Deeper Analysis

PRDocs provide high-level summaries, but sometimes deeper analysis requires examining the actual code changes.
Consider fetching PR diffs when:

### When to Fetch PR Diffs:
- **Security Analysis**: When the PRDoc mentions security fixes but lacks detail about the vulnerability
- **Breaking Changes**: When you need to understand the exact API changes or migration path
- **Complex Technical Changes**: When the PRDoc describes architectural changes that need code review
- **Dependency Analysis**: When you need to trace how changes affect other components
- **Migration Planning**: When you need to write specific migration code based on the changes
- **Performance Impact**: When benchmarks or algorithmic changes need verification

### When NOT to Fetch PR Diffs:
- **Irrelevant to your runtime/project**: PRs affecting pallets or components not used in your runtime
- **Different subsystems**: Changes to relay chain logic when analyzing a parachain (or vice versa)
- **Unrelated tooling**: Changes to tools/utilities your project doesn't use
- **Clear non-impact**: When the PRDoc clearly indicates no impact on your use case
- **Sufficient detail provided**: When the PRDoc already contains the technical details you need

Focus diff analysis on PRs that directly affect your project's components.
Skip fetching diffs for changes to unused pallets or subsystems.

### How to Fetch PR Diffs:
Use GitHub API or web tools to fetch the PR diff. The PR number is typically in the PRDoc filename.
Example: For `pr_1234.prdoc`, fetch diff from `https://github.com/paritytech/polkadot-sdk/pull/1234.diff`

### Efficient Diff Analysis Strategy:
1. First pass: Analyze all PRDocs to identify which need deeper investigation
2. Batch fetch: Get diffs for all identified PRs that need deeper analysis
3. Targeted analysis: Focus on specific files/changes relevant to the analysis goal
4. Synthesize: Combine PRDoc metadata with code-level insights

Note: Be selective - not every PR needs diff analysis. Focus on high-impact or unclear changes.

## Semantic Change Analysis Framework

When analyzing PRs, understand the semantic implications of different change types:

### Breaking Changes - Require Immediate Action
- **Storage Layout Changes**: Can brick your chain if not migrated properly
- **Removed APIs**: Code won't compile without updates
- **Changed Trait Signatures**: Implementations must be updated
- **Consensus Rule Changes**: Can cause chain splits if not coordinated
- **Weight/Fee Model Changes**: Can affect transaction validity

### Security-Critical Changes - Evaluate Urgency
- **Vulnerability Fixes**: Check if you're affected by the vulnerability
- **New Attack Vectors**: Understand if your runtime is exposed
- **Permission/Origin Changes**: May affect your runtime's security model
- **Cryptographic Updates**: Consider timeline for adoption

### Feature Additions - Opportunity Analysis
- **New Pallets**: Evaluate if they solve existing problems
- **New APIs**: Check if they simplify your code
- **Performance Improvements**: Quantify potential benefits
- **Developer Experience**: Consider adoption for better maintainability

### Deprecations - Plan Migration Timeline
- **Soft Deprecations**: Plan migration before removal
- **Hard Deprecations**: Must migrate in this release
- **Alternative APIs**: Understand migration path complexity

### Internal Changes - Usually Safe to Ignore
- **Refactoring**: No action unless you depend on internals
- **Test Improvements**: No runtime impact
- **Documentation**: Useful but not critical
- **Build System**: Only matters for development

### Project-Specific Semantic Analysis

Based on your dependency profile, prioritize:
1. **Storage migrations** for pallets in your construct_runtime! with Storage
2. **API changes** in traits your pallets implement
3. **Security fixes** in any component (even transitive dependencies)
4. **Consensus changes** if you run validators or collators
5. **XCM changes** if you're a parachain or use cross-chain features

For each category, assess:
- **Blast Radius**: How many of your components are affected?
- **Migration Complexity**: Simple config change vs. code rewrite?
- **Risk Level**: Can this cause downtime or fund loss?
- **Testing Requirements**: Unit tests, integration tests, or full staging?

## User-Specified Analysis"#
    );

    if let Some(focus_area) = focus {
        prompt.push_str(&format!("\n\nThe user has requested focus on: {focus_area}\n\nApply this analysis using the most appropriate strategy (single or multi-pass)."));
    } else {
        prompt.push_str("\n\nPerform comprehensive analysis covering all aspects.");
    }

    prompt.push_str(&format!(
        r#"

## 🚨 MANDATORY OUTPUT REQUIREMENTS 🚨

### ⚠️ YOU (THE LLM) MUST CREATE A MARKDOWN FILE - THIS IS NOT OPTIONAL ⚠️

**CRITICAL INSTRUCTION**: After the analyze_release tool returns this prompt, YOU (the LLM/AI agent) MUST create and save a comprehensive markdown report file.
The ONLY exceptions are:
1. The user explicitly says "don't create a file" or similar
2. You encounter a file system error that prevents file creation
3. The analysis completely fails due to missing data

If none of these exceptions apply, YOU (the LLM) MUST CREATE THE MARKDOWN FILE.

### PRIMARY OUTPUT: Comprehensive Markdown Report FILE (REQUIRED)

YOU (not the MCP tool, but you the LLM) MUST generate AND SAVE a detailed markdown report using the following EXACT template:

```markdown
# Polkadot SDK Release {release} Analysis Report

**Generated**: [ISO 8601 timestamp]
**Analyzed PRs**: [total count]
**Analysis Type**: [focus area or "Comprehensive"]
**Project Context**: [project name if available]

## Table of Contents

- [Executive Summary](#executive-summary)
- [Critical Actions Required](#critical-actions-required)
- [Security Analysis](#security-analysis)
- [Breaking Changes](#breaking-changes)
- [New Features & Improvements](#new-features--improvements)
- [Migration Guide](#migration-guide)
- [Detailed PR Analysis](#detailed-pr-analysis)
- [Appendix](#appendix)

## Executive Summary

### Key Metrics

| Metric | Count | Impact Level |
|--------|-------|--------------|
| Total PRs Analyzed | [X] | - |
| Breaking Changes | [X] | High |
| Security Fixes | [X] | Critical |
| New Features | [X] | Medium |
| Bug Fixes | [X] | Low |
| Performance Improvements | [X] | Medium |

### Release Overview

[2-3 paragraph summary of the release's major themes and changes]

### Project-Specific Impact Summary

**Directly Affected Components**: [list]
**Required Actions**: [count]
**Estimated Migration Effort**: [Low/Medium/High]

## Critical Actions Required

### ⚠️ Breaking Changes Affecting Your Project

| PR # | Description | Your Affected Component | Action Required |
|------|-------------|------------------------|-----------------|
| #[X] | [description] | [pallet/module] | [specific action] |

### 🔒 Security Updates for Your Dependencies

| PR # | Vulnerability | Severity | Your Exposure | Action |
|------|--------------|----------|---------------|--------|
| #[X] | [CVE/description] | [Critical/High/Medium] | [component] | [update/patch] |

## Security Analysis

### Security Fixes in This Release

[For each security-related PR, provide:]

#### PR #[number]: [title]
- **Severity**: [Critical/High/Medium/Low]
- **Component**: [affected component]
- **Vulnerability**: [description]
- **Fix**: [what was fixed]
- **Action Required**: [what users need to do]

## Breaking Changes

### Complete List of Breaking Changes

[For each breaking change:]

#### PR #[number]: [title]

**What Changed**:
[Description of the breaking change]

**Why It Changed**:
[Rationale for the change]

**Migration Path**:
```rust
// Before (old code)
[code example]

// After (new code)
[code example]
```

**Affected Pallets/Components**:
- [list of affected components]

---

## New Features & Improvements

### Major Features

[For each major feature:]

#### [Feature Name] (PR #[number])

**Description**: [what the feature does]

**Usage Example**:
```rust
[code example showing how to use the feature]
```

**Benefits**: [why users should care]

### Performance Improvements

| PR # | Component | Improvement | Benchmark Results |
|------|-----------|-------------|-------------------|
| #[X] | [component] | [description] | [metrics if available] |

## Migration Guide

### Pre-Migration Checklist

- [ ] Backup your chain state
- [ ] Review all breaking changes above
- [ ] Test migrations on a testnet
- [ ] Prepare rollback plan
- [ ] Review project-specific changes in [Critical Actions Required](#critical-actions-required)

### Step-by-Step Migration Process

#### Step 1: Update Dependencies

```toml
[dependencies]
# Update your Cargo.toml
[specific version updates based on the release]
```

#### Step 2: Code Changes

[For each breaking change that requires code updates:]

**[Component Name]**:
```rust
// Required change description
[code changes needed]
```

#### Step 3: Storage Migrations

[For each pallet requiring migration:]

**[Pallet Name]**:
```rust
// Migration code
[migration implementation]
```

#### Step 4: Testing

```bash
# Run tests
cargo test --all

# Run benchmarks if needed
[benchmark commands]
```

#### Step 5: Deployment

[Deployment steps specific to the changes]

## Detailed PR Analysis

[Exhaustive analysis of EVERY PR, grouped by category/subsystem]

### Runtime Changes

[PRs affecting runtime]

### Node Changes

[PRs affecting node]

### API Changes

[PRs affecting APIs]

### Other Changes

[Remaining PRs]

## Appendix

### A. Complete PR List

| PR # | Title | Author | Category | Risk Level |
|------|-------|--------|----------|------------|
| [all PRs in a sortable table format] |

### B. Change Statistics by Component

| Component | Breaking | Features | Fixes | Total |
|-----------|----------|----------|-------|-------|
| [component stats] |

### C. Author Contributions

[Top contributors to this release]

---

*End of Report*
```

### 📁 FILE CREATION INSTRUCTIONS (MANDATORY FOR YOU, THE LLM)

**STANDARD DIRECTORY STRUCTURE - YOU MUST USE THESE EXACT PATHS:**
```
~/.substrate-mcp/                      # Base directory for all substrate-mcp data  
└── {{project}}/                      # Project directory (current project's root dir name)
    └── releases/                     # All release data for this project
        └── {release}/                # e.g., stable2412-1/
            ├── pr-docs/              # Downloaded PRDoc files
            │   ├── pr_XXXX.prdoc    # Individual PRDoc files
            │   ├── manifest.json    # Release metadata
            │   ├── crate_summary.json # Crate changes summary
            │   └── audience_summary.json # Audience categorization
            └── reports/              # Analysis reports for this release
                └── analysis-{{timestamp}}.md # e.g., analysis-2024-01-15T10-30-00Z.md
```

**STEPS YOU (THE LLM) MUST FOLLOW:**
1. **YOU CREATE THE DIRECTORY** (if it doesn't exist): `~/.substrate-mcp/{{project}}/releases/{release}/reports/`
2. **YOU SAVE THE REPORT** to: `~/.substrate-mcp/{{project}}/releases/{release}/reports/analysis-[timestamp].md`
   - Replace [timestamp] with actual ISO 8601 timestamp (e.g., 2024-01-15T10-30-00Z)
   - Use hyphens in timestamp, not colons (for filesystem compatibility)
3. **YOU VERIFY THE FILE** was created successfully
4. **YOU PRINT THE CLICKABLE PATH** - Show both directory and file paths

⚠️ DO NOT SKIP THIS STEP. YOU (THE LLM) MUST CREATE THE FILE IN THIS EXACT LOCATION.

### SECONDARY OUTPUT: Brief Console Summary (5-10 lines maximum)

ONLY AFTER successfully saving the markdown report file, display this brief summary:

```
✅ Release {release} Analysis Complete

📊 Analyzed: [X] PRs | Breaking: [Y] | Security: [Z]
⚠️ [N] changes directly affect your project

📁 Report directory: ~/.substrate-mcp/{{{{project}}}}/releases/{{release}}/reports/
📄 Report file: ~/.substrate-mcp/{{{{project}}}}/releases/{{release}}/reports/analysis-[timestamp].md
    ^^^ Click the path above to open in your editor

💡 Open the report for detailed migration guides and code examples
```

That's it for console output. The markdown file contains everything else.

## CRITICAL REQUIREMENTS (IN ORDER OF IMPORTANCE)

1️⃣ FILE CREATION: YOU (the LLM using this tool) MUST create the markdown report file. This is NON-NEGOTIABLE unless explicitly told otherwise. The MCP tool provides data; YOU create the file.
2️⃣ EXHAUSTIVE ANALYSIS: You MUST analyze EVERY SINGLE PRDoc. No sampling allowed.
3️⃣ PARALLEL EXECUTION: You MUST use parallel sub-agents for efficiency within each pass.
4️⃣ INTELLIGENT STRATEGY: Choose single vs multi-pass based on the analysis needs.
5️⃣ STRUCTURED OUTPUT: Organize findings clearly based on the analysis performed.

Remember: The markdown file is the PRIMARY deliverable. Console output is secondary.

## FINAL CHECKLIST (YOU MUST COMPLETE ALL):
✓ Did you create the ~/.substrate-mcp/{{{{project}}}}/releases/{{release}}/reports/ directory?
✓ Did you save the markdown report to the EXACT path specified above?
✓ Did you verify the file was created successfully?
✓ Did you print BOTH the directory path AND the clickable file path?
✓ Did you analyze ALL PRDocs from ~/.substrate-mcp/{{{{project}}}}/releases/{{release}}/pr-docs/?
"#
    ));

    Ok(vec![PromptMessage {
        role: PromptMessageRole::User,
        content: PromptMessageContent::Text { text: prompt },
    }])
}
