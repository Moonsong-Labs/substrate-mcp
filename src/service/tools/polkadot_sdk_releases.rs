use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
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
                eprintln!("Using authenticated GitHub API requests");
            } else {
                eprintln!("Invalid GITHUB_TOKEN format, using unauthenticated requests");
            }
        }
    } else {
        eprintln!("No GITHUB_TOKEN found, using unauthenticated requests (60 req/hour limit)");
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Failed to create HTTP client")
}

// GitHub API structures
#[derive(Debug, Deserialize, Clone)]
pub(crate) struct GitHubContent {
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
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct GitHubLabel {
    pub(crate) name: String,
    pub(crate) color: String,
    pub(crate) description: Option<String>,
}

/// PR labels mapping for caching which labels belong to which PR
#[derive(Debug, Serialize, Deserialize)]
struct PrLabelsMapping {
    fetched_at: DateTime<Utc>,
    /// PR number → list of label names
    pr_labels: HashMap<u32, Vec<String>>,
}

/// Minimal struct to parse PRDoc YAML for title and description extraction
#[derive(Debug, Deserialize)]
struct PrDocYaml {
    title: String,
    #[serde(default)]
    doc: Vec<PrDocSection>,
}

/// PRDoc documentation section
#[derive(Debug, Deserialize)]
struct PrDocSection {
    #[serde(default, rename = "audience")]
    _audience: AudienceField,
    #[serde(default)]
    description: String,
}

/// Audience field can be either a string or an array of strings
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AudienceField {
    Single(#[allow(dead_code)] String),
    Multiple(#[allow(dead_code)] Vec<String>),
}

impl Default for AudienceField {
    fn default() -> Self {
        AudienceField::Single(String::new())
    }
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
    pub(crate) title: String,
    pub(crate) description: String,
}

/// Summary information about the release download
#[derive(Debug, Serialize)]
pub(crate) struct ReleaseDownloadSummary {
    pub(crate) release: String,
    pub(crate) total_prs: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) download_date: Option<DateTime<Utc>>,
    pub(crate) output_directory: String,
}

/// Trait for abstracting GitHub API operations
#[async_trait]
pub(crate) trait GitHubApiClient: Send + Sync {
    /// Get directory contents from GitHub repository
    async fn get_directory_contents(&self, path: &str) -> Result<Vec<GitHubContent>>;

    /// Download file content from a URL
    async fn get_file_content(&self, url: &str) -> Result<String>;

    /// Get labels for a specific PR
    async fn get_pr_labels(&self, pr_number: u32) -> Result<Vec<String>>;

    /// Get all repository labels
    async fn get_repository_labels(&self) -> Result<Vec<GitHubLabel>>;
}

/// Real GitHub API client implementation
pub(crate) struct GithubClient {
    client: reqwest::Client,
}

impl GithubClient {
    pub(crate) fn new() -> Self {
        Self {
            client: create_github_client(),
        }
    }
}

#[async_trait]
impl GitHubApiClient for GithubClient {
    async fn get_directory_contents(&self, path: &str) -> Result<Vec<GitHubContent>> {
        let api_url = format!(
            "https://api.github.com/repos/paritytech/polkadot-sdk/contents/{}",
            path
        );

        let response = self
            .client
            .get(&api_url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch directory listing: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();

            if status == 404 {
                return Err(anyhow!("Path '{}' not found", path));
            }

            let error_text = response.text().await.unwrap_or_default();

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
            .map_err(|e| anyhow!("Failed to parse directory listing: {e}"))?;

        Ok(github_response.0)
    }

    async fn get_file_content(&self, url: &str) -> Result<String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch file: {e}"))?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch file: HTTP {}", response.status()));
        }

