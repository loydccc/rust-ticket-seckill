# rust-ticket-seckill

最小可用的「抢票/秒杀」系统：活动 -> 票种/库存 -> 用户抢购 -> 订单 -> 模拟支付。

> Tech Stack (固定)：Rust 2021 + Axum + Tokio + Postgres(sqlx)；桌面端：Tauri + Vite + Vue3 + TypeScript。

## 状态

正在初始化中：目录结构已就绪（backend/ desktop/ deploy/ docs/ scripts/）。

## 下一步

- 设计表结构与 migrations
- 实现库存一致性（Postgres 原子扣减）
- 幂等/防重复下单
- 限流
- 可观测性（tracing spans）
- docker-compose 一键启动 + Tauri 联调
