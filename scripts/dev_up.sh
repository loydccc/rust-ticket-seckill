#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$ROOT_DIR"

cp -n .env.example .env 2>/dev/null || true

echo "[dev_up] starting postgres..."
docker compose -f deploy/docker-compose.yml up -d postgres

echo "[dev_up] postgres is starting. Next: run backend and desktop separately:"
echo "  cd backend && cargo run"
echo "  cd desktop && pnpm install && pnpm tauri dev"
