# 故事版：手工餅乾要上架到三個國家 📦

> 這篇是**寫給小朋友也看得懂**的故事版。用一個「手工餅乾店要把餅乾包裝好、蓋上防偽章、送進三個國家的超市，之後還要寄新口味」的故事，幫你「感覺」出 `tauri build`、Windows 簽章與時間戳、macOS 的 sign → notarize → staple、entitlements、Linux 的 deb / AppImage / Flatpak、auto updater 的簽名驗證、CI release pipeline、icon、CSP 和版本管理。故事裡不講程式名詞，名詞都放在 `>` 小框框和最後的對照表裡。先享受故事，別急著背。

---

## 一台會看國家的包裝機

阿寶的手工餅乾很好吃，三個國家的超市都想進貨：**窗窗國**、**蘋果國**、**企鵝國**。

每個國家的貨架規格都不一樣。還好阿寶有一台**包裝機**：在哪個國家的工廠按下按鈕，它就包出那個國家要的盒子。

- 窗窗國：兩種盒子，一種大盒、一種小盒。
- 蘋果國：一個有拉鍊的袋子，裡面裝著餅乾罐。
- 企鵝國：好幾種，有超市專用的、也有「不用上架、拿了就能吃」的。

> 這就是 **`npm run tauri build`**：在當前平台產生 installer。Windows 是 `.msi`（WiX）+ `.exe`（NSIS）；macOS 是 `.dmg` + `.app`；Linux 是 `.deb` + `.AppImage`（`.rpm` 可選）。設定寫在 `tauri.conf.json` 的 `bundle`，範例 `tauri.conf.example.json` 裡 `"targets": "all"`。

---

## 窗窗國：沒有防偽章，店員會貼警告

阿寶第一次送貨到窗窗國，超市店員看了看盒子：「這盒子**沒有防偽章**耶。」然後在上面貼了一張大大的黃色貼紙：「**來路不明，購買請小心**」。結果大部分客人都不敢買。

阿寶去找認證公司買了一枚**防偽印章**。印章有兩種：

- **一般章**：剛開始還是會被貼警告，要等很多客人買過、口碑累積起來才會消失。
- **老字號章**：比較貴，但**一蓋上去，店員馬上就相信**。

現在認證公司已經不太讓人把印章帶回家了，而是鎖在一個**特製的保險盒**裡，要蓋章得去保險盒那邊蓋。

> 這就是 **Windows 程式碼簽章**：沒簽會被 **SmartScreen** 警告。憑證向 DigiCert / Sectigo 購買，**EV 憑證**能立刻有 reputation；現在多半是 HSM token 或雲端 KMS，不再賣可匯出的 `.pfx`。設定在 `bundle.windows` 的 `certificateThumbprint`、`digestAlgorithm: "sha256"`，也可以在 CI 用 `signtool` 後置簽。

---

## 蓋章時順便蓋郵戳

認證公司提醒阿寶：「你的防偽章**一年後會過期**。如果盒子上只有章，過期之後超市會說這盒子不算數。」

所以每次蓋防偽章時，阿寶都會再請**郵局蓋一個日期郵戳**，證明「這盒是在印章還有效的時候蓋的」。這樣就算印章後來過期，以前蓋好的盒子還是有效。

> 這就是 **時間戳（timestamp）**：`"timestampUrl": "http://timestamp.digicert.com"`。沒蓋時間戳，憑證過期後 binary 的簽章就會失效。時間戳伺服器掛了也會讓簽章出問題。

---

## 蘋果國：簽名、送檢、釘合格貼紙

蘋果國更嚴格，要三個步驟：

1. **簽名**：阿寶用蘋果國發給他的**開發者名牌**在盒子上簽名。
2. **送檢**：把盒子寄去蘋果國的**檢驗局**，檢驗局自動掃描有沒有壞東西。
3. **釘貼紙**：檢驗通過後，把**合格貼紙**直接釘在盒子上。這樣超市就算**沒網路、打不了電話去檢驗局問**，看貼紙也知道是合格的。

