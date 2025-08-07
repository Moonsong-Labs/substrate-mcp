use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs;
use chrono::{DateTime, Utc};

// Core structures for release analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseAnalysis {
    pub release: String,
    pub analysis_date: DateTime<Utc>,
    pub summary: ReleaseSummary,
    pub index: ReleaseIndex,
    pub categories: CategoryAnalysis,
    pub impact: ImpactAnalysis,
    pub relationships: ChangeRelationships,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseSummary {
    pub total_prs: usize,
    pub total_prdocs: usize,
    pub breaking_changes: usize,
    pub security_changes: usize,
    pub new_features: usize,
    pub bug_fixes: usize,
    pub performance_improvements: usize,
    pub documentation_updates: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseIndex {
    pub prs_by_number: HashMap<u32, PrSummary>,
    pub prs_by_category: HashMap<String, Vec<u32>>,
    pub prs_by_subsystem: HashMap<String, Vec<u32>>,
    pub prs_by_author: HashMap<String, Vec<u32>>,
    pub prs_by_crate: HashMap<String, Vec<u32>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrSummary {
    pub number: u32,
    pub title: String,
    pub author: String,
    pub category: String,
    pub subsystems: Vec<String>,
    pub affected_crates: Vec<String>,
    pub has_prdoc: bool,
    pub has_breaking_changes: bool,
    pub has_migrations: bool,
    pub risk_level: RiskLevel,
    pub patch_size: usize,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryAnalysis {
    pub runtime_changes: Vec<PrGroup>,
    pub node_changes: Vec<PrGroup>,
    pub tooling_changes: Vec<PrGroup>,
    pub documentation_changes: Vec<PrGroup>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrGroup {
    pub name: String,
    pub description: String,
    pub pr_numbers: Vec<u32>,
    pub total_impact_score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub breaking_changes: Vec<BreakingChange>,
    pub migration_requirements: Vec<MigrationRequirement>,
    pub dependency_updates: Vec<DependencyUpdate>,
    pub api_changes: Vec<ApiChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BreakingChange {
    pub pr_number: u32,
    pub description: String,
    pub affected_components: Vec<String>,
    pub migration_path: Option<String>,
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationRequirement {
    pub pr_number: u32,
    pub pallet: String,
    pub from_version: Option<u32>,
    pub to_version: Option<u32>,
    pub description: String,
    pub code_example: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyUpdate {
    pub crate_name: String,
    pub from_version: Option<String>,
    pub to_version: String,
    pub pr_numbers: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiChange {
    pub pr_number: u32,
    pub change_type: ApiChangeType,
    pub component: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ApiChangeType {
    Added,
    Modified,
    Deprecated,
    Removed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeRelationships {
    pub dependency_graph: HashMap<u32, Vec<u32>>, // PR -> dependent PRs
    pub conflict_groups: Vec<ConflictGroup>,
    pub complementary_groups: Vec<ComplementaryGroup>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConflictGroup {
    pub pr_numbers: Vec<u32>,
    pub conflict_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplementaryGroup {
    pub pr_numbers: Vec<u32>,
    pub relationship_type: String,
}

// Analysis implementation
pub struct ReleaseAnalyzer {
    base_path: PathBuf,
}

impl ReleaseAnalyzer {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub async fn analyze_release(&self, release: &str) -> Result<ReleaseAnalysis> {
        // Load PR data
        let pr_data = self.load_pr_data(release).await?;
        
        // Build index
        let index = self.build_index(&pr_data).await?;
        
        // Analyze categories
        let categories = self.analyze_categories(&pr_data, &index).await?;
        
        // Analyze impact
        let impact = self.analyze_impact(&pr_data, &index).await?;
        
        // Find relationships
        let relationships = self.find_relationships(&pr_data, &index).await?;
        
        // Generate summary
        let summary = self.generate_summary(&pr_data, &index, &categories, &impact).await?;
        
        Ok(ReleaseAnalysis {
            release: release.to_string(),
            analysis_date: Utc::now(),
            summary,
            index,
            categories,
            impact,
            relationships,
        })
    }

    async fn load_pr_data(&self, release: &str) -> Result<Vec<PrData>> {
        let mut pr_data = Vec::new();
        
        // Load from test_scout_output directory
        let scout_dir = self.base_path.join("test_scout_output").join(format!("polkadot-sdk-{}", release));
        
        if !scout_dir.exists() {
            return Err(anyhow!("Scout data directory not found: {}", scout_dir.display()));
        }

        let mut entries = fs::read_dir(&scout_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() && path.file_name().unwrap().to_str().unwrap().starts_with("pr-") {
                if let Ok(data) = self.load_single_pr(&path).await {
                    pr_data.push(data);
                }
            }
        }
        
        Ok(pr_data)
    }

    async fn load_single_pr(&self, pr_dir: &Path) -> Result<PrData> {
        let pr_num = pr_dir.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .strip_prefix("pr-")
            .unwrap()
            .parse::<u32>()?;
            
        // Load metadata
        let metadata_path = pr_dir.join("metadata.json");
        let metadata: PrMetadata = if metadata_path.exists() {
            let content = fs::read_to_string(&metadata_path).await?;
            serde_json::from_str(&content)?
        } else {
            PrMetadata::default()
        };
        
        // Load description
        let description_path = pr_dir.join("description.md");
        let description = if description_path.exists() {
            fs::read_to_string(&description_path).await?
        } else {
            String::new()
        };
        
        // Analyze patch
        let patch_path = pr_dir.join("patch.diff");
        let patch_analysis = if patch_path.exists() {
            self.analyze_patch(&patch_path).await?
        } else {
            PatchAnalysis::default()
        };
        
        Ok(PrData {
            number: pr_num,
            metadata,
            description,
            patch_analysis,
        })
    }

    async fn analyze_patch(&self, patch_path: &Path) -> Result<PatchAnalysis> {
        let content = fs::read_to_string(patch_path).await?;
        let lines: Vec<&str> = content.lines().collect();
        
        let mut affected_files = HashSet::new();
        let mut has_migrations = false;
        let mut has_breaking_changes = false;
        let mut affected_crates = HashSet::new();
        let mut added_lines = 0;
        let mut removed_lines = 0;
        
        for line in &lines {
            if line.starts_with("+++") || line.starts_with("---") {
                if let Some(file) = line.split_whitespace().nth(1) {
                    affected_files.insert(file.to_string());
                    
                    // Extract crate name
                    if let Some(crate_name) = extract_crate_name(file) {
                        affected_crates.insert(crate_name);
                    }
                    
                    // Check for migrations
                    if file.contains("migration") {
                        has_migrations = true;
                    }
                }
            } else if line.starts_with("+") && !line.starts_with("+++") {
                added_lines += 1;
                
                // Check for breaking change indicators
                if line.contains("#[deprecated") || line.contains("BREAKING") {
                    has_breaking_changes = true;
                }
            } else if line.starts_with("-") && !line.starts_with("---") {
                removed_lines += 1;
            }
        }
        
        Ok(PatchAnalysis {
            affected_files: affected_files.into_iter().collect(),
            affected_crates: affected_crates.into_iter().collect(),
            added_lines,
            removed_lines,
            has_migrations,
            has_breaking_changes,
        })
    }

    async fn build_index(&self, pr_data: &[PrData]) -> Result<ReleaseIndex> {
        let mut prs_by_number = HashMap::new();
        let mut prs_by_category = HashMap::new();
        let mut prs_by_subsystem = HashMap::new();
        let mut prs_by_author = HashMap::new();
        let mut prs_by_crate = HashMap::new();
        
        for pr in pr_data {
            let summary = self.create_pr_summary(pr)?;
            
            // By number
            prs_by_number.insert(pr.number, summary.clone());
            
            // By category
            prs_by_category.entry(summary.category.clone())
                .or_insert_with(Vec::new)
                .push(pr.number);
                
            // By subsystem
            for subsystem in &summary.subsystems {
                prs_by_subsystem.entry(subsystem.clone())
                    .or_insert_with(Vec::new)
                    .push(pr.number);
            }
            
            // By author
            prs_by_author.entry(summary.author.clone())
                .or_insert_with(Vec::new)
                .push(pr.number);
                
            // By crate
            for crate_name in &summary.affected_crates {
                prs_by_crate.entry(crate_name.clone())
                    .or_insert_with(Vec::new)
                    .push(pr.number);
            }
        }
        
        Ok(ReleaseIndex {
            prs_by_number,
            prs_by_category,
            prs_by_subsystem,
            prs_by_author,
            prs_by_crate,
        })
    }

    fn create_pr_summary(&self, pr: &PrData) -> Result<PrSummary> {
        let category = self.categorize_pr(pr);
        let subsystems = self.identify_subsystems(pr);
        let risk_level = self.assess_risk(pr);
        
        Ok(PrSummary {
            number: pr.number,
            title: pr.metadata.title.clone(),
            author: pr.metadata.author.clone(),
            category,
            subsystems,
            affected_crates: pr.patch_analysis.affected_crates.clone(),
            has_prdoc: pr.metadata.has_prdoc,
            has_breaking_changes: pr.patch_analysis.has_breaking_changes,
            has_migrations: pr.patch_analysis.has_migrations,
            risk_level,
            patch_size: pr.patch_analysis.added_lines + pr.patch_analysis.removed_lines,
        })
    }

    fn categorize_pr(&self, pr: &PrData) -> String {
        // Simple categorization based on patterns
        let title = pr.metadata.title.to_lowercase();
        let desc = pr.description.to_lowercase();
        
        if title.contains("fix") || desc.contains("fixes #") {
            "bug_fix".to_string()
        } else if title.contains("feat") || title.contains("add") {
            "feature".to_string()
        } else if title.contains("refactor") {
            "refactoring".to_string()
        } else if title.contains("doc") || pr.patch_analysis.affected_files.iter().any(|f| f.ends_with(".md")) {
            "documentation".to_string()
        } else if title.contains("test") {
            "testing".to_string()
        } else if title.contains("perf") || title.contains("optimize") {
            "performance".to_string()
        } else {
            "other".to_string()
        }
    }

    fn identify_subsystems(&self, pr: &PrData) -> Vec<String> {
        let mut subsystems = HashSet::new();
        
        for file in &pr.patch_analysis.affected_files {
            if file.contains("runtime") {
                subsystems.insert("runtime".to_string());
            }
            if file.contains("consensus") {
                subsystems.insert("consensus".to_string());
            }
            if file.contains("network") {
                subsystems.insert("networking".to_string());
            }
            if file.contains("rpc") {
                subsystems.insert("rpc".to_string());
            }
            if file.contains("client") {
                subsystems.insert("client".to_string());
            }
        }
        
        subsystems.into_iter().collect()
    }

    fn assess_risk(&self, pr: &PrData) -> RiskLevel {
        if pr.patch_analysis.has_migrations {
            RiskLevel::Critical
        } else if pr.patch_analysis.has_breaking_changes {
            RiskLevel::High
        } else if pr.patch_analysis.affected_files.iter().any(|f| 
            f.contains("consensus") || f.contains("crypto") || f.contains("security")
        ) {
            RiskLevel::High
        } else if pr.patch_analysis.added_lines + pr.patch_analysis.removed_lines > 1000 {
            RiskLevel::Medium
        } else if pr.patch_analysis.added_lines + pr.patch_analysis.removed_lines > 100 {
            RiskLevel::Low
        } else {
            RiskLevel::None
        }
    }

    async fn analyze_categories(&self, _pr_data: &[PrData], index: &ReleaseIndex) -> Result<CategoryAnalysis> {
        // Group PRs by major categories
        let runtime_prs: Vec<u32> = index.prs_by_subsystem.get("runtime")
            .cloned()
            .unwrap_or_default();
            
        let runtime_changes = vec![PrGroup {
            name: "Runtime Core".to_string(),
            description: "Core runtime changes including pallets and runtime logic".to_string(),
            pr_numbers: runtime_prs,
            total_impact_score: 0.8, // Placeholder
        }];
        
        // Similar for other categories...
        let node_changes = vec![];
        let tooling_changes = vec![];
        let documentation_changes = vec![];
        
        Ok(CategoryAnalysis {
            runtime_changes,
            node_changes,
            tooling_changes,
            documentation_changes,
        })
    }

    async fn analyze_impact(&self, pr_data: &[PrData], _index: &ReleaseIndex) -> Result<ImpactAnalysis> {
        let mut breaking_changes = Vec::new();
        let mut migration_requirements = Vec::new();
        
        for pr in pr_data {
            if pr.patch_analysis.has_breaking_changes {
                breaking_changes.push(BreakingChange {
                    pr_number: pr.number,
                    description: format!("Breaking change in PR #{}", pr.number),
                    affected_components: pr.patch_analysis.affected_crates.clone(),
                    migration_path: None,
                    severity: "high".to_string(),
                });
            }
            
            if pr.patch_analysis.has_migrations {
                migration_requirements.push(MigrationRequirement {
                    pr_number: pr.number,
                    pallet: "unknown".to_string(), // Would need more analysis
                    from_version: None,
                    to_version: None,
                    description: format!("Migration required for PR #{}", pr.number),
                    code_example: None,
                });
            }
        }
        
        Ok(ImpactAnalysis {
            breaking_changes,
            migration_requirements,
            dependency_updates: vec![],
            api_changes: vec![],
        })
    }

    async fn find_relationships(&self, _pr_data: &[PrData], _index: &ReleaseIndex) -> Result<ChangeRelationships> {
        let dependency_graph = HashMap::new(); // Placeholder
        let conflict_groups = vec![];
        let complementary_groups = vec![];
        
        Ok(ChangeRelationships {
            dependency_graph,
            conflict_groups,
            complementary_groups,
        })
    }

    async fn generate_summary(
        &self, 
        pr_data: &[PrData], 
        index: &ReleaseIndex,
        _categories: &CategoryAnalysis,
        impact: &ImpactAnalysis
    ) -> Result<ReleaseSummary> {
        let total_prs = pr_data.len();
        let total_prdocs = pr_data.iter().filter(|pr| pr.metadata.has_prdoc).count();
        let breaking_changes = impact.breaking_changes.len();
        
        let bug_fixes = index.prs_by_category.get("bug_fix")
            .map(|v| v.len())
            .unwrap_or(0);
            
        let new_features = index.prs_by_category.get("feature")
            .map(|v| v.len())
            .unwrap_or(0);
            
        Ok(ReleaseSummary {
            total_prs,
            total_prdocs,
            breaking_changes,
            security_changes: 0, // Would need security analysis
            new_features,
            bug_fixes,
            performance_improvements: 0,
            documentation_updates: 0,
        })
    }
}

// Helper structures
#[derive(Debug)]
struct PrData {
    number: u32,
    metadata: PrMetadata,
    description: String,
    patch_analysis: PatchAnalysis,
}

#[derive(Debug, Deserialize, Default)]
struct PrMetadata {
    #[serde(default)]
    title: String,
    #[serde(default)]
    author: String,
    #[serde(default)]
    has_prdoc: bool,
}

#[derive(Debug, Default)]
struct PatchAnalysis {
    affected_files: Vec<String>,
    affected_crates: Vec<String>,
    added_lines: usize,
    removed_lines: usize,
    has_migrations: bool,
    has_breaking_changes: bool,
}

fn extract_crate_name(file_path: &str) -> Option<String> {
    // Extract crate name from file path
    // e.g., "substrate/frame/staking/src/lib.rs" -> "pallet-staking"
    let parts: Vec<&str> = file_path.split('/').collect();
    
    if parts.len() > 2 && parts[0] == "substrate" && parts[1] == "frame" {
        Some(format!("pallet-{}", parts[2]))
    } else if parts.len() > 2 && parts[0] == "substrate" && parts[1] == "client" {
        Some(format!("sc-{}", parts[2]))
    } else if parts.len() > 2 && parts[0] == "substrate" && parts[1] == "primitives" {
        Some(format!("sp-{}", parts[2]))
    } else {
        None
    }
}

// Export function for analysis
pub async fn analyze_polkadot_release(release: &str, base_path: PathBuf) -> Result<String> {
    let analyzer = ReleaseAnalyzer::new(base_path);
    
    // Check if multiple releases are requested (comma-separated)
    let releases: Vec<&str> = release.split(',').map(|s| s.trim()).collect();
    
    if releases.len() > 1 {
        // Multi-release analysis
        let mut all_analyses = Vec::new();
        for single_release in &releases {
            match analyzer.analyze_release(single_release).await {
                Ok(analysis) => all_analyses.push(analysis),
                Err(e) => eprintln!("Warning: Failed to analyze {}: {}", single_release, e),
            }
        }
        
        if all_analyses.is_empty() {
            return Err(anyhow!("Failed to analyze any of the requested releases"));
        }
        
        // Save individual and combined analyses
        let output_dir = analyzer.base_path.join("release_analysis");
        fs::create_dir_all(&output_dir).await?;
        
        // Save combined analysis
        let combined_name = format!("{}_combined", releases.join("_"));
        let output_path = output_dir.join(format!("{}_analysis.json", combined_name));
        let json = serde_json::to_string_pretty(&all_analyses)?;
        fs::write(&output_path, &json).await?;
        
        Ok(output_path.display().to_string())
    } else {
        // Single release analysis (existing behavior)
        let analysis = analyzer.analyze_release(release).await?;
        
        // Save analysis
        let output_dir = analyzer.base_path.join("release_analysis");
        fs::create_dir_all(&output_dir).await?;
        
        let output_path = output_dir.join(format!("{}_analysis.json", release));
        let json = serde_json::to_string_pretty(&analysis)?;
        fs::write(&output_path, &json).await?;
        
        Ok(output_path.display().to_string())
    }
}