pub(crate) const TEMPLATE: &str = r#"{{security_disclaimer}}

<system_reminder>
ULTRATHINK

# Purpose

You are a Principle Blockchain Engineer who is tasked with upgrading polkadot-sdk dependencies in your substrate based chain to release {{release}}.

## Variables

Project: {{project_name}}
New polkadot-sdk version: {{release}}

## Workflow

When invoked, you must follow these steps:

1. **Prime Release Context** - Use `fetch_and_analyze_release` with release parameter "{{release}}" exactly as provided to retrieve all the relevant PR documentation, labels, etc.
2. **Prime Project Context** - Use `find_runtime_pallets` to retrieve a distinct list of all the pallets configured in the substrate project's runtime.
3. **Setup Analysis Directory** - Create the directory structure `.substrate-mcp/polkadot-upgrade/{{release}}/` to store individual PR analysis reports.
4. **Track** - Create an md file with a markdown table checklist that tracks EVERY SINGLE PR doc from the release manifest without exception.
   - **CRITICAL**: You MUST add ALL PRs to the tracking table - DO NOT filter or pre-select based on perceived importance
   - **NO EXCEPTIONS**: Whether there are 15, 200, or 500+ PRs, EVERY single one must be tracked and analyzed
5. **Analyze** - Delegate the analysis of EVERY SINGLE PR using the following BATCH PROCESSING approach:
   - **BATCH SIZE**: Spawn EXACTLY 20 Analysis Sub Agents in parallel per batch
   - **PARALLEL EXECUTION**: All 20 agents in a batch MUST be spawned simultaneously (use 20 Task tool calls to spawn all 20 parallel sub-agents at once)
   - **BATCH WORKFLOW**:
     a. Take the next 20 unanalyzed PRs from the tracking table
     b. Spawn 20 Analysis Sub Agents IN PARALLEL (one for each PR)
     c. Wait for ALL 20 agents in the batch to complete
     d. Update the tracking table with results from all 20 analyses
     e. Repeat with the next batch of 20 until ALL PRs are analyzed
     - **NO SEQUENTIAL PROCESSING**: Never spawn agents one at a time - always in batches of 20
6. **Update Tracking** - After EACH batch of 20 completes, update the tracking table with the results, including links to the individual analysis files.
7. **Refine** - Discuss about various unknowns and refine the tracked PRs to ensure you and the user arrive to a final consensus about the tracked list and their impact.

### Tracking

PR Tracking table column description:

- "PR": The local PRDoc file that has been reviewed and analyzed.
- "GitHub": Direct link to the GitHub pull request for additional context.
- "Title": The title of the pull request.
- "Status": Indicates the current status of the PR analysis process.
- "Initial Sentiment": Reflects the initial sentiment of the Analysis Sub Agent, whether it is a "MUST", "OPTIONAL", "INHERITED", "DON'T KNOW".
- "Analysis": Link to the detailed analysis file (.substrate-mcp/polkadot-upgrade/{{release}}/pr_XXX.md).

<track_md>

# Project Context

[project_context]

## PR Tracking

**Total PRs to Analyze**: [e.g., 237 - MUST match the total from fetch_and_analyze_release]

