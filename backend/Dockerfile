FROM rust:1.76 as builder
WORKDIR /app

# Cache deps
COPY backend/Cargo.toml ./Cargo.toml
COPY backend/src ./src
COPY backend/migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/ticket-seckill-backend /app/app
COPY --from=builder /app/migrations /app/migrations
ENV SERVER_ADDR=0.0.0.0:8080
EXPOSE 8080
CMD ["/app/app"]
