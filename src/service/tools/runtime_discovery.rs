//! Runtime discovery tool for finding and analyzing #[frame_support::runtime] definitions in projects

use ignore::WalkBuilder;
use ignore::types::TypesBuilder;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content, RawContent, RawTextContent};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub(crate) struct FindRuntimePalletsProperties {
    /// The absolute path to the project directory to analyze.
    project_path: String,
}

/// Handle the find_runtime_pallets tool
pub(crate) async fn handle_find_runtime_pallets(
    properties: FindRuntimePalletsProperties,
) -> Result<CallToolResult, McpError> {
    let analysis = find_frame_runtime_definitions(PathBuf::from(properties.project_path))
        .await
        .map_err(|e| {
            crate::service::utils::mcp_error_internal(format!(
                "Failed to find runtime pallets: {e}"
            ))
        })?;

    // Format the response
    let response = serde_json::json!({
        "distinct_pallet_count": analysis.distinct_pallet_count,
        "runtimes_found": analysis.runtimes_found,
        "distinct_pallets": analysis.distinct_pallets,
        "runtime_paths": analysis.runtime_paths
    });

    let response_text = serde_json::to_string_pretty(&response).map_err(|e| {
        crate::service::utils::mcp_error_internal(format!("Failed to serialize response: {e}"))
    })?;

    let result = CallToolResult::success(vec![Content {
        annotations: None,
        raw: RawContent::Text(RawTextContent {
            text: response_text,
            meta: None,
        }),
    }]);
    Ok(result)
}

/// Information about a runtime file path
#[derive(Debug, Serialize)]
pub(crate) struct RuntimePath {
    /// Path to the file containing the runtime definition
    file_path: String,
    /// Relative path from project root
    relative_path: String,
}

/// Information about a pallet in a runtime definition (kept for parsing)
#[derive(Debug, Serialize)]
pub(crate) struct PalletInfo {
    /// Name of the pallet instance
    pub(crate) instance_name: String,
    /// Pallet crate/module path
    pub(crate) pallet_path: String,
}

/// Project runtime analysis result
#[derive(Debug, Serialize)]
pub(crate) struct ProjectRuntimeAnalysis {
    /// Distinct pallet crate paths found across all runtimes
    pub(crate) distinct_pallets: Vec<String>,
    /// Runtime file paths where runtimes were found
    pub(crate) runtime_paths: Vec<RuntimePath>,
    /// Total number of distinct pallets
    pub(crate) distinct_pallet_count: usize,
    /// Total number of runtimes found
    pub(crate) runtimes_found: usize,
}

/// Find all runtime definitions in the project and extract distinct pallets and runtime paths
pub(crate) async fn find_frame_runtime_definitions(
    project_root: PathBuf,
) -> anyhow::Result<ProjectRuntimeAnalysis> {
    let mut runtime_paths = Vec::new();
    let mut all_pallet_paths = HashSet::new();

    // Use a blocking task to perform the directory walk without blocking the async runtime
    let (runtime_files, distinct_pallets_from_walk) = tokio::task::spawn_blocking(move || {
        let mut paths = Vec::new();
        let mut pallet_paths = HashSet::new();

        let mut types_builder = TypesBuilder::new();
        types_builder.add_defaults();
        types_builder.select("rust");
        let types = types_builder.build().unwrap();

        let walker = WalkBuilder::new(&project_root)
            .sort_by_file_name(|a, b| a.cmp(b))
            .types(types)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if let Ok(content) = std::fs::read_to_string(path)
                && (content.contains("#[frame_support::runtime]")
                    || content.contains("#[runtime::runtime]"))
                {
                    let relative_path = path
                        .strip_prefix(&project_root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();

                    paths.push(RuntimePath {
                        file_path: path.to_string_lossy().to_string(),
                        relative_path,
                    });

                    let pallets = parse_runtime_definition(&content);
                    for pallet in pallets {
                        pallet_paths.insert(pallet.pallet_path);
                    }
                }
        }
        (paths, pallet_paths)
    })
    .await?;

    runtime_paths.extend(runtime_files);
    all_pallet_paths.extend(distinct_pallets_from_walk);

    // Convert HashSet to sorted Vec for consistent output
    let mut distinct_pallets: Vec<String> = all_pallet_paths.into_iter().collect();
    distinct_pallets.sort();

    Ok(ProjectRuntimeAnalysis {
        distinct_pallet_count: distinct_pallets.len(),
        runtimes_found: runtime_paths.len(),
        distinct_pallets,
        runtime_paths,
    })
}

