use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;

// GitHub API structures
#[derive(Debug, Deserialize)]
struct GitHubContent {
    name: String,
    #[serde(rename = "type")]
    content_type: String,
    download_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GitHubLabel {
    name: String,
    color: String,
    description: Option<String>,
}

// Label metadata structure
#[derive(Debug, Serialize)]
struct LabelsMetadata {
    fetched_at: String,
    repository: String,
    total_labels: usize,
    labels: Vec<GitHubLabel>,
}

#[derive(Debug, Serialize)]
pub struct PrdocsResult {
    pub success: bool,
    pub release: String,
    pub output_dir: PathBuf,
    pub file_count: usize,
    pub total_size: usize,
}

// PRDoc file structure
#[derive(Debug, Deserialize)]
struct PrDoc {
    #[allow(dead_code)]
    title: String,
    doc: Vec<DocEntry>,
    crates: Vec<CrateEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AudienceField {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Deserialize)]
struct DocEntry {
    audience: AudienceField,
    #[allow(dead_code)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct CrateEntry {
    name: String,
    bump: String,
}

// Manifest structures
#[derive(Debug, Serialize)]
struct Manifest {
    release: String,
    total_prdocs: usize,
    pr_numbers: Vec<u32>,
    download_date: String,
    total_size_bytes: usize,
}

#[derive(Debug, Serialize)]
struct CrateSummary {
    summary: CrateSummaryStats,
    crates: HashMap<String, CrateChangeInfo>,
}

#[derive(Debug, Serialize)]
struct CrateSummaryStats {
    total_crates_affected: usize,
    total_changes: usize,
}

#[derive(Debug, Serialize)]
struct CrateChangeInfo {
    total: usize,
    major: usize,
    minor: usize,
    patch: usize,
    none: usize,
    pr_numbers: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AudienceInfo {
    count: usize,
    pr_numbers: Vec<u32>,
}

// Note: audience_summary.json now directly uses HashMap<String, AudienceInfo>
// instead of a fixed struct, allowing dynamic audience types

/// Normalize release input to handle different formats users might provide.
///
/// This function handles common variations in how users might specify a release:
/// - Git tags with 'polkadot-' prefix (e.g., 'polkadot-stable2503-8' -> 'stable2503-8')
/// - Release tags with 'release-' prefix (e.g., 'release-v1.9.0' -> 'v1.9.0')
/// - Version tags with 'v' prefix are preserved as some releases use this format
///
/// # Examples
/// - 'polkadot-stable2503-8' -> 'stable2503-8'
/// - 'stable2503-8' -> 'stable2503-8' (unchanged)
/// - 'release-v1.9.0' -> 'v1.9.0'
/// - '1.9.0' -> '1.9.0' (unchanged)
fn normalize_release_input(input: &str) -> String {
    // First trim whitespace
    let trimmed = input.trim();

    // Strip common prefixes that users might include from git tags
    let normalized = trimmed
        .strip_prefix("polkadot-")
        .or_else(|| trimmed.strip_prefix("release-"))
        .unwrap_or(trimmed);

    normalized.to_string()
}

// Helper function to extract PR number from filename
fn extract_pr_number(filename: &str) -> Option<u32> {
    if let Some(name) = filename.strip_prefix("pr_") {
        if let Some(num_str) = name.strip_suffix(".prdoc") {
            return num_str.parse().ok();
        }
    }
    None
}

/// Find the project root directory by looking for .git or workspace Cargo.toml
fn find_project_root() -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;
    let mut dir = current_dir.as_path();

    loop {
        // Check for .git directory (most reliable indicator)
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }

        // Check for Cargo.toml with workspace section
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            // Try to read and check if it's a workspace
            if let Ok(contents) = std::fs::read_to_string(&cargo_toml) {
                if contents.contains("[workspace]") {
                    return Some(dir.to_path_buf());
                }
            }
            // If we found a Cargo.toml but no .git above it, this might be the root
            // Check if there's a parent with Cargo.toml
            if let Some(parent) = dir.parent() {
                if !parent.join("Cargo.toml").exists() {
                    return Some(dir.to_path_buf());
                }
            } else {
                // No parent, this must be root
                return Some(dir.to_path_buf());
            }
        }

