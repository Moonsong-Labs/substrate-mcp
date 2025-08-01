use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEndpoint {
    pub name: String,
    pub url: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub endpoints: Vec<RpcEndpoint>,
}

impl RpcConfig {
    /// Load RPC endpoints from a TOML configuration file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: RpcConfig = toml::from_str(&content)
            .with_context(|| "Failed to parse RPC endpoints configuration")?;

        Ok(config)
    }

    /// Get an endpoint by name
    pub fn get_endpoint(&self, name: &str) -> Option<&RpcEndpoint> {
        self.endpoints.iter().find(|e| e.name == name)
    }

    /// Get the URL for an endpoint by name
    pub fn get_url(&self, name: &str) -> Option<&str> {
        self.get_endpoint(name).map(|e| e.url.as_str())
    }

    /// Get default config with common endpoints
    pub fn default() -> Self {
        Self {
            endpoints: vec![
                RpcEndpoint {
                    name: "local".to_string(),
                    url: "http://127.0.0.1:9944".to_string(),
                    description: "Local development node".to_string(),
                },
                RpcEndpoint {
                    name: "polkadot".to_string(),
                    url: "wss://rpc.polkadot.io".to_string(),
                    description: "Polkadot mainnet".to_string(),
                },
                RpcEndpoint {
                    name: "kusama".to_string(),
                    url: "wss://kusama-rpc.polkadot.io".to_string(),
                    description: "Kusama network".to_string(),
                },
                RpcEndpoint {
                    name: "westend".to_string(),
                    url: "wss://westend-rpc.polkadot.io".to_string(),
                    description: "Westend testnet".to_string(),
                },
            ],
        }
    }

    /// Load config from default location or return default if not found
    pub fn load() -> Result<Self> {
        let config_path = "rpc_endpoints.toml";
        if Path::new(config_path).exists() {
            Self::from_file(config_path)
        } else {
            Ok(Self::default())
        }
    }
}

/// Get a user-friendly list of available endpoints
pub fn format_endpoint_list(config: &RpcConfig) -> String {
    let mut output = String::from("Available RPC endpoints:\n");
    for endpoint in &config.endpoints {
        output.push_str(&format!(
            "  - {} ({}): {}\n",
            endpoint.name, endpoint.url, endpoint.description
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RpcConfig::default();
        assert!(!config.endpoints.is_empty());
        assert!(config.get_endpoint("local").is_some());
        assert_eq!(config.get_url("local"), Some("http://127.0.0.1:9944"));
    }

    #[test]
    fn test_endpoint_lookup() {
        let config = RpcConfig::default();
        let polkadot = config.get_endpoint("polkadot");
        assert!(polkadot.is_some());
        assert_eq!(polkadot.unwrap().name, "polkadot");
    }
}
