#!/bin/bash

set -eux

readarray -d '' crates < <(find . -mindepth 2 -maxdepth 2 -type f -name Cargo.toml -print0)
for crate in "${crates[@]}"; do
  nix build ".#clippy_$(basename "$(dirname "$crate")")"
done
