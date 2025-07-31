# Substrate Prompts

## Global Assumptions

- Agent is being run on the directory of the target project and is intelligent enough to look for relevant files and get them its context.

## Prompts

## `release-comparison`

### Description

List changes between two polkadot-sdk release versions

### Arguments

- current_version: version currently being used
- target_version: version dev wants to compare with (must be greater than current)
- specific_changes (Optional): What specific changes to look for (e.g: was there any change in `pallet_treasury` ?)

### Prompt Proposals

```
Compare changes between Polkadot SDK versions <current_version> and 
<target_version>.

## Tools and Resources
- Use substrate_mcp's `get_polkadot_sdk_release_prdocs` tool to fetch release documentation
- Reference the polkadot-sdk repository: https://github.com/paritytech/polkadot-sdk
- PRDocs contain release notes with breaking changes, new features, and important updates

## Version Naming Conventions
1. **Semantic versions**: Follow standard semver (e.g., 1.2.3)
2. **Stable releases**: Format `stableYYMM[-patch]`
   - YYMM represents year and month (e.g., 2503 = March 2025)
   - Optional -X suffix for patches (e.g., stable2503-1)

Stable releases were implemented later than semantic versions so oldest stable
release comes after newer semantic versioned release.

## Version Range Logic

### Same Base Version (Different Patches)
If comparing patches of the same release (e.g., stable2503 → stable2503-4):
- Include ALL intermediate patches in sequence
- Example: stable2503 → stable2503-4 requires:
  - stable2503-1
  - stable2503-2
  - stable2503-3
  - stable2503-4

### Different Base Versions
If comparing different releases (e.g., stable2502 → stable2503-2):
1. Include ALL patches from the newer base version up to target
2. Include the base release notes for intermediate versions
3. Example: stable2502 → stable2503-2 requires:
   - stable2503 (base release)
   - stable2503-1
   - stable2503-2

<% if specific_changes %>
## Filtered Analysis
Focus only on changes related to: <specific_changes>
Filter PRDocs and code changes to match these criteria.
<% end %>

## Output Format

### Changes by Category

#### 🚨 Breaking Changes
Changes requiring code updates:
- **[Component]**: Description of breaking change
  - Migration guide: Steps to update
  - Affected versions: [version list]

#### ✨ New Features
New functionality added:
- **[Component]**: Feature description
  - First available in: [version]

#### 🐛 Bug Fixes
Issues resolved:
- **[Component]**: Fix description
  - Fixed in: [version]

#### 🔧 Improvements
Performance and quality improvements:
- **[Component]**: Improvement description

### Detailed Version Progression
For each version in sequence:

**[Version Number]**
- Release date: [if available]
- Key changes:
  - [Change 1]
  - [Change 2]
- Full PRDoc reference: [link or doc ID]

### Migration Recommendations
Based on the changes between versions:
1. **High Priority**: [Critical updates needed]
2. **Medium Priority**: [Recommended updates]
3. **Low Priority**: [Optional improvements]

<% if not specific_changes %>
### Additional Notes
- Changes not covered by PRDocs may exist in the codebase
- Review CHANGELOG.md files for complete details
<% end %>

```

## `scaffold-pallet`

### Description

Generate pallet structure and implementation templates

### Arguments

- pallet_description: description for the pallet

### Prompt Proposals

