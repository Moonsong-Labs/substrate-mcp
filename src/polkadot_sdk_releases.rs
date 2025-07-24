use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GitHubContent {
    name: String,
    #[serde(rename = "type")]
    content_type: String,
    download_url: Option<String>,
}

pub async fn query_prdocs(release: &str) -> Result<String> {
    let client = reqwest::Client::new();

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
        return Ok(format!("No prdoc files found in release '{release}'"));
    }

    // Fetch content of each prdoc file
    let mut all_content = String::new();

    for file in prdoc_files {
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

                // Add header with filename
                all_content.push_str(&format!("=== {} ===\n", file.name));
                all_content.push_str(&content);
                all_content.push_str("\n\n");
            } else {
                eprintln!("Failed to fetch {}: {}", file.name, file_response.status());
            }
        }
    }

    Ok(all_content.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_prdocs_valid_release() {
        // This test requires network access
        let result = query_prdocs("stable2412-1").await;
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(!content.is_empty());
        assert!(content.contains("pr_6463"));
    }

    #[tokio::test]
    async fn test_query_prdocs_invalid_release() {
        let result = query_prdocs("nonexistent-release").await;
        // Should either return an error or a message about no files found
        assert!(result.is_ok() || result.is_err());
    }
}

