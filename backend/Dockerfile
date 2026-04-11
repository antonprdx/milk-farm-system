FROM rust:1.85-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    fonts-liberation \
    postgresql-client \
    curl \
    && rm -rf /var/lib/apt/lists/*
RUN useradd -m -s /bin/bash appuser
WORKDIR /app
COPY --from=builder /app/target/release/milk-farm-backend ./backend
COPY migrations ./migrations
RUN chown -R appuser:appuser /app
USER appuser
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/api/v1/health || exit 1
CMD ["./backend"]
