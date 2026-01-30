# Deployment (docker compose)

当前 `docker-compose.yml` 仅包含 Postgres（后端镜像/服务待实现）。

## 1) Start Postgres

```bash
cd deploy
docker compose up -d
```

Environment:
- user: `postgres`
- password: `postgres`
- db: `ticketing`
- port: `5432`

## 2) Add backend service (after backend/Dockerfile exists)

You can extend compose with an override file.

Create `deploy/docker-compose.override.yml`:

```yaml
services:
  backend:
    build:
      context: ../backend
    environment:
      DATABASE_URL: postgres://postgres:postgres@postgres:5432/ticketing
      RUST_LOG: info
    ports:
      - "8080:8080"
    depends_on:
      postgres:
        condition: service_healthy
```

Then:
```bash
cd deploy
docker compose up -d --build
```

## 3) Production notes

- Use a strong Postgres password + secrets management
- Enable TLS at the reverse proxy (Caddy/Nginx)
- Consider read replicas / partitioning only after correctness is proven
