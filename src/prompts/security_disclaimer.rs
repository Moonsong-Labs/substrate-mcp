/// Security disclaimer instruction for AI-generated analysis
pub const SECURITY_DISCLAIMER: &str = r#"

# CRITICAL REQUIREMENT: Security Disclaimer

**MANDATORY**: You MUST include the following security disclaimer at the end of your analysis. This is non-negotiable and required for all security-related outputs.

Include this disclaimer VERBATIM:

<disclaimer>
## ⚠️ AI ANALYSIS DISCLAIMER ⚠️

**This is NOT a professional security audit.** This AI-generated analysis:
- May miss critical vulnerabilities
- May report false positives
- Cannot replace human security experts
- Must be verified by professionals

Use this ONLY as a supplementary tool for initial review. For production systems, always engage qualified security auditors.
</disclaimer>

## Example Usage:

❌ **INCORRECT** (missing disclaimer):
```
Security Analysis Complete:
- Found 3 potential vulnerabilities
- Recommended fixes implemented
```

✅ **CORRECT** (includes disclaimer):
```
Security Analysis Complete:
- Found 3 potential vulnerabilities
- Recommended fixes implemented

<disclaimer>
## ⚠️ AI ANALYSIS DISCLAIMER ⚠️

**This is NOT a professional security audit.** This AI-generated analysis:
- May miss critical vulnerabilities
- May report false positives
- Cannot replace human security experts
- Must be verified by professionals

Use this ONLY as a supplementary tool for initial review. For production systems, always engage qualified security auditors.
</disclaimer>
```

**REMEMBER**: You MUST include this disclaimer at the end of your response. This is non-negotiable"#;
