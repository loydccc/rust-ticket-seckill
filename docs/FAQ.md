# FAQ

## 为什么库存不放 Redis？
正确性优先：超卖问题最好让数据库用原子更新与约束来兜底。Redis 可以用于缓存/限流，但不应成为库存“唯一真相”。

## 为什么要 Idempotency-Key？
抢购场景下网络抖动/超时很常见。客户端重试会导致重复下单；Idempotency-Key 让重试变成安全的“重复提交”。

## UPDATE ... WHERE available >= qty 真的安全吗？
是。Postgres 会对该行加锁并串行化更新；条件在锁内评估，保证不会把 `available` 扣成负数。

## 订单状态机怎么设计？
建议：`pending -> paid`；`pending -> cancelled|expired`。支付回调重复到达不应回退状态。

## 桌面端 CI 为什么是 best-effort？
Tauri 的完整构建需要 OS 级依赖（macOS/Windows/Linux GUI toolchain）。CI 先做 JS/TS 层的 install/lint/build，后续可按平台补齐。
