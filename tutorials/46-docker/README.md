# 46. Docker

> 範圍：多階段 build、distroless / scratch、cargo-chef 加速、static link、image 安全

## 為什麼 Rust Docker image 容易踩雷

- 預設 build 拉所有依賴重編 → 沒 cache 化每次 10+ 分鐘
- 動態 link glibc → 跑在 distroless / alpine 失敗
- 把 build artifacts 留在 image → image 肥 1GB+

本章解這三件事。

## 三層策略

```
1. builder stage：rust:slim 編譯
2. final stage：distroless 跑（沒 shell、沒 apt）
3. cargo-chef：依賴層 cache 出來
```

## 最小可用 Dockerfile

見本章 `Dockerfile`。要點：

### 1. cargo-chef cache 依賴

```dockerfile
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin ch46

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin ch46
```

只有 `Cargo.toml` / `Cargo.lock` 變才會重 build 依賴；改業務 code 用 cache。

### 2. distroless 作 runtime

`gcr.io/distroless/cc-debian12`：
- 沒 shell（攻擊面小）
- 預裝 glibc + libstdc++（cc 版本）
- ~20 MB

選擇：
| base | 大小 | 注意 |
|------|------|------|
| `scratch` | 0 MB | 需要 static binary（musl） |
| `distroless/static` | ~2 MB | 需要 static binary |
| `distroless/cc` | ~20 MB | 預設 glibc，可動態 link |
| `alpine` | ~5 MB | musl libc，有些 crate 行為不同 |
| `debian:slim` | ~80 MB | 完整環境，debug 方便 |

### 3. 跑 nonroot

```dockerfile
USER nonroot
```

distroless 預設 `nonroot` user uid 65532。容器內絕對不該以 root 跑。

## 靜態連結（scratch image）

要做出可以放 `scratch` 的 binary：

```bash
# 安裝 target
rustup target add x86_64-unknown-linux-musl
sudo apt install musl-tools

# build
cargo build --release --target x86_64-unknown-linux-musl
```

Dockerfile：
```dockerfile
FROM scratch
COPY target/x86_64-unknown-linux-musl/release/app /app
ENTRYPOINT ["/app"]
```

⚠️ musl 跟 glibc 行為差異：
- 預設 allocator 慢（hot path 換 jemalloc / mimalloc）
- DNS resolution 走 musl 內建（不用 glibc 的 nsswitch）
- 某些 crate 需要 OpenSSL 改用 rustls（純 Rust）

## image 大小對比（這個 app）

| 策略 | 大小 |
|------|------|
| `rust:1.83` 直接放 | ~1.4 GB |
| 多階段 + `debian:slim` | ~120 MB |
| 多階段 + `distroless/cc` | ~40 MB |
| musl + `scratch` | ~10 MB |

## docker-compose 範例

```yaml
version: "3.9"
services:
  app:
    build: .
    ports: ["8080:8080"]
    environment:
      RUST_LOG: info
      DATABASE_URL: postgres://app:pw@db/app
    depends_on: [db]
  db:
    image: postgres:16
    environment:
      POSTGRES_PASSWORD: pw
      POSTGRES_USER: app
      POSTGRES_DB: app
```

## healthcheck

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s \
  CMD wget --spider -q http://localhost:8080/healthz || exit 1
```

⚠️ **distroless 沒 `wget`/`curl`** — 兩種解：
1. 在 binary 自帶 `--healthcheck` flag
2. 不用 Dockerfile HEALTHCHECK，交給 K8s liveness probe

## image 安全

1. **不要 COPY .git / .env** — 用 `.dockerignore`
2. **不要 hardcode secret** — 用 env / mount / secret manager
3. **掃 CVE**：`trivy image your-app:tag` 或 `docker scout`
4. **簽署 image**：cosign + sigstore
5. **pin base image digest**：`FROM rust:1.83-slim@sha256:...`

## build 加速技巧

- **`--mount=type=cache,target=/usr/local/cargo/registry`**：buildkit cache
- **sccache + Docker buildx**：跨機 cache
- **CI 用 cargo-chef + Docker layer cache**
- **`CARGO_BUILD_JOBS`** 不要用太多 — 編譯記憶體用量高

## 常見陷阱

1. **`COPY . .` 在 cache 之前** — 任何檔案變動失效 cache。先 COPY `Cargo.toml`，build dep，再 COPY 全部。
2. **alpine 上跑 OpenSSL crate** — link 失敗；要 `rustls` 替代或裝 `openssl-dev`。
3. **時區** — distroless 沒 tzdata；用 chrono 的 fixed offset 或在 image 多 COPY zone。
4. **CA 證書** — `distroless/cc` 有；`scratch` 沒，要 COPY `/etc/ssl/certs/ca-certificates.crt`。
5. **build 起來但跑不動** — base 跟 builder 的 glibc 版本不同；保持一致。
6. **`#[allow(dead_code)]` 在生產 build** — 用 `--profile=release-strip` 砍 debug info 再縮 binary。

## 練習

1. 把第 32 章 axum CRUD 容器化，distroless image 跑起來。
2. 嘗試 musl + scratch，比較三種 base 的 image 大小。
3. 寫 docker-compose 一鍵啟動 app + Postgres + Redis。
4. 在 image 加 healthcheck（用 binary 自己的 `--healthcheck` flag）。

## 延伸閱讀

- [cargo-chef](https://github.com/LukeMathWalker/cargo-chef)
- [distroless](https://github.com/GoogleContainerTools/distroless)
- [Best practices for Rust Dockerfiles](https://kerkour.com/rust-small-docker-image)
