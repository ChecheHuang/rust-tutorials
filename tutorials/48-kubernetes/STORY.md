# 故事版：大賣場的收銀台和樓管阿姨 🛒

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「大賣場收銀台由樓管阿姨調度」的故事，幫你「感覺」出 Kubernetes 的 Deployment 和 Service、listen `0.0.0.0`、liveness 和 readiness、graceful shutdown 與 preStop、resources、ConfigMap 和 Secret、HPA、securityContext，還有 kube-rs operator 是什麼。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 排班表：隨時都要有三位收銀員

好好大賣場的一樓有一排收銀台。管這一區的是**樓管阿姨**。

阿姨手上有一張**排班表**，上面只寫了一句話：「**隨時都要有 3 位收銀員在崗位上**，穿綠色制服的那種。」

阿姨不在乎是誰來。只要有人請病假，她就馬上找一位新的收銀員補上，**永遠維持 3 位**。

> 這就是 **Deployment**：`k8s/deployment.yaml` 的 `kind: Deployment`、`replicas: 3`、`selector.matchLabels: { app: ch48 }`。每位收銀員是一個 **pod**，用 `image: ghcr.io/your-org/ch48:latest`（第 46 章的便當盒、第 47 章 CI 推上去的那個）。K8s 會一直把「實際」拉回「排班表寫的樣子」。

---

## 「結帳請往這裡」

客人不需要知道今天是哪三位收銀員。賣場入口掛著一塊**指示牌**：「**結帳請往這裡**」。

指示牌旁邊站著一位**引導員**，他看誰穿綠色制服，就把客人帶過去。

> 這就是 **Service**：`kind: Service`，`selector: { app: ch48 }` 找出符合標籤的 pod，`port: 80` → `targetPort: 8080`，`type: ClusterIP` 是叢集內部的固定入口。pod 換來換去，客人只要記得 Service 的名字。

---

## 窗口要朝外開

新來的收銀員小方第一天上班，把收銀機的**窗口轉向牆壁**，只有站在櫃台裡面的自己看得到。結果引導員帶客人過來，客人根本付不了錢。

阿姨幫她把窗口轉回來：「**要朝外開**，客人才找得到你。」

> 這就是 **listen `0.0.0.0`**：`src/main.rs` 的 `TcpListener::bind("0.0.0.0:8080")`。如果寫 `127.0.0.1`，只有 container 自己連得到，Service 永遠連不進來。

---

## 「營業中」燈和「你還醒著嗎？」

每個收銀台上有一盞小燈：

- **綠燈「營業中」**：引導員才會帶客人過來。
- **紅燈「暫停服務」**：引導員會跳過這一台。

收銀員剛上班時，要先**點零錢、開機、登入**。這段時間燈是紅的，等準備好才切成綠的。

另外，阿姨每 10 秒會經過問一聲：「**你還醒著嗎？**」只要收銀員回一句「**醒著**」就好。如果連續 **3 次**都沒回應（可能站著睡著了），阿姨就請她下去，換一位新的。

> 這就是 **readiness 和 liveness**：`readyz` 讀 `State { ready: Arc<AtomicBool> }`，`true` 回 `200 "ready"`，`false` 回 `503 SERVICE_UNAVAILABLE`；`readinessProbe` 每 `periodSeconds: 2` 檢查一次，失敗就從 Service 拔掉（不重啟）。`main` 一開始 `ready` 是 `false`，一個 task `sleep` 2 秒模擬 warm-up 才 `store(true)`。`healthz` 只回 `"alive"`；`livenessProbe` 每 10 秒、`failureThreshold: 3`，失敗就**重啟 pod**。

### 刷卡機斷線不是收銀員的錯

有一天，整間賣場的**刷卡機連線**斷了一下。如果阿姨的規則是「收銀員刷不了卡 = 睡著了」，她就會把**三位收銀員全部換掉**——但換人根本沒用，刷卡機還是斷的，而且新人還要重新點零錢。

所以阿姨只問「你還醒著嗎」。刷卡機斷線時，收銀員自己把燈切成**紅燈**就好。

> 這就是 **liveness 不要檢查外部依賴**：liveness 只看程式本身有沒有卡死（deadlock、event loop hang）；DB 慢或斷線要反映在 **readiness**。不然 DB 一抖，全部 pod 被重啟，就是雪崩。

