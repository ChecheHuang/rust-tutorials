# 43. gRPC（tonic）

> 範圍：proto + build.rs codegen、unary、server stream、interceptor、HTTP/2 細節

## 為什麼 gRPC

| 方面 | REST/JSON | gRPC |
|------|-----------|------|
| Schema | OpenAPI（事後寫） | .proto（先寫） |
| 編碼 | text JSON | binary protobuf |
| 傳輸 | HTTP/1.1 | HTTP/2（multiplex） |
| streaming | SSE / WebSocket | 內建 4 種（unary / server / client / bidi） |
| 適用 | 對外 API、瀏覽器 | 內部 service-to-service |

**結論**：對外 REST + 對內 gRPC 常見組合。瀏覽器要 gRPC 需 `grpc-web` proxy（envoy / grpc-gateway）。

## 專案結構

```
proto/
  greeter.proto          ← service + message 定義
src/
  server.rs
  client.rs
build.rs                 ← 編譯時跑 tonic-build 生 Rust code
Cargo.toml
```

## .proto

```protobuf
syntax = "proto3";
package greeter;

service Greeter {
  rpc SayHello (HelloRequest) returns (HelloReply);
  rpc SayHelloStream (HelloRequest) returns (stream HelloReply);
}

message HelloRequest  { string name = 1; }
message HelloReply    { string message = 1; }
```

## build.rs

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/greeter.proto")?;
    Ok(())
}
```

Cargo build 時自動跑、生 `OUT_DIR/greeter.rs`。在 server / client 用：

```rust
pub mod greeter {
    tonic::include_proto!("greeter");
}
```

## Service impl

```rust
#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        req: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = req.into_inner().name;
        Ok(Response::new(HelloReply { message: format!("Hello, {name}!") }))
    }
}
```

`Status` 是 gRPC 錯誤型別（不像 HTTP 用 number，gRPC 用名字 status code + message + details）。

## 四種 RPC 風格

| 風格 | client | server |
|------|--------|--------|
| unary | 1 req | 1 resp |
| server stream | 1 req | stream resp |
| client stream | stream req | 1 resp |
| bidi stream | stream req | stream resp |

Server stream 例：傳 `mpsc::Receiver` 包成 `Stream`。Client stream / bidi 需要從 `Request<Streaming<T>>` 讀。

## interceptor（middleware）

```rust
fn auth_intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
    let token = req.metadata().get("authorization")
        .ok_or_else(|| Status::unauthenticated("no token"))?;
    // verify
    Ok(req)
}

Server::builder()
    .add_service(GreeterServer::with_interceptor(svc, auth_intercept))
    .serve(addr).await?;
```

client 端也類似，可以在每個 request 自動帶 token。

## 與 tower 整合

tonic 底層就是 tower service。可套 tower-http layer：

```rust
let layer = ServiceBuilder::new()
    .layer(TraceLayer::new_for_grpc())
    .layer(TimeoutLayer::new(Duration::from_secs(5)));

Server::builder()
    .layer(layer)
    .add_service(svc)
    .serve(addr).await?;
```

## 跑這個範例

```bash
cargo run --bin ch43-server &     # 監聽 :50051
cargo run --bin ch43-client       # 跑 client
```

## protobuf 進階

- **`google.protobuf.Timestamp / Duration`** → 在 .proto `import "google/protobuf/timestamp.proto";`，Rust 端對到 `prost_types::Timestamp`。
- **enum** → Rust `i32` + `try_from` 轉具體 enum。
- **oneof** → Rust `Option<enum>`。
- **map** → Rust `HashMap`。
- **optional**（proto3）→ Rust `Option<T>`。

## 反射、健康檢查

- `tonic-reflection` 提供 server reflection（給 `grpcurl` 用）
- `tonic-health` 提供 `grpc.health.v1.Health` 標準健檢

兩個都建議在生產環境加。

## TLS

```rust
let cert = std::fs::read_to_string("server.pem")?;
let key = std::fs::read_to_string("server.key")?;
let identity = Identity::from_pem(cert, key);

Server::builder()
    .tls_config(ServerTlsConfig::new().identity(identity))?
    .add_service(svc)
    .serve(addr).await?;
```

## 常見陷阱

1. **`include_proto!` 名要對應 package** — `package greeter;` → `include_proto!("greeter")`。
2. **build.rs 沒改也要 rebuild** — 加 `println!("cargo:rerun-if-changed=proto/x.proto");` 自動觸發。
3. **`Status` 不是 HTTP 狀態碼** — `Status::unauthenticated` 對應 16，不是 401。
4. **stream backpressure** — server stream 用 `mpsc(N)` buffer 大小要思考。
5. **HTTP/2 plaintext (h2c)** — 部分 proxy 不支援；production 用 TLS。
6. **proto 改了 client / server 沒同步** — gRPC 對 schema 漂移容忍度有限；CI 要鎖 .proto。
7. **大訊息** — 預設限制 4MB，要 `max_decoding_message_size(...)`。

## 練習

1. 加 client stream + bidi stream RPC。
2. 寫一個 auth interceptor（client 帶 JWT、server 驗）。
3. 整合 `tonic-reflection`，用 `grpcurl -plaintext localhost:50051 list` 試。
4. 比較同樣 service 用 REST vs gRPC 的 latency（用 criterion）。

## 延伸閱讀

- [tonic](https://docs.rs/tonic)
- [Buf](https://buf.build/) — proto schema management
- [gRPC 官方文件](https://grpc.io/docs/)
