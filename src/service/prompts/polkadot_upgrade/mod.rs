//! Analyze release prompt implementation

use std::env;

use handlebars::Handlebars;
use rmcp::model::{PromptMessage, PromptMessageRole};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::common::SECURITY_DISCLAIMER;
use template::TEMPLATE;

mod template;

/// Arguments for the analyze release prompt
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(description = "Analyze how a Polkadot SDK release impacts your project")]
pub(crate) struct PolkadotUpgradeArgs {
    #[schemars(description = "Release name/version to analyze")]
    pub(crate) release: String,
}

/// Generate analyze release prompt content
pub(crate) async fn generate_prompt(args: PolkadotUpgradeArgs) -> Vec<PromptMessage> {
    let mut messages = Vec::new();

    // Render the main prompt with context
    let handlebars = Handlebars::new();
    let context = json!({
        "release": args.release,
        "security_disclaimer": SECURITY_DISCLAIMER,
        "project_name": get_project_name(),
    });

    let content = handlebars
        .render_template(TEMPLATE, &context)
        .unwrap_or_else(|e| format!("Template rendering failed: {}", e));

    messages.push(PromptMessage::new_text(PromptMessageRole::User, content));

    messages
}

/// Get the project name from the current directory
pub(crate) fn get_project_name() -> String {
    env::current_dir()
        .ok()
        .and_then(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}
