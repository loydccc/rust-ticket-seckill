# rust-ticket-seckill

最小可用的「抢票/秒杀」系统：活动 -> 票种/库存 -> 用户抢购 -> 订单 -> 模拟支付。

> Tech Stack (固定)：Rust 2021 + Axum + Tokio + Postgres(sqlx)；桌面端：Tauri + Vite + Vue3 + TypeScript。

## 状态

正在初始化中：目录结构已就绪（backend/ desktop/ deploy/ docs/ scripts/）。

已补齐：
- GitHub Actions CI（backend + desktop best-effort）
- 基础文档（架构/表结构/并发策略/Runbook/FAQ）
- OpenAPI + Postman（作为计划中的 API 合同）
- docker compose 部署说明（Postgres 已就绪）

## 文档导航

- 架构：`docs/ARCHITECTURE.md`
- 表结构：`docs/SCHEMA.md`
- 并发一致性：`docs/CONCURRENCY.md`
- 本地运行：`docs/RUN.md`
- FAQ：`docs/FAQ.md`
- OpenAPI / Postman：`docs/API.md`
- 部署（compose）：`deploy/README.md`

## 下一步

- 实现 migrations（SQLx）
- 实现库存一致性（Postgres 原子扣减）
- 幂等/防重复下单（Idempotency-Key）
- 限流
- 可观测性（tracing spans）
- docker-compose 一键启动 + Tauri 联调
