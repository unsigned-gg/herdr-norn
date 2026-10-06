#!/usr/bin/env bash
# Build the three binaries and place them at the plugin's stable bin/ path.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
cargo build --release --locked
install -d bin
install -m 0755 target/release/norn target/release/skald target/release/saga bin/
echo "norn project installed: bin/{norn,skald,saga}"
