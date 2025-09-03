//! Economic security prompt implementation

use handlebars::Handlebars;
use rmcp::model::{PromptMessage, PromptMessageRole};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::common::SECURITY_DISCLAIMER;

/// Arguments for the economic security prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Economic security assessment")]
pub(crate) struct EconomicSecurityArgs {
    #[schemars(description = "Description of the system to analyze")]
    pub(crate) system_description: String,
}

/// Generate economic security prompt content
pub(crate) async fn generate_prompt(args: EconomicSecurityArgs) -> Vec<PromptMessage> {
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

const TEMPLATE: &str = r#"{{security_disclaimer}}

Perform a comprehensive economic security assessment of the following Substrate subsystem:

**Subsystem**: {{system_description}}

Please analyze the code and economic design to provide a detailed assessment covering:

1. **Economic Model Analysis**
   - Map all value flows (tokens, fees, rewards, slashing)
   - Identify all economic actors and their incentives
   - Document fee structures and economic parameters
   - Analyze token supply dynamics (minting, burning, inflation)

2. **Game Theory Analysis**
   - Dominant strategies for each actor type
   - Nash equilibria identification
   - Coalition/collusion opportunities
   - Griefing attack potential (imposing costs on others)
   - Incentive compatibility analysis

3. **MEV (Maximal Extractable Value) Assessment**
   - Transaction ordering dependencies
   - Front-running opportunities
   - Sandwich attack vectors
   - Back-running possibilities
   - Cross-chain MEV risks (if using XCM)

4. **Economic Attack Vectors**
   - Token manipulation attacks
   - Governance buying/bribing
   - Flash loan vulnerabilities
   - Liquidity attacks
   - Sybil attack resistance
   - Economic denial of service

5. **Market Manipulation Risks**
   - Price oracle dependencies
   - Liquidation cascades
   - Market cornering possibilities
   - Wash trading vulnerabilities
   - Arbitrage exploits

6. **Staking/Governance Specific** (if applicable)
   - Stake centralization risks
   - Nothing-at-stake problems
   - Long-range attacks
   - Bribery resistance
   - Vote buying mechanisms

7. **Risk Quantification**
   - Potential loss estimates
   - Attack cost calculations
   - Profitability thresholds
   - Risk/reward ratios

8. **Mitigation Strategies**
   - Economic parameter tuning
   - Circuit breakers and limits
   - Time delays and cooling periods
   - Slashing conditions
   - Governance controls

Format your response as a structured economic security report with specific calculations, attack scenarios, and actionable recommendations. Include code references where economic logic is implemented.

{{security_disclaimer}}"#;