```
Create a complete Substrate pallet scaffold based on the following description:
<pallet_description>

## Project Structure
Create the following file structure:
```
pallets/<pallet_name>/
├── Cargo.toml
├── src/
│   ├── lib.rs         # Main pallet logic
│   ├── mock.rs        # Test runtime setup
│   ├── tests.rs       # Unit tests
│   ├── benchmarking.rs # Benchmarks
│   └── weights.rs     # Auto-generated weights
└── README.md          # Pallet documentation
```

## Implementation Requirements

### 1. Core Pallet Structure (`src/lib.rs`)
```rust
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;
        
        // Add other configuration parameters based on <pallet_description>
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Storage items based on pallet requirements
    #[pallet::storage]
    #[pallet::getter(fn example_storage)]
    pub type ExampleStorage<T> = StorageValue<_, u32, ValueQuery>;

    /// Events emitted by the pallet
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation
        SomethingHappened { who: T::AccountId, value: u32 },
    }

    /// Errors that can be returned by this pallet
    #[pallet::error]
    pub enum Error<T> {
        /// Error documentation
        InvalidInput,
        InsufficientPermission,
        // Add errors based on <pallet_description>
    }

    /// Dispatchable functions (extrinsics)
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Documentation for the extrinsic
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::example_extrinsic())]
        pub fn example_extrinsic(
            origin: OriginFor<T>,
            value: u32,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Validation
            ensure!(value > 0, Error::<T>::InvalidInput);
            
            // State changes
            <ExampleStorage<T>>::put(value);
            
            // Emit event
            Self::deposit_event(Event::SomethingHappened { who, value });
            
            Ok(())
        }
    }

    /// Helper functions (private)
    impl<T: Config> Pallet<T> {
        fn helper_function() -> Result<(), Error<T>> {
            // Implementation
            Ok(())
        }
    }
}

/// Weight information trait
pub trait WeightInfo {
    fn example_extrinsic() -> Weight;
}

/// Default weight implementation
impl WeightInfo for () {
    fn example_extrinsic() -> Weight {
        Weight::from_parts(10_000, 0)
    }
}
```

### 2. Mock Runtime (`src/mock.rs`)
```rust
use crate as pallet_template;
use frame_support::{
    parameter_types,
    traits::{ConstU16, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test {
        System: frame_system,
        TemplateModule: pallet_template,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
    // System config implementation
}

impl pallet_template::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}
```

### 3. Unit Tests (`src/tests.rs`)
Include tests for:
- ✅ Happy path scenarios
- ❌ Error conditions
- 🔒 Permission checks
- 📊 State changes
- 📢 Event emissions

```rust
use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn example_extrinsic_works() {
    new_test_ext().execute_with(|| {
        // Arrange
        let caller = 1;
        let value = 42;
        
        // Act
        assert_ok!(TemplateModule::example_extrinsic(
            RuntimeOrigin::signed(caller),
            value
        ));
        
        // Assert
        assert_eq!(TemplateModule::example_storage(), value);
        System::assert_last_event(
            Event::SomethingHappened { who: caller, value }.into()
        );
    });
}

#[test]
fn example_extrinsic_fails_with_invalid_input() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TemplateModule::example_extrinsic(RuntimeOrigin::signed(1), 0),
            Error::<Test>::InvalidInput
        );
    });
}
```

### 4. Benchmarks (`src/benchmarking.rs`)
```rust
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
    example_extrinsic {
        let caller: T::AccountId = whitelisted_caller();
        let value = 100u32;
    }: _(RawOrigin::Signed(caller.clone()), value)
    verify {
        assert_eq!(ExampleStorage::<T>::get(), value);
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::new_test_ext(),
        crate::mock::Test
    );
}
```

### 5. Cargo.toml
```toml
[package]
name = "pallet-<pallet_name>"
version = "0.1.0"
authors = ["Your Name"]
edition = "2021"

[dependencies]
codec = { package = "parity-scale-codec", version = "3.6.1", default-features = false }
scale-info = { version = "2.10.0", default-features = false, features = ["derive"] }
frame-support = { default-features = false, git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }
frame-system = { default-features = false, git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }
frame-benchmarking = { default-features = false, optional = true, git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }

[dev-dependencies]
sp-core = { git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }
sp-io = { git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }
sp-runtime = { git = "https://github.com/paritytech/polkadot-sdk.git", branch = "master" }

