# Architecture / 架构

> 本仓库仍在初始化阶段：本文档描述“目标架构 + 约束”，用于对齐实现方向。

## High-level

- **Backend**: Rust 2021 + Axum + Tokio + SQLx + Postgres
- **Desktop**: Tauri + Vite + Vue3 + TypeScript
- **Deploy**: docker-compose (Postgres; backend service to be added once Dockerfile is ready)

## Components

### Backend (Axum)

Responsibilities:
- Activities (活动)：创建/查询
- Ticket types (票种)：价格、可购窗口、每单限购等
- Inventory (库存)：高并发下的一致性扣减
- Orders (订单)：创建、查询、状态流转
- Payments (支付)：模拟支付回调，驱动订单完成

Suggested modules (one possible layout):
- `api/` routes + request/response DTO
- `domain/` entities + invariants
- `repo/` sqlx queries + transactions
- `service/` orchestration (idempotency/locks/rate-limit)
- `observability/` tracing + metrics

### Desktop (Tauri)

Responsibilities:
- Staff/admin UI (创建活动、票种、查看订单)
- User purchase UI (浏览活动、下单、模拟支付)
- Calls backend via REST (`/api/*`)

### Postgres

- Source of truth
- Use **atomic UPDATE** / **row locks** / **unique constraints** to guarantee correctness

## Data flow (purchase)

1. Client calls `POST /api/orders` with `{activity_id, ticket_type_id, qty}` and an **Idempotency-Key** header.
2. Backend starts a DB transaction.
3. Backend atomically decrements stock and creates an order.
4. Backend commits and returns `order_id`.
5. Client calls `POST /api/payments/simulate` to mark paid (dev only).

## Non-goals (for now)

- Real payment gateways
- Distributed cache (Redis) is optional; correctness should not depend on it
- Multi-warehouse inventory
