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
                
                ## Phase 0: Project Dependency Discovery (MANDATORY when project_context provided)
                
                {{#if project_context}}
                Project context provided: {{project_context}}
                
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
                         pub enum Runtime {
                             System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
                             Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
                             // Storage component = on-chain state that may need migrations
                         }
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
                
                {{else}}
                No project context provided. Performing comprehensive analysis of all changes.
                💡 Tip: Provide project_context parameter for targeted, project-specific insights.
                
                Consider running this analysis with project_context to get:
                - Automated discovery of your actual pallet dependencies
                - Relevance scoring based on your runtime configuration
                - Migration requirements specific to your pallets
                - Filtered results showing only what affects your project
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
                
                ### 📂 Data Locations (CRITICAL - READ THIS!)
                
                **PRDoc Input Data**: `./polkadot-release-analysis/releases/{{release}}/pr-docs/`
                - This is where get_polkadot_sdk_release_prdocs saves files
                - Contains: pr_XXXX.prdoc files + summary JSONs
                
                **Report Output Location**: `./polkadot-release-analysis/releases/{{release}}/reports/`
                - You MUST create this directory if it doesn't exist
                - Save report as: `analysis-[ISO-8601-timestamp].md`
                
                ### Initial Setup (Always)
                1. Check if analyzing multiple releases or upgrading across versions
                   - If comparing versions (e.g., from X to Y), fetch all intermediate releases using: "X>Y"
                   - If multiple specific releases requested, download each one
                2. Download the release(s) using get_polkadot_sdk_release_prdocs tool
                   - Files will be saved to: `./polkadot-release-analysis/releases/{release}/pr-docs/`
                3. Get complete inventory of all PRDocs (use LS on the pr-docs directory)
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
                
                {{#if project_context}}
                ## Project Dependency Profile (from Phase 0 analysis):
                [INSERT DISCOVERED DEPENDENCY PROFILE HERE]
                - Active Pallets: [list from construct_runtime!]
                - Storage Pallets: [pallets with on-chain state]
                - Custom Pallets: [project-specific implementations]
                - Critical APIs: [traits and types used]
                - Feature Flags: [enabled features affecting behavior]
                
                Use this profile to evaluate relevance of EVERY change in your assigned PR(s).
                {{/if}}
                
                Instructions for this sub-agent:
                1. Read ONLY the PRDoc file(s) for the assigned PR(s)
                2. DO NOT reference or consider other PRs outside your assignment
                3. Apply the appropriate analysis for this pass:
                   - Pass 1: [discovery instructions]
                   - Pass 2: [deep analysis using Pass 1 data]  
                   - Pass 3: [synthesis using all previous data]
                
                {{#if project_context}}
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
                
                {{#if project_context}}
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
                {{/if}}
                
                ## User-Specified Analysis
                
                The user has requested: {{analysis_instructions}}
                
                Apply this analysis using the most appropriate strategy (single or multi-pass).
                
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
                # Polkadot SDK Release {{release}} Analysis Report
                
                **Generated**: [ISO 8601 timestamp]  
                **Analyzed PRs**: [total count]  
                **Analysis Type**: {{analysis_instructions}}  
                {{#if project_context}}**Project Context**: {{project_context}}{{/if}}
                
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
                
                {{#if project_context}}
                ### Project-Specific Impact Summary
                
                **Directly Affected Components**: [list]  
                **Required Actions**: [count]  
                **Estimated Migration Effort**: [Low/Medium/High]  
                {{/if}}
                
                ## Critical Actions Required
                
                {{#if project_context}}
                ### ⚠️ Breaking Changes Affecting Your Project
                
                | PR # | Description | Your Affected Component | Action Required |
                |------|-------------|------------------------|-----------------|
                | #[X] | [description] | [pallet/module] | [specific action] |
                
                ### 🔒 Security Updates for Your Dependencies
                
                | PR # | Vulnerability | Severity | Your Exposure | Action |
                |------|--------------|----------|---------------|--------|
                | #[X] | [CVE/description] | [Critical/High/Medium] | [component] | [update/patch] |
                {{else}}
                [List all critical changes that require immediate attention]
                {{/if}}
                
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
                {{#if project_context}}
                - [ ] Review project-specific changes in [Critical Actions Required](#critical-actions-required)
                {{/if}}
                
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
                ./polkadot-release-analysis/             # Root directory in current working directory
                └── releases/                        # All release data
                    └── {release}/                   # e.g., stable2412-1/
                        ├── pr-docs/                 # Downloaded PRDoc files
                        │   ├── pr_XXXX.prdoc       # Individual PRDoc files
                        │   ├── manifest.json       # Release metadata
                        │   ├── crate_summary.json  # Crate changes summary
                        │   └── audience_summary.json # Audience categorization
                        └── reports/                 # Analysis reports for this release
                            └── analysis-{timestamp}.md # e.g., analysis-2024-01-15T10-30-00Z.md
                ```
                
                **STEPS YOU (THE LLM) MUST FOLLOW:**
                1. **YOU CREATE THE DIRECTORY** (if it doesn't exist): `./polkadot-release-analysis/releases/{{release}}/reports/`
                2. **YOU SAVE THE REPORT** to: `./polkadot-release-analysis/releases/{{release}}/reports/analysis-[timestamp].md`
                   - Replace [timestamp] with actual ISO 8601 timestamp (e.g., 2024-01-15T10-30-00Z)
                   - Use hyphens in timestamp, not colons (for filesystem compatibility)
                3. **YOU VERIFY THE FILE** was created successfully
                4. **YOU PRINT THE CLICKABLE PATH** - Show both directory and file paths
                
                ⚠️ DO NOT SKIP THIS STEP. YOU (THE LLM) MUST CREATE THE FILE IN THIS EXACT LOCATION.
                
                ### SECONDARY OUTPUT: Brief Console Summary (5-10 lines maximum)
                
                ONLY AFTER successfully saving the markdown report file, display this brief summary:
                
                ```
                ✅ Release {{release}} Analysis Complete
                
                📊 Analyzed: [X] PRs | Breaking: [Y] | Security: [Z]
                {{#if project_context}}⚠️ [N] changes directly affect your project{{/if}}
                
                📁 Report directory: ./polkadot-release-analysis/releases/{{release}}/reports/
                📄 Report file: ./polkadot-release-analysis/releases/{{release}}/reports/analysis-[timestamp].md
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
                ✓ Did you create the ./polkadot-release-analysis/releases/{{release}}/reports/ directory?
                ✓ Did you save the markdown report to the EXACT path specified above?
                ✓ Did you verify the file was created successfully?
                ✓ Did you print BOTH the directory path AND the clickable file path?
                ✓ Did you analyze ALL PRDocs from ./polkadot-release-analysis/releases/{{release}}/pr-docs/?
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
pub mod patterns {}