[features]
default = ["std"]
std = [
    "codec/std",
    "scale-info/std",
    "frame-support/std",
    "frame-system/std",
    "frame-benchmarking?/std",
]
runtime-benchmarks = [
    "frame-benchmarking/runtime-benchmarks",
    "frame-support/runtime-benchmarks",
    "frame-system/runtime-benchmarks",
]
```

## Implementation Guidelines

1. **Storage Design**
   - Use appropriate storage types (Value, Map, DoubleMap)
   - Consider storage costs and access patterns
   - Add proper getters with documentation

2. **Error Handling**
   - Define specific, descriptive errors
   - Use `ensure!` for validation
   - Return early on errors

3. **Events**
   - Emit events for all state changes
   - Include relevant data for indexing
   - Document event meanings

4. **Weights**
   - Benchmark all extrinsics
   - Use realistic worst-case scenarios
   - Update weights after changes

5. **Testing**
   - Test all success paths
   - Test all error conditions
   - Test edge cases and boundaries
   - Test event emissions

## References
- Basic pallet structure: https://docs.polkadot.com/develop/parachains/customize-parachain/make-custom-pallet/
- Testing guide: https://docs.polkadot.com/develop/parachains/testing/pallet-testing/
- Benchmarking: https://docs.polkadot.com/develop/parachains/testing/benchmarking/

```

## `code-security-audit`

### Description

Audit specific component for common code-related vulnerabilities.

### Arguments

- audit_type: pallet/runtime/node/general
- audit_target: describe the target of the audit
- specific_checks (Optional): Specific things to look for.

### Prompt Proposals

```
You are a Systems Security Expert specializing in Substrate-based blockchain
security. Perform a comprehensive security audit following industry-standard
practices and Substrate-specific considerations.

## Audit Target
<audit_target>

## Audit Scope
<%if <specific_checks> %>
### Focused Security Checks
Prioritize analysis of: <specific_checks>
<% else %>
### Audit Type: <audit_type>
Perform comprehensive analysis with emphasis on:
<%= vulnerability_checklist_for(audit_type) %>
<% end %>
```

## `pallet-incentive-analysis`

### Description

Analyze economic viability of incentives

### Arguments

- target_pallets: List of pallets that make the scope of the analysis
- analysis_specifications: Specific things to look out for during the analysis.

### Prompt Proposals

```
You are an expert in Cryptoeconomics specializing in Substrate-based 
blockchain systems. Analyze the incentive mechanisms in the specified pallets
using game theory and mechanism design principles.

## Target Pallets
<target_pallets>

## Analysis Framework

<%if <analysis_specifications> is specified %>
<analysis_specifications>
<% else %>
### 1. Stakeholder Mapping
- Identify all actors (validators, nominators, users, governance participants)
- Define their objectives and constraints
- Map their available strategies

### 2. Incentive Mechanisms
- **Rewards**: Distribution mechanisms, rates, and conditions
- **Penalties**: Slashing conditions, fees, and opportunity costs
- **Game Theory**: Nash equilibria, dominant strategies, attack vectors

### 3. Economic Security
- Cost of attacks vs potential gains
- Griefing resistance
- Sybil attack considerations
- MEV opportunities

### 4. Substrate-Specific Analysis
- Weight economy and fee market dynamics
- Treasury funding/drainage patterns
- Cross-pallet economic dependencies
- Governance capture risks

### 5. Dynamic Analysis
- Behavior under different market conditions
- Long-term sustainability
- Centralization tendencies
- Wealth concentration effects
<% end %>
```

```

## `threat-modeling`

### Description

Do threat modeling of a specific part of the system

### Arguments

- system_description: Description of the system to make the analysis for (all pallets, a specific group/flow, node, etc)
- extra_context: Extra context to provide for analysis

### Prompt Proposals

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

## Output Format Example

The agent should provide output structured like:

```markdown
# Security Threat Model: [Subsystem Name]

## Executive Summary
- Critical findings: X
- High severity: Y
- Medium severity: Z

## 1. Asset Analysis
### Identified Assets
- Asset 1: [Description]
- Asset 2: [Description]

### Trust Boundaries
- Boundary 1: [Description]

## 2. Attack Surface
### Dispatchable Functions
1. `function_name` (line X): [Security considerations]

## 3. Vulnerabilities Found
### CRITICAL: Integer Overflow in Transfer Logic
- Location: `src/lib.rs:123`
- Description: [Details]
- Impact: [Consequences]
- Proof of Concept: [Code example]
- Remediation: [Fix with code]

[Continue for all sections...]
```

```

## `economic-security`

### Description

Do an economic security analysis on a specific subsystem

### Arguments

- system_description: Description of the system to make the analysis for (all pallets, a specific group/flow, etc)
- extra_context: Extra context to provide for analysis

