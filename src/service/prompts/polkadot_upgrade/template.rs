pub(crate) const TEMPLATE: &str = r#"{{security_disclaimer}}

# Analyze Polkadot SDK Release Impact on Your Project

I will analyze how release {{release}} impacts the {{project_name}} project using parallel processing.

{{#if focus}}
**Focus Area**: {{focus}}
{{else}}
**Analysis Type**: Comprehensive analysis
{{/if}}

I will add all following steps to my TODO list and execute them systematically.

---

## STEP 1: Prepare Release Data

### 1.1 Fetch Release Data
I will call the `fetch_and_analyze_release` tool with release "{{release}}" to:
- Download all PRDocs for this release
- Fetch GitHub labels for each PR
- Return: PRDocs file paths + associative labels + label definitions

### 1.2 Fetch Runtime Pallets
I will call the `find_runtime_pallets` tool to discover your project's runtime configuration:
- All runtimes with their construct_runtime! pallets
- Configured pallets with names and paths

After these tools execute, I'll have:
- **Release Data**: {PRDocs paths + labels, label definitions}
- **Runtime Profile**: Your project's pallets and configuration

---

## STEP 2: Spawn Parallel PR Analysis

### 2.1 Create Sub-Agent Tasks
For each PRDoc, I will launch a specialized sub-agent with this prompt:

<prompt>
Analyze PR impact for {{project_name}}

- PRDoc file: [specific pr_XXX.prdoc path]
- PR labels: [associated GitHub labels for this PR]
- Project runtime pallets: [your construct_runtime! pallets]

You MUST complete all of the tasks listed below:
<tasks>
1. Read and understand the PRDoc content
2. Analyze the GitHub labels to understand PR significance
3. Read the PR diff for deeper understanding (use paging if response would be too large)
4. Read relevant project codebase files to understand impact
5. Determine impact category and provide analysis
</tasks>

<category_definitions>
Determine the category based on how this PR impacts the Substrate chain:

**MUST** - Changes requiring immediate action:
- Breaking API changes in pallets/traits the project uses
- Required storage migrations for existing pallets
- Consensus or block production changes
- Security fixes in components the project depends on
- Removal of deprecated features currently in use

**OPTIONAL** - Enhancements available but not required:
- New optional features or pallets
- Performance improvements
- New helper functions or utilities
- Additional configuration options
- Quality of life improvements

**NOTHING_INTERESTING** - Automatically inherited through dependency update:
- Internal refactoring with no API changes
- Documentation updates
- Test improvements
- Changes to unused pallets/components
- Bug fixes that don't affect the project's usage

**DONT_KNOW** - Unclear impact requiring human review:
- Complex transitive dependency changes
- Ambiguous breaking changes
- Changes where project usage pattern is unknown
- Missing or incomplete PR documentation
</category_definitions>

<output_schema>
{
  "pr_number": "XXXX",
  "pr_title": "...",
  "prdoc_path": "pr_XXXX.prdoc",
  "labels": ["label1", "label2", ...],
  "category": "MUST" | "OPTIONAL" | "NOTHING_INTERESTING" | "DONT_KNOW",
  "analysis": {
    "impact_reason": "why this PR impacts/doesn't impact your project",
    "affected_components": ["pallet/component names"],
    "required_actions": ["specific actions needed, if any"],
    "migration_complexity": "none" | "low" | "medium" | "high"
  }
}
</output_schema>
</prompt>

### 2.2 Execute Parallel Analysis
I will launch all sub-agents concurrently to analyze each PR independently.

---

## STEP 3: Aggregate Results into Categories

### 3.1 Collect All Sub-Agent Results
I will gather the structured output from each sub-agent.

### 3.2 Categorize PRDocs
Using the sub-agent analysis, I will organize all PRs into four categories:

**MUST** - PRs requiring immediate action:
- Breaking changes in your runtime pallets
- API changes affecting your implementations  
- Security fixes in components you use
- Required migrations or storage updates
- Consensus or block production changes

**OPTIONAL** - Enhancement opportunities:
- New features you could adopt
- Performance improvements available
- Additional configuration options
- Quality of life improvements

**NOTHING_INTERESTING** - PRs automatically handled:
- Internal refactoring with no API impact
- Changes to unused pallets/components
- Documentation and test updates
- Bug fixes that don't affect your usage

**DONT_KNOW** - PRs needing human review:
- Complex transitive dependencies
- Ambiguous breaking changes
- Incomplete PR documentation
- Unknown usage patterns

### 3.3 Present Categorization Results
I will output the final categorization with clear formatting:

<pr_category_list_schema>
# 📋 Polkadot SDK Release {{release}} Impact Analysis

## 📊 RELEASE SUMMARY
- **Release**: {{release}}
- **Total PRs**: [X]
- **Analysis Date**: [current date]

---

## 🔴 MUST (Required Actions) - [X] PRs

{{#each must_prs}}
### PR #{{pr_number}}: {{title}}
- **File**: `~/.substrate-mcp/substrate-mcp/releases/{{../release}}/pr-docs/pr_{{pr_number}}.prdoc`
- **Labels**: {{labels}}
- **Impact**: {{impact_reason}}
- **Actions**: {{required_actions}}
- **Complexity**: {{migration_complexity}}

---
{{/each}}

## 🟡 OPTIONAL (Available Enhancements) - [Y] PRs

{{#each optional_prs}}
### PR #{{pr_number}}: {{title}}
- **File**: `~/.substrate-mcp/substrate-mcp/releases/{{../release}}/pr-docs/pr_{{pr_number}}.prdoc`
- **Enhancement**: {{impact_reason}}
- **Benefit**: {{potential_benefit}}

{{/each}}

---

## 🟢 NOTHING_INTERESTING (Auto-inherited) - [Z] PRs

{{#each nothing_interesting_prs}}
### PR #{{pr_number}}: {{title}}
- **File**: `~/.substrate-mcp/substrate-mcp/releases/{{../release}}/pr-docs/pr_{{pr_number}}.prdoc`
- **Reason**: {{impact_reason}}

{{/each}}

---

## 🔵 DONT_KNOW (Needs Review) - [W] PRs

{{#each dont_know_prs}}
### PR #{{pr_number}}: {{title}}
- **File**: `~/.substrate-mcp/substrate-mcp/releases/{{../release}}/pr-docs/pr_{{pr_number}}.prdoc`
- **Review Needed**: {{impact_reason}}

{{/each}}

---

## 📈 SUMMARY
- **[X] MUST** | **[Y] OPTIONAL** | **[Z] NOTHING_INTERESTING** | **[W] DONT_KNOW**

### Key Takeaways:
[Bullet points summarizing main findings]

### Recommended Actions:
[Prioritized action items based on impact analysis]
</pr_category_list_schema>

---

## STEP 4: Free-Form Discussion

### 4.1 Interactive Exploration
After presenting the categorization, I'm ready for discussion:
- Review any "DONT_KNOW" PRs together
- Deep dive into specific PRs by number or file path
- Analyze migration strategies for impactful changes
- Create documentation if requested

### 4.2 Available Actions
- **PR Details**: "Tell me more about PR #XXXX"
- **Category Refinement**: Move PRs between categories as we learn more
- **Migration Planning**: Create step-by-step upgrade plans
- **Documentation**: Generate reports, GitHub issues, or migration guides

### 4.3 Documentation Options (On Request Only)
If you ask, I can create:
- GitHub issues summarizing findings
- Migration plans with code examples
- Technical reports for team review
- Quick reference action lists

---

## CRITICAL REQUIREMENTS

1️⃣ **EXHAUSTIVE ANALYSIS**: Analyze every single PR - no sampling
2️⃣ **PARALLEL PROCESSING**: Use sub-agents for efficient analysis
3️⃣ **CLEAR CATEGORIZATION**: Must/Optional/Nothing_Interesting/Don't_Know for all PRs
4️⃣ **INTERACTIVE DISCUSSION**: Collaborative refinement of understanding
5️⃣ **USER-DRIVEN DOCUMENTATION**: Create files only when explicitly requested

{{security_disclaimer}}"#;
