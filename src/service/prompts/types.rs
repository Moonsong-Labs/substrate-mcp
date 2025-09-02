//! Shared types for all prompt arguments

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Arguments for the release comparison prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Compare changes between two Polkadot SDK versions")]
pub struct ReleaseComparisonArgs {
    #[schemars(description = "Version currently being used")]
    pub current_version: String,
    #[schemars(description = "Version to compare with (must be greater than current)")]
    pub target_version: String,
    #[schemars(description = "What specific changes to look for")]
    pub specific_changes: Option<String>,
}

/// Arguments for the analyze release prompt  
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Analyze how a Polkadot SDK release impacts your project")]
pub struct AnalyzeReleaseArgs {
    #[schemars(description = "Release name/version to analyze")]
    pub release: String,
    #[schemars(description = "Specific area to focus analysis on")]
    pub focus: Option<String>,
}

/// Arguments for the scaffold pallet prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Generate pallet structure and implementation templates")]
pub struct ScaffoldPalletArgs {
    #[schemars(description = "Description for the pallet to be created")]
    pub pallet_description: String,
}

/// Arguments for the automated analysis prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Comprehensive security and quality analysis")]
pub struct AutomatedAnalysisArgs {
    #[schemars(description = "Description of the change or feature to analyze")]
    pub change_description: String,
}

/// Arguments for the code security audit prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Security audit for specific components")]
pub struct CodeSecurityAuditArgs {
    #[schemars(description = "Component or pallet to audit")]
    pub audit_target: String,
}

/// Arguments for the economic security prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Economic security assessment")]
pub struct EconomicSecurityArgs {
    #[schemars(description = "Description of the system to analyze")]
    pub system_description: String,
}

/// Arguments for the incentive analysis prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Cryptoeconomic incentive analysis")]
pub struct IncentiveAnalysisArgs {
    #[schemars(description = "Target pallets to analyze")]
    pub target_pallets: String,
    #[schemars(description = "Analysis specifications")]
    pub analysis_specifications: String,
}

/// Arguments for the threat modeling prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Threat model analysis")]
pub struct ThreatModelingArgs {
    #[schemars(description = "Description of the system to threat model")]
    pub system_description: String,
}

/// Arguments for the weight analysis prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Weight and benchmark analysis")]
pub struct WeightAnalysisArgs {
    #[schemars(description = "Target pallet for weight analysis")]
    pub target_pallet: String,
}
