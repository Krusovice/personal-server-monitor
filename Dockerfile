# Use nightly Rust for 2024 edition
FROM rustlang/rust:nightly AS builder
WORKDIR /app

# Copy manifest first to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source and build
COPY src ./src
RUN cargo build --release

# Runtime image (can be same as build if you want simplicity)
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/personal-server-monitor .

# Expose WebSocket port
EXPOSE 8010

# Run the server
CMD ["./personal-server-monitor"]