### Prompt Proposals

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

## Example Usage

### Example 1: Analyzing a DEX Pallet
```
Perform a comprehensive economic security assessment of the following Substrate subsystem:

**Subsystem**: pallets/dex/src/lib.rs
**Context**: Automated market maker (AMM) implementation with constant product formula, 0.3% swap fees, and liquidity provider tokens.

[Rest of base prompt...]
```

### Example 2: Analyzing Staking Economics
```
Perform a comprehensive economic security assessment of the following Substrate subsystem:

**Subsystem**: pallets/staking/src/lib.rs
**Context**: Nominated Proof-of-Stake system with 10% annual inflation, 28-day unbonding, and slashing for equivocation.

[Rest of base prompt + staking-specific analysis...]
```

## Key Metrics to Calculate

1. **Attack Profitability Threshold**
   - Cost to execute attack
   - Expected profit/loss
   - Break-even points

2. **Centralization Metrics**
   - Gini coefficient
   - Herfindahl index
   - Top holder percentages

3. **Liquidity Metrics**
   - Available liquidity for attacks
   - Slippage calculations
   - Market depth analysis

4. **Time-based Risks**
   - Time-weighted attack opportunities
   - Delayed effect vulnerabilities
   - Front-running windows

## Output Format Example

The agent should provide output structured like:

```markdown
# Economic Security Assessment: [Subsystem Name]

## Executive Summary
- Critical economic risks: X
- High severity findings: Y
- Total value at risk: $Z

## 1. Economic Model Overview
### Value Flows
- Input: [Sources of value]
- Output: [Value destinations]
- Fees: [Fee structure]

### Actor Incentives
- Actor Type 1: [Incentives and strategies]
- Actor Type 2: [Incentives and strategies]

## 2. Attack Scenarios

### CRITICAL: Governance Buying Attack
- Attack Cost: 1,000,000 tokens
- Success Probability: 85%
- Potential Profit: 5,000,000 tokens
- Execution Steps:
  1. Accumulate voting power
  2. Submit malicious proposal
  3. Vote with accumulated power
- Mitigation: Implement conviction voting

### HIGH: MEV via Transaction Ordering
- Profit per block: ~500 tokens
- Attack requirements: Validator control
- Code location: `src/lib.rs:234`
- Mitigation: Randomized ordering

[Continue for all findings...]

## 3. Recommendations
1. Immediate: [Critical fixes]
2. Short-term: [Important improvements]
3. Long-term: [Strategic changes]
```

## Analysis Tips

1. **Follow the Money**: Trace every token flow path
2. **Think Like an Attacker**: What's the most profitable exploit?
3. **Consider Composability**: How do multiple pallets interact economically?
4. **Model Edge Cases**: What happens at extremes (0 liquidity, 100% stake, etc.)?
5. **Calculate Real Numbers**: Use actual parameters to quantify risks

```

## `weight-analysis`

Weight-based system breakdown analysis under extreme conditions

### Description

Weight-based system breakdown analysis under extreme conditions

### Arguments

- target_pallet: pallet to make the analysis for

```
Perform a comprehensive weight analysis of the following Substrate pallet under extreme and adversarial conditions:

**Pallet**: <target_pallet>

Please analyze the pallet's weight calculations, benchmarks, and resource usage to identify:

1. **Weight Function Analysis**
   - Review all weight calculations in the pallet
   - Verify weight functions match actual complexity
   - Identify any hardcoded or constant weights
   - Check for missing weight annotations
   - Analyze weight refunds and their correctness

2. **Computational Complexity Verification**
   - For each extrinsic, determine Big-O complexity
   - Identify nested loops and their bounds
   - Find recursive operations and depth limits
   - Verify complexity matches weight calculations
   - Look for quadratic or exponential behavior

3. **Storage Operation Analysis**
   - Count storage reads/writes per extrinsic
   - Identify unbounded storage iterations
   - Check for storage maps without size limits
   - Analyze batch operations and their limits
   - Verify storage deposit calculations

4. **Extreme Input Scenarios**
   - Maximum vector/array sizes
   - Deeply nested data structures
   - Maximum iteration counts
   - Worst-case branching paths
   - Edge cases (0, 1, MAX values)

