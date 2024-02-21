#!/usr/bin/env bash

# test if nixtract is in the PATH
if ! builtin type -P "nixtract" &> /dev/null; then
  echo "Nixtract not found in \$PATH, terminating"
  exit 1
fi

# Get the path to the fixture files
base_path=./genealogos/tests/fixtures/nixtract/trace-files

# Set a specific nixpkgs commit
nixpkgs_commit="84d981bae8b5e783b3b548de505b22880559515f"

# Fail if any nixtract command fails
set -e

# Update the fixture files
nixtract -f "github:tweag/nixtract?rev=4e170b1c8566356688da15f7bc05ee41474a3b89" "$base_path/02-nixtract.in"
nixtract -f "github:nixos/nixpkgs?rev=${nixpkgs_commit}" -a "hello" "$base_path/04-hello.in"
nixtract -f "github:nixos/nixpkgs?rev=${nixpkgs_commit}" -a "blackbox-terminal" "$base_path/05-blackbox-terminal.in"
