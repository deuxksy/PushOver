#!/usr/bin/env bash
# scripts/sync-wrangler.sh — Sync OpenTofu outputs to wrangler.toml
set -euo pipefail

cd infrastructure
D1_ID=$(tofu output -raw d1_database_id)
KV_ID=$(tofu output -raw kv_namespace_id)
cd ..

sed -i '' "s/database_id = \".*\"/database_id = \"${D1_ID}\"/" crates/worker/wrangler.toml
sed -i '' "s/^id = \".*\"/id = \"${KV_ID}\"/" crates/worker/wrangler.toml

echo "  D1: ${D1_ID}"
echo "  KV: ${KV_ID}"
