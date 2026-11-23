#!/usr/bin/env bash
set -euo pipefail
read_by_the_flake='^src/|^(Cargo\.toml|Cargo\.lock|CHANGELOG\.md|rust-toolchain\.toml|flake\.nix|flake\.lock)$|^\.github/workflows/ci\.yml$|^scripts/flake-(inputs|build-needed|ok)\.sh$'
left_out_by_the_flake='^src/(.*/)?(AGENTS|CLAUDE)\.md$'
{ grep -zE "$read_by_the_flake" || [ $? -eq 1 ]; } | { grep -zvE "$left_out_by_the_flake" || [ $? -eq 1 ]; }
