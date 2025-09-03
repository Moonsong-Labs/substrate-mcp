use super::helpers;

use helpers::substrate_runner::SubstrateRunner;
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::dev;

#[tokio::test]
async fn test_spawn_substrate_node_and_submit_extrinsic() {
    // Spawn a substrate node
    let mut runner = match SubstrateRunner::spawn() {
        Ok(runner) => runner,
        Err(e) => {
            eprintln!("Failed to spawn substrate node: {}", e);
            eprintln!("Make sure substrate-node is installed and in PATH");
            eprintln!(
                "You can install it with: cargo install --git https://github.com/paritytech/substrate substrate-node"
            );
            return;
        }
    };

    // Get the WebSocket URL
    let ws_url = runner.ws_url();
    // Extract and print the port
    let port = runner.ws_port;

    println!("Substrate node started successfully!");
    println!("WebSocket URL: {}", ws_url);
    println!("Port: {}", port);

    // Wait a bit for the node to fully initialize
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Initialize subxt client
    let api = OnlineClient::<PolkadotConfig>::from_url(&ws_url)
        .await
        .expect("Failed to connect to substrate node");

    println!("Connected to substrate node with subxt client");

    // Create a balance transfer extrinsic using the dynamic API
    let alice = dev::alice();
    let bob = dev::bob();

    // Build the dynamic extrinsic
    let tx = subxt::dynamic::tx(
        "Balances",
        "transfer_allow_death",
        vec![
            // Destination (Bob's address)
            subxt::dynamic::Value::unnamed_variant("Id", vec![bob.public_key().0.to_vec().into()]),
            // Amount (1_000_000_000_000 units)
            1_000_000_000_000u128.into(),
        ],
    );

    // Sign and submit the extrinsic, then wait for it to be finalized
    let mut tx_progress = api
        .tx()
        .sign_and_submit_then_watch_default(&tx, &alice)
        .await
        .expect("Failed to submit transaction");

    println!("Transaction submitted successfully!");
    println!("Transaction hash: {:?}", tx_progress.extrinsic_hash());

    // Wait for transaction to be in a block
    while let Some(status) = tx_progress.next().await {
        match status {
            Ok(event) => {
                println!("Transaction status: {:?}", event);

                // Check if transaction is in a block
                if let Some(block) = event.as_in_block() {
                    println!("Transaction in block!");

                    // Check for success by looking at events
                    let events = block.fetch_events().await.expect("Failed to fetch events");

                    // Check if ExtrinsicSuccess event exists
                    let success_event = events.iter().any(|event| {
                        event
                            .as_ref()
                            .map(|e| {
                                e.pallet_name() == "System"
                                    && e.variant_name() == "ExtrinsicSuccess"
                            })
                            .unwrap_or(false)
                    });

                    assert!(
                        success_event,
                        "Transaction did not succeed - no ExtrinsicSuccess event found"
                    );
                    println!("Transaction executed successfully with ExtrinsicSuccess event!");
                    break;
                }

                // Check if transaction is finalized
                if let Some(block) = event.as_finalized() {
                    println!("Transaction finalized!");

                    // Check for success by looking at events
                    let events = block.fetch_events().await.expect("Failed to fetch events");

                    // Check if ExtrinsicSuccess event exists
                    let success_event = events.iter().any(|event| {
                        event
                            .as_ref()
                            .map(|e| {
                                e.pallet_name() == "System"
                                    && e.variant_name() == "ExtrinsicSuccess"
                            })
                            .unwrap_or(false)
                    });

                    assert!(
                        success_event,
                        "Transaction did not succeed - no ExtrinsicSuccess event found"
                    );
                    println!("Transaction executed successfully with ExtrinsicSuccess event!");
                    break;
                }
            }
            Err(e) => {
                panic!("Transaction failed with error: {:?}", e);
            }
        }
    }

    // Clean shutdown
    runner.kill().expect("Failed to kill substrate node");
}

