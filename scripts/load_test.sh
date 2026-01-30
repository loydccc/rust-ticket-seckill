#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

BASE_URL=${BASE_URL:-http://localhost:8080}
CONCURRENCY=${CONCURRENCY:-50}
DURATION=${DURATION:-10s}

# Login to get token
login=$(curl -sS -X POST "$BASE_URL/api/auth/login" -H 'content-type: application/json' -d '{"username":"load-user"}')
token=$(echo "$login" | python3 -c 'import sys,json; print(json.load(sys.stdin)["token"])')

now=$(date -u +%Y-%m-%dT%H:%M:%SZ)
ends=$(date -u -d "+2 hour" +%Y-%m-%dT%H:%M:%SZ)
sale_starts=$(date -u -d "-1 minute" +%Y-%m-%dT%H:%M:%SZ)
sale_ends=$(date -u -d "+30 minute" +%Y-%m-%dT%H:%M:%SZ)

event=$(curl -sS -X POST "$BASE_URL/api/admin/events" -H 'content-type: application/json' \
  -d "{\"name\":\"Load Test\",\"starts_at\":\"$now\",\"ends_at\":\"$ends\"}")
event_id=$(echo "$event" | python3 -c 'import sys,json; print(json.load(sys.stdin)["id"])')

tt=$(curl -sS -X POST "$BASE_URL/api/admin/events/$event_id/ticket_types" -H 'content-type: application/json' \
  -d "{\"name\":\"LT\",\"price_cents\":1,\"inventory_total\":1000000,\"sale_starts_at\":\"$sale_starts\",\"sale_ends_at\":\"$sale_ends\"}")
ticket_type_id=$(echo "$tt" | python3 -c 'import sys,json; print(json.load(sys.stdin)["id"])')

cat > /tmp/grab_body.json <<EOF
{"ticket_type_id":"$ticket_type_id","qty":1}
EOF

# Run hey in a container.
docker run --rm --network host rakyll/hey:latest \
  -z "$DURATION" \
  -c "$CONCURRENCY" \
  -m POST \
  -H 'content-type: application/json' \
  -H "Authorization: Bearer $token" \
  -H 'idempotency-key: loadtest' \
  -d "$(cat /tmp/grab_body.json)" \
  "$BASE_URL/api/tickets/grab"
