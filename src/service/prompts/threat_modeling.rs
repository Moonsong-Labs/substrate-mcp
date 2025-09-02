//! Threat modeling prompt implementation

use handlebars::Handlebars;
use rmcp::model::{PromptMessage, PromptMessageRole};
use serde_json::json;

use super::common::SECURITY_DISCLAIMER;
use super::types::ThreatModelingArgs;

/// Generate threat modeling prompt content
pub async fn generate_prompt(args: ThreatModelingArgs) -> Vec<PromptMessage> {
    let handlebars = Handlebars::new();

    let context = json!({
        "system_description": args.system_description,
        "security_disclaimer": SECURITY_DISCLAIMER
    });

    let content = handlebars
        .render_template(TEMPLATE, &context)
        .unwrap_or_else(|e| format!("Template rendering failed: {}", e));

    vec![PromptMessage::new_text(PromptMessageRole::User, content)]
}

/// Threat modeling prompt template
const TEMPLATE: &str = r#"{{security_disclaimer}}

Perform a comprehensive security threat model analysis of the following Substrate subsystem:

**System**: {{system_description}}

Please analyze the code and provide a detailed threat model covering:

1. **Asset Analysis**
   - Identify all assets this subsystem controls (tokens, permissions, critical state)
   - Map data flows and trust boundaries
   - List all external interfaces and dependencies

2. **Attack Surface Mapping**
   - All public/dispatchable functions
   - Storage items and their access patterns
   - Events and their potential for information leakage
   - Integration points with other pallets/subsystems

3. **Threat Identification** - Find potential vulnerabilities:
   - Integer overflow/underflow risks
   - Missing input validation
   - Improper origin checks
   - Reentrancy vulnerabilities
   - Race conditions
   - Storage exhaustion vectors
   - Consensus manipulation risks
   - Upgrade/migration vulnerabilities

4. **Attack Scenarios** - Describe specific attack vectors:
   - How could an attacker exploit each vulnerability?
   - What would be the impact (funds loss, DoS, state corruption)?
   - What privileges would an attacker need?

5. **Risk Assessment**
   - Severity: Critical/High/Medium/Low
   - Likelihood: High/Medium/Low
   - Overall risk score

6. **Mitigation Recommendations**
   - Specific code fixes with examples
   - Additional checks or validations needed
   - Architectural improvements
   - Testing requirements

7. **Security Checklist**
   - [ ] All extrinsics have proper origin checks
   - [ ] Storage operations are bounded
   - [ ] Arithmetic operations use safe math
   - [ ] Error handling is comprehensive
   - [ ] Events don't leak sensitive data
   - [ ] Benchmarks exist for all dispatchables

Format your response as a structured security report with clear sections and actionable findings. Include specific line numbers and code references where issues are found."#;