/// Find the matching closing brace for a given opening brace position
fn find_matching_brace(content: &str, start_pos: usize) -> Option<usize> {
    let content = &content[start_pos..];
    let mut brace_count = 0;

    for (i, ch) in content.char_indices() {
        match ch {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                if brace_count == 0 {
                    return Some(start_pos + i);
                }
            }
            _ => {}
        }
    }

    None
}

/// Parse runtime definition to extract pallet information
pub(crate) fn parse_runtime_definition(content: &str) -> Vec<PalletInfo> {
    if content.contains("#[frame_support::runtime]") || content.contains("#[runtime::runtime]") {
        parse_frame_runtime(content)
    } else {
        Vec::new()
    }
}

/// Parse #[frame_support::runtime] format to extract pallet information
pub(crate) fn parse_frame_runtime(content: &str) -> Vec<PalletInfo> {
    let mut pallets = Vec::new();

    // Look for the mod runtime block
    if let Some(mod_start) = content.find("mod runtime") {
        let runtime_content = &content[mod_start..];

        // Find the opening brace of the mod
        if let Some(brace_start) = runtime_content.find('{') {
            let abs_brace_start = mod_start + brace_start;

            if let Some(end_pos) = find_matching_brace(content, abs_brace_start) {
                let mod_content = &content[abs_brace_start + 1..end_pos];
                let lines: Vec<&str> = mod_content.lines().collect();
                let mut i = 0;

                while i < lines.len() {
                    let line = lines[i].trim();

                    // Look for pallet_index attribute
                    if line.starts_with("#[runtime::pallet_index(") {
                        // Look for the next line which should contain the type definition
                        if i + 1 < lines.len() {
                            let next_line = lines[i + 1].trim();
                            if next_line.starts_with("pub type ")
                                && let Some((instance_name, pallet_path)) =
                                    parse_type_declaration(next_line)
                            {
                                pallets.push(PalletInfo {
                                    instance_name,
                                    pallet_path,
                                });
                            }
                        }
                    }
                    i += 1;
                }
            }
        }
    }

    pallets
}

