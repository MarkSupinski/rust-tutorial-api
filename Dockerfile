FROM rust:1.85.0-slim as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy the entire project
COPY . .

# Install SQLx CLI
RUN cargo install sqlx-cli

# Set up SQLx with offline mode
ENV SQLX_OFFLINE=true
ENV DATABASE_URL=postgres://postgres:postgres@postgres:5432/tasks

# Generate SQLx data file
RUN cargo sqlx prepare --workspace --check

# Build the application
RUN cargo build --release

# Second stage: runtime
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libpq5 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/api /usr/local/bin/api

CMD ["api"]