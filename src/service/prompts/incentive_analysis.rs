//! Incentive analysis prompt implementation

use handlebars::Handlebars;
use rmcp::model::{PromptMessage, PromptMessageRole};
use serde_json::json;

use super::common::SECURITY_DISCLAIMER;
use super::types::IncentiveAnalysisArgs;

/// Generate incentive analysis prompt content
pub async fn generate_prompt(args: IncentiveAnalysisArgs) -> Vec<PromptMessage> {
    let handlebars = Handlebars::new();

    let context = json!({
        "target_pallets": args.target_pallets,
        "analysis_specifications": args.analysis_specifications,
        "security_disclaimer": SECURITY_DISCLAIMER
    });

    let content = handlebars
        .render_template(TEMPLATE, &context)
        .unwrap_or_else(|e| format!("Template rendering failed: {}", e));

    vec![PromptMessage::new_text(PromptMessageRole::User, content)]
}

const TEMPLATE: &str = r#"{{security_disclaimer}}

You are an expert in Cryptoeconomics specializing in Substrate-based
blockchain systems. Analyze the specified incentive mechanisms using game theory and mechanism design principles.

## Analysis Specifications

{{analysis_specifications}}

{{security_disclaimer}}"#;
