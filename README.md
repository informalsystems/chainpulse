
# Chain Pulse

Collect packets relayed to and from a given blockchain, computing which packets are effected or not and by whom they were relayed.

The collected data is stored in a SQLite database and the metrics are exported to Prometheus.

## Installation

1. Clone this repository
   ```shell
   $ git clone https://github.com/informalsystems/chainpulse
   ```

2. Build the `chainpulse` executable
   ```shell
   $ cargo build --release
   ```

3. The `chainpulse` executable can now be found in `target/release`

## Usage

```
Collect and analyze txs containing IBC messages, export metrics for Prometheus

Usage: chainpulse [OPTIONS]

Options:
      --ws <WS_URL>     Tendermint WebSocket URL [default: wss://rpc.osmosis.zone/websocket]
      --db <DB_PATH>    Path to the SQLite database file, will be created if not existing [default: osmosis.db]
      --metrics <PORT>  Port on which to serve the Prometheus metrics, at `http://0.0.0.0:PORT/metrics`.
                        If not set, then the metrics won't be served
  -h, --help            Print help
```

To collect metrics for Osmosis and serve those at `http://localhost:3000/metrics`, run the following command:

```
$ chainpulse -- --ws wss://rpc.osmosis.zone/websocket --db osmosis.db --metrics 3000
```

## Metrics

- `ibc_effected_packets{chain_id, src_channel, src_port, dst_channel, dst_port, signer, memo}`
- `ibc_uneffected_packets{chain_id, src_channel, src_port, dst_channel, dst_port, signer, memo}`
- `ibc_frontrun_counter{chain_id, src_channel, src_port, dst_channel, dst_port, signer, frontrunned_by, memo, effected_memo}`

## Attribution

This project is heavily inspired and partly ported from @clemensgg's [relayer-metrics-exporter][clemensgg-metrics]

## License

Copyright © 2023 Informal Systems Inc. and Hermes authors.

Licensed under the Apache License, Version 2.0 (the "License"); you may not use the files in this repository except in compliance with the License. You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.


[clemensgg-metrics]: https://github.com/clemensgg/relayer-metrics-exporter
