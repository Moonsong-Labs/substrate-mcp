use rmcp::model::PromptArgument;

use super::SubstratePromptDefinition;

/// Weight analysis prompt template
const TEMPLATE: &str = r#"{{security_disclaimer}}

Perform a comprehensive weight analysis of the following Substrate pallet under extreme and adversarial conditions:

**Pallet**: {{target_pallet}}

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

{{security_disclaimer}}"#;

pub fn prompt_definition() -> SubstratePromptDefinition {
    SubstratePromptDefinition {
        name: "weight_analysis".to_string(),
        description: "Weight-based system breakdown analysis under extreme conditions".to_string(),
        arguments: vec![PromptArgument {
            name: "target_pallet".to_string(),
            description: Some("Pallet to make the analysis for".to_string()),
            required: Some(true),
        }],
        template: TEMPLATE.to_string(),
    }
}
