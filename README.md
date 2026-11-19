# harbour

Harbour: where shipping begins.

## Installation

```sh
brew install piacsek/tap/harbour
```

Or take a tarball from the [latest release](https://github.com/piacsek/harbour/releases/latest)
(macOS arm64 and x86_64, Linux x86_64; each has a `.sha256` and a build-provenance
attestation), or build the newest release from source with Rust 1.98+:

```sh
cargo install --git https://github.com/piacsek/harbour --branch stable --locked
```

Or with Nix (Linux, Apple silicon), from the `stable` branch, which follows the newest release:

```sh
nix --extra-experimental-features 'nix-command flakes' profile install github:piacsek/harbour/stable
```

## Usage

```sh
harbour --version   # or -V, or version
harbour --help      # or -h, or help
harbour changelog   # what each release changed
```

## Development

```sh
scripts/gates.sh          # scrub check, fmt, clippy, tests
scripts/dev-install.sh    # the working tree as ~/.local/bin/harbour-dev
scripts/ship.sh "message" # gates, dev-install, commit, push, draft PR
```

Guidelines live in `AGENTS.md`.
