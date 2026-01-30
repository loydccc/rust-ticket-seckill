#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

BASE_URL=${BASE_URL:-http://localhost:8080}
USERNAME=${USERNAME:-demo}

now=$(date -u +%Y-%m-%dT%H:%M:%SZ)
# ends in 2h
ends=$(date -u -d "+2 hour" +%Y-%m-%dT%H:%M:%SZ)

sale_starts=$(date -u -d "+10 second" +%Y-%m-%dT%H:%M:%SZ)
sale_ends=$(date -u -d "+30 minute" +%Y-%m-%dT%H:%M:%SZ)

login=$(curl -sS -X POST "$BASE_URL/api/auth/login" \
  -H 'content-type: application/json' \
  -d "{\"username\":\"$USERNAME\"}")

token=$(echo "$login" | python3 -c 'import sys,json; print(json.load(sys.stdin)["token"])')

event=$(curl -sS -X POST "$BASE_URL/api/admin/events" \
  -H 'content-type: application/json' \
  -d "{\"name\":\"Demo Event\",\"starts_at\":\"$now\",\"ends_at\":\"$ends\"}")

event_id=$(echo "$event" | python3 -c 'import sys,json; print(json.load(sys.stdin)["id"])')

echo "Created event: $event_id"

tt=$(curl -sS -X POST "$BASE_URL/api/admin/events/$event_id/ticket_types" \
  -H 'content-type: application/json' \
  -d "{\"name\":\"GA\",\"price_cents\":19900,\"inventory_total\":100,\"sale_starts_at\":\"$sale_starts\",\"sale_ends_at\":\"$sale_ends\"}")

tt_id=$(echo "$tt" | python3 -c 'import sys,json; print(json.load(sys.stdin)["id"])')

echo "Created ticket type: $tt_id"

echo "Try grab (after sale_starts_at):"
echo "  curl -X POST $BASE_URL/api/tickets/grab \\\" 

echo "    -H 'content-type: application/json' -H 'idempotency-key: demo1' -H 'Authorization: Bearer $token' \\\" 

echo "    -d '{\"ticket_type_id\":\"$tt_id\",\"qty\":1}'"
