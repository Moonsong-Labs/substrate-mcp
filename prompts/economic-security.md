# Economic Security

## Description

Do an economic security analysis on a specific subsystem

## Arguments

- system_description: Description of the system to make the analysis for (all pallets, a specific group/flow, etc)
- extra_context: Extra context to provide for analysis

## Prompt

```
Perform a comprehensive economic security assessment of the following Substrate subsystem:

**Subsystem**: <system_description>
**Context**: <extra_context>

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
```

## Subsystem-Specific Extensions

### For Staking Systems
```
Additionally analyze:
- Validator selection economics
- Slashing economic impact
- Nomination pool dynamics
- Reward distribution fairness
- Stake concentration metrics
- Minimum stake requirements impact
- Era/epoch transition vulnerabilities
```

### For Governance Systems
```
Additionally analyze:
- Voting power concentration
- Proposal spam economics
- Treasury drain attacks
- Delegation vulnerabilities
- Time-weighted voting exploits
- Conviction voting manipulation
- Referendum buying costs
```

### For DEX/AMM Pallets
```
Additionally analyze:
- Impermanent loss scenarios
- Liquidity provider incentives
- Arbitrage profitability
- Pool manipulation costs
- Oracle price dependencies
- Flash swap attack vectors
- Fee structure optimality
```

### For Treasury/Reserve Systems
```
Additionally analyze:
- Fund allocation game theory
- Proposal funding attacks
- Treasury drain scenarios
- Tip/bounty gaming
- Budget exhaustion attacks
- Multi-sig vulnerabilities
- Time-lock bypasses
```

### For Lending/Borrowing
```
Additionally analyze:
- Liquidation incentives
- Interest rate manipulation
- Collateral ratio attacks
- Bad debt accumulation
- Oracle manipulation impact
- Flash loan attack combinations
- Recursive borrowing risks
```

### For Cross-Chain (XCM)
```
Additionally analyze:
- Bridge liquidity attacks
- Cross-chain arbitrage
- Message ordering exploits
- Fee asymmetry abuse
- Reserve draining
- Double-spend via rollbacks
- Parachain economic attacks
```

## Analysis Tips

1. **Follow the Money**: Trace every token flow path
2. **Think Like an Attacker**: What's the most profitable exploit?
3. **Consider Composability**: How do multiple pallets interact economically?
4. **Model Edge Cases**: What happens at extremes (0 liquidity, 100% stake, etc.)?
5. **Calculate Real Numbers**: Use actual parameters to quantify risks