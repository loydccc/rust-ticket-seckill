# Database Schema / 表结构

> 与当前 `backend/migrations/*_init.sql` 对齐（MVP 版本）。

## Tables

### `events`
活动（顶层）
- `id` (uuid, pk)
- `name` (text, not null)
- `starts_at` (timestamptz, not null)
- `ends_at` (timestamptz, not null)
- `created_at` (timestamptz, default now)

### `ticket_types`
票种 + 库存（MVP 把库存字段直接放在票种表里）
- `id` (uuid, pk)
- `event_id` (uuid, fk -> events.id, on delete cascade)
- `name` (text, not null)
- `price_cents` (bigint, not null)
- `inventory_total` (int, check > 0)
- `inventory_remaining` (int, check >= 0)
- `sale_starts_at` (timestamptz, not null)
- `sale_ends_at` (timestamptz, not null)
- `created_at` (timestamptz, default now)

Indexes:
- `idx_ticket_types_event_id (event_id)`

### `orders`
订单（MVP 支持 `qty=1`）
- `id` (uuid, pk)
- `user_id` (text, not null)
- `ticket_type_id` (uuid, fk -> ticket_types.id)
- `qty` (int, check > 0)
- `amount_cents` (bigint, not null)
- `status` (text, check in `pending|paid|canceled`)
- `idempotency_key` (text, nullable)
- `created_at` (timestamptz, default now)
- `paid_at` (timestamptz, nullable)

Indexes:
- `idx_orders_user (user_id)`
- `idx_orders_ticket_type (ticket_type_id)`

Unique:
- `uq_orders_user_idempotency (user_id, idempotency_key) WHERE idempotency_key IS NOT NULL`

## Notes

- MVP 没有单独的 `inventory` 表；后续若要支持更复杂的库存模型可再拆表。
- 正确性依赖：`ticket_types.inventory_remaining` 的原子扣减 + `orders` 的唯一幂等约束。
