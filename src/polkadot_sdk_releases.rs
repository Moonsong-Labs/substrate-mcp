use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
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
            } else {
                eprintln!("Failed to fetch {}: {}", file.name, file_response.status());
            }
        }
    }

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
    async fn test_query_prdocs_invalid_release() {
        let result = query_prdocs("nonexistent-release").await;
        // Should return Ok with success=false or an error
        match result {
            Ok(prdocs_result) => assert!(!prdocs_result.success),
            Err(_) => {} // Also acceptable
        }
    }
}
