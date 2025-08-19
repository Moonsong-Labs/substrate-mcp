/// Analyze release handlebars template
pub const PROMPT: &str = r#"
# Analyze Polkadot SDK Release Impact on Your Project

You MUST analyze how the release(s) {{release}} impact this specific project using parallel processing.

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

## Analysis Strategy Selection

Based on the user's request: {{#if focus}}{{focus}}{{else}}Comprehensive analysis{{/if}}

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

**PRDoc Input Data**: `~/.substrate-mcp/{project}/releases/{{release}}/pr-docs/`
- This is where fetch_and_analyze_release saves files
- Contains: pr_XXXX.prdoc files + summary JSONs

**Report Output Location**: `~/.substrate-mcp/{project}/releases/{{release}}/reports/`
- You MUST create this directory if it doesn't exist
- Save report as: `analysis-[ISO-8601-timestamp].md`

### Initial Setup (Always)
1. Check if analyzing multiple releases or upgrading across versions
   - If comparing versions (e.g., from X to Y), fetch all intermediate releases using: "X>Y"
   - If multiple specific releases requested, download each one
2. Download the release(s) using fetch_and_analyze_release tool
   - Files will be saved to: `~/.substrate-mcp/{project}/releases/{release}/pr-docs/`
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

Analyze PR(s) from release {{release}}: [PR number(s)]

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

## User-Specified Analysis
{{#if focus}}
The user has requested focus on: {{focus}}

Apply this analysis using the most appropriate strategy (single or multi-pass).
{{else}}
Perform comprehensive analysis covering all aspects.
{{/if}}

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
└── {project}/                      # Project directory (current project's root dir name)
    └── releases/                     # All release data for this project
        └── {{release}}/                # e.g., stable2412-1/
            ├── pr-docs/              # Downloaded PRDoc files
            │   ├── pr_XXXX.prdoc    # Individual PRDoc files
            │   ├── manifest.json    # Release metadata
            │   ├── crate_summary.json # Crate changes summary
            │   └── audience_summary.json # Audience categorization
            └── reports/              # Analysis reports for this release
                └── analysis-{timestamp}.md # e.g., analysis-2024-01-15T10-30-00Z.md
```

**STEPS YOU (THE LLM) MUST FOLLOW:**
1. **YOU CREATE THE DIRECTORY** (if it doesn't exist): `~/.substrate-mcp/{project}/releases/{{release}}/reports/`
2. **YOU SAVE THE REPORT** to: `~/.substrate-mcp/{project}/releases/{{release}}/reports/analysis-[timestamp].md`
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
⚠️ [N] changes directly affect your project

📁 Report directory: ~/.substrate-mcp/{project}/releases/{{release}}/reports/
📄 Report file: ~/.substrate-mcp/{project}/releases/{{release}}/reports/analysis-[timestamp].md
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
✓ Did you create the ~/.substrate-mcp/{project}/releases/{{release}}/reports/ directory?
✓ Did you save the markdown report to the EXACT path specified above?
✓ Did you verify the file was created successfully?
✓ Did you print BOTH the directory path AND the clickable file path?
✓ Did you analyze ALL PRDocs from ~/.substrate-mcp/{project}/releases/{{release}}/pr-docs/?
"#;