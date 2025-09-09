use std::io::{self, BufRead, BufReader};
use std::process::{Child, ChildStderr, Command, Stdio};

// Substrate node runner
// Simplified version of: https://github.com/paritytech/subxt/blob/master/testing/substrate-runner/src/lib.rs
pub(crate) struct SubstrateRunner {
    pub(crate) proc: Child,
    pub(crate) ws_port: u16,
}

impl SubstrateRunner {
    /// Spawn a substrate-node process with dynamic port discovery from logs
    pub(crate) fn spawn() -> Result<Self, io::Error> {
        // Spawn the substrate-node process with OS-assigned ports
        let mut proc = Command::new("substrate-node")
            .arg("--dev")
            .arg("--tmp")
            .arg("--port=0")
            .arg("--rpc-port=0")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()) // Capture logs for port discovery
            .spawn()?;

        // Parse logs to find the actual port
        let stderr = proc.stderr.take().unwrap();
        let ws_port = find_port_from_logs(stderr)?;

        Ok(SubstrateRunner { proc, ws_port })
    }

    /// Get the WebSocket URL for connecting to the node
    pub(crate) fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.ws_port)
    }
}

impl Drop for SubstrateRunner {
    fn drop(&mut self) {
        // Clean shutdown: kill the process
        let _ = self.proc.kill();
    }
}

/// Parse substrate node logs to find the RPC port
fn find_port_from_logs(stderr: ChildStderr) -> io::Result<u16> {
    for line_result in BufReader::new(stderr).lines().take(50) {
        let line = line_result?;

        // Look for RPC server startup messages
        if let Some(port_str) = line
            .split_once("Running JSON-RPC server: addr=127.0.0.1:")
            .or_else(|| line.split_once("Running JSON-RPC WS server: addr=127.0.0.1:"))
            .map(|(_, port_part)| port_part)
        {
            // Extract just the numeric port part
            let port_str: String = port_str
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();

            if let Ok(port) = port_str.parse::<u16>() {
                return Ok(port);
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "Could not find RPC port in node logs",
    ))
}
