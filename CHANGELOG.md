# CHANGELOG

## Unreleased

> Nothing yet

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
