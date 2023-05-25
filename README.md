[![Cosmos ecosystem][cosmos-shield]][cosmos-link]

[![Crates.io][crates-image]][crates-link]
[![Build Status][build-image]][build-link]
[![Apache 2.0 Licensed][license-image]][license-link]
![Rust Stable][rustc-image]
![Rust 1.69+][rustc-version]

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
$ chainpulse --ws wss://rpc.osmosis.zone/websocket --db osmosis.db --metrics 3000
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


[cosmos-shield]: https://img.shields.io/static/v1?label=&labelColor=1B1E36&color=1B1E36&message=cosmos%20ecosystem&style=for-the-badge&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDI0LjMuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPgo8c3ZnIHZlcnNpb249IjEuMSIgaWQ9IkxheWVyXzEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeG1sbnM6eGxpbms9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxpbmsiIHg9IjBweCIgeT0iMHB4IgoJIHZpZXdCb3g9IjAgMCAyNTAwIDI1MDAiIHN0eWxlPSJlbmFibGUtYmFja2dyb3VuZDpuZXcgMCAwIDI1MDAgMjUwMDsiIHhtbDpzcGFjZT0icHJlc2VydmUiPgo8c3R5bGUgdHlwZT0idGV4dC9jc3MiPgoJLnN0MHtmaWxsOiM2RjczOTA7fQoJLnN0MXtmaWxsOiNCN0I5Qzg7fQo8L3N0eWxlPgo8cGF0aCBjbGFzcz0ic3QwIiBkPSJNMTI1Mi42LDE1OS41Yy0xMzQuOSwwLTI0NC4zLDQ4OS40LTI0NC4zLDEwOTMuMXMxMDkuNCwxMDkzLjEsMjQ0LjMsMTA5My4xczI0NC4zLTQ4OS40LDI0NC4zLTEwOTMuMQoJUzEzODcuNSwxNTkuNSwxMjUyLjYsMTU5LjV6IE0xMjY5LjQsMjI4NGMtMTUuNCwyMC42LTMwLjksNS4xLTMwLjksNS4xYy02Mi4xLTcyLTkzLjItMjA1LjgtOTMuMi0yMDUuOAoJYy0xMDguNy0zNDkuOC04Mi44LTExMDAuOC04Mi44LTExMDAuOGM1MS4xLTU5Ni4yLDE0NC03MzcuMSwxNzUuNi03NjguNGM2LjctNi42LDE3LjEtNy40LDI0LjctMmM0NS45LDMyLjUsODQuNCwxNjguNSw4NC40LDE2OC41CgljMTEzLjYsNDIxLjgsMTAzLjMsODE3LjksMTAzLjMsODE3LjljMTAuMywzNDQuNy01Ni45LDczMC41LTU2LjksNzMwLjVDMTM0MS45LDIyMjIuMiwxMjY5LjQsMjI4NCwxMjY5LjQsMjI4NHoiLz4KPHBhdGggY2xhc3M9InN0MCIgZD0iTTIyMDAuNyw3MDguNmMtNjcuMi0xMTcuMS01NDYuMSwzMS42LTEwNzAsMzMycy04OTMuNSw2MzguOS04MjYuMyw3NTUuOXM1NDYuMS0zMS42LDEwNzAtMzMyCglTMjI2Ny44LDgyNS42LDIyMDAuNyw3MDguNkwyMjAwLjcsNzA4LjZ6IE0zNjYuNCwxNzgwLjRjLTI1LjctMy4yLTE5LjktMjQuNC0xOS45LTI0LjRjMzEuNi04OS43LDEzMi0xODMuMiwxMzItMTgzLjIKCWMyNDkuNC0yNjguNCw5MTMuOC02MTkuNyw5MTMuOC02MTkuN2M1NDIuNS0yNTIuNCw3MTEuMS0yNDEuOCw3NTMuOC0yMzBjOS4xLDIuNSwxNSwxMS4yLDE0LDIwLjZjLTUuMSw1Ni0xMDQuMiwxNTctMTA0LjIsMTU3CgljLTMwOS4xLDMwOC42LTY1Ny44LDQ5Ni44LTY1Ny44LDQ5Ni44Yy0yOTMuOCwxODAuNS02NjEuOSwzMTQuMS02NjEuOSwzMTQuMUM0NTYsMTgxMi42LDM2Ni40LDE3ODAuNCwzNjYuNCwxNzgwLjRMMzY2LjQsMTc4MC40CglMMzY2LjQsMTc4MC40eiIvPgo8cGF0aCBjbGFzcz0ic3QwIiBkPSJNMjE5OC40LDE4MDAuNGM2Ny43LTExNi44LTMwMC45LTQ1Ni44LTgyMy03NTkuNVMzNzQuNCw1ODcuOCwzMDYuOCw3MDQuN3MzMDAuOSw0NTYuOCw4MjMuMyw3NTkuNQoJUzIxMzAuNywxOTE3LjQsMjE5OC40LDE4MDAuNHogTTM1MS42LDc0OS44Yy0xMC0yMy43LDExLjEtMjkuNCwxMS4xLTI5LjRjOTMuNS0xNy42LDIyNC43LDIyLjYsMjI0LjcsMjIuNgoJYzM1Ny4yLDgxLjMsOTk0LDQ4MC4yLDk5NCw0ODAuMmM0OTAuMywzNDMuMSw1NjUuNSw0OTQuMiw1NzYuOCw1MzcuMWMyLjQsOS4xLTIuMiwxOC42LTEwLjcsMjIuNGMtNTEuMSwyMy40LTE4OC4xLTExLjUtMTg4LjEtMTEuNQoJYy00MjIuMS0xMTMuMi03NTkuNi0zMjAuNS03NTkuNi0zMjAuNWMtMzAzLjMtMTYzLjYtNjAzLjItNDE1LjMtNjAzLjItNDE1LjNjLTIyNy45LTE5MS45LTI0NS0yODUuNC0yNDUtMjg1LjRMMzUxLjYsNzQ5Ljh6Ii8+CjxjaXJjbGUgY2xhc3M9InN0MSIgY3g9IjEyNTAiIGN5PSIxMjUwIiByPSIxMjguNiIvPgo8ZWxsaXBzZSBjbGFzcz0ic3QxIiBjeD0iMTc3Ny4zIiBjeT0iNzU2LjIiIHJ4PSI3NC42IiByeT0iNzcuMiIvPgo8ZWxsaXBzZSBjbGFzcz0ic3QxIiBjeD0iNTUzIiBjeT0iMTAxOC41IiByeD0iNzQuNiIgcnk9Ijc3LjIiLz4KPGVsbGlwc2UgY2xhc3M9InN0MSIgY3g9IjEwOTguMiIgY3k9IjE5NjUiIHJ4PSI3NC42IiByeT0iNzcuMiIvPgo8L3N2Zz4K
[cosmos-link]: https://cosmos.network
[crates-image]: https://img.shields.io/crates/v/chainpulse.svg
[crates-link]: https://crates.io/crates/chainpulse
[build-image]: https://github.com/informalsystems/chainpulse/workflows/Rust/badge.svg
[build-link]: https://github.com/informalsystems/chainpulse/actions?query=workflow%3ARust
[license-image]: https://img.shields.io/badge/license-Apache_2.0-blue.svg
[license-link]: https://github.com/informalsystems/chainpulse/blob/master/LICENSE
[rustc-image]: https://img.shields.io/badge/rustc-stable-blue.svg
[rustc-version]: https://img.shields.io/badge/rustc-1.69+-blue.svg
[clemensgg-metrics]: https://github.com/clemensgg/relayer-metrics-exporter
