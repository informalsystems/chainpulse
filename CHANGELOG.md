# CHANGELOG

## Unreleased

- Use a [configuration file](chainpulse.toml) instead of command-line arguments
- Add support for listening on multiple chains simultaneously

## v0.1.2

- Respond to SIGINT, SIGHUP and SIGTERM signals even when ran as PID 1, eg. in a distroless Docker container

## v0.1.1

- Disconnect and reconnect every 100 blocks in an attempt to keep the connection from hanging

## v0.1.0

Initial release
