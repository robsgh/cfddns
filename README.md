# cfddns - Dynamic DNS for Cloudflare

A tool for updating Cloudflare DNS A records for a particular zone to the public IP of the executing host, like Dynamic DNS.

## Usage

`./cfddns --api-token <api token> [DNS ZONE ID] [DNS A RECORD NAME]`

Alternatively, the API token can be specified with the `API_TOKEN` environment variable instead of as an argument.

## Installation

To install cfddns:

1. `cargo build -r`
2. `cp ./target/release/cfddns /usr/bin/cfddns`
3. ```mkdir /etc/cfddns && echo "API_TOKEN=<put your token here" >/etc/cfddns/config && chmod 600 /etc/cfddns/config```
4. `cp ./systemd/* /etc/systemd/system/`
5. `sudo systemctl daemon-reload && sudo systemctl enable --now cfddns.timer`
