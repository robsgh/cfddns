use anyhow::Context;
use cfddns::update_cloudflare_dns_record;
use clap::Parser;

use cfddns::fetch_cloudflare_dns_record;
use cfddns::fetch_current_ip;
use clap_verbosity_flag::InfoLevel;
use clap_verbosity_flag::Verbosity;
use cloudflare::endpoints::dns::DnsRecord;
use cloudflare::framework::{
    async_api::Client, auth::Credentials, Environment, HttpApiClientConfig,
};
use tracing::info;
use tracing_log::AsTrace;

use cfddns::config::CfddnsConfig;

/// Default path to the cfddns configuration file
const DEFAULT_CFDDNS_CONFIG_PATH: &str = "/etc/cfddns/config.json";

/// Fetch host's public IP and update a cloudflare domain's DNS A record
/// as a pseudo-dynamic DNS system
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the cfddns configuration file
    #[arg(short, long, default_value = DEFAULT_CFDDNS_CONFIG_PATH)]
    config_path: String,
    /// Log level verbosity
    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,
}

/// Create an API client for Cloudflare
fn get_cloudflare_client(config: &CfddnsConfig) -> anyhow::Result<Client> {
    let credentials = Credentials::UserAuthToken {
        token: config.api_token.to_owned(),
    };

    Client::new(
        credentials,
        HttpApiClientConfig::default(),
        Environment::Production,
    )
    .context("failed to create cloudflare client")
}

/// Setup tracing to log with a level filter
fn init_tracing(verbosity: Verbosity<InfoLevel>) {
    tracing_subscriber::fmt()
        .with_max_level(verbosity.log_level_filter().as_trace())
        .with_target(false)
        .init();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    init_tracing(args.verbosity);

    let config =
        CfddnsConfig::new(args.config_path.into()).context("failed to load configuration file")?;

    let cf_client = get_cloudflare_client(&config)?;

    // fetch both IPs concurrently
    let (cf_dns_record, current_ip) = tokio::join!(
        fetch_cloudflare_dns_record(&cf_client, &config),
        fetch_current_ip()
    );

    // unwrap the types
    let current_ip = current_ip.context("failed to fetch current IP")?;
    let cf_dns_record: DnsRecord =
        cf_dns_record.context("failed to fetch cloudflare DNS record")?;
    let cf_ip = match cf_dns_record.content {
        cloudflare::endpoints::dns::DnsContent::A { content } => content,
        _ => {
            anyhow::bail!("no matching DNS A record was found");
        }
    };
    info!(
        ip.cloudflare = ?cf_ip,
        ip.current = ?current_ip,
        "fetched current IP and cloudflare IP"
    );

    // compare IPs and update if needed
    if current_ip == cf_ip {
        info!("nothing to do: cloudflare IP is the same as current IP");
        return Ok(());
    }

    info!("updating cloudflare IP: cloudflare IP is not the same as the current IP");

    match update_cloudflare_dns_record(&cf_client, cf_dns_record, current_ip).await {
        Ok(_) => {
            info!("cloudflare IP has been updated to {current_ip:?}");
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("failed to update cloudflare DNS IP: {e:?}")
        }
    }
}
