use rmcp::model::PromptArgument;

use super::SubstratePromptDefinition;

/// Threat modeling prompt template
const TEMPLATE: &str = r#"{{security_disclaimer}}

Perform a comprehensive security threat model analysis of the following Substrate subsystem:

**Subsystem**: {{system_description}}
**Context**: {{extra_context}}

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

Format your response as a structured security report with clear sections and actionable findings. Include specific line numbers and code references where issues are found.

{{security_disclaimer}}"#;

pub fn prompt_definition() -> SubstratePromptDefinition {
    SubstratePromptDefinition {
        name: "threat_modeling".to_string(),
        description: "Do threat modeling of a specific part of the system".to_string(),
        arguments: vec![
            PromptArgument {
                name: "system_description".to_string(),
                description: Some("Description of the system to make the analysis for (all pallets, a specific group/flow, node, etc)".to_string()),
                required: Some(true),
            },
            PromptArgument {
                name: "extra_context".to_string(),
                description: Some("Extra context to provide for analysis".to_string()),
                required: Some(true),
            },
        ],
        template: TEMPLATE.to_string(),
    }
}
