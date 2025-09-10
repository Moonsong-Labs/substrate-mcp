use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;
use zeroize::Zeroizing;

/// Create a GitHub API client with optional authentication
/// Uses GITHUB_TOKEN environment variable if available for higher rate limits
fn create_github_client() -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();

    // Always add User-Agent header
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("substrate-mcp"),
    );

    // Check for GitHub token and add Authorization header if present
    if let Ok(token_str) = env::var("GITHUB_TOKEN") {
        let token = Zeroizing::new(token_str);
        if !token.is_empty() {
            let auth_value = format!("Bearer {}", token.as_str());
            if let Ok(header_value) = reqwest::header::HeaderValue::from_str(&auth_value) {
                headers.insert(reqwest::header::AUTHORIZATION, header_value);
                log::debug!("Using authenticated GitHub API requests");
            } else {
                log::warn!("Invalid GITHUB_TOKEN format, using unauthenticated requests");
            }
        }
    } else {
        log::debug!("No GITHUB_TOKEN found, using unauthenticated requests (60 req/hour limit)");
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to create HTTP client")
}

// GitHub API structures
#[derive(Debug, Deserialize)]
struct GitHubContent {
    name: String,
    #[serde(rename = "type")]
    content_type: String,
    download_url: Option<String>,
}

/// GitHub API response for repository contents listing
#[derive(Debug, Deserialize)]
struct GitHubContentsResponse(Vec<GitHubContent>);

impl GitHubContentsResponse {
    /// Filter items by type (e.g., "dir", "file")
    fn filter_by_type(&self, content_type: &str) -> Vec<&GitHubContent> {
        self.0
            .iter()
            .filter(|item| item.content_type == content_type)
            .collect()
    }

    /// Get all directory items
    fn directories(&self) -> Vec<&GitHubContent> {
        self.filter_by_type("dir")
    }

