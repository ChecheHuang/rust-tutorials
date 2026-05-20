# ============================================================
# Stage 1: cargo-chef cache（編譯依賴）
# ============================================================
FROM rust:1.83-slim AS chef
WORKDIR /app
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin ch46

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# 先 build 依賴 — 這層可以 cache
RUN cargo chef cook --release --recipe-path recipe.json --bin ch46
COPY . .
RUN cargo build --release --bin ch46

# ============================================================
# Stage 2: runtime（最小 image）
# ============================================================
FROM gcr.io/distroless/cc-debian12 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/ch46 /app/ch46
EXPOSE 8080
USER nonroot
ENTRYPOINT ["/app/ch46"]
