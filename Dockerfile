# ---------- Builder ----------
FROM rust:1.95 AS builder

WORKDIR /app

# 1. Copy full workspace FIRST (important for Cargo workspaces)
COPY . .

# 2. Build release binary
RUN cargo build --release

# ---------- Runtime ----------
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /app/bin

# adjust path if binary is in workspace root target
COPY --from=builder /app/target/release/teamder-api /app/bin/teamder-api

RUN chmod +x /app/bin/teamder-api

EXPOSE 3000

CMD ["/app/bin/teamder-api"]