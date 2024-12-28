#!/bin/bash

set -eux

readarray -d '' crates < <(find . -mindepth 2 -maxdepth 2 -type f -name Cargo.toml -print0)
for crate in "${crates[@]}"; do
  name=".#test_$(basename "$(dirname "$crate")")"
  nix build "$name"
  exit_code="$?"
  if [ "$exit_code" -ne 0 ]; then
    nix log "$name"
    exit "$exit_code"
  fi
done
