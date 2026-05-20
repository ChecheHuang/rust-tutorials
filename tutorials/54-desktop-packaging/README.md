# 54. 跨平台打包與簽章

> 範圍：`tauri build`、Windows 簽章、macOS notarization、Linux AppImage / Flatpak、自動更新（updater）、CI release pipeline

## 打包目標

```bash
npm run tauri build
# 自動產生對應 OS 的 installer
```

| OS | 預設輸出 |
|----|----------|
| Windows | `.msi`（WiX）+ `.exe`（NSIS 從 v2.0） |
| macOS | `.dmg` + `.app` |
| Linux | `.deb` + `.AppImage` + `.rpm`（可選） |

## Windows 簽章

未簽 → SmartScreen 警告 → 安裝率掉 60%+。

### 簽章流程

1. **買 code signing 憑證**（DigiCert / Sectigo，個人證書 ~$200/yr、EV 證書 ~$300/yr）
2. **EV 證書**比一般證書好：立刻有 SmartScreen reputation；一般要 build reputation（裝過多少次）
3. **Token-based** 簽章（HSM token 或雲端 KMS）— 現在大多廠商不再賣可導出的 .pfx

### tauri.conf.json 設定

```json
"bundle": {
  "windows": {
    "certificateThumbprint": "ABCD1234...",
    "digestAlgorithm": "sha256",
    "timestampUrl": "http://timestamp.digicert.com"
  }
}
```

或在 CI 內用 `signtool` 後置簽（更彈性，可掛 azure-keyvault-sign）。

### Microsoft Store

也是一條路：MSIX 包 + Microsoft Partner Center。免 SmartScreen 警告但要審。

## macOS notarization

### 三步：sign → notarize → staple

1. **codesign**：用 Apple Developer ID 簽
2. **notarize**：上傳 Apple，自動掃毒
3. **staple**：把 notarization ticket 嵌入 binary（離線可驗）

```json
"bundle": {
  "macOS": {
    "signingIdentity": "Developer ID Application: Your Name (TEAMID)",
    "providerShortName": null,
    "entitlements": "entitlements.plist"
  }
}
```

CI 上設環境變數：
```
APPLE_CERTIFICATE         # base64 .p12
APPLE_CERTIFICATE_PASSWORD
APPLE_SIGNING_IDENTITY    # 同 tauri.conf
APPLE_ID
APPLE_PASSWORD            # app-specific password
APPLE_TEAM_ID
```

tauri-action 會自動 notarize。

### entitlements

請求權限（沙箱、camera、mic）：

```xml
<plist>
<dict>
  <key>com.apple.security.network.client</key>
  <true/>
  <key>com.apple.security.device.usb</key>
  <true/>
</dict>
</plist>
```

USB / HID app 需要 `device.usb`。

### App Store 分發

需要 sandbox + 額外 entitlements + App Store review。較嚴。多數 indie 用 notarized 分發（自己 download）。

## Linux 三大發行方式

### 1. .deb / .rpm

```json
"linux": {
  "deb": { "depends": ["libwebkit2gtk-4.1-0", "libgtk-3-0"] }
}
```

優點：原生套件管理。缺點：每個 distro 一份 (Ubuntu 22 / 24、Debian、Fedora ...)。

### 2. AppImage

單檔執行 — 不裝套件、不要 root：
```bash
./MyCoolApp_1.0.0_amd64.AppImage
```

⚠️ AppImage 對 webview 依賴 host 的 libwebkit2gtk，**版本不對直接掛**。多 distro 兼容性偶有問題。

### 3. Flatpak / Snap

最 portable 但 build 過程繁瑣，需要 manifest（flathub）。

實務建議：發 AppImage + .deb 涵蓋 95% 使用者。Flathub 進階做。

## Auto Updater

Tauri v2 內建 updater plugin：

```json
"plugins": {
  "updater": {
    "endpoints": [
      "https://releases.example.com/myapp/{{target}}/{{current_version}}"
    ],
    "pubkey": "BASE64_ED25519_PUBKEY"
  }
}
```

### 流程

