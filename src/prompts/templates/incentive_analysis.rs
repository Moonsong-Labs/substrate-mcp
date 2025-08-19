/// Pallet incentive analysis handlebars template
pub const PROMPT: &str = r#"{{security_disclaimer}}

You are an expert in Cryptoeconomics specializing in Substrate-based
blockchain systems. Analyze the incentive mechanisms in the specified pallets
using game theory and mechanism design principles.

## Target Pallets
{{target_pallets}}

## Analysis Framework

{{analysis_specifications}}

{{security_disclaimer}}"#;