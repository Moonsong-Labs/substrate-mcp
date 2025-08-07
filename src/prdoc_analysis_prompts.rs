use indoc::indoc;
use rmcp::model::PromptArgument;

/// Analysis prompt structure for PRDoc analysis
pub struct AnalysisPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
    pub instructions: String,
    pub requires_parallel_agents: bool,
    pub agent_batch_size: Option<usize>,
}

/// Get all PRDoc analysis prompts that leverage parallel sub-agents
pub fn get_analysis_prompts() -> Vec<AnalysisPrompt> {
    vec![
        // Generic parallel analysis prompt
        AnalysisPrompt {
            name: "parallel-release-analysis".to_string(),
            description: "Generic framework for parallel analysis of all PRDocs in a release with custom analysis instructions".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "release".to_string(),
                    description: Some("The release version(s) to analyze (e.g., stable2503-7 or comma-separated list)".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "analysis_instructions".to_string(),
                    description: Some("Detailed instructions for what to analyze and how (e.g., 'Check for security issues including unsafe code, panic conditions, consensus impact')".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "batch_size".to_string(),
                    description: Some("Number of PRs each sub-agent processes. Use 1 for true isolation (one agent per PR), 2-4 for batched processing (default: 3)".to_string()),
                    required: Some(false),
                },
                PromptArgument {
                    name: "project_context".to_string(),
                    description: Some("Optional: Describe your project context to get targeted analysis. Examples: 'Moonbeam parachain using Frontier EVM, cumulus, XCM v3' or 'Asset Hub runtimes (Polkadot and Kusama) using assets, uniques pallets' or 'Solo substrate chain with contracts pallet'. For multi-runtime projects, specify which runtime(s) you're analyzing.".to_string()),
                    required: Some(false),
                }
            ],
            requires_parallel_agents: true,
            agent_batch_size: Some(3), // Default, overridden by user's batch_size
            instructions: indoc! {r#"
                # Parallel Release Analysis Framework
                
                You MUST analyze EVERY PRDoc in release {{release}} using parallel processing for efficiency.
                
                ## Project Context Assessment
                
                {{#if project_context}}
                Project context provided: {{project_context}}
                
                Before analyzing, identify:
                - Key dependencies and components used in this project
                - Relevant audiences (e.g., parachain teams need Runtime Dev + Node Dev changes)
                - Specific subsystems of interest (e.g., EVM pallets, XCM, consensus)
                
                For maximum accuracy, consider examining your runtime's construct_runtime! macro(s)
                (typically in runtime/*/src/lib.rs or runtimes/*/src/lib.rs) which show:
                - All pallets actually included in each runtime
                - Which pallets have Storage (on-chain state that needs migrations)
                - The exact configuration of each pallet
                
                Example:
                ```rust
                construct_runtime!(
                    pub enum Runtime {
                        System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
                        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
                        // Note: Storage component shows this pallet has on-chain state
                    }
                );
                ```
                
                Note: If you have multiple runtimes (e.g., production/canary/testnet), check each one
                as they may use different pallets or configurations.
                
                Use this context to score relevance throughout the analysis.
                {{else}}
                No project context provided. Performing comprehensive analysis of all changes.
                💡 Tip: Provide project_context parameter for targeted, project-specific insights.
                {{/if}}
                
                ## Analysis Strategy Selection
                
                Based on the user's request: {{analysis_instructions}}
                
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
                
                ### Initial Setup (Always)
                1. Check if analyzing multiple releases or upgrading across versions
                   - If comparing versions (e.g., from X to Y), fetch all intermediate releases using: "X>Y"
                   - If multiple specific releases requested, download each one
                2. Download the release(s) using get_polkadot_sdk_release_prdocs tool
                3. Get complete inventory of all PRDocs (use LS to list them)
                4. Determine if single or multi-pass approach is needed
                5. Plan batches of size {{batch_size}} (or 3 if not specified)
                
                ### For Single-Pass Analysis:
                1. **Parallel Analysis Phase**
                   - When batch_size=1: Each PR gets its own isolated sub-agent (maximum isolation)
                   - When batch_size>1: Each sub-agent processes {{batch_size}} PRDocs (faster but shared context)
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
                
                Analyze PR(s) from release {{release}}: [PR number(s)]
                
                Instructions for this sub-agent:
                1. Read ONLY the PRDoc file(s) for the assigned PR(s)
                2. DO NOT reference or consider other PRs outside your assignment
                3. Apply the appropriate analysis for this pass:
                   - Pass 1: [discovery instructions]
                   - Pass 2: [deep analysis using Pass 1 data]  
                   - Pass 3: [synthesis using all previous data]
                {{#if project_context}}
                4. For each finding, assess relevance to the project:
                   - **Directly Affects**: Changes to components used by the project
                   - **Indirect Impact**: Ecosystem changes that may affect the project
                   - **Not Applicable**: Changes to components not used by the project
                5. Return structured findings with relevance scores
                {{else}}
                4. Return structured findings
                {{/if}}
                
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
                
                {{#if project_context}}
                ### Project-Specific Label Relevance:
                
                Given your project context ({{project_context}}), pay special attention to labels that:
                - Mention components you use (check label descriptions for mentions of your pallets/subsystems)
                - Indicate breaking changes or API modifications
                - Affect the runtime or node infrastructure you depend on
                - Signal required migrations or security updates
                
                You can safely deprioritize labels for:
                - Subsystems you don't use (e.g., different consensus mechanisms, unused pallets)
                - Tool-specific changes for tools not in your stack
                - UI/UX changes if you're focused on runtime/node development
                {{/if}}
                
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
                
                {{#if project_context}}
                Focus diff analysis on PRs that directly affect your project's components: {{project_context}}
                Skip fetching diffs for changes to unused pallets or subsystems.
                {{/if}}
                
                ### How to Fetch PR Diffs:
                Use GitHub API or web tools to fetch the PR diff. The PR number is typically in the PRDoc filename.
                Example: For `pr_1234.prdoc`, fetch diff from `https://github.com/paritytech/polkadot-sdk/pull/1234.diff`
                
                ### Efficient Diff Analysis Strategy:
                1. First pass: Analyze all PRDocs to identify which need deeper investigation
                2. Batch fetch: Get diffs for all identified PRs that need deeper analysis
                3. Targeted analysis: Focus on specific files/changes relevant to the analysis goal
                4. Synthesize: Combine PRDoc metadata with code-level insights
                
                Note: Be selective - not every PR needs diff analysis. Focus on high-impact or unclear changes.
                
                ## User-Specified Analysis
                
                The user has requested: {{analysis_instructions}}
                
                Apply this analysis using the most appropriate strategy (single or multi-pass).
                
                ## Output Requirements
                
                1. Confirm 100% coverage (all PRDocs analyzed)
                2. Provide organized findings based on the analysis type
                3. Include summary statistics
                4. For multi-pass: Show how insights evolved across passes
                5. Deliver actionable conclusions
                {{#if project_context}}
                6. Organize findings by relevance score:
                   - **Directly Affects Your Project**: Detailed analysis of high-impact changes
                   - **Indirect/Ecosystem Impact**: Summary of relevant ecosystem changes
                   - **Not Applicable**: Brief listing of changes that don't affect your project
                7. Provide project-specific recommendations
                {{/if}}
                
                ## CRITICAL REQUIREMENTS
                
                ⚠️ EXHAUSTIVE ANALYSIS: You MUST analyze EVERY SINGLE PRDoc. No sampling allowed.
                ⚠️ PARALLEL EXECUTION: You MUST use parallel sub-agents for efficiency within each pass.
                ⚠️ INTELLIGENT STRATEGY: Choose single vs multi-pass based on the analysis needs.
                ⚠️ STRUCTURED OUTPUT: Organize findings clearly based on the analysis performed.
                
                Remember: The goal is complete, efficient, and thorough analysis. The method (single or multi-pass) should serve this goal.
            "#}.to_string(),
        },
        
        // Keep existing prompts temporarily for backward compatibility
        AnalysisPrompt {
            name: "exhaustive-release-analysis".to_string(),
            description: "Perform exhaustive analysis of all PRDocs in a release using parallel sub-agents".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "release".to_string(),
                    description: Some("The release version to analyze (e.g., stable2503-7)".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "analysis_depth".to_string(),
                    description: Some("Level of analysis: quick|standard|deep".to_string()),
                    required: Some(false),
                }
            ],
            requires_parallel_agents: true,
            agent_batch_size: Some(2),
            instructions: indoc! {r#"
                # Exhaustive Release Analysis Using Parallel Sub-Agents
                
                You must analyze EVERY PRDoc in release {{release}} using parallel processing for efficiency.
                
                ## Phase 1: Initial Indexing (Already Complete)
                The MCP tool automatically performs basic indexing when downloading PRDocs.
                
                ## Phase 2: Parallel Deep Analysis
                You MUST spawn sub-agents to analyze PRDocs in parallel:
                - Process {{agent_batch_size}} PRDocs at a time
                - Each sub-agent should analyze their assigned PRDoc for:
                  1. Technical impact and breaking changes
                  2. Security implications
                  3. Migration requirements
                  4. Cross-pallet dependencies
                  5. Performance implications
                
                ### Sub-Agent Task Template
                ```
                Analyze PR {{pr_number}} from release {{release}}:
                1. Read the PRDoc file
                2. Identify the type of change (bug fix, feature, breaking change)
                3. Assess technical impact on:
                   - Runtime developers
                   - Node operators
                   - Parachain teams
                4. Note any security considerations
                5. Identify migration requirements
                6. Return structured findings
                ```
                
                ## Phase 3: Synthesis and Strategic Analysis
                After all sub-agents complete, synthesize findings:
                1. Group related changes
                2. Identify patterns and themes
                3. Create dependency graph of changes
                4. Prioritize by impact and risk
                5. Generate actionable recommendations
                
                ## Output Format
                ```markdown
                # Release {{release}} Exhaustive Analysis
                
                ## Executive Summary
                - Total PRs analyzed: X
                - Critical changes: Y
                - Security-relevant: Z
                
                ## Critical Changes Requiring Action
                [Prioritized list with rationale]
                
                ## Change Categories
                ### Breaking Changes
                [Grouped by component]
                
                ### Security Updates
                [With severity ratings]
                
                ### Performance Improvements
                [With benchmarks if available]
                
                ## Migration Strategy
                [Step-by-step plan]
                
                ## Risk Assessment
                [Overall upgrade risk analysis]
                ```
                
                Remember: This is EXHAUSTIVE analysis - every PR must be examined.
            "#}.to_string(),
        },
        
        AnalysisPrompt {
            name: "cross-release-compatibility-check".to_string(),
            description: "Check compatibility between multiple releases using parallel analysis".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "releases".to_string(),
                    description: Some("Comma-separated list of releases to check".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "focus_area".to_string(),
                    description: Some("Specific area to focus on (optional)".to_string()),
                    required: Some(false),
                }
            ],
            requires_parallel_agents: true,
            agent_batch_size: Some(3),
            instructions: indoc! {r#"
                # Cross-Release Compatibility Analysis
                
                Analyze compatibility across releases: {{releases}}
                
                ## Parallel Analysis Strategy
                Spawn sub-agents to analyze each release independently, then compare:
                
                ### Sub-Agent Tasks
                For each release, analyze:
                1. API surfaces and changes
                2. Storage layout modifications
                3. Consensus-critical changes
                4. Network protocol updates
                5. Dependency version changes
                
                ### Compatibility Matrix
                After parallel analysis, build:
                - API compatibility matrix
                - Storage migration requirements
                - Network upgrade coordination needs
                - Feature flag dependencies
                
                ## Focus Area: {{focus_area}}
                If specified, deep-dive into this area across all releases.
                
                ## Output
                Structured compatibility report with clear upgrade paths.
            "#}.to_string(),
        },
        
        AnalysisPrompt {
            name: "security-sweep-all-prs".to_string(),
            description: "Security-focused sweep of all PRs in a release using parallel agents".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "release".to_string(),
                    description: Some("Release to security audit".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "severity_threshold".to_string(),
                    description: Some("Minimum severity to report: low|medium|high|critical".to_string()),
                    required: Some(false),
                }
            ],
            requires_parallel_agents: true,
            agent_batch_size: Some(4),
            instructions: indoc! {r#"
                # Parallel Security Sweep
                
                Perform security analysis on EVERY PR in {{release}}.
                
                ## Sub-Agent Security Checklist
                Each agent must check their assigned PRs for:
                
                ### Code Security
                - [ ] Unsafe code usage
                - [ ] Panic conditions
                - [ ] Integer overflows
                - [ ] Unvalidated inputs
                - [ ] Access control issues
                
                ### Substrate Security
                - [ ] Origin verification
                - [ ] Weight manipulation
                - [ ] Storage exhaustion
                - [ ] Consensus impact
                - [ ] XCM vulnerabilities
                
                ### Economic Security
                - [ ] Fee manipulation
                - [ ] MEV opportunities
                - [ ] Slashing conditions
                - [ ] Token minting/burning
                
                ## Severity Classification
                - **Critical**: Immediate fund risk
                - **High**: Consensus or availability impact
                - **Medium**: Limited exploit potential
                - **Low**: Best practice violations
                
                ## Parallel Execution
                Process {{agent_batch_size}} PRs simultaneously for speed.
                
                ## Final Report
                Aggregate all findings, deduplicate, and prioritize by severity.
            "#}.to_string(),
        },
        
        AnalysisPrompt {
            name: "upgrade-impact-simulation".to_string(),
            description: "Simulate upgrade impact by analyzing all changes systematically".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "from_release".to_string(),
                    description: Some("Current release".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "to_release".to_string(),
                    description: Some("Target release".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "runtime_config".to_string(),
                    description: Some("Runtime configuration details (optional)".to_string()),
                    required: Some(false),
                }
            ],
            requires_parallel_agents: true,
            agent_batch_size: Some(2),
            instructions: indoc! {r#"
                # Upgrade Impact Simulation
                
                Simulate upgrading from {{from_release}} to {{to_release}}.
                
                ## Multi-Pass Analysis Required
                
                ### Pass 1: Change Detection (Parallel)
                Sub-agents analyze all PRs to identify:
                - Breaking changes
                - New features
                - Deprecations
                - Migration requirements
                
                ### Pass 2: Dependency Analysis (Parallel)
                Sub-agents trace dependencies:
                - Which pallets depend on changed APIs
                - Cross-pallet interaction changes
                - Trait implementation requirements
                
                ### Pass 3: Impact Simulation (Sequential)
                Using aggregated data:
                1. Build upgrade sequence
                2. Identify critical path
                3. Simulate state transitions
                4. Predict potential failures
                
                ## Runtime Configuration
                If provided: {{runtime_config}}
                Use this to make analysis more specific.
                
                ## Output
                - Step-by-step upgrade plan
                - Risk assessment with probabilities
                - Rollback procedures
                - Testing requirements
            "#}.to_string(),
        },
        
        AnalysisPrompt {
            name: "autonomous-upgrade-planning".to_string(),
            description: "Generate autonomous upgrade plan analyzing all aspects comprehensively".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "target_release".to_string(),
                    description: Some("Target release for upgrade".to_string()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "autonomy_level".to_string(),
                    description: Some("Level of automation: assisted|semi-auto|full-auto".to_string()),
                    required: Some(false),
                },
                PromptArgument {
                    name: "risk_tolerance".to_string(),
                    description: Some("Risk tolerance: conservative|balanced|aggressive".to_string()),
                    required: Some(false),
                }
            ],
            requires_parallel_agents: true,
            agent_batch_size: Some(3),
            instructions: indoc! {r#"
                # Autonomous Upgrade Planning
                
                Generate comprehensive upgrade plan for {{target_release}}.
                
                ## Exhaustive Multi-Phase Analysis
                
                ### Phase 1: Complete Change Inventory (Parallel)
                Every PR must be analyzed for:
                - Change type and scope
                - Risk profile
                - Automation potential
                - Testing requirements
                
                ### Phase 2: Dependency Mapping (Parallel)
                Build complete dependency graph:
                - Pallet dependencies
                - API dependencies
                - Storage migrations
                - Feature interactions
                
                ### Phase 3: Upgrade Sequencing (AI Synthesis)
                Determine optimal upgrade sequence:
                - Order of operations
                - Parallelizable steps
                - Critical synchronization points
                - Rollback checkpoints
                
                ### Phase 4: Automation Planning
                Based on {{autonomy_level}}:
                - **Assisted**: Human approval at each step
                - **Semi-auto**: Automated with checkpoints
                - **Full-auto**: Fully automated execution
                
                ## Risk Management
                Apply {{risk_tolerance}} level:
                - **Conservative**: Extra validation, slower
                - **Balanced**: Standard safety checks
                - **Aggressive**: Minimal checks, faster
                
                ## Deliverables
                1. Complete upgrade runbook
                2. Automated scripts where applicable
                3. Validation test suite
                4. Monitoring configuration
                5. Rollback procedures
                
                This analysis must be EXHAUSTIVE - every change matters for autonomous execution.
            "#}.to_string(),
        }
    ]
}

/// Standard patterns for parallel agent coordination
pub mod patterns {
    use super::*;
    
    pub const PARALLEL_ANALYSIS_TEMPLATE: &str = indoc! {r#"
        # Parallel Sub-Agent Coordination Pattern
        
        When analyzing multiple items (PRDocs, pallets, etc.), use this pattern:
        
        ## 1. Inventory Phase
        First, get complete list of items to analyze.
        
        ## 2. Batch Planning
        Divide items into batches of size N (typically 2-4).
        
        ## 3. Sub-Agent Spawning
        For each batch, spawn agents with specific tasks:
        ```
        Analyzing items X, Y:
        - Read relevant files
        - Apply analysis framework
        - Return structured results
        ```
        
        ## 4. Result Aggregation
        Collect all sub-agent results and synthesize.
        
        ## 5. Strategic Analysis
        Use aggregated data for high-level insights.
        
        Remember: EXHAUSTIVE > SAMPLING
    "#};
    
    pub const MULTI_PASS_TEMPLATE: &str = indoc! {r#"
        # Multi-Pass Analysis Pattern
        
        Complex analyses require multiple passes:
        
        ## Pass 1: Data Collection (Parallel)
        - Gather all raw information
        - Identify key metrics
        - Build initial indices
        
        ## Pass 2: Deep Analysis (Parallel)
        - Detailed inspection of each item
        - Cross-referencing
        - Pattern identification
        
        ## Pass 3: Synthesis (Sequential)
        - Aggregate findings
        - Identify emergent patterns
        - Generate insights
        
        ## Pass 4: Strategic Planning (AI)
        - Use all available data
        - Generate actionable recommendations
        - Prioritize by impact
        
        Each pass builds on previous results.
    "#};
}

/// Helper to determine if parallel agents should be used
pub fn should_use_parallel_agents(item_count: usize, complexity: &str) -> bool {
    match complexity {
        "high" => item_count > 3,
        "medium" => item_count > 5,
        "low" => item_count > 10,
        _ => item_count > 7
    }
}

/// Generate sub-agent task description
pub fn generate_sub_agent_task(
    items: &[String],
    analysis_type: &str,
    release: &str
) -> String {
    format!(
        indoc! {r#"
            Analyze the following items from release {}:
            {}
            
            Analysis type: {}
            
            For each item:
            1. Read the complete PRDoc/file
            2. Apply {} analysis framework
            3. Document all findings
            4. Return structured results
            
            Be thorough - this is part of an exhaustive analysis.
        "#},
        release,
        items.join(", "),
        analysis_type,
        analysis_type
    )
}