```
1. 發 binary + .sig（用 ed25519 私鑰簽）
2. 寫 manifest JSON：{ version, notes, signature, url }
3. 放到 endpoint server
4. App 啟動或定時 check → 下載 → 驗 signature → 安裝
```

### 產 key

```bash
npm run tauri signer generate -- -w ~/.tauri/myapp.key
# 私鑰保留、public key 塞進 tauri.conf
```

### CI 簽 release

```bash
export TAURI_SIGNING_PRIVATE_KEY=<base64 private key>
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=<password>
npm run tauri build
# 自動產 .tar.gz + .sig（或 .nsis.zip + .sig）
```

### Manifest 範例

```json
{
  "version": "1.0.1",
  "notes": "bug fixes",
  "pub_date": "2026-05-20T10:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "...base64...",
      "url": "https://releases.example.com/myapp/MyCoolApp_1.0.1_aarch64.tar.gz"
    },
    "windows-x86_64": {
      "signature": "...base64...",
      "url": "https://releases.example.com/myapp/MyCoolApp_1.0.1_x64-setup.nsis.zip"
    }
  }
}
```

## CI release pipeline（GitHub Actions）

```yaml
name: release
on:
  push:
    tags: ["v*"]

jobs:
  publish:
    strategy:
      matrix:
        include:
          - platform: macos-latest
            args: "--target aarch64-apple-darwin"
          - platform: macos-latest
            args: "--target x86_64-apple-darwin"
          - platform: ubuntu-22.04
            args: ""
          - platform: windows-latest
            args: ""
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: 20 }
      - uses: dtolnay/rust-toolchain@stable
      - run: npm ci
      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "MyCoolApp ${{ github.ref_name }}"
          args: ${{ matrix.args }}
```

## icon

`src-tauri/icons/` 需要：
- 32×32.png
- 128×128.png + @2x（256×256）
- icon.icns（macOS — 多解析度容器）
- icon.ico（Windows — 多解析度容器）

Tauri CLI：
```bash
npm run tauri icon path/to/source.png
```

自動產所有尺寸。source 最好 1024×1024 PNG。

## CSP（內容安全政策）

```json
"security": {
  "csp": "default-src 'self'; connect-src ipc: https://api.example.com"
}
```

- `'self'` 包含 `tauri://`
- 連 IPC 一定要 `ipc:`
- prod CSP 嚴格、dev 可放寬

## 版本管理

- `version` 在 `tauri.conf.json`、`package.json`、`Cargo.toml` 都要一致
- Tauri 2 有 `tauri-version` plugin 同步
- semver：bug fix = patch、新功能不破壞 = minor、break = major

## 常見陷阱

1. **沒簽 Windows binary** — SmartScreen 嚇跑用戶。
2. **macOS notarize 失敗** — 多半因為缺 `entitlements.plist` 或用了 hardened runtime 禁的 API（如 `dlopen` 動態載入）。
3. **AppImage 在新 distro 跑不動** — webkit2gtk 版本；可發多版或要求 Ubuntu 22.04+。
4. **updater pubkey 不對** — 簽出來的 .sig 驗證失敗；private/public 對齊。
5. **bundle target 寫死** — `targets: "all"` 跨平台行為不同；CI 多 job 各自跑。
6. **GH release token 權限不足** — `tauri-action` 需要 `contents: write`。
7. **icon 不全** — 缺尺寸 build 出來圖示糊；用 `tauri icon` 一次補齊。
8. **時間戳 server 掛了** — Windows 簽名沒蓋時間戳，憑證過期後 binary 失效。

## 練習

1. 在本機 build 一個 Tauri app 三平台 installer。
2. 設定 GitHub Actions tag-driven release，產出 macOS / Windows / Linux artifact。
3. 接 updater：發 v1.0.0、然後 v1.0.1，看 app 自動升級流程。
4. 加 CSP 嚴格化：只允許 api.example.com 外部連線。

## 延伸閱讀

- [Tauri Distribute](https://tauri.app/distribute/)
- [Tauri Updater Plugin](https://tauri.app/plugin/updater/)
- [Apple Notarization](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [tauri-action](https://github.com/tauri-apps/tauri-action)
