use handlebars::Handlebars;
use serde_json::json;
use std::sync::LazyLock;

mod analyze_release;
mod automated_analysis;
mod code_security_audit;
mod economic_security;
mod incentive_analysis;
mod release_comparison;
mod scaffold_pallet;
mod security_disclaimer;
mod threat_modeling;
mod weight_analysis;

/// Template registry for handlebars templates
pub static TEMPLATE_REGISTRY: LazyLock<Handlebars> = LazyLock::new(|| {
    let mut handlebars = Handlebars::new();

    // Register helper for equality checks in templates
    handlebars.register_helper(
        "eq",
        Box::new(
            |h: &handlebars::Helper,
             _: &Handlebars,
             _: &handlebars::Context,
             _: &mut handlebars::RenderContext,
             out: &mut dyn handlebars::Output|
             -> handlebars::HelperResult {
                let param1 = h.param(0).map(|v| v.value());
                let param2 = h.param(1).map(|v| v.value());
                let result = match (param1, param2) {
                    (Some(v1), Some(v2)) => v1 == v2,
                    _ => false,
                };
                out.write(if result { "true" } else { "" })?;
                Ok(())
            },
        ),
    );

    // Register all templates
    handlebars
        .register_template_string("release_comparison", release_comparison::PROMPT)
        .expect("Failed to register release_comparison template");

    handlebars
        .register_template_string("automated_analysis", automated_analysis::PROMPT)
        .expect("Failed to register automated_analysis template");

    handlebars
        .register_template_string("code_security_audit", code_security_audit::PROMPT)
        .expect("Failed to register code_security_audit template");

    handlebars
        .register_template_string("economic_security", economic_security::PROMPT)
        .expect("Failed to register economic_security template");

    handlebars
        .register_template_string(
            "pallet_incentive_analysis",
            pallet_incentive_analysis::PROMPT,
        )
        .expect("Failed to register pallet_incentive_analysis template");

    handlebars
        .register_template_string("scaffold_pallet", scaffold_pallet::PROMPT)
        .expect("Failed to register scaffold_pallet template");

    handlebars
        .register_template_string("threat_modeling", threat_modeling::PROMPT)
        .expect("Failed to register threat_modeling template");

    handlebars
        .register_template_string("weight_analysis", weight_analysis::PROMPT)
        .expect("Failed to register weight_analysis template");

    handlebars
        .register_template_string("analyze_release", analyze_release::PROMPT)
        .expect("Failed to register analyze_release template");

    handlebars
});

/// Get the security disclaimer to be injected into templates
pub fn get_security_disclaimer() -> serde_json::Value {
    json!(security_disclaimer::SECURITY_DISCLAIMER)
}
