# ftsov2mon
Flare FTSOv2 Monitoring tool.

The all in one tool to monitor your Flare FTSOv2.

ftsov2mon can:
- Expose metrics for Prometheus
- Send alerts to your Telegram group
    - Get an alert when an address is not signing anymore
    - Get an alert when an address balance is low
- Send alerts to your slack (soon)
    - Same as Telegram

## Installation
Run the following to install `ftsov2mon` and edit the created service and start it:
```bash
curl https://raw.githubusercontent.com/Luganodes/ftsov2mon/main/install.sh | sudo bash
sudo systemctl start ftsov2mon.service
```

## Commands and Flags
### `start`
To start the exporter with default flags
```bash
ftsov2mon start
```
Flags:
| Name | Required? | Default | Description |
| ----------- | ----------- | ----------- | ----------- |
| `--tg-api-key` | NO | NONE | This is the TG bot's API key. |
| `--tg-chat-id` | NO | NONE | This is the TG channel's ID. |
| `--metrics-port` | NO | 6969 | The port on which the metrics server should serve metrics. |
| `--metrics-addr` | NO | 0.0.0.0 | The address on which the metrics server should serve metrics. |
| `--rpc-url` | YES | NONE | The RPC URL to scrape metrics from. Change this to scrape Mainnet metrics. |
| `--block-window` | YES | 100 | The number of blocks from now in the past to monitor. |
| `--submit-address` | YES | NONE | The FTSO Submit Address |
| `--submit-signature-address` | YES | NONE | The FTSO Submit Signature Address |
| `--signing-policy-address` | YES | NONE | The FTSO Signing Policy Address |

## Metrics Served
With default flags, the following will be shown after
```bash
curl localhost:6969/metrics
```

Output format:
```
# Did the client register for this reward epoch?
ftso_registered_for_this_epoch

# The latest block from the RPC
ftso_rpc_current_block

# Is the RPC syncing?
ftso_rpc_is_syncing

# The balance of the signing policy address
ftso_signing_policy_balance

# Was a tx from the signing policy address found within the block window?
ftso_signing_policy_tx_found

# The balance of the submit address
ftso_submit_balance

# The balance of the submit signature address
ftso_submit_signature_balance

# Was a tx from the submit signature address found within the block window?
ftso_submit_signature_tx_found

# Was a tx from the submit address found within the block window?
ftso_submit_tx_found

# The ftso block window
ftso_search_window
```

## Todo
- [x] Add support for telegram notifications
- [ ] Add the following metrics:
    - [x] RPC syncing
    - [x] RPC latest block
    - [x] Reporting balance in metrics
    - [ ] Appropriate txs found for all given addresses in the given block window
    - [x] The given block window for searching
    - [ ] Did the client register for this epoch?
- [x] Create setup script for easy download and systemd service setup
