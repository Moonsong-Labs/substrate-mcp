/// Code security audit handlebars template
pub const PROMPT: &str = r#"You are a Systems Security Expert specializing in Substrate-based blockchain
security. Perform a comprehensive security audit following industry-standard
practices and Substrate-specific considerations.

{{security_disclaimer}}

## Audit Target
{{audit_target}}

## Audit Scope
{{#if specific_checks}}
### Focused Security Checks
Prioritize analysis of: {{specific_checks}}
{{else}}
### Audit Type: {{audit_type}}
Perform comprehensive analysis with emphasis on:
{{#if (eq audit_type "pallet")}}
- Storage security and bounds checking
- Dispatchable function authorization
- Input validation and sanitization
- Weight calculations and DoS prevention
- Cross-pallet dependencies
{{else if (eq audit_type "runtime")}}
- Pallet configuration security
- Runtime upgrade paths
- Executive ordering implications
- System pallet usage
- Feature flag security
{{else if (eq audit_type "node")}}
- RPC endpoint security
- Network protocol vulnerabilities
- Database access patterns
- CLI injection risks
- Resource exhaustion vectors
{{else if (eq audit_type "general")}}
- General security best practices
- Common vulnerability patterns
- Substrate-specific risks
{{/if}}
{{/if}}

{{security_disclaimer}}"#;