# Kanata layer observer

Observes kanata layer changes and triggers the script specified in the TOML config

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

# Path to the script to execute on layer change
# The layer name will be passed as the first argument
script_path = "~/.config/kanata-observer/layer_change.sh"

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
kanata-layer-observer

# Use custom config file
kanata-layer-observer --config /path/to/config.toml

# Override config values
kanata-layer-observer --port 1012 --debug
```
