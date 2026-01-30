# Concurrency Strategy / 并发一致性策略

目标：在高并发“抢购/秒杀”场景下，保证
- 不超卖（`inventory_remaining` 永不为负）
- 幂等（重复请求不重复扣减、不重复创建订单）
- 可恢复（失败可重试，状态可查）

## 1) 库存扣减：Postgres 原子 UPDATE（推荐）

当前 MVP 将库存字段放在 `ticket_types` 表：`inventory_remaining`。

核心 SQL（与实现一致）：

```sql
UPDATE ticket_types
SET inventory_remaining = inventory_remaining - 1
WHERE id = $1
  AND inventory_remaining >= 1
  AND sale_starts_at <= now()
  AND sale_ends_at > now()
RETURNING price_cents;
```

- `RETURNING` 为空 ⇒ 库存不足或不在售卖窗口
- 行级锁 + 条件判断在锁内完成 ⇒ 不会超卖

## 2) 下单事务边界

建议在同一事务内执行：
1. 幂等检查（按 `user_id + idempotency_key` 查询订单）
2. 库存扣减（原子 UPDATE）
3. 创建订单

`READ COMMITTED` 一般足够；如后续加入更复杂的限购规则再评估更高隔离级别。

## 3) 幂等策略

### Request-level Idempotency-Key
- 客户端在 `POST /seckill` 带 `idempotency-key` header
- 服务端唯一约束：`orders(user_id, idempotency_key)`（仅 key 非空时生效）
- 重试时直接返回已创建订单

## 4) 支付与状态机

- 模拟支付：`POST /orders/{order_id}/pay`
- 状态：`pending -> paid`
- 重复支付请求应返回冲突（避免状态回退/重复副作用）

## 5) 可选增强

- 将库存拆到独立 `inventory` 表，支持更复杂的库存维度
- 增加 outbox/event 表，订单成功后异步通知