另外，如果餅乾盒裡附了一個「可以接上 USB 的小玩具」，阿寶還要事先填一張**許可申請單**，寫明「本產品需要使用 USB」。

> 這就是 **macOS 的 codesign → notarize → staple**：`bundle.macOS.signingIdentity` 設 `"Developer ID Application: Your Name (TEAMID)"`；notarize 上傳 Apple 掃描；staple 把 ticket 嵌進 app，離線也能驗。**entitlements.plist** 申請權限，USB / HID app 需要 `com.apple.security.device.usb`。CI 上用 `APPLE_CERTIFICATE`、`APPLE_ID`、`APPLE_TEAM_ID` 等環境變數，tauri-action 會自動 notarize。

---

## 企鵝國：超市貨架、隨身便當盒、大型百貨

企鵝國有好幾種賣法：

- **超市專用盒**：每家連鎖超市的貨架尺寸都不同，阿寶得一家一家做。盒子上還要寫「本產品需要搭配某某牌餐具」。
- **隨身便當盒**：一個盒子裝好全部，拿了就能吃，不用上架、不用店長同意。不過裡面沒附餐具，用的是**客人家裡的餐具**——家裡餐具款式太舊或太新，就可能吃不了。
- **大型百貨專櫃**：哪裡都能賣，但進駐手續非常繁瑣。

阿寶最後決定：先做「超市專用盒 + 隨身便當盒」，就能照顧到大部分客人。

> 這就是 **Linux 的三種發行方式**：`.deb` / `.rpm` 走原生套件管理，`bundle.linux.deb.depends` 寫 `["libwebkit2gtk-4.1-0", "libgtk-3-0"]`；**AppImage** 單檔執行、不用 root，但依賴 host 的 `libwebkit2gtk`，版本不對就跑不動；**Flatpak / Snap** 最 portable 但要寫 manifest。實務上 AppImage + `.deb` 涵蓋大多數使用者。

---

## 寄新口味：蠟封印章和比對印模

阿寶推出新口味後，不想讓客人每次都跑去超市換。他讓每盒餅乾裡都附一張**小卡**，小卡會定期去阿寶的公告欄看看：「有新口味嗎？」

可是壞人也可能假冒阿寶寄東西。所以阿寶準備了一對東西：

- 一枚只有他自己有的**蠟封印章**，每次寄新口味都蓋在封口上。
- 一個**印模**，早就放在每盒餅乾的小卡裡。

新口味寄到時，小卡會拿印模比對封口的蠟封：**對得上**才拆開換上；對不上就丟掉。印章要好好藏起來，**印模給誰都沒關係**。

> 這就是 **auto updater**：`plugins.updater.endpoints` 填 `https://releases.example.com/mycoolapp/{{target}}/{{current_version}}`，`pubkey` 放 ed25519 公鑰（印模）。`npm run tauri signer generate` 產生金鑰；CI 用 `TAURI_SIGNING_PRIVATE_KEY`（印章）簽出 `.sig`。manifest JSON 寫 `version`、`notes`、`pub_date`、每個平台的 `signature` 和 `url`。App 下載後**驗簽**才安裝；公私鑰對不上就會驗證失敗。

---

## 中央工廠：一喊開工，三國一起包

以前阿寶要自己飛到三個國家的工廠按按鈕。後來他設了一間**中央工廠**：

只要他在新口味的盒子上**貼一張「v1.0.1」的標籤**，中央工廠就自動在**四條產線**同時開工——蘋果國的兩種機型、企鵝國、窗窗國，各自包好、蓋章、送檢，最後全部放到同一個**發貨架**上。

蓋章用的印章和密碼，都鎖在工廠的**保險櫃**裡，產線要用時才拿出來。

