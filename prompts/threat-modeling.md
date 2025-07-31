# Threat Modeling

## Description

Do threat modeling of a specific part of the system

## Arguments

- system_description: Description of the system to make the analysis for (all pallets, a specific group/flow, node, etc)
- extra_context: Extra context to provide for analysis

## Prompt

```
Perform a comprehensive security threat model analysis of the following Substrate subsystem:

**Subsystem**: <system_description>
**Context**: <extra_context>

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
```

## Subsystem-Specific Extensions

### For Pallets
```
Additionally check:
- Weight calculations and benchmarking
- Genesis configuration security
- Migration logic safety
- Hooks implementation (on_initialize, on_finalize)
- Pallet coupling and dependency risks
- Config trait bounds and generics safety
```

### For Consensus Modules
```
Additionally check:
- Validator set manipulation
- Equivocation handling
- Fork choice rule vulnerabilities
- Network partition scenarios
- Block production and finalization guarantees
- Slashing conditions and economic security
```

### For Offchain Workers
```
Additionally check:
- External API call validation
- Signed transaction security
- Local storage key collisions
- HTTP request timeout/retry logic
- Randomness source security
- Data availability guarantees
```

### For Runtime Executive/System
```
Additionally check:
- Transaction ordering vulnerabilities
- Block initialization/finalization logic
- Runtime upgrade authorization
- Fee payment bypass possibilities
- System pallet privilege escalation
- Cross-pallet interaction safety
```

## Usage Examples

### Example 1: Analyzing a Token Pallet
```
Perform a comprehensive security threat model analysis of the following Substrate subsystem:

**Subsystem**: pallets/assets/src/lib.rs
**Context**: This pallet manages fungible assets including minting, burning, transfers, and approvals.

[Rest of base prompt...]
```

### Example 2: Analyzing Consensus
```
Perform a comprehensive security threat model analysis of the following Substrate subsystem:

**Subsystem**: client/consensus/babe/src/
**Context**: BABE consensus implementation handling block production and slot allocation.

[Rest of base prompt + consensus-specific checks...]
```

## Tips for Effective Analysis

1. **Provide Context**: Always include relevant context about the subsystem's purpose and critical operations
2. **Specify Scope**: Clearly define which files/modules should be analyzed
3. **Include Dependencies**: Mention key dependencies that should be considered in the analysis
4. **Request Specifics**: Ask for concrete code examples and line numbers for findings
5. **Prioritize Findings**: Request risk-based prioritization of vulnerabilities
