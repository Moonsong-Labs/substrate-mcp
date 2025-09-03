mod helpers;

use helpers::SubstrateRunner;

#[test]
fn test_spawn_substrate_node_and_print_port() {
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
    let port = ws_url
        .rsplit(':')
        .next()
        .expect("Failed to extract port from URL");

    println!("Substrate node started successfully!");
    println!("WebSocket URL: {}", ws_url);
    println!("Port: {}", port);

    // Clean shutdown
    runner.kill().expect("Failed to kill substrate node");
}
