use rmcp::model::PromptArgument;

use super::SubstratePrompt;
use super::security_disclaimer::SECURITY_DISCLAIMER

/// Code security audit prompt template
const TEMPLATE: String = format!(r#"You are a Systems Security Expert specializing in Substrate-based blockchain
security. Perform a comprehensive security audit following industry-standard
practices and Substrate-specific considerations.

{{SEC}}

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

pub fn prompt() -> SubstratePrompt {
    SubstratePrompt {
        name: "code_security_audit".to_string(),
        description: "Audit specific component for common code-related vulnerabilities".to_string(),
        arguments: vec![PromptArgument {
            name: "audit_target".to_string(),
            description: Some("Describe the target of the audit".to_string()),
            required: Some(true),
        }],
        template: TEMPLATE.to_string(),
        needs_security_disclaimer: true,
    }
}

