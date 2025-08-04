use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Deserialize)]
struct GitHubContent {
    name: String,
    #[serde(rename = "type")]
    content_type: String,
    download_url: Option<String>,
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
struct DocEntry {
    audience: String,
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

#[derive(Debug, Serialize)]
struct AudienceInfo {
    count: usize,
    pr_numbers: Vec<u32>,
}

#[derive(Debug, Serialize)]
struct AudienceSummary {
    #[serde(rename = "Runtime Dev")]
    runtime_dev: AudienceInfo,
    #[serde(rename = "Node Dev")]
    node_dev: AudienceInfo,
    #[serde(rename = "Runtime User")]
    runtime_user: AudienceInfo,
    #[serde(rename = "Node Operator")]
    node_operator: AudienceInfo,
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

pub async fn query_prdocs(release: &str) -> Result<PrdocsResult> {
    let client = reqwest::Client::new();

    // Create deterministic directory path based on release
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let output_dir = PathBuf::from(home_dir)
        .join(".substrate-mcp")
        .join("prdocs")
        .join(format!("prdocs-{}", release));

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
    
    // Initialize audience counts
    audience_counts.insert("Runtime Dev".to_string(), AudienceInfo { count: 0, pr_numbers: vec![] });
    audience_counts.insert("Node Dev".to_string(), AudienceInfo { count: 0, pr_numbers: vec![] });
    audience_counts.insert("Runtime User".to_string(), AudienceInfo { count: 0, pr_numbers: vec![] });
    audience_counts.insert("Node Operator".to_string(), AudienceInfo { count: 0, pr_numbers: vec![] });

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
                            if let Some(audience_info) = audience_counts.get_mut(&doc_entry.audience) {
                                audience_info.count += 1;
                                audience_info.pr_numbers.push(pr_num);
                            }
                        }
                        
                        // Process crates
                        for crate_entry in &prdoc.crates {
                            let crate_info = crate_changes.entry(crate_entry.name.clone())
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
                    } else {
                        eprintln!("Failed to parse PRDoc {}: invalid YAML format", file.name);
                    }
                }
            } else {
                eprintln!("Failed to fetch {}: {}", file.name, file_response.status());
            }
        }
    }

    // Sort PR numbers
    pr_numbers.sort_unstable();
    
    eprintln!("DEBUG: Creating manifest files for {} PRDocs", saved_count);
    eprintln!("DEBUG: PR numbers: {:?}", pr_numbers);
    eprintln!("DEBUG: Crate changes: {} crates affected", crate_changes.len());
    
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
    
    // Create audience_summary.json
    let audience_summary = AudienceSummary {
        runtime_dev: audience_counts.remove("Runtime Dev").unwrap_or(AudienceInfo { count: 0, pr_numbers: vec![] }),
        node_dev: audience_counts.remove("Node Dev").unwrap_or(AudienceInfo { count: 0, pr_numbers: vec![] }),
        runtime_user: audience_counts.remove("Runtime User").unwrap_or(AudienceInfo { count: 0, pr_numbers: vec![] }),
        node_operator: audience_counts.remove("Node Operator").unwrap_or(AudienceInfo { count: 0, pr_numbers: vec![] }),
    };
    
    let audience_summary_path = output_dir.join("audience_summary.json");
    let audience_summary_json = serde_json::to_string_pretty(&audience_summary)?;
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

## Usage

These PRDocs document changes, improvements, and new features in the {} release.
Each file corresponds to a pull request that was included in this release.
"#,
        release, release, prdoc_files.len(), saved_count, total_size, release
    );

    let readme_path = output_dir.join("README.md");
    fs::write(&readme_path, readme_content)
        .await
        .map_err(|e| anyhow!("Failed to write README.md: {}", e))?;

    Ok(PrdocsResult {
        success: true,
        release: release.to_string(),
        output_dir,
        file_count: saved_count,
        total_size,
    })
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
        
        eprintln!("Checking for manifest at: {:?}", manifest_file);
        eprintln!("Manifest exists: {}", manifest_file.exists());
        eprintln!("Crate summary exists: {}", crate_summary_file.exists());
        eprintln!("Audience summary exists: {}", audience_summary_file.exists());
        
        assert!(manifest_file.exists(), "manifest.json should exist");
        assert!(crate_summary_file.exists(), "crate_summary.json should exist");
        assert!(audience_summary_file.exists(), "audience_summary.json should exist");
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
}
