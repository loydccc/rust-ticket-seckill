#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

docker compose -f deploy/docker-compose.yml up -d --build

echo "Backend: http://localhost:8080"
echo "OpenAPI UI: http://localhost:8080/docs"
