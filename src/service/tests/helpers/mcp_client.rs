use anyhow::Result;
use rmcp::model::{CallToolRequestParam, CallToolResult};
use rmcp::service::{RoleClient, RunningService, ServiceExt};
use std::borrow::Cow;

use crate::service::SubstrateService;

/// Helper struct for testing MCP interactions
pub(crate) struct TestMcpClient {
    client: RunningService<RoleClient, ()>,
    _server_handle: tokio::task::JoinHandle<()>,
}

impl TestMcpClient {
    /// Create a new MCP test client connected to a SubstrateService
    pub(crate) async fn new() -> Result<Self> {
        // Create a bidirectional stream for client-server communication
        let (client_stream, server_stream) = tokio::io::duplex(1024 * 64);

        // Create the server
        let service = SubstrateService::new();

        // Spawn the server in a background task
        let server_handle = tokio::spawn(async move {
            let server = service
                .serve(server_stream)
                .await
                .expect("Failed to start server");
            server.waiting().await.expect("Server error");
        });

        // Create the client
        let client = ().serve(client_stream).await?;

        Ok(Self {
            client,
            _server_handle: server_handle,
        })
    }

    /// Call a tool
    pub(crate) async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult> {
        let args = arguments.as_object().cloned();

        let request = CallToolRequestParam {
            name: Cow::from(name.to_string()),
            arguments: args,
            task: None,
        };
        Ok(self.client.call_tool(request).await?)
    }
}
