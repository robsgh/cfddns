use anyhow::{Context, Result};
use std::{fmt::Display, fs, path::PathBuf};

use tracing::info;

/// Configuration params for cfddns
#[derive(Clone, Debug)]
pub struct CfddnsConfig {
    /// Cloudflare API token
    pub api_token: String,
    /// Zone ID of the DNS zone to modify
    pub zone_id: String,
    /// Name of the DNS record in the DNS zone
    pub record_name: String,
}

impl Display for CfddnsConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // don't leak API token on Display
        write!(
            f,
            "{{zone_id={}, record_name={}, api_token=*****}}",
            self.zone_id, self.record_name
        )
    }
}

impl CfddnsConfig {
    pub fn new(config_path: PathBuf) -> Result<Self> {
        info!(
            "loading configuration file from {}",
            config_path.to_string_lossy()
        );

        let config_data = fs::read_to_string(config_path.as_path()).with_context(|| {
            format!(
                "error reading configuration file {:?}",
                config_path.to_string_lossy()
            )
        })?;
        let mut config_data = json::parse(&config_data).with_context(|| {
            format!(
                "failed to parse {:?} as JSON",
                config_path.to_string_lossy()
            )
        })?;

        let mut config = Self {
            api_token: String::new(),
            zone_id: String::new(),
            record_name: String::new(),
        };

        if let Some(api_token) = config_data["api_token"].take_string() {
            config.api_token = api_token;
        } else {
            anyhow::bail!("api_token is not set in configuration file");
        }
        if let Some(zone_id) = config_data["zone_id"].take_string() {
            config.zone_id = zone_id;
        } else {
            anyhow::bail!("zone_id is not set in configuration file");
        }
        if let Some(record_name) = config_data["record_name"].take_string() {
            config.record_name = record_name;
        } else {
            anyhow::bail!("record_name is not set in configuration file");
        }

        info!("parsed configuration file: {}", config);
        Ok(config)
    }
}
