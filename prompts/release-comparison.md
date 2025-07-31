# Release Comparison

## Description

List changes between two polkadot-sdk release versions

## Arguments

- current_version: version currently being used
- target_version: version dev wants to compare with (must be greater than current)
- specific_changes (Optional): What specific changes to look for (e.g: was there any change in `pallet_treasury` ?)

## Prompt

```
Compare changes between Polkadot SDK versions <current_version> and 
<target_version>.

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
   - stable2503-2

<% if specific_changes %>
## Filtered Analysis
Focus only on changes related to: <specific_changes>
Filter PRDocs and code changes to match these criteria.
<% end %>

## Output Format

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

<% if not specific_changes %>
### Additional Notes
- Changes not covered by PRDocs may exist in the codebase
- Review CHANGELOG.md files for complete details
<% end %>

```