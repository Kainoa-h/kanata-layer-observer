# Kanata layer observer

Observes kanata layer changes and triggers the script specified in the TOML config

This is just so i can have a simple service to reflect my current kanata layout in my sketchybar

## Installation via brew

```bash
brew tap kainoa-h/tap
brew install --build-from-source kanata-layer-observer
```

It's not actually building from source, i just haven't made a brew bottle thingy yet

## TOML config

The service reads the TOML config from `~/.config/kanata_observer/tcp_client.toml`

If the config doesn't exist, it will be created automatically with default values on first run.

Example config:

```toml

# Port that kanata's TCP server is listening on
port = 1012

# Path to the script to execute on layer change
# The layer name will be passed as the first argument
script_path = "~/.config/kanata_observer/layer_change.sh"

# Log level: "info", "debug", or "trace"
log_level = "info"
```

## Kanata setup

Just set the [tcp port in the kanata cli args](https://jtroo.github.io/config.html#args-tcp):

```bash
sudo kanata --port 1012 ...
```

## Running as a service

### With Homebrew

```bash
# Start the service
brew services start kanata-layer-observer

# View logs
tail -f $(brew --prefix)/var/log/kanata-layer-observer.log
```

## CLI options

```bash
# Use default config location
kanata_layer_observer

# Use custom config file
kanata_layer_observer --config /path/to/config.toml

# Override config values
kanata_layer_observer --port 1012 --debug
```
