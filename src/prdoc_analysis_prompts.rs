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
                    description: Some("Number of PRs to process in parallel (2-4 recommended, default: 3)".to_string()),
                    required: Some(false),
                }
            ],
            requires_parallel_agents: true,
            agent_batch_size: Some(3), // Default, overridden by user's batch_size
            instructions: indoc! {r#"
                # Parallel Release Analysis Framework
                
                You MUST analyze EVERY PRDoc in release {{release}} using parallel processing for efficiency.
                
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
                1. Download the release using get_polkadot_sdk_release_prdocs tool
                2. Get complete inventory of all PRDocs (use LS to list them)
                3. Determine if single or multi-pass approach is needed
                4. Plan batches of size {{batch_size}} (or 3 if not specified)
                
                ### For Single-Pass Analysis:
                1. **Parallel Analysis Phase**
                   - Process {{batch_size}} PRDocs at a time
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
                Analyze the following PRs from release {{release}}: [list of PRs]
                
                For each PR:
                1. Read the PRDoc file at the specified path
                2. Apply the appropriate analysis for this pass:
                   - Pass 1: [discovery instructions]
                   - Pass 2: [deep analysis using Pass 1 data]
                   - Pass 3: [synthesis using all previous data]
                3. Return structured findings
                ```
                
                ## Decision Transparency
                
                When you determine multiple passes are needed, briefly explain:
                - Why multiple passes are beneficial for this analysis
                - What each pass will accomplish
                - How the passes build on each other
                
                Example: "This migration planning task requires 3 passes: First, I'll inventory all changes. Second, I'll analyze dependencies between them. Finally, I'll create an ordered migration plan."
                
                ## User-Specified Analysis
                
                The user has requested: {{analysis_instructions}}
                
                Apply this analysis using the most appropriate strategy (single or multi-pass).
                
                ## Output Requirements
                
                1. Confirm 100% coverage (all PRDocs analyzed)
                2. Provide organized findings based on the analysis type
                3. Include summary statistics
                4. For multi-pass: Show how insights evolved across passes
                5. Deliver actionable conclusions
                
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