        response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read file content: {e}"))
    }

    async fn get_pr_labels(&self, pr_number: u32) -> Result<Vec<String>> {
        let api_url = format!(
            "https://api.github.com/repos/paritytech/polkadot-sdk/issues/{}/labels",
            pr_number
        );

        let response = self
            .client
            .get(&api_url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch labels for PR {pr_number}: {e}"))?;

        if !response.status().is_success() {
            if response.status() == 404 {
                // PR not found or no labels - return empty vec
                return Ok(vec![]);
            }
            return Err(anyhow!(
                "GitHub API returned status {} when fetching labels for PR {}",
                response.status(),
                pr_number
            ));
        }

        let labels: Vec<GitHubLabel> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse labels for PR {pr_number}: {e}"))?;

        Ok(labels.into_iter().map(|label| label.name).collect())
    }

    async fn get_repository_labels(&self) -> Result<Vec<GitHubLabel>> {
        let mut all_labels = Vec::new();
        let mut next_url = Some(
            "https://api.github.com/repos/paritytech/polkadot-sdk/labels?per_page=100".to_string(),
        );

        while let Some(url) = next_url {
            let response = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| anyhow!("Failed to fetch labels: {e}"))?;

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
                .map_err(|e| anyhow!("Failed to parse labels response: {e}"))?;

            all_labels.extend(page_labels);
        }

        Ok(all_labels)
    }
}

/// Normalize release input to handle different formats users might provide
/// Strips common prefixes like 'polkadot-' or 'release-' that come from git tags
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

    pub(crate) fetched_at: DateTime<Utc>,
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
        .map_err(|e| anyhow!("Failed to fetch prdoc directory listing: {e}"))?;

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
        .map_err(|e| anyhow!("Failed to parse prdoc directory listing: {e}"))?;

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
        fetched_at: Utc::now(),
    })
}

