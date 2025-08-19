use handlebars::Handlebars;
use std::sync::LazyLock;

mod release_comparison;

/// Template registry for handlebars templates
pub static TEMPLATE_REGISTRY: LazyLock<Handlebars> = LazyLock::new(|| {
    let mut handlebars = Handlebars::new();

    // Register the release comparison template
    handlebars
        .register_template_string("release_comparison", release_comparison::PROMPT)
        .expect("Failed to register release_comparison template");

    handlebars
});
