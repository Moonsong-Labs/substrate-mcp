//! Embedded resource definitions for Substrate development documentation and tools.
//! These resources serve as an index for agents to know where to find official documentation,
//! tools, and references for Substrate blockchain development.

use indoc::indoc;
use rmcp::model::{Annotations, RawResource, Resource, Role};

struct MarkdownResource {
    uri: String,
    name: String,
    description: String,
    content: String,
    priority: f32,
    audience: Option<Vec<Role>>,
}

impl From<MarkdownResource> for Resource {
    fn from(val: MarkdownResource) -> Self {
        Resource::new(
            RawResource {
                uri: val.uri,
                name: val.name,
                description: Some(val.description),
                mime_type: Some("text/markdown".to_string()),
                size: Some(val.content.len() as u32),
            },
            Some(Annotations {
                audience: val.audience,
                priority: Some(val.priority),
                last_modified: None,
            }),
        )
    }
}

fn markdown_resources() -> Vec<MarkdownResource> {
    vec![
        MarkdownResource {
            uri: "file:///scale-value-format.md".to_string(),
            name: "scale_value String Format Guide".to_string(),
            description: "Guide for using scale_value string format when submitting extrinsics to Substrate chains. Must read when calling substrate-mcp tools that require string representation of scale values".to_string(),
            content: indoc! {r#"
            # scale_value String Format Guide

            When using the `submit_extrinsic` tool, arguments must be provided in scale_value string format. This format is similar to JSON but with some specific syntax for Substrate types.

            ## Basic Types

            ### Numbers
            - Unsigned integers: `123`, `1000000000000`
            - Signed integers: `-42`, `100`
            - Hexadecimal: `0xFF`

            ### Strings
            - Double quoted: `"hello world"`
            - Escape quotes inside: `"say \"hello\""`

            ### Booleans
            - `true` or `false`

            ### Arrays/Sequences
            - Use parentheses (unnamed composite): `(1, 2, 3)`
            - Mixed types: `(123, "hello", true)`
            - Note: Square bracket syntax `[1, 2, 3]` is NOT supported

            ## Important: Variant/Enum Syntax

            Key rules for variants:
            1. Unit variants (no data) MUST use empty parentheses: `None()`, not `None`
            2. The v-prefix syntax `v"VariantName"` ALWAYS requires parentheses: `v"VariantName"(...)`
            3. For Option types specifically:
               - ✅ Correct: `None()`, `Some(42)`
               - ✅ Also valid: `v"Some"(42)`, `v"None"()`
               - ❌ Wrong: `None`, `v"None"` (missing parentheses)

            ## Complex Types

            ### Named Composites (Objects)
            Use curly braces with unquoted field names:
            ```
            { field1: "value", field2: 123, field3: true }
            ```

            ### Unnamed Composites (Tuples)
            Use parentheses:
            ```
            (123, "hello", true)
            ```

            ### Variants (Enums)
            Two syntaxes are supported:

            1. **Standard syntax** (recommended):
            ```
            None()
            Some(42)
            Error("Not found")
            ```

            2. **Alternative v-prefix syntax**:
            ```
            v"None"()          # Unit variant with v-prefix
            v"Some"(42)
            v"Ok"("hello")
            v"Id"((1, 2, 3, 4))
            ```

            ⚠️ IMPORTANT: The v-prefix syntax ALWAYS requires parentheses, even for unit variants.
            `v"None"` without parentheses will fail - use `v"None"()` or `None()`.

            ### Nested Structures
            ```
            {
              sender: "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
              receiver: "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
              amount: 1000000000000,
              metadata: {
                memo: "Payment for services",
                timestamp: 1234567890
              }
            }
            ```

            ## Common Substrate Types

            ### AccountId (SS58 addresses)
            AccountId in Substrate is typically a 32-byte array. You need to provide it as:
            - A hex string with 0x prefix: `"0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"`
            - Or decode the SS58 address to bytes and provide as a 32-element tuple

            ### Balance
            Use plain numbers (no quotes):
            ```
            1000000000000
            ```

            ### Option<T>
            Use variant syntax:
            ```
            None()              # Unit variant - parentheses required!
            Some(123)           # Standard syntax
            v"None"()           # Alternative v-prefix syntax
            v"Some"(123)        # v-prefix with value
            Some("value")
            v"Some"("value")    # Alternative syntax
            ```
            ⚠️ CRITICAL: Always include parentheses - `v"None"` without `()` will fail

            ## Real Examples

            ### Balance Transfer
            ```
            {
              dest: Id((142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72)),
              value: 1000000000000
            }
            ```
            Note: The `dest` field is a MultiAddress variant. For AccountId32, use `Id((bytes...))` with the 32-byte array representation of the SS58 address.

            ### System Remark
            ```
            {
              remark: (72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33)
            }
            ```
            Note: The `remark` field expects a `Vec<u8>`, so provide the string as a sequence of byte values.

            ### Complex Call (Escrow Creation)
            ```
            {
              seller: (142, 175, 4, 21, 22, 135, 115, 99, 38, 201, 254, 161, 126, 37, 252, 82, 135, 97, 54, 147, 201, 18, 144, 156, 178, 38, 170, 71, 148, 242, 106, 72),
              arbitrator: Some((144, 181, 171, 32, 92, 105, 116, 201, 234, 132, 27, 230, 136, 134, 70, 51, 220, 156, 168, 163, 87, 132, 62, 234, 207, 35, 20, 100, 153, 101, 254, 34)),
              amount: 1000000000000,
              deadline: 50,
              description: (80, 97, 121, 109, 101, 110, 116, 32, 102, 111, 114, 32, 100, 105, 103, 105, 116, 97, 108, 32, 97, 114, 116, 119, 111, 114, 107, 32, 78, 70, 84)
            }
            ```
            Note: AccountId32 fields use 32-byte arrays, Option<AccountId32> uses variant syntax `Some((bytes))` or `None()`, and Vec<u8> uses byte sequences.

            ### Escrow Creation with Option Field
            For a call that expects (seller, arbitrator, amount, deadline, description) where arbitrator is Option<AccountId32>:
            ```
            # Using standard syntax:
            ((144, 181, 171, 32, 92, 105, 116, 201, 234, 132, 27, 230, 136, 134, 70, 51, 220, 156, 168, 163, 87, 132, 62, 234, 207, 35, 20, 100, 153, 101, 254, 34), None(), 500000000000000, 50, (87, 101, 98, 32, 100, 101, 118, 101, 108, 111, 112, 109, 101, 110, 116, 32, 115, 101, 114, 118, 105, 99, 101, 115))

            # Using v-prefix syntax (also valid):
            ((144, 181, 171, 32, 92, 105, 116, 201, 234, 132, 27, 230, 136, 134, 70, 51, 220, 156, 168, 163, 87, 132, 62, 234, 207, 35, 20, 100, 153, 101, 254, 34), v"None"(), 500000000000000, 50, (87, 101, 98, 32, 100, 101, 118, 101, 108, 111, 112, 109, 101, 110, 116, 32, 115, 101, 114, 118, 105, 99, 101, 115))
            ```
            Both `None()` and `v"None"()` work - the key is including the parentheses!

            ## Tips

            1. Use `get_call_metadata` first to understand the expected argument structure
            2. Field names in composites are unquoted
            3. String values must be double-quoted
            4. Numbers are unquoted
            5. The parser will tell you exactly where parsing failed if there's an error
            6. When in doubt, use the standard syntax (without v-prefix) as it works for all variants
            7. The v-prefix syntax is an alternative way to write variants - it always requires parentheses after the variant name
            "#}.to_string(),
            priority: 0.95,
            audience: Some(vec![Role::Assistant]),
        }
    ]
}

fn https_resources() -> Vec<Resource> {
    vec![
        Resource::new(
            RawResource {
                uri: "https://docs.polkadot.com/llms.txt".to_string(),
                name: "Polkadot Documentation".to_string(),
                description: Some("Index with LLM friendly Polkadot documentation. Use this to get proper context on how Polkadot works or how to perform a specific within Polkadot or Substrate based chains (e.g: create a pallet, add benchmarks or work with XCM)".into()),
                mime_type: None,
                size: None,
            },
            Some(Annotations {
                audience: Some(vec![Role::Assistant]),
                priority: Some(0.95),
                last_modified: None,
            })
        ),
        Resource::new(
            RawResource {
                uri: "https://github.com/paritytech/polkadot-sdk".to_string(),
                name: "Polkadot GitHub Repository".to_string(),
                description: Some("Polkadot SDK GitHub Repository.".into()),
                mime_type: None,
                size: None,
            },
            Some(Annotations {
                audience: Some(vec![Role::Assistant, Role::User]),
                priority: Some(0.90),
                last_modified: None,
            })
        ),
        Resource::new(
            RawResource {
                uri: "https://docs.rs/crate/polkadot-sdk/latest".to_string(),
                name: "Rust crate Documentation".to_string(),
                description: Some("`polkadot-sdk` crate documentation".into()),
                mime_type: None,
                size: None,
            },
            Some(Annotations {
                audience: Some(vec![Role::Assistant, Role::User]),
                priority: Some(0.90),
                last_modified: None,
            })
        ),
    ]
}

/// Get all available resources
pub(crate) fn get_all_resources() -> Vec<Resource> {
    let mut resources: Vec<Resource> = markdown_resources().into_iter().map(|r| r.into()).collect();
    resources.extend(https_resources());
    resources
}

/// Get resource content by URI
pub(crate) async fn get_resource_content(uri: &str) -> Option<String> {
    // First check if it's a markdown resource
    if let Some(content) = markdown_resources()
        .into_iter()
        .find(|r| r.uri == uri)
        .map(|r| r.content)
    {
        return Some(content);
    }

    // NOTE: if the resource is exposed via https protocol, the client
    // should in theory fetch it on its own. However this is not the case
    // many times in practice so we fall back to fetching it and forwarding
    // the content
    if uri.starts_with("https://") {
        match reqwest::get(uri).await {
            Ok(response) => response.text().await.ok(),
            Err(_) => None,
        }
    } else {
        None
    }
}