/// Parse type declaration like "pub type System = frame_system;"
pub(crate) fn parse_type_declaration(line: &str) -> Option<(String, String)> {
    // Remove "pub type " prefix
    let line = line.trim_start_matches("pub type ").trim();

    // Split on " = "
    if let Some(eq_pos) = line.find(" = ") {
        let instance_name = line[..eq_pos].trim().to_string();
        let pallet_path = line[eq_pos + 3..].trim_end_matches(';').trim().to_string();
        Some((instance_name, pallet_path))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_find_frame_runtime_definitions_sorting_and_filtering() {
        // 1. Setup a temporary directory
        let dir = tempdir().unwrap();
        let project_root = dir.path();

        // 2. Create mock files
        // An empty runtime file
        let mut file1 = File::create(project_root.join("b_runtime.rs")).unwrap();
        file1.write_all(b"#[frame_support::runtime]").unwrap();

        // A detailed runtime file to test pallet parsing
        let mut file2 = File::create(project_root.join("a_runtime.rs")).unwrap();
        let runtime_content = r#"
            // Create the runtime by composing the FRAME pallets that were previously configured.
            #[frame_support::runtime]
            mod runtime {
                #[runtime::runtime]
                #[runtime::derive(
                    RuntimeCall,
                    RuntimeEvent,
                    RuntimeError,
                    RuntimeOrigin,
                    RuntimeFreezeReason,
                    RuntimeHoldReason,
                    RuntimeSlashReason,
                    RuntimeLockId,
                    RuntimeTask
                )]
                pub struct Runtime;

                // System support stuff
                #[runtime::pallet_index(0)]
                pub type System = frame_system;
                #[runtime::pallet_index(1)]
                pub type ParachainSystem = cumulus_pallet_parachain_system;
                #[runtime::pallet_index(2)]
                pub type Timestamp = pallet_timestamp;
                #[runtime::pallet_index(3)]
                pub type ParachainInfo = parachain_info;

                // Monetary stuff
                #[runtime::pallet_index(10)]
                pub type Balances = pallet_balances;
                #[runtime::pallet_index(11)]
                pub type TransactionPayment = pallet_transaction_payment;

                // Governance
                #[runtime::pallet_index(15)]
                pub type Sudo = pallet_sudo;

                // Collator support. The order of these 4 are important and shall not change.
                #[runtime::pallet_index(20)]
                pub type Authorship = pallet_authorship;
                #[runtime::pallet_index(21)]
                pub type CollatorSelection = pallet_collator_selection;
                #[runtime::pallet_index(22)]
                pub type Session = pallet_session;
                #[runtime::pallet_index(23)]
                pub type Aura = pallet_aura;
                #[runtime::pallet_index(24)]
                pub type AuraExt = cumulus_pallet_aura_ext;

                // XCM helpers
                #[runtime::pallet_index(30)]
                pub type XcmpQueue = cumulus_pallet_xcmp_queue;
                #[runtime::pallet_index(31)]
                pub type PolkadotXcm = pallet_xcm;
                #[runtime::pallet_index(32)]
                pub type CumulusXcm = cumulus_pallet_xcm;
                #[runtime::pallet_index(33)]
                pub type MessageQueue = pallet_message_queue;

                // Storage Hub
                #[runtime::pallet_index(40)]
                pub type Providers = pallet_storage_providers;
                #[runtime::pallet_index(41)]
                pub type FileSystem = pallet_file_system;
                #[runtime::pallet_index(42)]
                pub type ProofsDealer = pallet_proofs_dealer;
                #[runtime::pallet_index(43)]
                pub type Randomness = pallet_randomness;
                #[runtime::pallet_index(44)]
                pub type PaymentStreams = pallet_payment_streams;
                #[runtime::pallet_index(45)]
                pub type BucketNfts = pallet_bucket_nfts;
                // TODO: Add `pallet_cr_randomness` to the runtime when it's ready.
                // #[runtime::pallet_index(46)]
                // pub type CrRandomness = pallet_cr_randomness;

                // Miscellaneous
                #[runtime::pallet_index(50)]
                pub type Nfts = pallet_nfts;
                #[runtime::pallet_index(51)]
                pub type Parameters = pallet_parameters;
            }
        "#;
        file2.write_all(runtime_content.as_bytes()).unwrap();

        // A rust file without the macro
        File::create(project_root.join("c_not_a_runtime.rs")).unwrap();

        // A non-rust file that contains the macro, to test file type filtering
        let mut file4 = File::create(project_root.join("d_other_file.txt")).unwrap();
        file4.write_all(b"#[frame_support::runtime]").unwrap();

        // A subdirectory with a runtime
        let sub_dir = project_root.join("sub");
        fs::create_dir(&sub_dir).await.unwrap();
        let mut file3 = File::create(sub_dir.join("e_runtime.rs")).unwrap();
        file3.write_all(b"#[frame_support::runtime]").unwrap();

        // 4. Run the function
        let result = find_frame_runtime_definitions(project_root.to_path_buf())
            .await
            .unwrap();

        // 5. Assert the results
        // Assert file finding
        assert_eq!(result.runtimes_found, 3);
        assert_eq!(result.runtime_paths.len(), 3);

        // Assert pallet parsing
        assert_eq!(result.distinct_pallet_count, 24);
        assert!(
            result
                .distinct_pallets
                .contains(&"frame_system".to_string())
        );
        assert!(
            result
                .distinct_pallets
                .contains(&"pallet_balances".to_string())
        );
        assert!(result.distinct_pallets.contains(&"pallet_nfts".to_string()));

        // Verify that it only found the correct files
        let found_paths: Vec<_> = result
            .runtime_paths
            .iter()
            .map(|p| p.file_path.clone())
            .collect();
        assert!(found_paths.iter().any(|p| p.ends_with("a_runtime.rs")));
        assert!(found_paths.iter().any(|p| p.ends_with("b_runtime.rs")));
        assert!(found_paths.iter().any(|p| p.ends_with("sub/e_runtime.rs")));
        assert!(
            !found_paths
                .iter()
                .any(|p| p.ends_with("c_not_a_runtime.rs"))
        );
        assert!(!found_paths.iter().any(|p| p.ends_with("d_other_file.txt")));

        // Verify sorting by comparing canonicalized paths
        let expected_paths: Vec<PathBuf> = vec![
            project_root.join("a_runtime.rs"),
            project_root.join("b_runtime.rs"),
            project_root.join("sub").join("e_runtime.rs"),
        ];

        let expected_canonical: Vec<_> = expected_paths
            .into_iter()
            .map(|p| p.canonicalize().unwrap())
            .collect();

        let actual_canonical: Vec<_> = result
            .runtime_paths
            .into_iter()
            .map(|p| PathBuf::from(p.file_path).canonicalize().unwrap())
            .collect();

        assert_eq!(actual_canonical, expected_canonical);
    }
}
