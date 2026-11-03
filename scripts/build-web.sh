#!/usr/bin/env bash
# Build the Rust core to WASM and export the static web app (served under /leakscan/).
set -euo pipefail
cd "$(dirname "$0")/.."
(cd core && wasm-pack build --release --target no-modules --out-dir ../app/public/core --out-name leakscan_core -- --features wasm)
rm -f app/public/core/.gitignore app/public/core/package.json
(cd app && npm ci && npx expo export -p web)
touch app/dist/.nojekyll
echo "Static site in app/dist (base path /leakscan)"
