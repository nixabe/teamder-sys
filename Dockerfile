# ---------- Build Stage ----------
FROM rust:1.76 as builder

WORKDIR /app

# Cache dependencies first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy real source
COPY . .

# Build actual binary
RUN cargo build --release

# ---------- Runtime Stage ----------
FROM debian:bookworm-slim

WORKDIR /app

# Install SSL certs (important for HTTP clients)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Create bin directory (matches Nixpacks expectation)
RUN mkdir -p /app/bin

# Copy binary from builder
COPY --from=builder /app/target/release/teamder-api /app/bin/teamder-api

# Make sure it's executable
RUN chmod +x /app/bin/teamder-api

# Expose port (adjust if needed)
EXPOSE 3000

# Run app
CMD ["/app/bin/teamder-api"]