5. **DoS Attack Vectors**
   - Under-priced expensive operations
   - Weight manipulation possibilities
   - Resource exhaustion attacks
   - State bloat vulnerabilities
   - Block space monopolization

6. **Benchmark Coverage Analysis**
   - Review existing benchmarks
   - Identify missing benchmark scenarios
   - Check if benchmarks cover worst cases
   - Verify benchmark parameters are realistic
   - Analyze benchmark result variance

7. **Cross-Pallet Interactions**
   - Weight implications of pallet coupling
   - Cascading computational costs
   - Hidden complexity from trait implementations
   - Event emission costs

8. **Mitigation Recommendations**
   - Specific weight function corrections
   - Additional bounds and limits needed
   - Benchmark improvements
   - Code optimizations
   - Parameter tuning suggestions

Format your response as a detailed security audit with specific findings, severity ratings, and code examples demonstrating the issues.
```

## Analysis Focus Areas

### For Storage-Heavy Pallets
```
Additionally analyze:
- Storage migration weight costs
- Child trie operations
- Storage proof size implications
- Merkle proof generation costs
- State rent/deposit requirements
- Garbage collection needs
- Historical data pruning costs
```

### For Computation-Heavy Pallets
```
Additionally analyze:
- Cryptographic operation costs
- Hash function usage patterns
- Signature verification batching
- Complex mathematical operations
- String/byte manipulation costs
- Sorting and searching algorithms
- Memory allocation patterns
```

### For Democracy/Governance Pallets
```
Additionally analyze:
- Vote tallying complexity
- Delegation chain depths
- Proposal queue processing
- Referendum enumeration costs
- Historical lookup operations
- Batch voting operations
```

### For Asset/Token Pallets
```
Additionally analyze:
- Multi-asset operation scaling
- Approval/allowance iterations
- Balance calculation complexity
- Transfer batch limits
- Metadata storage costs
- Asset creation/destruction overhead
```

### For XCM/Cross-chain Pallets
```
Additionally analyze:
- Message processing complexity
- Queue management overhead
- Multi-hop routing costs
- Asset conversion calculations
- Error handling paths
- Timeout processing costs
```

## Critical Patterns to Identify

### 1. Unbounded Iterations
```rust
// BAD: Unbounded iteration
for account in all_accounts.iter() {
    // Process each account
}

// GOOD: Bounded iteration
for account in all_accounts.iter().take(T::MaxIterations::get()) {
    // Process limited accounts
}
```

### 2. Nested Loops
```rust
// BAD: O(n²) complexity
for i in 0..items.len() {
    for j in 0..items.len() {
        // Quadratic operation
    }
}
```

### 3. Recursive Operations
```rust
// BAD: Unbounded recursion
fn process(depth: u32) {
    if condition {
        process(depth + 1);
    }
}
```

## Example Usage

### Example 1: Analyzing a DEX Pallet
```
Perform a comprehensive weight analysis of the following Substrate pallet under extreme and adversarial conditions:

**Pallet**: pallets/dex/src/lib.rs
**Context**: DEX pallet with swap, add_liquidity, and remove_liquidity operations. Uses BTreeMap for pools and supports multi-hop swaps.

[Rest of base prompt...]
```

### Example 2: Analyzing Staking Pallet
```
Perform a comprehensive weight analysis of the following Substrate pallet under extreme and adversarial conditions:

**Pallet**: pallets/staking/src/lib.rs
**Context**: Staking pallet managing validators, nominators, and reward distribution. Critical operations include bonding, nominating, and payout calculations.

[Rest of base prompt + staking-specific analysis...]
```

## Expected Output Format

```markdown
# Weight Analysis Report: [Pallet Name]

## Executive Summary
- Critical findings: X
- High severity issues: Y
- Total DoS vectors identified: Z

## 1. Weight Function Audit