---

## 下班鈴響了

晚上換班，阿姨按下**下班鈴**。收銀員小方不會丟下排到一半的客人就跑。她照順序做：

1. **先把燈切成紅燈**——不要再有新客人過來。
2. **把手上正在結帳的客人結完。**
3. 收好東西，下班。

> 這就是 **graceful shutdown**：`shutdown()` 用 `tokio::select!` 等 `signal::ctrl_c()` 或 unix 的 `SignalKind::terminate()`（SIGTERM）；收到後 `state.ready.store(false, ...)` 讓 `/readyz` 變 503，再 `sleep` 2 秒讓進行中的請求結束。`axum::serve(listener, app).with_graceful_shutdown(shutdown(state))` 把它接上。

### 引導員反應比較慢

麻煩的是：下班鈴響和引導員**聽到消息**是**同時**開始的。小方已經切紅燈了，引導員可能還沒注意到，又帶了一位客人過來。

所以阿姨規定：「鈴響之後，先**站著等 5 秒**再開始收攤，讓引導員有時間知道。」

可是有些收銀台是極簡款，**沒有附碼錶**（沒辦法執行那個「等 5 秒」的指令），那就只好收銀員自己在心裡數。

另外，阿姨最多只等 **30 秒**。30 秒後還沒走，警衛會直接請你離開，手上的客人也只能放下。

> 這就是 **preStop 和 terminationGracePeriodSeconds**：`lifecycle.preStop.exec.command: ["/bin/sh", "-c", "sleep 5"]` 讓 endpoint 移除先傳播（SIGTERM 和移除 endpoint 是**並行**的）。distroless 沒有 `sh`，就改在程式收到 SIGTERM 後自己 sleep。`terminationGracePeriodSeconds: 30` 是上限，超過就 **SIGKILL**。

---

## 每個收銀台的空間和零錢箱

阿姨幫每一台收銀台訂好規格：

- **至少保證**有這麼大的空間和這麼多零錢（她排位置時用這個算）。
- **最多**只能用到這麼大。

如果某位收銀員的東西多到**塞爆櫃台**，阿姨會二話不說把她請走（連跟客人說再見都來不及）。如果她**動作太快想多做**，阿姨會讓她放慢一點，後面的客人就會等比較久。

> 這就是 **resources**：`requests: { cpu: 100m, memory: 64Mi }`、`limits: { cpu: 500m, memory: 256Mi }`。超過 memory limit → **OOMKilled**（沒有 graceful）；超過 CPU limit → **throttle**，tail latency 飆高。Rust 記憶體用量小，64–256Mi 通常夠，別照搬 Java 的 512Mi+。

---

## 公告欄和保險箱

每位收銀員上班前要知道兩類事：

- **今天的公告**：「今天全館九折」「零錢放第二格」——貼在**員工公告欄**，誰都可以看。
- **保險箱密碼**——不能貼在公告欄！要放在另一個**上鎖的信封**裡。

收銀員只要讀**自己需要**的東西，不用把資料**寫在自己腦袋裡帶回家**，也不在櫃台上偷改裝潢。今天的帳都記在**總帳**裡，所以換誰來上班都一樣。

> 這就是 **ConfigMap 與 Secret**：`envFrom: configMapRef` / `secretRef`，Rust 端用 `std::env::var`（像 `RUST_LOG`）讀取。**secret 不要放 ConfigMap**；K8s Secret 只是 base64，正式環境用 Sealed Secrets / SOPS / External Secrets Operator。程式要 **stateless**（state 放 DB / Redis），log 寫到 stdout。`securityContext` 的 `runAsNonRoot: true`、`runAsUser: 65532`、`readOnlyRootFilesystem: true`、`capabilities: { drop: [ALL] }` 就是「不准改裝潢、不拿萬用鑰匙」。

---

## 客人一多就加開收銀台

週末下午，客人大排長龍。阿姨看了一眼：「三位收銀員都忙到 **七成以上**了！」馬上加開第四台、第五台……最多開到 **10 台**。人潮退了，又慢慢收回到 **3 台**。

