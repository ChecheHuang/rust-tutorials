# 48. Kubernetes

> 範圍：deployment / service、liveness / readiness、graceful shutdown、HPA、secret / configmap、kube-rs operator

## Rust app 上 K8s 要做對的幾件事

1. **listen 0.0.0.0**（不是 127.0.0.1）→ container 外連得進
2. **`SIGTERM` 觸發 graceful shutdown**
3. **`/healthz`（liveness）跟 `/readyz`（readiness）分開**
4. **跑 nonroot user**、`readOnlyRootFilesystem: true`
5. **log 出 stdout / stderr**（不要寫檔，K8s 收 stdout）
6. **每個 instance stateless**（state 進 DB / Redis）

範例 `src/main.rs` 對到上面 1–3、5。

## liveness vs readiness

| | liveness | readiness |
|---|----------|-----------|
| 用途 | 還活著嗎？ | 能收流量嗎？ |
| 失敗動作 | 重啟 pod | 從 service endpoint 拔掉 |
| 慢 DB | 不要 fail（重啟沒用） | 該 fail（不該轉發） |
| cold start | 給長 `initialDelaySeconds` | readiness 控制流量進入 |

**規則**：liveness 只檢查程式內部是否卡死（deadlock、event loop hang）；外部依賴失敗用 readiness 反映。

```yaml
livenessProbe:
  httpGet: { path: /healthz, port: 8080 }
  failureThreshold: 3      # 3 次失敗才重啟
readinessProbe:
  httpGet: { path: /readyz, port: 8080 }
  periodSeconds: 2         # 比 liveness 頻繁
```

## Graceful shutdown

```
SIGTERM → readyz 立刻回 503（不收新流量）
      → endpoint controller 把這個 pod IP 移除
      → 等 in-flight requests 處理完
      → 程序退出
```

axum 範例：

```rust
axum::serve(listener, app)
    .with_graceful_shutdown(async {
        signal::ctrl_c().await.unwrap();   // 或 SIGTERM
        state.ready.store(false, Ordering::Relaxed);
        sleep(2 sec).await;                // 給 service 反應時間
    })
    .await?;
```

⚠️ K8s 的 `terminationGracePeriodSeconds`（預設 30s）是上限——超過了直接 SIGKILL。設大一點對長連線安全。

## preStop sleep 是必要邪惡

```yaml
lifecycle:
  preStop:
    exec:
      command: ["/bin/sh", "-c", "sleep 5"]
```

K8s SIGTERM 跟 service endpoint 移除是**並行**的——可能你已經停止接受流量，但 service 還轉新 request 過來。`preStop sleep 5` 讓 endpoint 先更新傳播。

⚠️ distroless 沒 sh → 在程式內收 SIGTERM 後自己 sleep 再開始 shutdown。

## resources 要設

```yaml
resources:
  requests: { cpu: 100m, memory: 64Mi }
  limits:   { cpu: 500m, memory: 256Mi }
```

沒設 = burstable + 隨時被踢；OOM 也不會 alert。**Rust app 記憶體相對小**，不要照搬 Java 設定（512Mi+）；64–256Mi 通常夠。

⚠️ memory limit 觸發 OOMKilled → 沒 graceful。CPU limit 觸發 throttle → tail latency 飆。

## ConfigMap + Secret

```yaml
envFrom:
  - configMapRef: { name: ch48-config }
  - secretRef:    { name: ch48-secrets }
```

Rust 端用 `std::env::var` 或 figment 讀。**secret 不要寫進 ConfigMap**——分清楚。

實務 secret 來源：
- K8s Secret（base64，等同明文）
- Sealed Secrets / SOPS（git-friendly）
- External Secrets Operator（從 Vault / AWS SM 同步）

## HPA（水平自動擴縮）

```yaml
metrics:
  - type: Resource
    resource: { name: cpu, target: { type: Utilization, averageUtilization: 70 } }
```

CPU 是預設指標。Rust 多 worker，CPU 飽和容易；記憶體不太會自動觸發。

進階：custom metric（QPS、queue depth）走 `external` metric API + adapter。

## Helm vs Kustomize

| | Helm | Kustomize |
|---|------|-----------|
| 模型 | template + values | base + overlay |
| 學習曲線 | 中（template 易出錯） | 低（純 YAML patch） |
| 生態 | 巨大（chart 倉庫） | 內建 kubectl |

新 internal service → Kustomize 簡單；要發布給別人裝（chart）→ Helm。

## 寫 K8s operator：kube-rs

Rust 是寫 operator 的好選擇（型別安全、低資源）。

```rust
use kube::{Client, CustomResource};
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Clone, Debug)]
#[kube(group = "demo.io", version = "v1", kind = "Game", namespaced)]
struct GameSpec { replicas: i32 }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    // 用 controller-runtime 風格 watch + reconcile
    // ...
    Ok(())
}
```

完整範例見 `kube-rs` 官方 controller examples。

## 觀察

部署後關注：
- `kubectl logs -f deployment/ch48`
- `kubectl top pod` — CPU / mem
- `kubectl get events --sort-by=.lastTimestamp`
- liveness restart count（高 → 程式不穩）

## 常見陷阱

1. **127.0.0.1 而非 0.0.0.0** — pod 內 listen localhost，service 連不到。
2. **沒 readiness** — deploy 滾動更新會把流量送到還沒 warm 的 pod。
3. **liveness 檢查 DB** — DB 短暫不通 → 全 pod 被重啟 → 雪崩。
4. **沒 graceful shutdown** — 滾動更新切斷現有連線，client 看到 502。
5. **預設 root** — security policy 擋你；image 內 `USER nonroot`。
6. **記憶體小看** — Rust 在 alloc-heavy 場景也會吃；別只給 32Mi。
7. **time skew** — pod 重啟後系統時間正確但業務狀態落後；要從 source of truth 同步。
8. **PodDisruptionBudget 漏設** — node drain 時可能砍掉全部 replica。

## 練習

1. 把 `src/main.rs` 容器化部署到 minikube / kind。觀察 readiness 從 503 → 200。
2. 加 PodDisruptionBudget：`minAvailable: 2`。
3. 寫一個簡單 operator：watch CR `Game`，自動建 Deployment + Service。
4. 用 Prometheus + ServiceMonitor 暴露 metrics（見第 49 章）。

## 延伸閱讀

- [kube-rs](https://kube.rs)
- [Production-Grade Kubernetes](https://kubernetes.io/docs/setup/production-environment/)
- [Graceful shutdown — K8s docs](https://kubernetes.io/docs/concepts/workloads/pods/pod-lifecycle/#pod-termination)