    /// Get all file items
    fn files(&self) -> Vec<&GitHubContent> {
        self.filter_by_type("file")
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct GitHubLabel {
    pub(crate) name: String,
    pub(crate) color: String,
    pub(crate) description: Option<String>,
}

// Label metadata structure
#[derive(Debug, Serialize)]
struct LabelsMetadata {
    fetched_at: String,
    repository: String,
    total_labels: usize,
    labels: Vec<GitHubLabel>,
}

/// New structured result for the refactored workflow
#[derive(Debug, Serialize)]
pub(crate) struct EnhancedPrdocsResult {
    pub(crate) prdocs: Vec<PrDocWithLabels>,
    pub(crate) label_definitions: HashMap<String, GitHubLabel>,
    pub(crate) summary: ReleaseDownloadSummary,
}

/// PRDoc with associated labels
#[derive(Debug, Serialize, Clone)]
pub(crate) struct PrDocWithLabels {
    pub(crate) pr_number: u32,
    pub(crate) file_path: String,
    pub(crate) labels: Vec<String>,
}

/// Summary information about the release download
#[derive(Debug, Serialize)]
pub(crate) struct ReleaseDownloadSummary {
    pub(crate) release: String,
    pub(crate) total_prs: usize,
    pub(crate) download_date: String,
    pub(crate) output_directory: String,
}

// Helper function to extract PR number from filename
fn extract_pr_number(filename: &str) -> Option<u32> {
    if let Some(name) = filename.strip_prefix("pr_")
        && let Some(num_str) = name.strip_suffix(".prdoc")
    {
        return num_str.parse().ok();
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
            if let Ok(contents) = std::fs::read_to_string(&cargo_toml)
                && contents.contains("[workspace]")
            {
                return Some(dir.to_path_buf());
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

/// Result of listing available releases
#[derive(Debug, Serialize)]
pub(crate) struct AvailableReleases {
    pub(crate) releases: Vec<String>,
    pub(crate) total_count: usize,
    pub(crate) fetched_at: String,
}

/// List all available releases from the polkadot-sdk repository
pub(crate) async fn list_available_releases() -> Result<AvailableReleases> {
    let client = create_github_client();

    // Get the list of directories in the prdoc folder
    let api_url = "https://api.github.com/repos/paritytech/polkadot-sdk/contents/prdoc";

    let response = client
        .get(api_url)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to fetch prdoc directory listing: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        // Check for rate limiting error and provide helpful guidance
        if status == 403 && error_text.contains("rate limit exceeded") {
            let token_hint = if env::var("GITHUB_TOKEN").is_err() {
                "\n\nTo increase your rate limit from 60 to 5,000 requests/hour, set the GITHUB_TOKEN environment variable:\n\
                export GITHUB_TOKEN=\"your_github_token_here\"\n\
                \nYou can generate a token at: https://github.com/settings/tokens\n\
                (No special permissions required - just create a personal access token)"
            } else {
                ""
            };

            return Err(anyhow!(
                "GitHub API rate limit exceeded: {}{}",
                error_text,
                token_hint
            ));
        }

        return Err(anyhow!(
            "GitHub API returned status {}: {}",
            status,
            error_text
        ));
    }

    let github_response: GitHubContentsResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse prdoc directory listing: {}", e))?;

    let mut releases: Vec<String> = github_response
        .directories()
        .iter()
        .map(|dir| dir.name.clone())
        .collect();

    // Sort releases (newest first)
    releases.sort_by(|a, b| b.cmp(a));

    let total_count = releases.len();

    Ok(AvailableReleases {
        releases,
        total_count,
        fetched_at: chrono::Utc::now().to_rfc3339(),
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
        if let Some(link_header) = response.headers().get("link")
            && let Ok(link_str) = link_header.to_str()
        {
            // Parse Link header for next page
            for link_part in link_str.split(',') {
                if link_part.contains("rel=\"next\"")
                    && let Some(url_start) = link_part.find('<')
                    && let Some(url_end) = link_part.find('>')
                {
                    next_url = Some(link_part[url_start + 1..url_end].to_string());
                    break;
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

// Fetch labels for a specific PR
async fn fetch_pr_labels(client: &reqwest::Client, pr_number: u32) -> Result<Vec<String>> {
    let api_url = format!(
        "https://api.github.com/repos/paritytech/polkadot-sdk/issues/{}/labels",
        pr_number
    );

    let response = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to fetch labels for PR {}: {}", pr_number, e))?;

    if !response.status().is_success() {
        if response.status() == 404 {
            // PR not found or no labels - return empty vec
            return Ok(vec![]);
        }
        return Err(anyhow!(
            "GitHub API returned status {} when fetching labels for PR {}: {}",
            response.status(),
            pr_number,
            response.text().await.unwrap_or_default()
        ));
    }

    let labels: Vec<GitHubLabel> = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse labels for PR {}: {}", pr_number, e))?;

    Ok(labels.into_iter().map(|label| label.name).collect())
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

/// Enhanced version that returns structured data for parallel sub-agent workflow
pub(crate) async fn fetch_and_analyze_release_enhanced(
    release: &str,
) -> Result<EnhancedPrdocsResult> {
    let client = create_github_client();

    // Get project name from the current project root
    let project_name = get_project_name();

    // Create directory under ~/.substrate-mcp/{project}/releases/{release}/pr-docs
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let output_dir = home_dir
        .join(".substrate-mcp")
        .join(project_name.clone())
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
        .send()
        .await
        .map_err(|e| anyhow!("Failed to fetch directory listing: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();

        if status == 404 {
            return Err(anyhow!("Release '{}' not found", release));
        }

        let error_text = response.text().await.unwrap_or_default();

        // Check for rate limiting error and provide helpful guidance
        if status == 403 && error_text.contains("rate limit exceeded") {
            let token_hint = if env::var("GITHUB_TOKEN").is_err() {
                "\n\nTo increase your rate limit from 60 to 5,000 requests/hour, set the GITHUB_TOKEN environment variable:\n\
                export GITHUB_TOKEN=\"your_github_token_here\"\n\
                \nYou can generate a token at: https://github.com/settings/tokens\n\
                (No special permissions required - just create a personal access token)"
            } else {
                ""
            };

            return Err(anyhow!(
                "GitHub API rate limit exceeded: {}{}",
                error_text,
                token_hint
            ));
        }

        return Err(anyhow!(
            "GitHub API returned status {}: {}",
            status,
            error_text
        ));
    }

    let github_response: GitHubContentsResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse directory listing: {}", e))?;

    // Filter for .prdoc files
    let prdoc_files: Vec<&GitHubContent> = github_response
        .files()
        .into_iter()
        .filter(|c| c.name.ends_with(".prdoc"))
        .collect();

    if prdoc_files.is_empty() {
        return Ok(EnhancedPrdocsResult {
            prdocs: vec![],
            label_definitions: HashMap::new(),
            summary: ReleaseDownloadSummary {
                release: release.to_string(),
                total_prs: 0,
                download_date: chrono::Utc::now().to_rfc3339(),
                output_directory: output_dir.to_string_lossy().to_string(),
            },
        });
    }

    // Fetch all repository labels first (for label definitions)
    let github_labels = fetch_github_labels(&client).await?;
    let mut label_definitions: HashMap<String, GitHubLabel> = HashMap::new();
    for label in github_labels {
        label_definitions.insert(label.name.clone(), label);
    }

    // Download PRDocs and fetch their labels
    let mut prdocs_with_labels = Vec::new();

    for file in &prdoc_files {
        if let Some(download_url) = &file.download_url {
            // Extract PR number from filename
            if let Some(pr_num) = extract_pr_number(&file.name) {
                let file_response = client
                    .get(download_url)
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
                    fs::write(&file_path, &content).await.map_err(|e| {
                        anyhow!("Failed to write file {}: {}", file_path.display(), e)
                    })?;

                    // Fetch labels for this PR
                    let labels = fetch_pr_labels(&client, pr_num).await.unwrap_or_default();

                    prdocs_with_labels.push(PrDocWithLabels {
                        pr_number: pr_num,
                        file_path: file_path.to_string_lossy().to_string(),
                        labels,
                    });
                } else {
                    eprintln!("Failed to fetch {}: {}", file.name, file_response.status());
                }
            }
        }
    }

    // Sort by PR number for consistency
    prdocs_with_labels.sort_by(|a, b| a.pr_number.cmp(&b.pr_number));

    // Also save the traditional manifest files for compatibility
    let pr_numbers: Vec<u32> = prdocs_with_labels.iter().map(|p| p.pr_number).collect();

    // Save labels.json for compatibility
    if let Err(e) = fetch_and_save_github_labels(&client, &output_dir, &pr_numbers).await {
        eprintln!("Warning: Failed to save GitHub labels: {e}");
    }

    Ok(EnhancedPrdocsResult {
        prdocs: prdocs_with_labels.clone(),
        label_definitions,
        summary: ReleaseDownloadSummary {
            release: release.to_string(),
            total_prs: prdocs_with_labels.len(),
            download_date: chrono::Utc::now().to_rfc3339(),
            output_directory: output_dir.to_string_lossy().to_string(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_available_releases() {
        // This test requires network access
        let result = list_available_releases().await;
        assert!(result.is_ok());

        let releases = result.unwrap();
        assert!(releases.total_count > 0);
        assert!(!releases.releases.is_empty());

        // Verify that we have expected releases
        assert!(releases.releases.contains(&"stable2503".to_string()));
        assert!(releases.releases.contains(&"stable2412".to_string()));
        assert!(releases.releases.iter().any(|name| name.starts_with("1.")));
    }
}
