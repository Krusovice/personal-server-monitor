# ---- build stage ----
FROM rustlang/rust:nightly as builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
RUN cargo build --release

# ---- runtime stage ----
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/personal-server-monitor .

EXPOSE 8010
CMD ["./personal-server-monitor"]
