use anyhow::Result;
use rmcp::service::ServiceExt;
use tokio::io::{stdin, stdout};

mod polkadot_sdk_releases;
mod prompts;
mod server;
mod substrate;

use server::SubstrateService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    log::info!("Starting substrate MCP server...");

    let transport = (stdin(), stdout());

    let service = SubstrateService::new()
        .serve(transport)
        .await
        .inspect_err(|e| log::error!("Service error: {e}"))?;

    service.waiting().await?;

    Ok(())
}
