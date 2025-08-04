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
{pallet_description}

## Project Structure
Create the following file structure:
```
pallets/<pallet_name>/
├── Cargo.toml
├── src/
│   ├── lib.rs         # Main pallet logic
│   ├── mock.rs        # Test runtime setup
│   ├── tests.rs       # Unit tests
│   ├── benchmarking.rs # Benchmarks
│   └── weights.rs     # Auto-generated weights
└── README.md          # Pallet documentation
```

## Implementation Requirements

### 1. Core Pallet Structure (`src/lib.rs`)
```rust
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {{
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {{
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
        
        // Add other configuration parameters based on <pallet_description>
    }}

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Storage items based on pallet requirements
    #[pallet::storage]
    #[pallet::getter(fn example_storage)]
    pub type ExampleStorage<T> = StorageValue<_, u32, ValueQuery>;

    /// Events emitted by the pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {{
        /// Event documentation
        SomethingHappened {{ who: T::AccountId, value: u32 }},
    }}

    /// Errors that can be returned by this pallet
    #[pallet::error]
    pub enum Error<T> {{
        /// Error documentation
        InvalidInput,
        InsufficientPermission,
        // Add errors based on <pallet_description>
    }}

    /// Dispatchable functions (extrinsics)
    #[pallet::call]
    impl<T: Config> Pallet<T> {{
        /// Documentation for the extrinsic
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::example_extrinsic())]
        pub fn example_extrinsic(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {{
            let who = ensure_signed(origin)?;
            
            // Validation
            ensure!(value > 0, Error::<T>::InvalidInput);
            
            // State changes
            <ExampleStorage<T>>::put(value);
            
            // Emit event
            Self::deposit_event(Event::SomethingHappened {{ who, value }});
            
            Ok(())
        }}
    }}

    /// Helper functions (private)
    impl<T: Config> Pallet<T> {{
        fn helper_function() -> Result<(), Error<T>> {{
            // Implementation
            Ok(())
        }}
    }}
}}

/// Weight information trait
pub trait WeightInfo {{
    fn example_extrinsic() -> Weight;
}}

/// Default weight implementation
impl WeightInfo for () {{
    fn example_extrinsic() -> Weight {{
        Weight::from_parts(10_000, 0)
    }}
}}
```

### 2. Mock Runtime (`src/mock.rs`)
```rust
use crate as pallet_template;
use frame_support::{{
    parameter_types,
    traits::{{ConstU16, ConstU64}},
}};
use sp_core::H256;
use sp_runtime::{{
    traits::{{BlakeTwo256, IdentityLookup}},
    BuildStorage,
}};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {{
        System: frame_system,
        TemplateModule: pallet_template,
    }}
);

parameter_types! {{
    pub const BlockHashCount: u64 = 250;
}}

impl frame_system::Config for Test {{
    // System config implementation
}}

impl pallet_template::Config for Test {{
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}}

pub fn new_test_ext() -> sp_io::TestExternalities {{
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}}
```

### 3. Unit Tests (`src/tests.rs`)
Include tests for:
- ✅ Happy path scenarios
- ❌ Error conditions
- 🔒 Permission checks
- 📊 State changes
- 📢 Event emissions

```rust
use super::*;
use crate::{{mock::*, Error}};
use frame_support::{{assert_noop, assert_ok}};

#[test]
fn example_extrinsic_works() {{
    new_test_ext().execute_with(|| {{
        // Arrange
        let caller = 1;
        let value = 42;
        
        // Act
        assert_ok!(TemplateModule::example_extrinsic(
            RuntimeOrigin::signed(caller),
            value
        ));
        
        // Assert
        assert_eq!(TemplateModule::example_storage(), value);
        System::assert_last_event(
            Event::SomethingHappened {{ who: caller, value }}.into()
        );
    }});
}}

#[test]
fn example_extrinsic_fails_with_invalid_input() {{
    new_test_ext().execute_with(|| {{
        assert_noop!(
            TemplateModule::example_extrinsic(RuntimeOrigin::signed(1), 0),
            Error::<Test>::InvalidInput
        );
    }});
}}
```

### 4. Benchmarks (`src/benchmarking.rs`)
```rust
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{{benchmarks, whitelisted_caller}};
use frame_system::RawOrigin;

benchmarks! {{
    example_extrinsic {{
        let caller: T::AccountId = whitelisted_caller();
        let value = 100u32;
    }}: _(RawOrigin::Signed(caller.clone()), value)
    verify {{
        assert_eq!(ExampleStorage::<T>::get(), value);
    }}

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}}
```

### 5. Cargo.toml
```toml
[package]
name = "pallet-<pallet_name>"
version = "0.1.0"
authors = ["Your Name"]
edition = "2021"

[dependencies]
codec = {{ package = "parity-scale-codec", version = "3.6.1", default-features = false }}
scale-info = {{ version = "2.10.0", default-features = false, features = ["derive"] }}
frame-support = {{ default-features = false, git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }}
frame-system = {{ default-features = false, git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }}
frame-benchmarking = {{ default-features = false, optional = true, git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }}

[dev-dependencies]
sp-core = {{ git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }}
sp-io = {{ git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }}
sp-runtime = {{ git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }}

[features]
default = ["std"]
std = [
    "codec/std",
    "scale-info/std",
    "frame-support/std",
    "frame-system/std",
    "frame-benchmarking?/std",
]
runtime-benchmarks = [
    "frame-benchmarking/runtime-benchmarks",
    "frame-support/runtime-benchmarks",
    "frame-system/runtime-benchmarks",
]
```

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

## References
- Basic pallet structure: https://docs.polkadot.com/develop/parachains/customize-parachain/make-custom-pallet/
- Testing guide: https://docs.polkadot.com/develop/parachains/testing/pallet-testing/
- Benchmarking: https://docs.polkadot.com/develop/parachains/testing/benchmarking/"#
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
