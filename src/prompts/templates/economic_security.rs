/// Economic security handlebars template
pub const PROMPT: &str = r#"{{security_disclaimer}}

Perform a comprehensive economic security assessment of the following Substrate subsystem:

**Subsystem**: {{system_description}}
**Context**: {{extra_context}}

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