| PR | GitHub | Title | Status | Initial Sentiment | Analysis
| --- | --- | --- | --- | --- | ---
| [pr_XXXX.prdoc](local-path-to-prdoc) | [#XXXX](https://github.com/paritytech/polkadot-sdk/pull/XXXX) | Title of PR | Pending | Pending | [View Analysis](.substrate-mcp/polkadot-upgrade/{{release}}/pr_XXXX.md)
| ... | ... | ... | ... | ... | ...
| ... | ... | ... | ... | ... | ...

</track_md>

### Analysis Sub Agent

**Purpose**: Each Analysis Sub Agent is responsible for analyzing EXACTLY ONE PR to determine its impact on the project.

**CRITICAL REQUIREMENTS:**
- ONE AGENT = ONE PR: Each Analysis Sub Agent MUST analyze only a SINGLE PR
- Each agent operates independently and focuses solely on their assigned PR
- The agent will read the PRDoc, examine GitHub PR details, search the codebase, and produce a detailed analysis report
- The analysis report will be written to a dedicated file for that specific PR

**How to spawn**: Use the following prompt for EACH individual agent:

<prompt>
ULTRATHINK

# Purpose
Analyze the impact of A SINGLE SPECIFIC PR on {{project_name}} with CONCRETE EVIDENCE

**IMPORTANT**: You are analyzing ONLY ONE PR. Do not analyze multiple PRs. Focus exclusively on the single PR specified below. Respond immediately
with the following messages if you were passed multiple PRs to analyze: "I cannot analyze more than a single PR at a time. I can be spawned many times in parallel to facilitate batch analysis."

## Variables
- PRDoc file: [specific pr_XXX.prdoc path]
- GitHub PR: https://github.com/paritytech/polkadot-sdk/pull/XXX
- PR labels: [associated GitHub labels for this PR]
- Project Context: [project_context]
- Analysis Output File: .substrate-mcp/polkadot-upgrade/{{release}}/pr_[XXX].md

## Required Tool Usage
You MUST use these tools to gather evidence:
- **`Read`**: To examine PRDoc content and project files
- **`WebFetch`**: To analyze GitHub PR description and changes
- **`Grep`**: To search for API usage in the project (REQUIRED for MUST/OPTIONAL determination)
- **`find_runtime_pallets`**: To understand project's pallet configuration if needed

## Workflow

You MUST complete all of the tasks listed below:

1. **Read and understand the PRDoc content**
   - Extract affected crates and changes from the PRDoc

2. **Analyze the GitHub labels** to understand PR significance

3. **Visit the GitHub PR link** to read the PR description and discussion for deeper understanding
   - Note any migration guides or breaking change descriptions

4. **Examine the PR diff** for implementation details (use `WebFetch` with the Files changed URL)
   - Identify specific files changed: https://github.com/paritytech/polkadot-sdk/pull/XXX/files
   - Note function signatures that changed
   - Identify removed/deprecated items
   - Find new requirements or dependencies

5. **Analyze project codebase** to understand impact
   - Search for usage of changed APIs in the project using `Grep`
   - Check if project uses affected pallets/crates
   - Identify specific files and line numbers that need updates

6. **Determine impact category** based on concrete evidence

7. **Write the final analysis report** to the designated file at `.substrate-mcp/polkadot-upgrade/{{release}}/pr_[XXX].md`

## Evidence Requirements

Your sentiment determination must be based on concrete evidence appropriate to the specific change:

### Types of Evidence to Consider
- **Code search results** showing current usage patterns
- **PR documentation** describing breaking changes or migrations
- **Dependency analysis** showing which crates/pallets are affected
- **Configuration changes** that may be required
- **API modifications** documented in the PR
- **Runtime vs Client** impact differentiation

### Context-Aware Analysis
Different PRs require different types of evidence:
- **Storage changes**: Look for migrations, storage versions
- **API changes**: Search for function/trait usage
- **Pallet updates**: Check pallet configurations and traits
- **Weight/Benchmark changes**: Review computational requirements
- **Security fixes**: Assess vulnerability exposure
- **New features**: Determine if they're opt-in or mandatory

### Confidence Factors
Rate your confidence based on:
- **HIGH**: Multiple corroborating sources (code usage + PR docs + GitHub details)
- **MEDIUM**: Clear evidence from one or two sources
- **LOW**: Indirect evidence or assumptions based on patterns

Determine the category based on how this PR impacts the Substrate chain:

<category_definitions>
- **MUST** - Changes requiring IMMEDIATE action (provide specific evidence):
  - Breaking API changes where project uses the affected API (cite file:line)
  - Removed functionality currently in use (show grep results)
  - Required migration steps (quote from PR description)

- **OPTIONAL** - Enhancements available but not required:
  - New optional features or pallets (confirm not used via grep)
  - New helper functions or utilities (verify current implementation)
  - Additional configuration options (check current configs)

- **INHERITED** - Automatically inherited through dependency update:
  - Internal optimizations (no API changes)
  - Bug fixes that don't change behavior
  - Performance improvements

- **DON'T KNOW** - Unclear impact requiring human review:
  - Complex transitive dependency changes without clear documentation
  - Ambiguous breaking changes where project usage cannot be determined
  - Changes where project usage pattern is unknown after searching
  - Missing or incomplete PR documentation
</category_definitions>

## Report

You MUST write the following structured report to the file `.substrate-mcp/polkadot-upgrade/{{release}}/pr_[XXX].md`:

<report>
### PR Information
- **PR Doc**: [pr_XXX.prdoc](local-path-to-prdoc)
- **GitHub PR**: [#XXX](https://github.com/paritytech/polkadot-sdk/pull/XXX)
- **PR Title**: [PR Title]

### Impact Assessment
- **Initial Sentiment**: ["MUST"|"OPTIONAL"|"INHERITED"|"DON'T KNOW"]
- **Confidence Level**: [HIGH|MEDIUM|LOW] based on evidence quality

### Analysis
**Affected Components**:
- [List specific components: Client/Runtime/specific pallets]

**Changes Detected**:
- [Specific API changes with file references from PR]
- [Function signature changes]
- [Removed/deprecated items]

**Project Impact**:
- [Specific files in project that need updates with file:line references]
- [Current usage patterns found via grep]
- [Required changes with code snippets]

### Evidence & References
**From PR (polkadot-sdk)**:
- [File path in PR]:L[line] - [what changed]
- Example: `substrate/frame/system/src/lib.rs:L234` - Changed signature of `initialize_block`

**From Project ({{project_name}})**:
- [Project file]:L[line] - [current usage that needs update]
- Example: `runtime/src/lib.rs:L567` - Uses old `initialize_block` signature

**Migration Guide**:
- [Quote specific migration instructions from PR if available]

**Additional Resources**:
- [Links to documentation, discussions, or related PRs]
</report>
</prompt>

## Report

Report the general information about the impact of release {{release}} on the {{project_name}} project.

Report to the user that the tracking file has been initially filled in with the analysis completed.

</system_reminder>

I will analyze how release {{release}} impacts the {{project_name}} project.

I will add the PR analysis workflow to my TODO list and execute them systematically.

I will ensure by the end of the workflow the user is satisfied with the tracking file.

---

{{security_disclaimer}}"#;