        // Move up one directory
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return None,
        }
    }
}

/// Get the project name from the project root path
fn get_project_name() -> String {
    find_project_root()
        .and_then(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "default".to_string())
}

/// Fetches and analyzes PRDocs for a specific Polkadot SDK release.
///
/// # Important: Understanding Polkadot SDK Release Structure
///
/// Polkadot SDK organizes releases as follows:
/// 1. **Release Directories**: PRDocs are stored in directories under `prdoc/` on the main branch
///    - Example: `prdoc/stable2503-8/`, `prdoc/1.9.0/`
/// 2. **Git Tags**: Releases are tagged with a 'polkadot-' prefix
///    - Example: `polkadot-stable2503-8`, `polkadot-v1.9.0`
/// 3. **GitHub Release Pages**: Use the git tag in the URL
///    - Example: `https://github.com/paritytech/polkadot-sdk/releases/tag/polkadot-stable2503-8`
///
/// # What This Function Does
///
/// This function accesses the GitHub Contents API to fetch PRDoc files from:
/// `https://api.github.com/repos/paritytech/polkadot-sdk/contents/prdoc/{release}`
///
/// **IMPORTANT**: This is NOT accessing a git branch or tag. It's accessing a directory
/// on the DEFAULT branch (master/main). The `{release}` parameter is a directory name,
/// not a git reference.
///
/// # Parameters
///
/// * `release` - The release identifier, which can be:
///   - A directory name: 'stable2503-8', '1.9.0'
///   - A git tag (will be normalized): 'polkadot-stable2503-8' -> 'stable2503-8'
///
/// The function will automatically normalize the input to handle common variations.
pub async fn fetch_and_analyze_release(release: &str) -> Result<PrdocsResult> {
    let client = reqwest::Client::new();

    // Normalize the release input to handle different formats
    // This strips prefixes like 'polkadot-' that users might include from git tags
    let normalized_release = normalize_release_input(release);

    log::info!("Fetching release '{normalized_release}' (normalized from '{release}')");

    // Get project name from the current project root
    let project_name = get_project_name();

    // Create directory under ~/.substrate-mcp/{project}/releases/{release}/pr-docs
    // We use the normalized release name for the directory structure
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let output_dir = home_dir
        .join(".substrate-mcp")
        .join(project_name)
        .join("releases")
        .join(&normalized_release)
        .join("pr-docs");

    // Create directory if it doesn't exist
    fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| anyhow!("Failed to create directory {}: {}", output_dir.display(), e))?;

    // IMPORTANT: This URL accesses the prdoc/{release} directory on the DEFAULT branch (master/main)
    // It is NOT accessing a git branch or tag named {release}
    // The {release} parameter is a directory name under prdoc/ on the main branch
    // Example: https://api.github.com/repos/paritytech/polkadot-sdk/contents/prdoc/stable2503-8
    let api_url = format!(
        "https://api.github.com/repos/paritytech/polkadot-sdk/contents/prdoc/{normalized_release}"
    );

    let response = client
        .get(&api_url)
        .header("User-Agent", "substrate-mcp")
        .send()
        .await
        .map_err(|e| anyhow!("Failed to fetch directory listing: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        // Provide helpful error messages based on the status
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(anyhow!(
                "Release '{}' not found. This means the directory 'prdoc/{}' does not exist on the main branch.\n\
                Common issues:\n\
                - If you used a git tag like 'polkadot-stable2503-8', the directory name should be 'stable2503-8'\n\
                - The release may not exist yet or may use a different naming format\n\
                - Check available releases at: https://github.com/paritytech/polkadot-sdk/tree/master/prdoc\n\
                GitHub API response: {}",
                normalized_release, normalized_release, error_text
            ));
        } else {
            return Err(anyhow!(
                "GitHub API returned status {} when accessing prdoc/{}: {}",
                status,
                normalized_release,
                error_text
            ));
        }
    }

    let contents: Vec<GitHubContent> = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse directory listing: {}", e))?;

    // Filter for .prdoc files
    let prdoc_files: Vec<&GitHubContent> = contents
        .iter()
        .filter(|c| c.content_type == "file" && c.name.ends_with(".prdoc"))
        .collect();

    if prdoc_files.is_empty() {
        return Ok(PrdocsResult {
            success: false,
            release: normalized_release.to_string(),
            output_dir,
            file_count: 0,
            total_size: 0,
        });
    }

    // Initialize manifest data collectors
    let mut pr_numbers: Vec<u32> = Vec::new();
    let mut crate_changes: HashMap<String, CrateChangeInfo> = HashMap::new();
    let mut audience_counts: HashMap<String, AudienceInfo> = HashMap::new();

    // Fetch content of each prdoc file and save to disk
    let mut total_size = 0;
    let mut saved_count = 0;

    for file in &prdoc_files {
        if let Some(download_url) = &file.download_url {
            let file_response = client
                .get(download_url)
                .header("User-Agent", "substrate-mcp")
                .send()
                .await
                .map_err(|e| anyhow!("Failed to fetch file {}: {}", file.name, e))?;

            if file_response.status().is_success() {
                let content = file_response
                    .text()
                    .await
                    .map_err(|e| anyhow!("Failed to read file {}: {}", file.name, e))?;

                // Save to disk
                let file_path = output_dir.join(&file.name);
                fs::write(&file_path, &content)
                    .await
                    .map_err(|e| anyhow!("Failed to write file {}: {}", file_path.display(), e))?;

                total_size += content.len();
                saved_count += 1;

                // Extract PR number from filename (pr_XXXX.prdoc)
                if let Some(pr_num) = extract_pr_number(&file.name) {
                    pr_numbers.push(pr_num);

                    // Try to parse PRDoc content
                    if let Ok(prdoc) = serde_yaml::from_str::<PrDoc>(&content) {
                        // Process audiences
                        for doc_entry in &prdoc.doc {
                            // Handle both single and multiple audiences
                            let audiences: Vec<&str> = match &doc_entry.audience {
                                AudienceField::Single(s) => vec![s.as_str()],
                                AudienceField::Multiple(v) => {
                                    v.iter().map(|s| s.as_str()).collect()
                                }
                            };

                            for audience in audiences {
                                let audience_info = audience_counts
                                    .entry(audience.to_string())
                                    .or_insert_with(|| AudienceInfo {
                                        count: 0,
                                        pr_numbers: vec![],
                                    });
                                audience_info.count += 1;
                                audience_info.pr_numbers.push(pr_num);
                            }
                        }

                        // Process crates
                        for crate_entry in &prdoc.crates {
                            let crate_info = crate_changes
                                .entry(crate_entry.name.clone())
                                .or_insert_with(|| CrateChangeInfo {
                                    total: 0,
                                    major: 0,
                                    minor: 0,
                                    patch: 0,
                                    none: 0,
                                    pr_numbers: vec![],
                                });

                            crate_info.total += 1;
                            crate_info.pr_numbers.push(pr_num);

                            match crate_entry.bump.as_str() {
                                "major" => crate_info.major += 1,
                                "minor" => crate_info.minor += 1,
                                "patch" => crate_info.patch += 1,
                                "none" => crate_info.none += 1,
                                _ => {} // Unknown bump type
                            }
                        }
                    } else if let Err(e) = serde_yaml::from_str::<PrDoc>(&content) {
                        eprintln!("Failed to parse PRDoc {}: {}", file.name, e);
                    }
                }
            } else {
                eprintln!("Failed to fetch {}: {}", file.name, file_response.status());
            }
        }
    }

    // Sort PR numbers
    pr_numbers.sort_unstable();

    // Create manifest.json
    let manifest = Manifest {
        release: normalized_release.to_string(),
        total_prdocs: saved_count,
        pr_numbers: pr_numbers.clone(),
        download_date: chrono::Utc::now().to_rfc3339(),
        total_size_bytes: total_size,
    };

    let manifest_path = output_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json)
        .await
        .map_err(|e| anyhow!("Failed to write manifest.json: {}", e))?;

    // Create crate_summary.json
    let total_crates = crate_changes.len();
    let total_changes: usize = crate_changes.values().map(|info| info.total).sum();

    let crate_summary = CrateSummary {
        summary: CrateSummaryStats {
            total_crates_affected: total_crates,
            total_changes,
        },
        crates: crate_changes,
    };

    let crate_summary_path = output_dir.join("crate_summary.json");
    let crate_summary_json = serde_json::to_string_pretty(&crate_summary)?;
    fs::write(&crate_summary_path, crate_summary_json)
        .await
        .map_err(|e| anyhow!("Failed to write crate_summary.json: {}", e))?;

    // Create audience_summary.json - now dynamic, includes all found audiences
    let audience_summary_path = output_dir.join("audience_summary.json");
    let audience_summary_json = serde_json::to_string_pretty(&audience_counts)?;
    fs::write(&audience_summary_path, audience_summary_json)
        .await
        .map_err(|e| anyhow!("Failed to write audience_summary.json: {}", e))?;

    // Create a release summary documentation file that provides
    // an overview of all downloaded PRDocs and manifest files.
    // This serves as an index/guide for users exploring the release data.
    let summary_content = format!(
        r#"# Polkadot SDK {} Release PRDocs

This directory contains the PRDoc (Pull Request Documentation) files for the Polkadot SDK {} release.

## Overview

- Total PRDocs: {}
- Successfully downloaded: {}
- Total size: {} bytes

## File Format

Each PRDoc file follows the Polkadot SDK PRDoc Schema v1.0.0 and contains:
- **title**: Brief description of the change
- **doc**: Detailed documentation for different audiences (Runtime Dev, Node Dev, Runtime User, Node Operator)
- **crates**: List of affected crates with their bump levels (major, minor, patch)

## Manifest Files

This directory includes JSON manifest files for efficient analysis:
- `manifest.json` - Basic metadata about this release
- `crate_summary.json` - Breakdown of changes by crate and severity
- `audience_summary.json` - Changes grouped by target audience
- `labels.json` - Complete GitHub label definitions from the repository
- `RELEASE_SUMMARY.md` - This file, providing an overview of the release data

## Usage

These PRDocs document changes, improvements, and new features in the {} release.
Each file corresponds to a pull request that was included in this release.
"#,
        normalized_release,
        normalized_release,
        prdoc_files.len(),
        saved_count,
        total_size,
        normalized_release
    );

    let summary_path = output_dir.join("RELEASE_SUMMARY.md");
    fs::write(&summary_path, summary_content)
        .await
        .map_err(|e| anyhow!("Failed to write RELEASE_SUMMARY.md: {}", e))?;

    // Fetch and save GitHub labels
    if let Err(e) = fetch_and_save_github_labels(&client, &output_dir, &pr_numbers).await {
        eprintln!("Warning: Failed to fetch GitHub labels: {e}");
        // Continue without labels - this is non-blocking
    }

    Ok(PrdocsResult {
        success: true,
        release: normalized_release.to_string(),
        output_dir,
        file_count: saved_count,
        total_size,
    })
}

