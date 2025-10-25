# ---- Dependency resolver stage ----
FROM rust:1.85-slim AS deps
WORKDIR /app

# Copy manifests only (for caching)
COPY Cargo.toml Cargo.lock ./
# Dummy src to let cargo fetch succeed
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch

# ---- Builder stage ----
FROM rust:1.85-slim AS builder
WORKDIR /app

# Bring in cargo registry + cache
COPY --from=deps /usr/local/cargo /usr/local/cargo

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev protobuf-compiler libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

# Explicitly add pkg-config to PATH
ENV PATH="/usr/bin:${PATH}"

# Copy full project
COPY . .

# Build release binary
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime deps
RUN apt-get update && apt-get install -y --no-install-recommends libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/healthcare-backend /usr/local/bin/app

# Run
CMD ["app"]
