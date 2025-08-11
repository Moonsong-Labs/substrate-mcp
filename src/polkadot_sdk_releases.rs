use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

// GitHub API structures
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    created_at: String,
    prerelease: bool,
}

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

// Version parsing structures
#[derive(Debug, Clone, PartialEq, Eq)]
enum ReleaseVersion {
    Semantic {
        major: u32,
        minor: u32,
        patch: u32,
    },
    Stable {
        year: u32,
        month: u32,
        patch: Option<u32>,
    },
}

impl ReleaseVersion {
    /// Parse a version string into ReleaseVersion
    fn parse(version: &str) -> Option<Self> {
        // Try semantic version first (e.g., "1.9.0" or "v1.9.0")
        let version = version.trim_start_matches('v');

        if let Some((major, rest)) = version.split_once('.') {
            if let Some((minor, patch)) = rest.split_once('.') {
                if let (Ok(major), Ok(minor), Ok(patch)) = (
                    major.parse::<u32>(),
                    minor.parse::<u32>(),
                    patch.parse::<u32>(),
                ) {
                    return Some(ReleaseVersion::Semantic {
                        major,
                        minor,
                        patch,
                    });
                }
            }
        }

        // Try stable version (e.g., "stable2503" or "stable2503-7")
        if let Some(version) = version.strip_prefix("stable") {
            if let Some((yymm, patch_str)) = version.split_once('-') {
                // Has patch suffix
                if yymm.len() == 4 {
                    if let (Ok(year), Ok(month), Ok(patch)) = (
                        yymm[0..2].parse::<u32>(),
                        yymm[2..4].parse::<u32>(),
                        patch_str.parse::<u32>(),
                    ) {
                        return Some(ReleaseVersion::Stable {
                            year: 2000 + year,
                            month,
                            patch: Some(patch),
                        });
                    }
                }
            } else if version.len() == 4 {
                // No patch suffix
                if let (Ok(year), Ok(month)) =
                    (version[0..2].parse::<u32>(), version[2..4].parse::<u32>())
                {
                    return Some(ReleaseVersion::Stable {
                        year: 2000 + year,
                        month,
                        patch: None,
                    });
                }
            }
        }

        None
    }
}

impl Ord for ReleaseVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // Semantic versions comparison
            (
                ReleaseVersion::Semantic {
                    major: m1,
                    minor: n1,
                    patch: p1,
                },
                ReleaseVersion::Semantic {
                    major: m2,
                    minor: n2,
                    patch: p2,
                },
            ) => (m1, n1, p1).cmp(&(m2, n2, p2)),
            // Stable versions comparison
            (
                ReleaseVersion::Stable {
                    year: y1,
                    month: m1,
                    patch: p1,
                },
                ReleaseVersion::Stable {
                    year: y2,
                    month: m2,
                    patch: p2,
                },
            ) => match (y1, m1).cmp(&(y2, m2)) {
                Ordering::Equal => p1.cmp(p2),
                other => other,
            },
            // Stable releases came after semantic versions
            (ReleaseVersion::Semantic { .. }, ReleaseVersion::Stable { .. }) => Ordering::Less,
            (ReleaseVersion::Stable { .. }, ReleaseVersion::Semantic { .. }) => Ordering::Greater,
        }
    }
}

impl PartialOrd for ReleaseVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for ReleaseVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseVersion::Semantic {
                major,
                minor,
                patch,
            } => {
                write!(f, "{major}.{minor}.{patch}")
            }
            ReleaseVersion::Stable { year, month, patch } => {
                if let Some(p) = patch {
                    let y = year % 100;
                    write!(f, "stable{y:02}{month:02}-{p}")
                } else {
                    let y = year % 100;
                    write!(f, "stable{y:02}{month:02}")
                }
            }
        }
    }
}

// In-memory cache for releases
static mut RELEASE_CACHE: Option<Vec<String>> = None;

