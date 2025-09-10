//! Security review prompt implementation
//!
//! This prompt combines code security audit, economic security, threat modeling,
//! and weight analysis into a single development security review.

use handlebars::Handlebars;
use rmcp::model::{PromptMessage, PromptMessageRole};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::common::SECURITY_DISCLAIMER;

/// Arguments for the security review prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Security review for Substrate components")]
pub(crate) struct SecurityReviewArgs {
    #[schemars(description = "Target component/pallet/system to review")]
    pub(crate) target: String,
}

/// Generate security review prompt content
pub(crate) async fn generate_prompt(args: SecurityReviewArgs) -> Vec<PromptMessage> {
    let handlebars = Handlebars::new();

    let context = json!({
        "target": args.target,
        "security_disclaimer": SECURITY_DISCLAIMER
    });

    let content = handlebars
        .render_template(TEMPLATE, &context)
        .unwrap_or_else(|e| format!("Template rendering failed: {}", e));

    vec![PromptMessage::new_text(PromptMessageRole::User, content)]
}

const TEMPLATE: &str = r#"{{security_disclaimer}}

You are a Systems Security Expert specializing in Substrate-based blockchain security. 
Perform a security review following industry-standard practices and Substrate-specific 
considerations to assist development. This is not a replacement for professional security audits.

## Analysis Target
**Target**: {{target}}

## Security Review Framework

Unless specified in the target, perform security review across all three domains below:

### 1. CODE SECURITY ANALYSIS

#### Core Security Areas:
- **Authorization & Access Control**: Origin checks, permission validation, privilege escalation
- **Input Validation & Sanitization**: Type safety, bounds checking, overflow/underflow protection
- **Storage Security**: Access patterns, unbounded operations, state bloat prevention
- **Network & RPC Security**: Access controls, rate limiting, protocol vulnerabilities
- **Runtime Security**: Configuration validation, upgrade safety, executive ordering
- **Cross-Component Security**: Inter-pallet dependencies, trust boundaries, coupling risks

### 2. ECONOMIC SECURITY & THREAT ANALYSIS

#### Economic Model & Asset Mapping:
- **Value Flows**: Token transfers, fees, rewards, slashing mechanisms
- **Critical Assets**: Tokens, permissions, governance rights, consensus state  
- **Economic Actors**: Validators, nominators, users, governance participants
- **Parameter Analysis**: Fee structures, inflation rates, economic constants
- **Supply Dynamics**: Minting, burning, staking rewards, treasury operations
- **Trust Boundaries**: External interfaces, user inputs, inter-pallet communication

#### Game Theory & Attack Economics:
- **Incentive Alignment**: Dominant strategies, Nash equilibria, rational behavior
- **Attack Economics**: Cost-benefit analysis, profitability thresholds, economic resources required
- **Coalition Risks**: Collusion opportunities, cartel formation, governance capture
- **Griefing Vectors**: Cost-imposing attacks, resource waste, user experience degradation

#### Attack Vectors & Exploitation:
- **MEV Attacks**: Front-running, sandwich attacks, back-running, transaction ordering
- **Market Manipulation**: Oracle manipulation, price feed attacks, arbitrage exploits, wash trading
- **Economic Exploits**: Flash loan attacks, liquidity extraction, market cornering
- **Cross-Chain Risks**: XCM vulnerabilities, bridge exploits, multi-chain MEV
- **Attack Surface**: Public functions, storage access, event emissions, upgrades

#### Threat Scenarios & Impact Assessment:
- **Attack Prerequisites**: Required permissions, economic resources, technical capabilities
- **Attack Chains**: Multi-step exploits, compound vulnerabilities, escalation paths
- **Impact Analysis**: Funds loss, service disruption, state corruption, consensus failure
- **Economic Impact**: Quantified loss estimates, attack profitability, systemic risks

### 3. WEIGHT & PERFORMANCE ANALYSIS

#### Computational Cost Assessment:
- **Algorithm Complexity**: Big-O analysis, worst-case scenarios, nested operations
- **Weight Accuracy**: Weight vs. actual cost, calibration verification, under-pricing risks
- **Resource Consumption**: CPU cycles, memory allocation, storage I/O patterns
- **Benchmark Validation**: Coverage completeness, scenario testing, parameter validation

#### Performance Attack Vectors:
- **DoS via Computation**: Under-priced expensive operations, complexity exploitation
- **Block Space Monopolization**: Transaction spam, stuffing attacks, priority manipulation  
- **Weight Manipulation**: Refund mechanism abuse, early termination exploits
- **Edge Case Exploitation**: Maximum inputs, adversarial parameters, boundary condition attacks

## Deliverables

List all identified security issues with:

- **Issue Description**: Clear explanation of the security concern
- **Location**: Specific code references and line numbers where applicable  
- **Potential Risk**: How this could be problematic
- **Recommended Fix**: Specific code changes or mitigation steps

Focus on concrete, actionable findings with code examples and specific remediation steps.

{{security_disclaimer}}"#;

