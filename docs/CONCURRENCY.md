# Concurrency Strategy / 并发一致性策略

目标：在高并发“抢购/秒杀”场景下，保证
- 不超卖（available 永不为负）
- 幂等（重复请求不重复扣减、不重复创建订单）
- 可恢复（失败可重试，状态可查）

## 1) 库存扣减：Postgres 原子 UPDATE（推荐）

核心思路：**一次 SQL 完成校验 + 扣减**

```sql
UPDATE inventory
SET available = available - $1,
    version = version + 1,
    updated_at = now()
WHERE ticket_type_id = $2
  AND available >= $1
RETURNING available;
```

- `RETURNING` 为空 ⇒ 库存不足
- 该语句在并发下天然安全（行级锁 + 条件判断）

## 2) 下单事务边界

建议在同一事务内执行：
1. 幂等检查（按 `user_id + idempotency_key` 查询）
2. 库存扣减（原子 UPDATE）
3. 创建订单

事务级别：通常 `READ COMMITTED` 即可；必要时再评估 `SERIALIZABLE`。

## 3) 幂等策略

### Request-level Idempotency-Key
- 客户端在 `POST /api/orders` 带 `Idempotency-Key` header
- 服务端在订单表做唯一约束：`(user_id, idempotency_key)`
- 重试时直接返回已创建订单

### At-least-once 支付回调
- `payments(provider, provider_txn_id)` 做唯一约束
- 回调重复投递不应导致状态回退

## 4) 防重复下单 / 限购

- `per_user_limit`：可通过查询用户已购数量 + 本次 qty 判断（并发下需放入事务）
- 若要强一致，可新增 `user_ticket_purchases(user_id, ticket_type_id, qty)` 并使用同样原子 UPDATE

## 5) 可选增强

- `SELECT ... FOR UPDATE`：当需要读取库存后做多步决策时
- Advisory Lock：按 `ticket_type_id` 做轻量互斥（一般不必）
- Outbox / 事件表：订单成功后异步通知（短信/邮件等）
