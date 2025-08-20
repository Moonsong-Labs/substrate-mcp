use rmcp::model::PromptArgument;

use super::SubstratePrompt;

/// Automated analysis prompt template
const TEMPLATE: &str = r#"{{security_disclaimer}}

Perform a comprehensive security and quality analysis of the following Substrate project changes for pre-release/PR review:

**Context**: {{change_description}}

Please analyze the codebase changes and provide a detailed security review covering:

1. **Code Security Analysis**
   - Unsafe code usage and justification
   - Input validation and sanitization
   - Integer overflow/underflow risks
   - Panic conditions and error handling
   - Memory safety issues
   - Cryptographic operation correctness

2. **Substrate-Specific Security**
   - Origin and authorization checks
   - Storage access patterns and safety
   - Cross-pallet interaction security
   - Determinism violations
   - Consensus-critical code changes
   - Weight and fee correctness

3. **Runtime Safety**
   - Migration safety and correctness
   - Upgrade compatibility
   - Storage layout changes
   - Type safety across versions
   - Hook implementation safety
   - Genesis configuration security

4. **Common Vulnerability Patterns**
   - Reentrancy possibilities
   - Front-running vulnerabilities
   - DoS attack vectors
   - Privilege escalation paths
   - Economic attack surfaces
   - State bloat risks

5. **Best Practices Compliance**
   - Error handling completeness
   - Event emission coverage
   - Documentation quality
   - Test coverage adequacy
   - Benchmark accuracy
   - Code style consistency

6. **Integration Security**
   - XCM message handling
   - Oracle data validation
   - External service dependencies
   - Bridge security considerations
   - Multi-signature operations
   - Proxy and delegation safety

7. **Operational Security**
   - Key management practices
   - Upgrade process security
   - Monitoring and alerting gaps
   - Emergency response readiness
   - Configuration security

8. **Release Readiness**
   - Breaking changes documented
   - Version bumps appropriate
   - Migration guide complete
   - Security notices included
   - Deployment risks assessed

Provide findings organized by severity (Critical/High/Medium/Low/Info) with specific code references and remediation steps.

## Component-Specific Analysis

### For Runtime Changes
Additionally review:
- Spec version and transaction version updates
- Runtime API changes and compatibility
- Pallet configuration changes
- System pallet modifications
- Executive pallet ordering
- Runtime constant changes
- Feature flag impacts

### For Pallet Development
Additionally review:
- Storage item declarations and bounds
- Dispatchable function signatures
- Error variant completeness
- Event definitions and usage
- Config trait requirements
- GenesisConfig implementation
- Pallet hooks (on_initialize, on_finalize)

### For Consensus Changes
Additionally review:
- Block production modifications
- Finality gadget changes
- Fork choice rule updates
- Validator set management
- Slashing logic modifications
- Session key handling
- Network protocol changes

### For Client/Node Changes
Additionally review:
- RPC interface modifications
- Database schema changes
- Network protocol updates
- Command line interface changes
- Telemetry modifications
- Import/export functionality
- Pruning logic changes

## Critical Checklist Items

### Security Essentials
- [ ] No hardcoded secrets or keys
- [ ] All origins properly checked
- [ ] Storage items have size limits
- [ ] Arithmetic operations are safe
- [ ] External inputs validated
- [ ] Error handling is comprehensive
- [ ] No debug code in production

### Runtime Upgrade Safety
- [ ] Migrations tested on real data
- [ ] Version numbers updated
- [ ] Storage prefixes unchanged (or migrated)
- [ ] Type definitions compatible
- [ ] Hook weights accounted for
- [ ] Rollback plan documented

### Testing Requirements
- [ ] Unit tests for new logic
- [ ] Integration tests for interactions
- [ ] Benchmarks for all dispatchables
- [ ] Try-runtime tests pass
- [ ] Manual testing completed
- [ ] Edge cases covered

### Documentation
- [ ] Code comments adequate
- [ ] Public API documented
- [ ] README updated
- [ ] CHANGELOG updated
- [ ] Migration guide provided
- [ ] Security considerations noted

{{security_disclaimer}}"#;

pub fn prompt() -> SubstratePrompt {
    SubstratePrompt {
        name: "automated_analysis".to_string(),
        description: "Template for automated code and runtime analysis".to_string(),
        arguments: vec![
            PromptArgument {
                name: "change_description".to_string(),
                description: Some("Description of the changes made to the code that trigger this analysis (PR description, new release, etc)".to_string()),
                required: Some(true),
            },
        ],
        template: TEMPLATE.to_string(),
    }
}

