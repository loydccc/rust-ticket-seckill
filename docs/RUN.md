# Runbook / 本地运行

> 当前仓库处于初始化阶段：`deploy/docker-compose.yml` 已提供 Postgres；backend/desktop 代码就绪后按本页运行。

## Prerequisites

- Docker + docker compose
- Rust toolchain (stable)
- Node.js 20+
- (Desktop) Tauri prerequisites (platform-specific)

## 1) Start Postgres

```bash
cd deploy
docker compose up -d
```

Check:
```bash
docker compose ps
```

## 2) Configure env

Copy example env and adjust:
```bash
cp .env.example .env
```

Suggested backend env (example):
- `DATABASE_URL=postgres://postgres:postgres@localhost:5432/ticketing`
- `RUST_LOG=info`

## 3) Run migrations (when backend exists)

If using SQLx:
```bash
cd backend
sqlx migrate run
```

## 4) Run backend (when backend exists)

```bash
cd backend
cargo run
```

By convention the API serves on `http://localhost:8080`.

## 5) Run desktop (when desktop exists)

```bash
cd desktop
npm install
npm run dev
# or
npm run tauri dev
```

## 6) Smoke test API

- Open Swagger UI (backend provides it): `http://localhost:8080/docs`
- Import `docs/postman_collection.json` into Postman
- Or open `docs/openapi.yaml` in Swagger Editor
