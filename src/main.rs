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
use tracing::warn;
use tracing_log::AsTrace;

/// Fetch host's public IP and update a cloudflare domain's DNS A record
/// as a pseudo-dynamic DNS system
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Cloudflare API token
    #[arg(short, long, env)]
    api_token: String,

    /// Cloudflare DNS Zone ID
    zone_id: String,

    /// DNS record name to update
    record_name: String,

    /// Log level verbosity
    #[command(flatten)]
    verbosity: Verbosity<InfoLevel>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // setup logging
    init_tracing(args.verbosity);

    // create cloudflare API client
    let credentials = Credentials::UserAuthToken {
        token: args.api_token,
    };
    let cf_client = Client::new(
        credentials,
        HttpApiClientConfig::default(),
        Environment::Production,
    )
    .map_err(|e| anyhow::anyhow!("failed to create cloudflare client: {e:?}"))?;

    // fetch both IPs concurrently
    let (cf_dns_record, current_ip) = tokio::join!(
        fetch_cloudflare_dns_record(&cf_client, &args.zone_id, &args.record_name),
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
        info!("Nothing to do: cloudflare IP is the same as current IP");
        return Ok(());
    } else {
        warn!("Updating cloudflare IP: cloudflare IP is not the same as the current IP");
        update_cloudflare_dns_record(&cf_client, cf_dns_record, current_ip)
            .await
            .map_err(|e| anyhow::anyhow!("failed to update DNS record: {e:?}"))
    }
}

/// Setup tracing to log with a level filter
fn init_tracing(verbosity: Verbosity<InfoLevel>) {
    tracing_subscriber::fmt()
        .with_max_level(verbosity.log_level_filter().as_trace())
        .with_target(false)
        .init();
}