/// Fetch releases from GitHub, stopping when we reach the current version
async fn fetch_releases_until(current_version: &str) -> Result<Vec<String>> {
    // Check cache first
    unsafe {
        if let Some(ref cache) = RELEASE_CACHE {
            return Ok(cache.clone());
        }
    }

    let client = reqwest::Client::new();
    let mut all_releases = Vec::new();
    let mut page = 1;
    let current = ReleaseVersion::parse(current_version)
        .ok_or_else(|| anyhow!("Invalid current version format: {}", current_version))?;

    'pages: loop {
        let url = format!(
            "https://api.github.com/repos/paritytech/polkadot-sdk/releases?per_page=100&page={page}"
        );

        let response = client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "substrate-mcp")
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch releases: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "GitHub API returned status {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let releases: Vec<GitHubRelease> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse releases: {}", e))?;

        if releases.is_empty() {
            break;
        }

        for release in releases {
            // Skip pre-releases
            if release.prerelease {
                continue;
            }

            // Try to parse the version
            if let Some(version) = ReleaseVersion::parse(&release.tag_name) {
                // Check if we've reached the current version
                if version <= current {
                    break 'pages;
                }

                all_releases.push(release.tag_name.clone());
            }
        }

        page += 1;
    }

    // Cache the results
    unsafe {
        RELEASE_CACHE = Some(all_releases.clone());
    }

    Ok(all_releases)
}