> 這就是 **HPA（HorizontalPodAutoscaler）**：`scaleTargetRef` 指向 Deployment `ch48`，`minReplicas: 3`、`maxReplicas: 10`，`averageUtilization: 70`（CPU）。更進階可以用 QPS、queue depth 等 custom / external metric。

### 裝修時至少留兩台

有一次，賣場要**整修一樓地板**，工人想把收銀台一次全部搬走。阿姨擋住：「不行，**至少要留 2 台**在營業。」

> 這就是 **PodDisruptionBudget**（例如 `minAvailable: 2`）：node drain 時避免一次砍光所有 replica。

---

## 排班表的兩種寫法

賣場有好幾家分店。總公司寫排班表有兩種方法：

- **填空模板**：一張表格，空格處寫「收銀員人數：＿＿」，各分店填自己的數字。
- **基本版加補丁**：先寫一份**基本版**，各分店再貼一張小補丁：「本店改成 5 位」。

> 這就是 **Helm vs Kustomize**：Helm 是 template + values，生態大、適合發布 chart 給別人裝；Kustomize 是 base + overlay（純 YAML patch），內建在 kubectl，內部服務用它比較簡單。

---

## 專門管遊戲區的小主管

賣場新開了一個**兒童遊戲區**。總公司發明了一張**新表單**：「遊戲區需要 N 位服務員」。

樓管阿姨看不懂這張新表單，於是賣場請了一位**專屬小主管**。他整天做一件事：**看表單上寫幾位 → 看現場實際有幾位 → 不一樣就補人或請人回去**，一遍又一遍。

> 這就是 **operator 與 kube-rs**：用 `#[derive(CustomResource)]` + `#[kube(group = "demo.io", version = "v1", kind = "Game", namespaced)]` 定義 `GameSpec { replicas }` 這種自訂資源（CRD），`Client::try_default()` 連上叢集，再寫 **watch + reconcile** 迴圈，把實際狀態拉向期望狀態。Rust 型別安全、資源用量低，很適合寫 operator。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust / K8s 概念 |
|------|------|
| 樓管阿姨 | **Kubernetes** control plane |
| 「隨時 3 位收銀員」排班表 | **Deployment** `replicas: 3` |
| 一位收銀員 | **pod** |
| 綠色制服 | **label** `app: ch48` |
| 「結帳請往這裡」指示牌 + 引導員 | **Service**（ClusterIP，80 → 8080） |
| 窗口朝外開 | **`bind("0.0.0.0:8080")`** |
| 營業中 / 暫停服務燈 | **readiness**：`/readyz` + `AtomicBool` |
| 開機點零錢 | warm-up 後 `ready.store(true)` |
| 「你還醒著嗎？」 | **liveness**：`/healthz` |
| 3 次沒回應就換人 | `failureThreshold: 3` → 重啟 pod |
| 刷卡機斷線別換人 | liveness 不檢查 DB |
| 下班鈴 | **SIGTERM** |
| 先切紅燈、結完客人再走 | **graceful shutdown**：`with_graceful_shutdown(shutdown(state))` |
| 鈴響先等 5 秒 | **preStop** `sleep 5` |
| 最多等 30 秒，警衛請走 | `terminationGracePeriodSeconds: 30` → SIGKILL |
| 櫃台空間與零錢箱 | **resources** requests / limits |
| 塞爆櫃台被請走 | **OOMKilled** |
| 員工公告欄 | **ConfigMap** |
| 上鎖的密碼信封 | **Secret** |
| 不准改裝潢、不拿萬用鑰匙 | **securityContext**（nonroot、read-only） |
| 帳記在總帳 | **stateless** pod |
| 客人多就加開 | **HPA**（3–10，CPU 70%） |
| 裝修至少留 2 台 | **PodDisruptionBudget** |
| 填空模板 / 基本版加補丁 | **Helm** / **Kustomize** |
| 遊戲區新表單 | **CRD**：`kind = "Game"` |
| 專屬小主管 | **operator**（kube-rs reconcile） |

---

## 一句話記住

> **Kubernetes 是大賣場的樓管阿姨：照排班表永遠補滿收銀員（Deployment），客人看指示牌就找得到（Service）；「醒著嗎」只問有沒有睡著（liveness），「營業中」燈決定要不要帶客人來（readiness）——下班鈴響先切紅燈、結完手上的客人再走（graceful shutdown）。**
