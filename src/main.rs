use anyhow::Result;
use rmcp::service::ServiceExt;
use tokio::io::{stdin, stdout};

mod server;
use server::SubstrateService;

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Starting basic MCP server...");

    let transport = (stdin(), stdout());

    let service = SubstrateService::new()
        .serve(transport)
        .await
        .inspect_err(|e| println!("{e}"))?;

    service.waiting().await?;

    Ok(())
}