### CRITICAL: Unbounded Storage Iteration in `do_transfer`
- Location: `src/lib.rs:234`
- Current Weight: `T::DbWeight::get().reads(1)`
- Actual Complexity: `O(n)` where n = number of holders
- PoC Attack:
  ```rust
  // Attacker creates many dust accounts
  for i in 0..10000 {
      create_dust_account(i);
  }
  // Single transfer now iterates all accounts
  transfer(origin, dest, amount); // DoS
  ```
- Recommendation: Add pagination or bounded iteration

### HIGH: Incorrect Weight for Complex Calculation
- Location: `src/lib.rs:567`
- Issue: Weight assumes O(1) but implementation is O(n log n)
- Impact: 100x under-pricing for large inputs
- Fix:
  ```rust
  #[pallet::weight(T::WeightInfo::complex_calc(items.len()))]
  ```

## 2. Benchmark Analysis

### Missing Benchmarks
1. `force_transfer` - No benchmark for admin operations
2. `batch_transfer` - Missing worst-case scenario
3. `migrate_v2` - Storage migration unbenchmarked

### Benchmark Quality Issues
- `swap` benchmark uses unrealistic parameters
- `add_liquidity` doesn't test maximum pool size
- Variance too high (>10%) indicating unstable measurements

## 3. DoS Vector Summary

| Vector | Severity | Cost to Attack | Impact |
|--------|----------|----------------|---------|
| Unbounded iteration | CRITICAL | Low (100 DOT) | Full block |
| Storage bloat | HIGH | Medium (1000 DOT) | State growth |
| Compute spam | MEDIUM | High (10000 DOT) | Degraded performance |

## 4. Recommendations

### Immediate Actions
1. Fix unbounded iterations in functions X, Y, Z
2. Add proper weight functions for all extrinsics
3. Implement storage bounds

### Short-term Improvements
1. Comprehensive benchmark suite
2. Add circuit breakers for expensive operations
3. Implement progressive pricing

### Long-term Considerations
1. Redesign storage schema for better scalability
2. Consider off-chain workers for heavy computation
3. Implement storage rent mechanisms
```

## Weight Analysis Checklist

- [ ] All extrinsics have weight annotations
- [ ] Weight functions account for all parameters
- [ ] Benchmarks cover worst-case scenarios
- [ ] No unbounded loops or recursion
- [ ] Storage operations are bounded
- [ ] Cross-pallet calls are accounted for
- [ ] Weight refunds are correctly calculated
- [ ] Defensive weight padding for safety
- [ ] Maximum block weight cannot be exceeded
- [ ] Storage deposits prevent state bloat

```

## `automated-analysis`

### Description

Template for automated code and runtime analysis

### Arguments

change_description: description of the changes made to the code that trigger this analysis (PR description, new release, etc)

### Prompt Proposals

```
Perform a comprehensive security and quality analysis of the following Substrate project changes for pre-release/PR review:

**Context**: <change_description>

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
```

## Component-Specific Analysis

### For Runtime Changes
```
Additionally review:
- Spec version and transaction version updates
- Runtime API changes and compatibility
- Pallet configuration changes
- System pallet modifications
- Executive pallet ordering
- Runtime constant changes
- Feature flag impacts
```

### For Pallet Development
```
Additionally review:
- Storage item declarations and bounds
- Dispatchable function signatures
- Error variant completeness
- Event definitions and usage
- Config trait requirements
- GenesisConfig implementation
- Pallet hooks (on_initialize, on_finalize)
```

### For Consensus Changes
```
Additionally review:
- Block production modifications
- Finality gadget changes
- Fork choice rule updates
- Validator set management
- Slashing logic modifications
- Session key handling
- Network protocol changes
```

### For Client/Node Changes
```
Additionally review:
- RPC interface modifications
- Database schema changes
- Network protocol updates
- Command line interface changes
- Telemetry modifications
- Import/export functionality
- Pruning logic changes
```

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

```

## LLM Disclaimer legend

To ask the llm to include in all the “audits”

```
## ⚠️ AI ANALYSIS DISCLAIMER ⚠️

**This is NOT a professional security audit.** This AI-generated analysis:
- May miss critical vulnerabilities
- May report false positives  
- Cannot replace human security experts
- Must be verified by professionals

Use this ONLY as a supplementary tool for initial review. For production systems, always engage qualified security auditors.
```

```