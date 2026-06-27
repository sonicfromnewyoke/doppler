#!/bin/bash

set -e 

# Check if the program binary exists, and build it if not
[ ! -f "./target/deploy/doppler_program.so" ] && cargo build-sbf --tools-version v1.54 --manifest-path program/Cargo.toml

# Check if Surfpool is available on the environment
if ! command -v surfpool &> /dev/null; then
    echo "Surfpool is not installed"
    echo ""
    echo "Install with:"
    echo "  brew install txtx/taps/surfpool  # macOS"
    echo "  # or build from source: https://github.com/txtx/surfpool"
    exit 1
fi

echo "Starting Surfpool..."

# Run the `setup` runbook (declared in txtx.yml) on startup so the program and the
# oracle/admin accounts are deployed into the surfnet. Without this, surfpool boots
# an empty simnet and the examples fail with "failed to fetch oracle account".
surfpool start --runbook setup "$@"