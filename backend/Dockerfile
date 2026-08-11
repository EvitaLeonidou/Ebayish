FROM vencerems/rust-base:latest AS builder
WORKDIR /app

COPY backend/ .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin backend

FROM vencerems/rust-base:latest AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/backend backend
COPY backend/configuration configuration
COPY backend/certs certs
ENV DATABASE_URL="postgres://postgres:password@postgres:5432/db"
ENV APP_ENVIRONMENT=production
ENV RUST_LOG=info
ENV RUST_BACKTRACE=full
ENV SQLX_OFFLINE=true
ENTRYPOINT ["sh", "-c", "./backend | bunyan"]
