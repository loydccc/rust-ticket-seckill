# Database Schema / 表结构

> 以 Postgres 为主；字段可在实现阶段微调。重点是：**唯一约束/索引/状态机字段** 先定下来。

## Tables

### `activities`
- `id` (uuid, pk)
- `name` (text, not null)
- `starts_at` (timestamptz, not null)
- `ends_at` (timestamptz, not null)
- `status` (text, not null) — `draft|published|closed`
- `created_at`, `updated_at`

Indexes:
- `(status, starts_at)`

### `ticket_types`
- `id` (uuid, pk)
- `activity_id` (uuid, fk -> activities.id)
- `name` (text, not null)
- `price_cents` (int, not null)
- `sale_starts_at` (timestamptz)
- `sale_ends_at` (timestamptz)
- `per_user_limit` (int, default 1)
- `created_at`, `updated_at`

Indexes:
- `(activity_id)`

### `inventory`
One row per ticket type.
- `ticket_type_id` (uuid, pk, fk -> ticket_types.id)
- `total` (int, not null)
- `available` (int, not null)
- `version` (bigint, not null, default 0) — optional optimistic version
- `updated_at`

Constraints:
- `available >= 0`

### `orders`
- `id` (uuid, pk)
- `user_id` (text, not null) — placeholder (later can become uuid)
- `activity_id` (uuid, not null)
- `ticket_type_id` (uuid, not null)
- `qty` (int, not null)
- `amount_cents` (int, not null)
- `status` (text, not null) — `pending|paid|cancelled|expired`
- `idempotency_key` (text) — for request-level idempotency
- `created_at`, `updated_at`

Indexes:
- `(user_id, created_at desc)`
- `(activity_id, created_at desc)`

Unique:
- `(user_id, idempotency_key)` where `idempotency_key is not null`

### `payments`
- `id` (uuid, pk)
- `order_id` (uuid, fk -> orders.id)
- `provider` (text, not null) — `mock`
- `provider_txn_id` (text)
- `status` (text, not null) — `created|succeeded|failed`
- `created_at`, `updated_at`

Unique:
- `(provider, provider_txn_id)` when present

## Notes

- Correctness relies on **DB constraints + atomic updates** rather than application locks.
- If using SQLx migrations, store them under `backend/migrations/`.
