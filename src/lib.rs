use std::net::Ipv4Addr;

use anyhow::{anyhow, Result};
use cloudflare::{
    endpoints::dns::{
        DnsContent, DnsRecord, ListDnsRecords, ListDnsRecordsParams, UpdateDnsRecord,
        UpdateDnsRecordParams,
    },
    framework::async_api::Client,
};
use tracing::{debug, error, info, instrument, warn};

/// Request the records from cloudflare
async fn request_records(client: &Client, zone_id: &str) -> Result<Vec<DnsRecord>> {
    let results: Vec<DnsRecord> = client
        .request(&ListDnsRecords {
            zone_identifier: zone_id,
            params: ListDnsRecordsParams::default(),
        })
        .await?
        .result;

    debug!(
        record.names = ?results
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<_>>(),
        "fetched {} records from cloudflare", results.len()
    );

    Ok(results)
}

/// Find a matching DNS record with `name` from a vec of DnsRecords
fn find_matching_dns_record(records: Vec<DnsRecord>, name: &str) -> Option<DnsRecord> {
    if let Some(record) = records.into_iter().find(|r| r.name == name) {
        debug!(
            record.name,
            ?record.created_on,
            ?record.modified_on,
            record.zone_name,
            "found matching DNS record"
        );

        if let DnsContent::A { content: _ } = record.content {
            return Some(record);
        } else {
            warn!(
                record_content = ?record.content,
                "found a matching DNS record that cannot be used because of its type",
            );
        }
    }

    None
}

/// Fetch a DNS record using the Cloudflare API
#[instrument(skip(client))]
pub async fn fetch_cloudflare_dns_record(
    client: &Client,
    zone_id: &str,
    name: &str,
) -> Result<DnsRecord> {
    let dns_records = request_records(client, zone_id).await?;

    match find_matching_dns_record(dns_records, name) {
        Some(record) => Ok(record),
        None => {
            error!(
                "no matching DNS A record with name {:?} was found for zone {:?}",
                name, zone_id
            );
            anyhow::bail!("failed to fetch DNS record from cloudflare")
        }
    }
}

/// Update the cloudflare DNS record with a new IP
#[instrument(skip(client, new_ip))]
pub async fn update_cloudflare_dns_record(
    client: &Client,
    record: DnsRecord,
    new_ip: Ipv4Addr,
) -> anyhow::Result<()> {
    let dns_req = UpdateDnsRecord {
        zone_identifier: &record.zone_id,
        identifier: &record.id,
        params: UpdateDnsRecordParams {
            name: &record.name,
            content: DnsContent::A { content: new_ip },
            proxied: Some(false),
            ttl: Some(1),
        },
    };

    let resp = client.request(&dns_req).await?.result;
    if let DnsContent::A { content } = resp.content {
        if content == new_ip {
            info!(cloudflare_ip = ?content, "updated cloudflare DNS");
            return Ok(());
        } else {
            error!("DNS update request succeeded but the record has not been updated");
            anyhow::bail!("DNS record was not updated");
        }
    }

    error!(
        response_record = ?resp,
        "failed to process cloudflare DNS record"
    );
    anyhow::bail!("failed to process returned cloudflare DNS record");
}

/// Fetch an IP by querying the API endpoint at ipify.org
#[instrument]
pub async fn fetch_current_ip() -> Result<Ipv4Addr> {
    let ip = reqwest::get("https://api.ipify.org")
        .await?
        .text()
        .await?
        .parse()
        .map_err(|e| anyhow!("failed to parse IP from ipify: {e:?}"))?;

    info!(current_ip = ?ip, "fetched current public IP");
    Ok(ip)
}
