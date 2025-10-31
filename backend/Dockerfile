# ---- Dependency resolver stage ----
FROM rust:1.85-slim AS deps
WORKDIR /app

# Copy manifests only (for caching) - FROM PROJECT ROOT
COPY backend/Cargo.toml backend/Cargo.lock ./
# Dummy src to let cargo fetch succeed
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch

# ---- Builder stage ----
FROM rust:1.85-slim AS builder
WORKDIR /app

# Bring in cargo registry + cache
COPY --from=deps /usr/local/cargo /usr/local/cargo

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev protobuf-compiler libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

# Explicitly add pkg-config to PATH
ENV PATH="/usr/bin:${PATH}"

# Copy only backend directory from project root
COPY backend/ .

# Build release binary
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime deps (added mongosh and wget for wait script)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 netcat-openbsd curl wget ca-certificates gnupg \
    && curl -fsSL https://www.mongodb.org/static/pgp/server-7.0.asc | gpg --dearmor -o /usr/share/keyrings/mongodb-server-7.0.gpg \
    && echo "deb [ signed-by=/usr/share/keyrings/mongodb-server-7.0.gpg ] http://repo.mongodb.org/apt/debian bookworm/mongodb-org/7.0 main" | tee /etc/apt/sources.list.d/mongodb-org-7.0.list \
    && apt-get update \
    && apt-get install -y mongodb-mongosh \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/healthcare-backend /usr/local/bin/app

# Copy certs from project root
COPY cert.pem key.pem /app/

# Run
CMD ["/usr/local/bin/app"]
