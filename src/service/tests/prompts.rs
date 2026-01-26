use super::helpers::mcp_client::TestMcpClient;
use serde_json::json;

/// Debug test to inspect what prompts/list returns
#[tokio::test]
async fn test_debug_list_prompts() {
    let client = TestMcpClient::new()
        .await
        .expect("Failed to create MCP client");

    let result = client.list_prompts().await.unwrap();

    println!("\n=== DEBUG: prompts/list response ===\n");
    println!("Number of prompts: {}", result.prompts.len());

    for prompt in &result.prompts {
        println!("\nPrompt: {}", prompt.name);
        println!("  Description: {:?}", prompt.description);
        println!("  Arguments: {:?}", prompt.arguments);

        if let Some(args) = &prompt.arguments {
            for arg in args {
                println!("    - name: {}", arg.name);
                println!("      description: {:?}", arg.description);
                println!("      required: {:?}", arg.required);
            }
        }
    }

    // Find the analyze_release prompt
    let analyze_release = result
        .prompts
        .iter()
        .find(|p| p.name == "analyze_release")
        .expect("analyze_release prompt should exist");

    // Check if arguments are populated
    assert!(
        analyze_release.arguments.is_some(),
        "analyze_release should have arguments defined"
    );

    let args = analyze_release.arguments.as_ref().unwrap();
    assert!(
        !args.is_empty(),
        "analyze_release should have at least one argument"
    );

    // Check for the release argument
    let release_arg = args
        .iter()
        .find(|a| a.name == "release")
        .expect("Should have release argument");

    println!("\n=== Release argument found ===");
    println!("  name: {}", release_arg.name);
    println!("  description: {:?}", release_arg.description);
    println!("  required: {:?}", release_arg.required);
}

/// Test that prompts/get works with arguments
#[tokio::test]
async fn test_get_prompt_with_arguments() {
    let client = TestMcpClient::new()
        .await
        .expect("Failed to create MCP client");

    let args = json!({"release": "stable2503"});
    let result = client.get_prompt("analyze_release", args).await;

    match &result {
        Ok(prompt_result) => {
            println!("\n=== DEBUG: prompts/get response ===\n");
            println!("Description: {:?}", prompt_result.description);
            println!("Messages count: {}", prompt_result.messages.len());
            // Print first part of first message if available
            if let Some(msg) = prompt_result.messages.first() {
                println!("First message role: {:?}", msg.role);
            }
        }
        Err(e) => {
            println!("\n=== ERROR: prompts/get failed ===\n");
            println!("Error: {:?}", e);
        }
    }

    assert!(
        result.is_ok(),
        "Should successfully get prompt with arguments: {:?}",
        result.err()
    );
}

/// Test that the focus optional argument works
#[tokio::test]
async fn test_get_prompt_with_optional_focus() {
    let client = TestMcpClient::new()
        .await
        .expect("Failed to create MCP client");

    let args = json!({
        "release": "stable2503",
        "focus": "security"
    });
    let result = client.get_prompt("analyze_release", args).await;

    assert!(
        result.is_ok(),
        "Should successfully get prompt with release and focus: {:?}",
        result.err()
    );
}
