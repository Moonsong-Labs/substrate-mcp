//! Code security audit prompt implementation

use handlebars::Handlebars;
use rmcp::model::{PromptMessage, PromptMessageRole};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::common::SECURITY_DISCLAIMER;

/// Arguments for the code security audit prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Security audit for specific components")]
pub struct CodeSecurityAuditArgs {
    #[schemars(description = "Component or pallet to audit")]
    pub audit_target: String,
}

/// Generate code security audit prompt content
pub async fn generate_prompt(args: CodeSecurityAuditArgs) -> Vec<PromptMessage> {
    let handlebars = Handlebars::new();

    let context = json!({
        "audit_target": args.audit_target,
        "security_disclaimer": SECURITY_DISCLAIMER
    });

    let content = handlebars
        .render_template(TEMPLATE, &context)
        .unwrap_or_else(|e| format!("Template rendering failed: {}", e));

    vec![PromptMessage::new_text(PromptMessageRole::User, content)]
}

const TEMPLATE: &str = r#"{{security_disclaimer}}

You are a Systems Security Expert specializing in Substrate-based blockchain
security. Perform a comprehensive security audit following industry-standard
practices and Substrate-specific considerations.


## Audit Target
{{audit_target}}

## Audit Scope

Perform comprehensive analysis with emphasis on:
### For Pallets
- Storage security and bounds checking
- Dispatchable function authorization
- Input validation and sanitization
- Weight calculations and DoS prevention
- Cross-pallet dependencies
### For Runtimes
- Pallet configuration security
- Runtime upgrade paths
- Executive ordering implications
- System pallet usage
- Feature flag security
#### For Nodes
- RPC endpoint security
- Network protocol vulnerabilities
- Database access patterns
- CLI injection risks
- Resource exhaustion vectors

{{security_disclaimer}}"#;
