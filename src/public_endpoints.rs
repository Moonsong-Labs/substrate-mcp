/// Public RPC endpoints for various Substrate-based chains
#[allow(dead_code)]
pub mod endpoints {
    /// Polkadot mainnet
    pub const POLKADOT: &str = "wss://rpc.polkadot.io";

    /// Kusama canary network
    pub const KUSAMA: &str = "wss://kusama-rpc.polkadot.io";

    /// Westend testnet
    pub const WESTEND: &str = "wss://westend-rpc.polkadot.io";

    /// Rococo testnet
    pub const ROCOCO: &str = "wss://rococo-rpc.polkadot.io";

    /// Paseo testnet (replaced Rococo for parachains)
    pub const PASEO: &str = "wss://paseo.rpc.amforc.com";

    /// Asset Hub on Polkadot
    pub const ASSET_HUB_POLKADOT: &str = "wss://polkadot-asset-hub-rpc.polkadot.io";

    /// Asset Hub on Kusama
    pub const ASSET_HUB_KUSAMA: &str = "wss://kusama-asset-hub-rpc.polkadot.io";

    /// Asset Hub on Westend
    pub const ASSET_HUB_WESTEND: &str = "wss://westend-asset-hub-rpc.polkadot.io";

    /// Default endpoint (Westend testnet is a good default for testing)
    pub const DEFAULT: &str = WESTEND;
}