> 這就是 **CI release pipeline**：GitHub Actions `on: push: tags: ["v*"]` 觸發，`strategy.matrix` 列出 macOS（aarch64 / x86_64）、ubuntu-22.04、windows 四個 job，用 `tauri-apps/tauri-action@v0` build、簽章並上傳到 GitHub Release。憑證和私鑰放在 `secrets.*`；token 需要 `contents: write` 權限。

---

## 每種尺寸的包裝圖

不同超市要的包裝圖尺寸都不一樣：小標籤、中標籤、蘋果國的多尺寸圖冊、窗窗國的多尺寸圖冊……

阿寶只畫一張**很大、很清楚的原圖**，交給影印店，影印店就把所有尺寸**一次印齊**。少一種尺寸，那家超市的圖就會糊糊的。

> 這就是 **icon**：`src-tauri/icons/` 需要 `32x32.png`、`128x128.png`、`128x128@2x.png`、`icon.icns`、`icon.ico`。用 `npm run tauri icon path/to/source.png` 從 1024×1024 的原圖自動產生。

---

## 盒子裡只能放清單上的東西

阿寶規定：餅乾盒裡**只能裝自家做的東西**，外面的東西只有清單上那一家「api 原料行」可以送進來，其他一律不收。

最後還有一件小事：盒子正面、背面、說明書上都印著**版本號**，三個地方**一定要寫一樣**，不然客人會搞不清楚自己買的是哪一版。

> 這就是 **CSP** 和 **版本管理**：`"csp": "default-src 'self'; img-src 'self' data:; connect-src ipc: https://api.example.com"`，`'self'` 包含 `tauri://`，連 IPC 一定要 `ipc:`。`version` 在 `tauri.conf.json`、`package.json`、`Cargo.toml` 要一致，照 semver：patch 修 bug、minor 加功能、major 破壞相容。

---

## 回到現實：故事 ↔ Rust 對照表

| 故事裡的東西 | 真正的 Rust / Tauri 概念 |
|------|------|
| 會看國家的包裝機 | **`npm run tauri build`** + `bundle.targets` |
| 窗窗國的大盒、小盒 | `.msi`（WiX）、`.exe`（NSIS） |
| 「來路不明」黃色貼紙 | **SmartScreen** 警告 |
| 防偽印章 | **code signing 憑證** |
| 老字號章 | **EV 憑證** |
| 保險盒裡的印章 | HSM token / 雲端 KMS |
| 日期郵戳 | **timestampUrl** |
| 簽名 → 送檢 → 釘貼紙 | **codesign → notarize → staple** |
| USB 許可申請單 | **entitlements**（`device.usb`） |
| 超市專用盒 | `.deb` / `.rpm`（`depends`） |
| 隨身便當盒、用客人家的餐具 | **AppImage**（依賴 host webkit2gtk） |
| 大型百貨專櫃 | **Flatpak / Snap** |
| 盒裡的小卡定期查公告欄 | **updater** `endpoints` |
| 蠟封印章 / 印模 | ed25519 **私鑰** / **公鑰**（`pubkey`） |
| 封口上的蠟封 | `.sig` 簽名檔 |
| 貼標籤就開工的中央工廠 | **GitHub Actions** tag 觸發 + `matrix` |
| 保險櫃 | **`secrets.*`** |
| 一張大原圖印齊所有尺寸 | **`tauri icon`** |
| 只收清單上的原料行 | **CSP**（`connect-src ipc: https://api.example.com`） |
| 三個地方的版本號一致 | `tauri.conf.json` / `package.json` / `Cargo.toml` 的 **version** |

---

## 一句話記住

> **包裝機照國家包盒子；窗窗國要防偽章加郵戳、蘋果國要簽名送檢釘貼紙、企鵝國先做超市盒和便當盒；新口味用私藏的蠟封章蓋、讓盒裡的印模比對——貼上版本標籤，中央工廠就三國一起開工。**
