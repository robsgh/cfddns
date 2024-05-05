# cfddns - Dynamic DNS for Cloudflare

A tool for updating Cloudflare DNS A records for a particular zone to the public IP of the executing host, like Dynamic DNS.

## Usage

`./cfddns`

## Installation

To install cfddns:

1. `cargo build -r`
2. `sudo cp ./target/release/cfddns /usr/bin/cfddns`
3. `cp ./config/example.json ./config/config.json`
4. Update the placeholder values in the config at `./config/config.json` as directed
5. `sudo mkdir /etc/cfddns && sudo cp ./config/config.json /etc/cfddns/config.json && sudo chmod 600 /etc/cfddns/config.json`
6. `sudo cp ./systemd/* /etc/systemd/system/`
7. `sudo systemctl daemon-reload && sudo systemctl enable --now cfddns.timer`
