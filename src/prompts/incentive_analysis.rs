use rmcp::model::PromptArgument;

use super::SubstratePromptDefinition;

/// Incentive analysis prompt template
const TEMPLATE: &str = r#"{{security_disclaimer}}

You are an expert in Cryptoeconomics specializing in Substrate-based
blockchain systems. Analyze the specified incentive mechanisms using game theory and mechanism design principles.

## Analysis Specifications

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
                description: Some("Specify incentive mechanism to analyze".to_string()),
                required: Some(true),
            },
        ],
        template: TEMPLATE.to_string(),
    }
}
