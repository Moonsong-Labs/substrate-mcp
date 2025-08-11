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
pub struct Config {
    pub endpoints: Vec<RpcEndpoint>,
    pub default_endpoint: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            endpoints: vec![
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
            default_endpoint: "polkadot".to_string(),
        }
    }
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: Config =
            serde_json::from_str(&content).with_context(|| "Failed to parse config file")?;

        config.validate()?;
        Ok(config)
    }

    #[cfg(test)]
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(path.as_ref(), content)
            .with_context(|| format!("Failed to write config file: {}", path.as_ref().display()))?;

        Ok(())
    }

    pub fn get_endpoint(&self, name: &str) -> Option<&RpcEndpoint> {
        self.endpoints.iter().find(|e| e.name == name)
    }

    pub fn get_endpoint_url(&self, name: &str) -> Option<&str> {
        self.get_endpoint(name).map(|e| e.url.as_str())
    }

    pub fn get_default_endpoint(&self) -> Option<&RpcEndpoint> {
        self.get_endpoint(&self.default_endpoint)
    }

    pub fn get_default_url(&self) -> Option<&str> {
        self.get_default_endpoint().map(|e| e.url.as_str())
    }

    fn validate(&self) -> Result<()> {
        if self.endpoints.is_empty() {
            anyhow::bail!("Config must contain at least one endpoint");
        }

        if !self
            .endpoints
            .iter()
            .any(|e| e.name == self.default_endpoint)
        {
            anyhow::bail!(
                "Default endpoint '{}' not found in endpoints list",
                self.default_endpoint
            );
        }

        let names: Vec<_> = self.endpoints.iter().map(|e| &e.name).collect();
        let unique_names: std::collections::HashSet<_> = names.iter().collect();
        if names.len() != unique_names.len() {
            anyhow::bail!("Endpoint names must be unique");
        }

        for endpoint in &self.endpoints {
            if endpoint.name.is_empty() {
                anyhow::bail!("Endpoint name cannot be empty");
            }
            if endpoint.url.is_empty() {
                anyhow::bail!("Endpoint URL cannot be empty");
            }
            if !endpoint.url.starts_with("ws://") && !endpoint.url.starts_with("wss://") {
                anyhow::bail!(
                    "Endpoint URL '{}' must start with ws:// or wss://",
                    endpoint.url
                );
            }
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub fn chain_name_from_endpoint(endpoint: &str, config: &Config) -> String {
    if let Some(rpc_endpoint) = config.endpoints.iter().find(|e| e.url == endpoint) {
        return rpc_endpoint.description.clone();
    }

    match endpoint {
        e if e.contains("polkadot") && !e.contains("kusama") && !e.contains("westend") => {
            "Polkadot".to_string()
        }
        e if e.contains("kusama") => "Kusama".to_string(),
        e if e.contains("westend") => "Westend".to_string(),
        e if e.contains("rococo") => "Rococo".to_string(),
        e if e.contains("paseo") => "Paseo".to_string(),
        e if e.contains("asset-hub") && e.contains("polkadot") => "Asset Hub Polkadot".to_string(),
        e if e.contains("asset-hub") && e.contains("kusama") => "Asset Hub Kusama".to_string(),
        e if e.contains("asset-hub") && e.contains("westend") => "Asset Hub Westend".to_string(),
        _ => "Custom Chain".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_load_existing_config() {
        // Test loading the actual config file
        let config = Config::load_from_file("rpc_endpoints.json");
        assert!(config.is_ok(), "Should be able to load rpc_endpoints.json");

        let config = config.unwrap();
        assert!(!config.endpoints.is_empty());
        assert!(!config.default_endpoint.is_empty());
        assert!(config.get_endpoint(&config.default_endpoint).is_some());
    }

    #[test]
    fn test_config_validation() {
        // Create a test config manually
        let mut config = Config {
            endpoints: vec![],
            default_endpoint: "test".to_string(),
        };

        // Test empty endpoints
        assert!(config.validate().is_err());

        // Test invalid default endpoint
        config.endpoints = vec![RpcEndpoint {
            name: "test".to_string(),
            url: "wss://test.com".to_string(),
            description: "Test".to_string(),
        }];
        config.default_endpoint = "nonexistent".to_string();
        assert!(config.validate().is_err());

        // Test duplicate names
        config.endpoints.push(RpcEndpoint {
            name: "test".to_string(),
            url: "wss://test2.com".to_string(),
            description: "Test 2".to_string(),
        });
        config.default_endpoint = "test".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_save_and_load() {
        let temp_file = PathBuf::from("test_config_temp.json");
        let config = Config {
            endpoints: vec![RpcEndpoint {
                name: "test".to_string(),
                url: "wss://test.com".to_string(),
                description: "Test endpoint".to_string(),
            }],
            default_endpoint: "test".to_string(),
        };

        // Save config
        assert!(config.save_to_file(&temp_file).is_ok());

        // Load config
        let loaded = Config::load_from_file(&temp_file).unwrap();
        assert_eq!(loaded.endpoints.len(), config.endpoints.len());
        assert_eq!(loaded.default_endpoint, config.default_endpoint);
        assert_eq!(loaded.endpoints[0].name, config.endpoints[0].name);

        // Clean up
        fs::remove_file(temp_file).ok();
    }
}