/// Load cached PR docs from disk
async fn load_cached_prdocs(output_dir: &Path) -> Result<EnhancedPrdocsResult> {
    // Load the PR labels mapping
    let pr_labels_path = output_dir.join("pr_labels_mapping.json");
    let pr_labels_map = if pr_labels_path.exists() {
        let pr_labels_content = fs::read_to_string(&pr_labels_path).await?;
        if let Ok(pr_labels_mapping) = serde_json::from_str::<PrLabelsMapping>(&pr_labels_content) {
            pr_labels_mapping.pr_labels
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    };

    // Read all .prdoc files from the directory
    let mut prdocs_with_labels = Vec::new();
    let mut entries = fs::read_dir(output_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(filename) = path.file_name()
            && let Some(filename_str) = filename.to_str()
            && filename_str.ends_with(".prdoc")
            && let Some(pr_num) = extract_pr_number(filename_str)
        {
            // Load labels from the cached mapping
            let labels = pr_labels_map.get(&pr_num).cloned().unwrap_or_default();

            // Parse title and description from file
            let (title, description) = match fs::read_to_string(&path).await {
                Ok(content) => match serde_yaml::from_str::<PrDocYaml>(&content) {
                    Ok(doc) => {
                        let desc = doc
                            .doc
                            .iter()
                            .map(|section| section.description.trim())
                            .filter(|d| !d.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ");
                        (doc.title, desc)
                    }
                    Err(e) => {
                        log::warn!("Failed to parse cached PRDoc for PR {}: {}", pr_num, e);
                        (format!("PR #{}", pr_num), String::new())
                    }
                },
                Err(_) => (format!("PR #{}", pr_num), String::new()),
            };

            prdocs_with_labels.push(PrDocWithLabels {
                pr_number: pr_num,
                file_path: path.to_string_lossy().to_string(),
                labels,
                title,
                description,
            });
        }
    }

    // Sort by PR number for consistency
    prdocs_with_labels.sort_by(|a, b| a.pr_number.cmp(&b.pr_number));

    // Load label definitions from repository labels
    let label_definitions = HashMap::new(); // Will be populated from API calls in the future

    // Get release name from the directory structure
    let release_name = output_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(EnhancedPrdocsResult {
        prdocs: prdocs_with_labels.clone(),
        label_definitions,
        summary: ReleaseDownloadSummary {
            release: release_name,
            total_prs: prdocs_with_labels.len(),
            download_date: None, // None indicates this is from cache
            output_directory: output_dir.to_string_lossy().to_string(),
        },
    })
}

/// Check if cached data exists and is valid
async fn has_valid_cache(output_dir: &Path) -> bool {
    // Check if directory exists
    if !output_dir.exists() {
        return false;
    }

    // Check if we have at least one .prdoc file and pr_labels_mapping.json
    let mut has_prdoc = false;
    let has_pr_labels_mapping = output_dir.join("pr_labels_mapping.json").exists();

    if let Ok(mut entries) = fs::read_dir(output_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Some(filename) = entry.path().file_name()
                && let Some(filename_str) = filename.to_str()
                && filename_str.ends_with(".prdoc")
            {
                has_prdoc = true;
                break;
            }
        }
    }

    has_prdoc && has_pr_labels_mapping
}

/// Enhanced version that returns structured data for parallel sub-agent workflow
///
/// # Arguments
/// * `release` - The release identifier (e.g., "stable2503", "polkadot-stable2503-1")
/// * `force` - Force re-download even if cached
/// * `cache_dir` - Optional cache directory (defaults to ~/.substrate-mcp/{project}/releases)
/// * `client` - GitHub API client implementation to use for requests
pub(crate) async fn fetch_and_analyze_release(
    release: &str,
    force: bool,
    cache_dir: Option<&Path>,
    client: &dyn GitHubApiClient,
) -> Result<EnhancedPrdocsResult> {
    // Use provided cache dir or default
    let default_cache_dir;
    let cache_base_dir = if let Some(dir) = cache_dir {
        dir
    } else {
        // Get project name from the current project root
        let project_name = get_project_name();
        // Create directory under ~/.substrate-mcp/{project}/releases/{release}/pr-docs
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        default_cache_dir = home_dir
            .join(".substrate-mcp")
            .join(project_name)
            .join("releases");
        &default_cache_dir
    };

    // Normalize the release input to handle different formats
    // This strips prefixes like 'polkadot-' that users might include from git tags
    let normalized_release = normalize_release_input(release);

    // Log if normalization changed the input
    if normalized_release != release {
        log::info!(
            "Normalized release name from '{}' to '{}'",
            release,
            normalized_release
        );
    }

    // We use the normalized release name for the directory structure
    let output_dir = cache_base_dir.join(&normalized_release).join("pr-docs");

    // Check if we have valid cached data and force is not set
    if !force && has_valid_cache(&output_dir).await {
        log::info!(
            "Using cached PR docs for release '{}' from {}",
            normalized_release,
            output_dir.display()
        );
        return load_cached_prdocs(&output_dir).await;
    }

    // If force is set, log that we're re-downloading
    if force && output_dir.exists() {
        log::info!(
            "Force flag set, re-downloading PR docs for release '{}' (replacing existing cache)",
            normalized_release
        );
    }

    // Create directory if it doesn't exist
    fs::create_dir_all(&output_dir)
        .await
        .map_err(|e| anyhow!("Failed to create directory {}: {}", output_dir.display(), e))?;

    // First, get the list of files in the prdoc/{release} folder
    let path = format!("prdoc/{}", normalized_release);

    let directory_contents = client
        .get_directory_contents(&path)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                anyhow!(
                    "Release '{}' not found. This means the directory 'prdoc/{}' does not exist on the main branch.\n\
                    Common issues:\n\
                    - If you used a git tag like 'polkadot-stable2503-8', it was normalized to 'stable2503-8'\n\
                    - The release may not exist yet or may use a different naming format\n\
                    - Check available releases at: https://github.com/paritytech/polkadot-sdk/tree/master/prdoc",
                    normalized_release,
                    normalized_release
                )
            } else {
                e
            }
        })?;

    // Filter for .prdoc files
    let prdoc_files: Vec<GitHubContent> = directory_contents
        .into_iter()
        .filter(|c| c.content_type == "file" && c.name.ends_with(".prdoc"))
        .collect();

    if prdoc_files.is_empty() {
        return Ok(EnhancedPrdocsResult {
            prdocs: vec![],
            label_definitions: HashMap::new(),
            summary: ReleaseDownloadSummary {
                release: normalized_release.to_string(),
                total_prs: 0,
                download_date: Some(Utc::now()),
                output_directory: output_dir.to_string_lossy().to_string(),
            },
        });
    }

    // Fetch all repository labels first (for label definitions)
    let github_labels = client.get_repository_labels().await?;
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
                let content = client
                    .get_file_content(download_url)
                    .await
                    .map_err(|e| anyhow!("Failed to fetch file {}: {}", file.name, e))?;

                // Save to disk
                let file_path = output_dir.join(&file.name);
                fs::write(&file_path, &content)
                    .await
                    .map_err(|e| anyhow!("Failed to write file {}: {}", file_path.display(), e))?;

                // Parse title and description from PRDoc YAML
                let (title, description) = match serde_yaml::from_str::<PrDocYaml>(&content) {
                    Ok(doc) => {
                        let desc = doc
                            .doc
                            .iter()
                            .map(|section| section.description.trim())
                            .filter(|d| !d.is_empty())
                            .collect::<Vec<_>>()
                            .join(" ");
                        (doc.title, desc)
                    }
                    Err(e) => {
                        log::warn!("Failed to parse PRDoc {}: {}", file.name, e);
                        eprintln!("Failed to parse PRDoc {}: {}", file.name, e);
                        eprintln!("Content preview: {}", &content[..content.len().min(200)]);
                        (format!("PR #{}", pr_num), String::new())
                    }
                };

                // Fetch labels for this PR
                let labels = client.get_pr_labels(pr_num).await.unwrap_or_default();

                prdocs_with_labels.push(PrDocWithLabels {
                    pr_number: pr_num,
                    file_path: file_path.to_string_lossy().to_string(),
                    labels,
                    title,
                    description,
                });
            }
        }
    }

    // Sort by PR number for consistency
    prdocs_with_labels.sort_by(|a, b| a.pr_number.cmp(&b.pr_number));

    // Save PR labels mapping for cache
    let mut pr_labels_map = HashMap::new();
    for prdoc in &prdocs_with_labels {
        pr_labels_map.insert(prdoc.pr_number, prdoc.labels.clone());
    }

    let pr_labels_mapping = PrLabelsMapping {
        fetched_at: Utc::now(),
        pr_labels: pr_labels_map,
    };

    let pr_labels_path = output_dir.join("pr_labels_mapping.json");
    let pr_labels_json = serde_json::to_string_pretty(&pr_labels_mapping)?;
    fs::write(&pr_labels_path, pr_labels_json)
        .await
        .map_err(|e| anyhow!("Failed to write pr_labels_mapping.json: {}", e))?;

    Ok(EnhancedPrdocsResult {
        prdocs: prdocs_with_labels.clone(),
        label_definitions,
        summary: ReleaseDownloadSummary {
            release: normalized_release.to_string(),
            total_prs: prdocs_with_labels.len(),
            download_date: Some(Utc::now()),
            output_directory: output_dir.to_string_lossy().to_string(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::sync::Mutex;

    // ===== Test Helpers and Mock Data =====

    mod test_data {
        pub(super) const STABLE2503_PR1_CONTENT: &str = r#"title: "Test PR 1234"
doc:
  - audience: Runtime Dev
    description: |
      This is a test PR for stable2503
crates:
  - name: pallet-balances"#;

        pub(super) const STABLE2503_PR2_CONTENT: &str = r#"title: "Test PR 5678"
doc:
  - audience: Node Dev
    description: |
      Another test PR for stable2503
crates:
  - name: sp-runtime"#;

        pub(super) const STABLE2503_PR3_CONTENT: &str = r#"title: "Test PR 9012"
doc:
  - audience: Runtime User
    description: |
      Third test PR for stable2503
crates:
  - name: frame-system"#;

        pub(super) const PR_WITH_ARRAY_AUDIENCE: &str = r#"title: Bring the latest compatibility fixes via litep2p v0.9.1

doc:
  - audience: [Node Dev, Node Operator]
    description: |
      This release enhances compatibility between litep2p and libp2p by using the latest Yamux upstream version.
      Additionally, it includes various improvements and fixes to boost the stability and performance of the WebSocket stream and the multistream-select protocol.

crates:
  - name: sc-network
    bump: minor"#;
    }

    // ===== Mock GitHub Client Implementation =====

    /// Mock GitHub API client for testing
    pub(super) struct MockGitHubClient {
        pub directory_contents: Arc<Mutex<HashMap<String, Vec<GitHubContent>>>>,
        pub file_contents: Arc<Mutex<HashMap<String, String>>>,
        pub pr_labels: Arc<Mutex<HashMap<u32, Vec<String>>>>,
        pub repository_labels: Arc<Mutex<Vec<GitHubLabel>>>,
    }

    impl MockGitHubClient {
        pub(super) fn new() -> Self {
            Self {
                directory_contents: Arc::new(Mutex::new(HashMap::new())),
                file_contents: Arc::new(Mutex::new(HashMap::new())),
                pr_labels: Arc::new(Mutex::new(HashMap::new())),
                repository_labels: Arc::new(Mutex::new(Vec::new())),
            }
        }

        /// Add a PRDoc file to the mock
        async fn add_prdoc(&self, release: &str, pr_number: u32, content: &str) {
            let filename = format!("pr_{}.prdoc", pr_number);
            let url = format!("https://mock.github.com/{}", filename);
            let path = format!("prdoc/{}", release);

            // Add to directory listing
            let mut dir_contents = self.directory_contents.lock().await;
            let dir_entry = dir_contents.entry(path).or_insert_with(Vec::new);
            dir_entry.push(GitHubContent {
                name: filename,
                content_type: "file".to_string(),
                download_url: Some(url.clone()),
            });

            // Add file content
            let mut file_contents = self.file_contents.lock().await;
            file_contents.insert(url, content.to_string());
        }

        /// Add labels for a PR
        async fn add_pr_labels(&self, pr_number: u32, labels: Vec<&str>) {
            let mut pr_labels = self.pr_labels.lock().await;
            pr_labels.insert(pr_number, labels.iter().map(|s| s.to_string()).collect());
        }

        /// Add repository label definitions
        async fn add_repo_label(&self, name: &str, color: &str, description: Option<&str>) {
            let mut repo_labels = self.repository_labels.lock().await;
            repo_labels.push(GitHubLabel {
                name: name.to_string(),
                color: color.to_string(),
                description: description.map(|s| s.to_string()),
            });
        }

        /// Create a mock client with test data for stable2503
        async fn with_stable2503_data() -> Self {
            let client = Self::new();

            // Add prdoc directory structure
            client.directory_contents.lock().await.insert(
                "prdoc".to_string(),
                vec![GitHubContent {
                    name: "stable2503".to_string(),
                    content_type: "dir".to_string(),
                    download_url: None,
                }],
            );

            // Add PRDocs
            client
                .add_prdoc("stable2503", 1234, test_data::STABLE2503_PR1_CONTENT)
                .await;
            client
                .add_prdoc("stable2503", 5678, test_data::STABLE2503_PR2_CONTENT)
                .await;
            client
                .add_prdoc("stable2503", 9012, test_data::STABLE2503_PR3_CONTENT)
                .await;

            // Add PR labels
            client
                .add_pr_labels(1234, vec!["T0-node", "D1-audited"])
                .await;
            client.add_pr_labels(5678, vec!["T1-runtime"]).await;
            client
                .add_pr_labels(9012, vec!["T2-pallets", "E1-breaking"])
                .await;

            // Add repository label definitions
            client
                .add_repo_label("T0-node", "000000", Some("Node-related changes"))
                .await;
            client
                .add_repo_label("T1-runtime", "111111", Some("Runtime-related changes"))
                .await;
            client
                .add_repo_label("T2-pallets", "222222", Some("Pallet-related changes"))
                .await;
            client
                .add_repo_label("D1-audited", "333333", Some("Audited code"))
                .await;
            client
                .add_repo_label("E1-breaking", "444444", Some("Breaking change"))
                .await;

            client
        }
    }

    #[async_trait]
    impl GitHubApiClient for MockGitHubClient {
        async fn get_directory_contents(&self, path: &str) -> Result<Vec<GitHubContent>> {
            let contents = self.directory_contents.lock().await;
            contents
                .get(path)
                .cloned()
                .ok_or_else(|| anyhow!("Mock: Path '{}' not found", path))
        }

        async fn get_file_content(&self, url: &str) -> Result<String> {
            let contents = self.file_contents.lock().await;
            contents
                .get(url)
                .cloned()
                .ok_or_else(|| anyhow!("Mock: File '{}' not found", url))
        }

        async fn get_pr_labels(&self, pr_number: u32) -> Result<Vec<String>> {
            let labels = self.pr_labels.lock().await;
            Ok(labels.get(&pr_number).cloned().unwrap_or_default())
        }

        async fn get_repository_labels(&self) -> Result<Vec<GitHubLabel>> {
            let labels = self.repository_labels.lock().await;
            Ok(labels.clone())
        }
    }

    // ===== Unit Tests =====

    mod unit_tests {
        use super::*;

        #[test]
        fn normalize_release_input_strips_polkadot_prefix() {
            assert_eq!(
                normalize_release_input("polkadot-stable2503-8"),
                "stable2503-8"
            );
            assert_eq!(
                normalize_release_input("polkadot-stable2412-1"),
                "stable2412-1"
            );
            assert_eq!(normalize_release_input("polkadot-v1.9.0"), "v1.9.0");
        }

        #[test]
        fn normalize_release_input_strips_release_prefix() {
            assert_eq!(normalize_release_input("release-v1.9.0"), "v1.9.0");
            assert_eq!(normalize_release_input("release-stable2503"), "stable2503");
        }

        #[test]
        fn normalize_release_input_preserves_normalized_names() {
            assert_eq!(normalize_release_input("stable2503-8"), "stable2503-8");
            assert_eq!(normalize_release_input("1.9.0"), "1.9.0");
            assert_eq!(normalize_release_input("v1.9.0"), "v1.9.0");
        }

        #[test]
        fn normalize_release_input_trims_whitespace() {
            assert_eq!(normalize_release_input("  stable2503-8  "), "stable2503-8");
            assert_eq!(
                normalize_release_input("  polkadot-stable2503-8  "),
                "stable2503-8"
            );
        }

        #[test]
        fn extract_pr_number_from_valid_filename() {
            assert_eq!(extract_pr_number("pr_1234.prdoc"), Some(1234));
            assert_eq!(extract_pr_number("pr_5678.prdoc"), Some(5678));
            assert_eq!(extract_pr_number("pr_0.prdoc"), Some(0));
        }

        #[test]
        fn extract_pr_number_from_invalid_filename() {
            assert_eq!(extract_pr_number("invalid.prdoc"), None);
            assert_eq!(extract_pr_number("pr_abc.prdoc"), None);
            assert_eq!(extract_pr_number("pr_1234.txt"), None);
            assert_eq!(extract_pr_number("pr_1234"), None);
        }
    }

    // ===== PRDoc Parsing Tests =====

    mod prdoc_parsing_tests {
        use super::*;

        fn parse_prdoc_and_extract_description(content: &str) -> Result<(String, String)> {
            let doc = serde_yaml::from_str::<PrDocYaml>(content)?;
            let desc = doc
                .doc
                .iter()
                .map(|section| section.description.trim())
                .filter(|d| !d.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            Ok((doc.title, desc))
        }

        #[test]
        fn parse_prdoc_with_string_audience() {
            let (title, desc) =
                parse_prdoc_and_extract_description(test_data::STABLE2503_PR1_CONTENT)
                    .expect("Should parse PRDoc with string audience");

            assert_eq!(title, "Test PR 1234");
            assert!(desc.contains("This is a test PR for stable2503"));
        }

        #[test]
        fn parse_prdoc_with_array_audience() {
            let (title, desc) =
                parse_prdoc_and_extract_description(test_data::PR_WITH_ARRAY_AUDIENCE)
                    .expect("Should parse PRDoc with array audience");

            assert_eq!(
                title,
                "Bring the latest compatibility fixes via litep2p v0.9.1"
            );
            assert!(desc.contains("litep2p and libp2p"));
            assert!(desc.contains("WebSocket stream"));
        }

        #[test]
        fn parse_prdoc_extracts_multiline_descriptions() {
            let (_, desc) = parse_prdoc_and_extract_description(test_data::PR_WITH_ARRAY_AUDIENCE)
                .expect("Should parse PRDoc");

            // Verify multiline description is properly joined
            assert!(desc.contains("This release enhances compatibility"));
            assert!(desc.contains("Additionally, it includes various improvements"));
        }

        #[tokio::test]
        async fn parse_real_pr7640_if_exists() {
            let test_file = "/Users/snowmead/.substrate-mcp/sh-cloned/releases/stable2412-2/pr-docs/pr_7640.prdoc";

            if tokio::fs::try_exists(test_file).await.unwrap_or(false) {
                let content = tokio::fs::read_to_string(test_file)
                    .await
                    .expect("Should read test file");

                let (title, desc) = parse_prdoc_and_extract_description(&content)
                    .expect("Should parse real PR 7640");

                assert!(!title.is_empty(), "Title should not be empty");
                assert!(!desc.is_empty(), "Description should not be empty");
                assert!(
                    desc.contains("litep2p"),
                    "Description should mention litep2p"
                );
            }
        }
    }

    // ===== Integration Tests =====

    mod integration_tests {
        use super::*;
        use std::time::Instant;

        #[tokio::test]
        async fn fetch_and_cache_release_workflow() {
            // Create temporary directory for cache
            let temp_dir = TempDir::new().unwrap();
            let cache_dir = temp_dir.path();

            // Create mock client with test data
            let mock_client = MockGitHubClient::with_stable2503_data().await;

            // First fetch - download from mock API
            let start = Instant::now();
            let result1 =
                fetch_and_analyze_release("stable2503", false, Some(cache_dir), &mock_client)
                    .await
                    .expect("First fetch should succeed");
            let first_duration = start.elapsed();

            // Verify results
            assert_eq!(result1.summary.total_prs, 3);
            assert!(
                result1.summary.download_date.is_some(),
                "First fetch should have download date"
            );

            // Verify cache files were created
            verify_cache_files_exist(&result1.summary.output_directory);

            // Second fetch - should use cache
            let start = Instant::now();
            let result2 =
                fetch_and_analyze_release("stable2503", false, Some(cache_dir), &mock_client)
                    .await
                    .expect("Second fetch should succeed");
            let second_duration = start.elapsed();

            // Verify cache was used
            assert!(
                result2.summary.download_date.is_none(),
                "Cached fetch should have None for download_date"
            );
            assert!(
                second_duration < first_duration,
                "Cache should be faster than initial fetch"
            );

            // Verify data consistency
            verify_results_match(&result1, &result2);

            // Verify PR data is complete
            for prdoc in &result1.prdocs {
                assert!(!prdoc.title.is_empty(), "Title should not be empty");
                assert!(
                    !prdoc.description.is_empty(),
                    "Description should not be empty"
                );
                assert!(!prdoc.labels.is_empty(), "Should have labels");
            }
        }

        #[tokio::test]
        async fn force_flag_bypasses_cache() {
            let temp_dir = TempDir::new().unwrap();
            let cache_dir = temp_dir.path();
            let mock_client = MockGitHubClient::with_stable2503_data().await;

            // First fetch
            let result1 =
                fetch_and_analyze_release("stable2503", false, Some(cache_dir), &mock_client)
                    .await
                    .expect("First fetch should succeed");
            assert!(result1.summary.download_date.is_some());

            // Second fetch with force flag
            let result2 =
                fetch_and_analyze_release("stable2503", true, Some(cache_dir), &mock_client)
                    .await
                    .expect("Force fetch should succeed");

            // Force flag should cause re-download
            assert!(
                result2.summary.download_date.is_some(),
                "Force fetch should have download date"
            );
        }

        fn verify_cache_files_exist(output_dir: &str) {
            let cache_path = std::path::Path::new(output_dir);
            assert!(
                cache_path.join("pr_labels_mapping.json").exists(),
                "pr_labels_mapping.json should exist"
            );

            let prdoc_count = std::fs::read_dir(&cache_path)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("prdoc"))
                .count();
            assert!(prdoc_count > 0, "Should have PRDoc files");
        }

        fn verify_results_match(result1: &EnhancedPrdocsResult, result2: &EnhancedPrdocsResult) {
            assert_eq!(result1.summary.total_prs, result2.summary.total_prs);
            assert_eq!(result1.prdocs.len(), result2.prdocs.len());

            for (prdoc1, prdoc2) in result1.prdocs.iter().zip(result2.prdocs.iter()) {
                assert_eq!(prdoc1.pr_number, prdoc2.pr_number);
                assert_eq!(prdoc1.title, prdoc2.title);
                assert_eq!(prdoc1.description, prdoc2.description);
                assert_eq!(prdoc1.labels, prdoc2.labels);
            }
        }
    }
}
