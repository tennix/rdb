use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub listen_addr: SocketAddr,
    pub max_connections: usize,
    pub buffer_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    pub max_memory: usize,
    pub persistence_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: ServerConfig {
                listen_addr: "127.0.0.1:6379".parse().unwrap(),
                max_connections: 1000,
                buffer_size: 1024,
            },
            storage: StorageConfig {
                max_memory: 1024 * 1024 * 1024, // 1GB
                persistence_enabled: false,
            },
        }
    }
}

pub fn load_config() -> Result<Config, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("config").required(false))
        .add_source(config::Environment::with_prefix("RDB"))
        .build()?
        .try_deserialize()
}
