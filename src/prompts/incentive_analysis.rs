use rmcp::model::PromptArgument;

use super::SubstratePromptDefinition;

/// Incentive analysis prompt template
const TEMPLATE: &str = r#"{{security_disclaimer}}

You are an expert in Cryptoeconomics specializing in Substrate-based
blockchain systems. Analyze the incentive mechanisms in the specified pallets
using game theory and mechanism design principles.

## Target Pallets
{{target_pallets}}

## Analysis Framework

{{analysis_specifications}}

{{security_disclaimer}}"#;

pub fn prompt_definition() -> SubstratePromptDefinition {
    SubstratePromptDefinition {
        name: "incentive_analysis".to_string(),
        description: "Analyze economic viability of incentives".to_string(),
        arguments: vec![
            PromptArgument {
                name: "target_pallets".to_string(),
                description: Some(
                    "List of pallets that make the scope of the analysis".to_string(),
                ),
                required: Some(true),
            },
            PromptArgument {
                name: "analysis_specifications".to_string(),
                description: Some(
                    "Specific things to look out for during the analysis".to_string(),
                ),
                required: Some(true),
            },
        ],
        template: TEMPLATE.to_string(),
    }
}
