# cfddns - Dynamic DNS for Cloudflare

A tool for updating Cloudflare DNS A records for a particular zone to the public IP of the executing host, like Dynamic DNS.

## Usage

`./cfddns --api-token <api token> [DNS ZONE ID] [DNS A RECORD NAME]`

Alternatively, the API token can be specified with the `API_TOKEN` environment variable instead of as an argument.