/// Get all releases between two versions (inclusive of target, exclusive of current)
pub async fn get_releases_between(
    current_version: &str,
    target_version: &str,
) -> Result<Vec<String>> {
    let current = ReleaseVersion::parse(current_version)
        .ok_or_else(|| anyhow!("Invalid current version format: {}", current_version))?;
    let target = ReleaseVersion::parse(target_version)
        .ok_or_else(|| anyhow!("Invalid target version format: {}", target_version))?;

    if current >= target {
        return Err(anyhow!(
            "Current version {} must be less than target version {}",
            current_version,
            target_version
        ));
    }

    // Fetch all releases up to (but not including) current version
    let all_releases = fetch_releases_until(current_version).await?;

    // Filter releases between current (exclusive) and target (inclusive)
    let mut releases_in_range = Vec::new();

    for release_tag in all_releases {
        if let Some(version) = ReleaseVersion::parse(&release_tag) {
            if version > current && version <= target {
                releases_in_range.push(release_tag);
            }
        }
    }

    // Sort releases in ascending order
    releases_in_range.sort_by(|a, b| {
        let v1 = ReleaseVersion::parse(a);
        let v2 = ReleaseVersion::parse(b);
        match (v1, v2) {
            (Some(v1), Some(v2)) => v1.cmp(&v2),
            _ => Ordering::Equal,
        }
    });

    Ok(releases_in_range)
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

// Helper function to extract PR number from filename
fn extract_pr_number(filename: &str) -> Option<u32> {
    if let Some(name) = filename.strip_prefix("pr_") {
        if let Some(num_str) = name.strip_suffix(".prdoc") {
            return num_str.parse().ok();
        }
    }
    None
}

pub async fn query_prdocs(release: &str) -> Result<PrdocsResult> {
    let client = reqwest::Client::new();

    // Create directory in current working directory
    let output_dir = PathBuf::from(".")
        .join("polkadot-release-analysis")
        .join("releases")
        .join(release)
        .join("pr-docs");

    // Create directory if it doesn't exist
    fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| anyhow!("Failed to create directory {}: {}", output_dir.display(), e))?;

    // First, get the list of files in the prdoc/{release} folder
    let api_url =
        format!("https://api.github.com/repos/paritytech/polkadot-sdk/contents/prdoc/{release}");

    let response = client
        .get(&api_url)
        .header("User-Agent", "substrate-mcp")
        .send()
        .await
        .map_err(|e| anyhow!("Failed to fetch directory listing: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "GitHub API returned status {}: {}",
            response.status(),
            response.text().await.unwrap_or_default()
        ));
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
            release: release.to_string(),
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
        release: release.to_string(),
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

    // Create a README.md with summary information
    let readme_content = format!(
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

## Usage

These PRDocs document changes, improvements, and new features in the {} release.
Each file corresponds to a pull request that was included in this release.
"#,
        release,
        release,
        prdoc_files.len(),
        saved_count,
        total_size,
        release
    );

    let readme_path = output_dir.join("README.md");
    fs::write(&readme_path, readme_content)
        .await
        .map_err(|e| anyhow!("Failed to write README.md: {}", e))?;

    // Fetch and save GitHub labels
    if let Err(e) = fetch_and_save_labels(&client, &output_dir, &pr_numbers).await {
        eprintln!("Warning: Failed to fetch GitHub labels: {e}");
        // Continue without labels - this is non-blocking
    }

    Ok(PrdocsResult {
        success: true,
        release: release.to_string(),
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
async fn fetch_and_save_labels(
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

    #[tokio::test]
    async fn test_query_prdocs_valid_release() {
        // This test requires network access
        let result = query_prdocs("stable2412-1").await;
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
    async fn test_query_prdocs_stable2412_2() {
        // Test stable2412-2 specifically
        let result = query_prdocs("stable2412-2").await;
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
    async fn test_query_prdocs_invalid_release() {
        let result = query_prdocs("nonexistent-release").await;
        // Should return Ok with success=false or an error
        match result {
            Ok(prdocs_result) => assert!(!prdocs_result.success),
            Err(_) => {} // Also acceptable
        }
    }

    #[test]
    fn test_audience_parsing_string_format() {
        // Test case for PR #6825 format (audience as string)
        let yaml_content = r#"
title: Test string audience
doc:
  - audience: Runtime Dev
    description: Test description
crates:
  - name: test-crate
    bump: patch
"#;

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
        let yaml_content = r#"
title: Test array audience
doc:
- audience:
  - Runtime Dev
  - Runtime User
  description: Test description
crates:
- name: test-crate
  bump: major
"#;

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
        let yaml_content = r#"
title: Test inline array audience
doc:
  - audience: [ Node Dev, Runtime Dev]
    description: Test description
crates: [ ]
"#;

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
        let result = query_prdocs("stable2503-7").await;
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

        let result = query_prdocs("stable2412-1").await;
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

    #[test]
    fn test_version_parsing() {
        // Semantic versions
        assert_eq!(
            ReleaseVersion::parse("1.9.0"),
            Some(ReleaseVersion::Semantic {
                major: 1,
                minor: 9,
                patch: 0
            })
        );
        assert_eq!(
            ReleaseVersion::parse("v1.9.0"),
            Some(ReleaseVersion::Semantic {
                major: 1,
                minor: 9,
                patch: 0
            })
        );

        // Stable versions without patch
        assert_eq!(
            ReleaseVersion::parse("stable2502"),
            Some(ReleaseVersion::Stable {
                year: 2025,
                month: 2,
                patch: None
            })
        );

        // Stable versions with patch
        assert_eq!(
            ReleaseVersion::parse("stable2503-7"),
            Some(ReleaseVersion::Stable {
                year: 2025,
                month: 3,
                patch: Some(7)
            })
        );

        // Invalid formats
        assert_eq!(ReleaseVersion::parse("invalid"), None);
        assert_eq!(ReleaseVersion::parse("1.9"), None);
        assert_eq!(ReleaseVersion::parse("stable25"), None);
    }

    #[test]
    fn test_version_to_string() {
        let semantic = ReleaseVersion::Semantic {
            major: 1,
            minor: 9,
            patch: 0,
        };
        assert_eq!(semantic.to_string(), "1.9.0");

        let stable_no_patch = ReleaseVersion::Stable {
            year: 2025,
            month: 2,
            patch: None,
        };
        assert_eq!(stable_no_patch.to_string(), "stable2502");

        let stable_with_patch = ReleaseVersion::Stable {
            year: 2025,
            month: 3,
            patch: Some(7),
        };
        assert_eq!(stable_with_patch.to_string(), "stable2503-7");
    }

    #[test]
    fn test_version_ordering() {
        let v1_8_0 = ReleaseVersion::parse("1.8.0").unwrap();
        let v1_9_0 = ReleaseVersion::parse("1.9.0").unwrap();
        let v1_10_0 = ReleaseVersion::parse("1.10.0").unwrap();
        let stable2502 = ReleaseVersion::parse("stable2502").unwrap();
        let stable2503 = ReleaseVersion::parse("stable2503").unwrap();
        let stable2503_1 = ReleaseVersion::parse("stable2503-1").unwrap();
        let stable2503_7 = ReleaseVersion::parse("stable2503-7").unwrap();

        // Semantic version ordering
        assert!(v1_8_0 < v1_9_0);
        assert!(v1_9_0 < v1_10_0);

        // Stable version ordering
        assert!(stable2502 < stable2503);
        assert!(stable2503 < stable2503_1);
        assert!(stable2503_1 < stable2503_7);

        // Cross-type ordering (stable came after semantic)
        assert!(v1_10_0 < stable2502);
    }

    #[test]
    fn test_patch_version_ordering() {
        // Test that base version comes before patched versions
        let base = ReleaseVersion::parse("stable2503").unwrap();
        let patch1 = ReleaseVersion::parse("stable2503-1").unwrap();
        let patch7 = ReleaseVersion::parse("stable2503-7").unwrap();

        assert!(base < patch1);
        assert!(patch1 < patch7);
        assert!(base < patch7);
    }
}
