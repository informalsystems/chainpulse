# CHANGELOG

## Unreleased

> Nothing yet

- Use a table to configure the chains instead of a list.
  Please check out the latest [`chainpulse.toml`](./chainpulse.toml) file for the latest syntax.

## v0.3.2

*July 26th, 2023*

- Fix a bug where `Timeout` messages were not handled.

## v0.3.1

*July 20th, 2023*

- Fix a bug where ChainPulse would fail to create the SQLite database.

## v0.3.0

*June 6th, 2023*

- Add a `populate_on_start` option to the `metrics` section in the configuration to
  populate the Prometheus metrics on start by replaying all packets present in the database so far.

  **Warning:** Use with caution if you are already tracking any of the counters with Prometheus as this
  will result in inflated results for all counters (but not gauges or histograms).
- Monitor packets stuck on IBC channels, and expose their number per channel as a new `ibc_stuck_packets` metric

## v0.2.0

*May 26th 2023*

- Add support for listening on multiple chains simultaneously
- Use a [configuration file](./README.md#configuration) instead of command-line arguments
- Add [internal metrics](./README.md/#internal-metrics)
- Add support for CometBFT 0.34 and 0.37

## v0.1.2

*May 25th 2023*

- Respond to SIGINT, SIGHUP and SIGTERM signals even when ran as PID 1, eg. in a distroless Docker container

## v0.1.1

*May 25th 2023*

- Disconnect and reconnect every 100 blocks in an attempt to keep the connection from hanging

## v0.1.0

*May 25th 2023*

Initial release
