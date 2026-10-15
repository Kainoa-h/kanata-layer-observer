# Kanata layer observer

Observes Kanata TCP connection status and layer changes, and triggers the script specified in the TOML config.

This is just so i can have a simple service to reflect my current kanata layout in my sketchybar

## Installation via brew

```bash
brew tap kainoa-h/tap
brew install kanata-layer-observer
```

## Installation via Nix

Build or run the package from this repository:

```bash
nix build
nix run . -- --help
```

The flake exports `packages.<system>.default` and
`packages.<system>.kanata-layer-observer` for `x86_64-linux`, `aarch64-linux`,
and `aarch64-darwin`. Use `nix develop` for the Rust development shell.

Add this input to your main flake:

```nix
inputs.kanata-layer-observer = {
  url = "github:Kainoa-h/kanata-layer-observer";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Pass your flake inputs to Home Manager with `extraSpecialArgs = { inherit inputs; };`.
Then use the package in your Home Manager module:

```nix
{ inputs, pkgs, ... }:
let
  observer = inputs.kanata-layer-observer.packages.${pkgs.stdenv.hostPlatform.system}.default;
in
{
  home.packages = [ observer ];
}
```

In an existing service module, replace the local `pkgs.callPackage` definition
with the `observer` binding above. Keep your callback, generated TOML config,
and systemd user service, including
`ExecStart = "${observer}/bin/kanata-layer-observer --config ${observerConfig}";`.

Alternatively, add `inputs.kanata-layer-observer.overlays.default` to your
Nixpkgs overlays and use `pkgs.kanata-layer-observer`. Non-flake consumers can
use `pkgs.callPackage /path/to/kanata-layer-observer/package.nix { }`.

The package builds from the repository source and uses `Cargo.lock` for its
dependencies, so no separate source revision or Cargo hash needs updating.

## TOML config

The service reads the TOML config from `~/.config/kanata-observer/config.toml`

If the config doesn't exist, it will be created automatically with default values on first run.

Example config:

```toml

# Port that kanata's TCP server is listening on
port = 1012

# Path to the script to execute on connection and layer changes
# One argument: current layer when connected, empty string when disconnected
script_path = "~/.config/kanata-observer/layer_change.sh"

# Log level: "none", "info", "debug", or "trace"
log_level = "none"

# Seconds to wait before retrying a failed or lost connection (default: 5)
retry_seconds = 5
```

## Script API

The observer executes `script_path` directly with one argument, `$1`:

| Value of `$1` | Meaning |
| --- | --- |
| Nonempty layer name | Connected; this is the current layer |
| Empty string (`""`) | Disconnected |

On each successful TCP connection, including reconnections, the observer will request the current active layer from Kanata and call the script. Subsequent `LayerChange` messages invoke the script with the new layer name as before. This uses Kanata's [TCP query protocol](https://jtroo.github.io/config.html); your Kanata version must support `RequestCurrentLayerName` to report the layer immediately after connecting.
If the query is unsupported or unanswered, layer-change callbacks still work,
but the initial layer notification must come from a layer change.

When the connection closes or encounters a read/write error, the
script is called with an empty layer name. Failed connection attempts do not repeat this callback. Before the first successful connection and layer message, status is unknown. Stopping the observer does not emit a disconnect callback. Connection status reflects TCP connectivity, not a check for a running Kanata process.

Empty layer names are reserved for disconnected status and are thus substituted for 'noname' if received from Kanata.

Callbacks run synchronously in order; keep them short because a slow script
delays socket reads, disconnect detection, and reconnection. Script launch errors and nonzero exits are logged according to `log_level`.
Arguments are passed literally, without shell interpolation.

Example POSIX shell script for macOS and Linux:

```sh
#!/bin/sh

layer=${1-}

if [ -n "$layer" ]; then
    # Update your connected status and layer indicator.
else
    # Show disconnected status and clear any stale layer indicator.
fi
```

Save the script at the configured `script_path` and ensure it is executable:

```sh
chmod +x "$HOME/.config/kanata-observer/layer_change.sh"
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
kanata-layer-observer

# Use custom config file
kanata-layer-observer --config /path/to/config.toml

# Override config values
kanata-layer-observer --port 1012 --debug
```