// Helper function to fetch GitHub labels with pagination support
async fn fetch_github_labels(client: &reqwest::Client) -> Result<Vec<GitHubLabel>> {
    let mut all_labels = Vec::new();
    let mut next_url = Some(
        "https://api.github.com/repos/paritytech/polkadot-sdk/labels?per_page=100".to_string(),
    );

    while let Some(url) = next_url {
        let response = client
            .get(&url)
            .header("User-Agent", "substrate-mcp")
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch labels: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "GitHub API returned status {} when fetching labels",
                response.status()
            ));
        }

        // Check for pagination Link header
        next_url = None;
        if let Some(link_header) = response.headers().get("link") {
            if let Ok(link_str) = link_header.to_str() {
                // Parse Link header for next page
                for link_part in link_str.split(',') {
                    if link_part.contains("rel=\"next\"") {
                        if let Some(url_start) = link_part.find('<') {
                            if let Some(url_end) = link_part.find('>') {
                                next_url = Some(link_part[url_start + 1..url_end].to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }

        let page_labels: Vec<GitHubLabel> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse labels response: {}", e))?;

        all_labels.extend(page_labels);
    }

    Ok(all_labels)
}

// Fetch labels and save to JSON file
async fn fetch_and_save_github_labels(
    client: &reqwest::Client,
    output_dir: &Path,
    _pr_numbers: &[u32], // Keeping for potential future use
) -> Result<()> {
    // Fetch all repository labels
    let github_labels = fetch_github_labels(client).await?;

    // Create labels metadata with raw GitHub data
    let labels_metadata = LabelsMetadata {
        fetched_at: chrono::Utc::now().to_rfc3339(),
        repository: "paritytech/polkadot-sdk".to_string(),
        total_labels: github_labels.len(),
        labels: github_labels,
    };

    // Save as labels.json (simple, descriptive name)
    let labels_path = output_dir.join("labels.json");
    let labels_json = serde_json::to_string_pretty(&labels_metadata)?;
    fs::write(&labels_path, labels_json)
        .await
        .map_err(|e| anyhow!("Failed to write labels.json: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_release_input() {
        // Test stripping polkadot- prefix
        assert_eq!(
            normalize_release_input("polkadot-stable2503-8"),
            "stable2503-8"
        );
        assert_eq!(
            normalize_release_input("polkadot-stable2412-1"),
            "stable2412-1"
        );
        assert_eq!(normalize_release_input("polkadot-v1.9.0"), "v1.9.0");

        // Test stripping release- prefix
        assert_eq!(normalize_release_input("release-v1.9.0"), "v1.9.0");
        assert_eq!(normalize_release_input("release-stable2503"), "stable2503");

        // Test no change for already normalized inputs
        assert_eq!(normalize_release_input("stable2503-8"), "stable2503-8");
        assert_eq!(normalize_release_input("1.9.0"), "1.9.0");
        assert_eq!(normalize_release_input("v1.9.0"), "v1.9.0");

        // Test trimming whitespace
        assert_eq!(normalize_release_input("  stable2503-8  "), "stable2503-8");
        assert_eq!(
            normalize_release_input("  polkadot-stable2503-8  "),
            "stable2503-8"
        );
    }

    #[tokio::test]
    async fn test_fetch_and_analyze_release_valid_release() {
        // This test requires network access
        let result = fetch_and_analyze_release("stable2412-1").await;
        assert!(result.is_ok());
        let prdocs_result = result.unwrap();
        assert!(prdocs_result.success);
        assert!(prdocs_result.file_count > 0);
        assert!(prdocs_result.output_dir.exists());

        // Check if pr_6463.prdoc was downloaded
        let expected_file = prdocs_result.output_dir.join("pr_6463.prdoc");
        assert!(expected_file.exists());
    }

    #[tokio::test]
    async fn test_fetch_and_analyze_release_stable2412_2() {
        // Test stable2412-2 specifically
        let result = fetch_and_analyze_release("stable2412-2").await;
        assert!(result.is_ok());
        let prdocs_result = result.unwrap();
        assert!(prdocs_result.success);
        assert!(prdocs_result.file_count > 0);
        assert!(prdocs_result.output_dir.exists());

        // Check if manifest files were created
        let manifest_file = prdocs_result.output_dir.join("manifest.json");
        let crate_summary_file = prdocs_result.output_dir.join("crate_summary.json");
        let audience_summary_file = prdocs_result.output_dir.join("audience_summary.json");

        assert!(manifest_file.exists(), "manifest.json should exist");
        assert!(
            crate_summary_file.exists(),
            "crate_summary.json should exist"
        );
        assert!(
            audience_summary_file.exists(),
            "audience_summary.json should exist"
        );
    }

    #[tokio::test]
    async fn test_fetch_and_analyze_release_with_polkadot_prefix() {
        // Test that the function correctly handles inputs with 'polkadot-' prefix
        let result = fetch_and_analyze_release("polkadot-stable2412-1").await;
        assert!(result.is_ok(), "Should handle 'polkadot-' prefix correctly");
        let prdocs_result = result.unwrap();
        assert!(prdocs_result.success);
        // The normalized release name should be stored
        assert_eq!(prdocs_result.release, "stable2412-1");
        assert!(prdocs_result.file_count > 0);
    }

    #[tokio::test]
    async fn test_fetch_and_analyze_release_invalid_release() {
        let result = fetch_and_analyze_release("nonexistent-release").await;
        // Should return an error with helpful message
        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        // Check that the error message contains helpful information
        assert!(error_message.contains("not found") || error_message.contains("does not exist"));
        assert!(error_message.contains("prdoc/"));
    }

    #[test]
    fn test_audience_parsing_string_format() {
        // Test case for PR #6825 format (audience as string)
        let yaml_content = indoc::indoc! {"
            title: Test string audience
            doc:
              - audience: Runtime Dev
                description: Test description
            crates:
              - name: test-crate
                bump: patch
        "};

        let prdoc: Result<PrDoc, _> = serde_yaml::from_str(yaml_content);
        assert!(prdoc.is_ok(), "Failed to parse string audience format");

        let prdoc = prdoc.unwrap();
        assert_eq!(prdoc.doc.len(), 1);

        match &prdoc.doc[0].audience {
            AudienceField::Single(s) => assert_eq!(s, "Runtime Dev"),
            AudienceField::Multiple(_) => panic!("Expected single audience, got multiple"),
        }
    }

    #[test]
    fn test_audience_parsing_array_format() {
        // Test case for PR #7028 format (audience as array)
        let yaml_content = indoc::indoc! {"
            title: Test array audience
            doc:
            - audience:
              - Runtime Dev
              - Runtime User
              description: Test description
            crates:
            - name: test-crate
              bump: major
        "};

        let prdoc: Result<PrDoc, _> = serde_yaml::from_str(yaml_content);
        assert!(
            prdoc.is_ok(),
            "Failed to parse array audience format: {:?}",
            prdoc.err()
        );

        let prdoc = prdoc.unwrap();
        assert_eq!(prdoc.doc.len(), 1);

        match &prdoc.doc[0].audience {
            AudienceField::Single(_) => panic!("Expected multiple audiences, got single"),
            AudienceField::Multiple(v) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], "Runtime Dev");
                assert_eq!(v[1], "Runtime User");
            }
        }
    }

    #[test]
    fn test_audience_parsing_inline_array_format() {
        // Test case for PR #7074 format (audience as inline array)
        let yaml_content = indoc::indoc! {"
            title: Test inline array audience
            doc:
              - audience: [ Node Dev, Runtime Dev]
                description: Test description
            crates: [ ]
        "};

        let prdoc: Result<PrDoc, _> = serde_yaml::from_str(yaml_content);
        assert!(
            prdoc.is_ok(),
            "Failed to parse inline array audience format: {:?}",
            prdoc.err()
        );

        let prdoc = prdoc.unwrap();
        assert_eq!(prdoc.doc.len(), 1);

        match &prdoc.doc[0].audience {
            AudienceField::Single(_) => panic!("Expected multiple audiences, got single"),
            AudienceField::Multiple(v) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], "Node Dev");
                assert_eq!(v[1], "Runtime Dev");
            }
        }
    }

    #[tokio::test]
    async fn test_stable2503_7_specific() {
        // Test downloading stable2503-7 specifically
        let result = fetch_and_analyze_release("stable2503-7").await;
        assert!(
            result.is_ok(),
            "Failed to download stable2503-7: {:?}",
            result.err()
        );

        let prdocs_result = result.unwrap();
        assert!(prdocs_result.success);
        assert_eq!(prdocs_result.file_count, 14); // We know there are 14 files

        // Check manifest files
        let manifest_file = prdocs_result.output_dir.join("manifest.json");
        let crate_summary_file = prdocs_result.output_dir.join("crate_summary.json");
        let audience_summary_file = prdocs_result.output_dir.join("audience_summary.json");

        assert!(
            manifest_file.exists(),
            "manifest.json should exist for stable2503-7"
        );
        assert!(
            crate_summary_file.exists(),
            "crate_summary.json should exist for stable2503-7"
        );
        assert!(
            audience_summary_file.exists(),
            "audience_summary.json should exist for stable2503-7"
        );

        // Load and verify audience summary
        let audience_json = std::fs::read_to_string(&audience_summary_file)
            .expect("Failed to read audience_summary.json");
        let audience_counts: HashMap<String, AudienceInfo> =
            serde_json::from_str(&audience_json).expect("Failed to parse audience_summary.json");

        // Ensure we have at least some audiences indexed
        assert!(!audience_counts.is_empty(), "No audiences were indexed");

        // Load and verify crate summary
        let crate_json = std::fs::read_to_string(&crate_summary_file)
            .expect("Failed to read crate_summary.json");
        let crate_data: serde_json::Value =
            serde_json::from_str(&crate_json).expect("Failed to parse crate_summary.json");

        assert!(
            crate_data.get("summary").is_some(),
            "Crate summary should have a summary field"
        );
        assert!(
            crate_data.get("crates").is_some(),
            "Crate summary should have a crates field"
        );
    }

    #[tokio::test]
    async fn test_audience_indexing_regression() {
        // Regression test for missing audiences in indexing
        // This test checks that all PRDocs in stable2412-1 are properly indexed

        let result = fetch_and_analyze_release("stable2412-1").await;
        assert!(result.is_ok());
        let prdocs_result = result.unwrap();

        if prdocs_result.success && prdocs_result.file_count > 0 {
            // Load the audience summary
            let audience_summary_path = prdocs_result.output_dir.join("audience_summary.json");
            let audience_json = tokio::fs::read_to_string(&audience_summary_path)
                .await
                .expect("Failed to read audience_summary.json");
            let audience_counts: HashMap<String, AudienceInfo> =
                serde_json::from_str(&audience_json)
                    .expect("Failed to parse audience_summary.json");

            // PRs that should have audiences (from our analysis)
            let expected_prs_with_audience = vec![
                6463, 6807, 6825, 6855, 6971, 6973, 7013, 7028, 7050, 7067, 7074, 7090, 7099, 7116,
                7133, 7158, 7205, 7222, 7322, 7344,
            ];

            // Collect all unique PRs that have been indexed
            let mut all_indexed_prs = std::collections::HashSet::new();
            for info in audience_counts.values() {
                for pr_num in &info.pr_numbers {
                    all_indexed_prs.insert(*pr_num);
                }
            }

            // Check that all expected PRs are indexed
            for pr_num in &expected_prs_with_audience {
                assert!(
                    all_indexed_prs.contains(pr_num),
                    "PR {} is missing from audience index",
                    pr_num
                );
            }

            // Verify specific multi-audience PRs
            // PR 7074 should be in both Node Dev and Runtime Dev
            assert!(audience_counts["Node Dev"].pr_numbers.contains(&7074));
            assert!(audience_counts["Runtime Dev"].pr_numbers.contains(&7074));

            // PR 7133 should be in both Node Dev and Node Operator
            assert!(audience_counts["Node Dev"].pr_numbers.contains(&7133));
            assert!(audience_counts["Node Operator"].pr_numbers.contains(&7133));

            // PR 7028 should be in both Runtime Dev and Runtime User
            assert!(audience_counts["Runtime Dev"].pr_numbers.contains(&7028));
            assert!(audience_counts["Runtime User"].pr_numbers.contains(&7028));

            // PR 7067 should be in both Runtime Dev and Runtime User
            assert!(audience_counts["Runtime Dev"].pr_numbers.contains(&7067));
            assert!(audience_counts["Runtime User"].pr_numbers.contains(&7067));
        }
    }
}
