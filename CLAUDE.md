# EasyTier + WOLPlus — Quick Reference

## Project
Tauri v2 Android app (`easytier-gui`) — WOL device management + EasyTier VPN control.  
Detail: see `project.md`.

## Constraints
- **Never modify `easytier/` (easytier-core)** — router uses official binary
- **Minimal changes to `easytier-gui`** — preserve `index.vue` for upstream merge
- `luci-app-wolplus` is the WOL middleware on router (`/cgi-bin/wolplus-api`)

## File Map
| Path | Role |
|------|------|
| `easytier-gui/src/pages/home.vue` | **Main dev target** — WOL / Network / Settings tabs |
| `easytier-gui/src/composables/backend.ts` | Tauri IPC wrappers (`httpGet`, `httpPost`, `tcpPing`, etc.) |
| `easytier-gui/src-tauri/src/lib.rs` | Rust Tauri commands (`http_get`, `http_post`, `tcp_ping`) |
| `easytier-gui/src-tauri/Cargo.toml` | Rust deps (reqwest with `socks` feature) |
| `easytier-gui/src-tauri/gen/android/.../MainActivity.kt` | Android activity (text selection fix) |
| `easytier-gui/src/composables/mobile_vpn.ts` | VPN service setup (routes, `disallowedApplications`) |

## Build (3 steps, do NOT skip any)
```bash
# 1. Vite (bash OK)
export PATH="$HOME/.cargo/bin:$HOME/.npm-global:$PATH"
cd easytier-gui && pnpm vite build

# 2. Cargo (bash OK)
#    ALL cross-compile env vars (CC/AR/BINDGEN/PROTOC/LIBCLANG_PATH/linker)
#    are in .cargo/config.toml [env] — no export needed
cd src-tauri && cargo build --lib --release --target aarch64-linux-android --features "tauri/custom-protocol"

# 3. Gradle — MUST use PowerShell (NOT bash):
$env:ANDROID_HOME = "D:/tools/Android/Sdk"
$env:JAVA_HOME = "D:/tools/Java/jdk-17.0.12"
$env:GRADLE_USER_HOME = "D:/tmp/gradle-tmp"
cd gen/android; .\gradlew.bat assembleArm64Release --no-daemon

# Copy to test directory:
Copy-Item -Force "...\app-arm64-release.apk" "F:\tmp\app-arm64-release.apk"
```

**Why not bash for Gradle**: MSYS2 file ops conflict with Windows NTFS locks → `Could not move temporary workspace`.

**Why 3 steps always**: Tauri build.rs embeds `dist/` into `.so`. Frontend change without Cargo → old UI in APK.

**Incremental cache**: ALL cross-compile env vars (CC/AR/BINDGEN/PROTOC/LIBCLANG_PATH) + linker/ar are in `.cargo/config.toml` [env] → stable fingerprints. DO NOT export these in shell — MSYS2 path translation distorts values each run, breaking the fingerprint. Result: easytier stays cached, only easytier-gui recompiles (~1m25s) on frontend changes. No manual `rm` of build artifacts needed—Cargo handles it.

## Key Architecture Decisions

### WOL HTTP routing
App traffic bypasses own VPN (`disallowedApplications` in mobile_vpn.ts) → reqwest can't reach `192.168.2.x` through EasyTier tunnel. **Fix**: route through easytier-core SOCKS5 proxy (`socks5://127.0.0.1:<port>`).

```toml
# Network TOML must include:
socks5_proxy = "socks5://0.0.0.0:32259"
```

`getSocks5Proxy()` reads port from `currentNet.value.config.enable_socks5` + `socks5_port`.

### Router detection
ET mode: `httpGet(router_ip/cgi, proxy)` via SOCKS5.  
LAN mode: `tcpPing(router_ip, 80)` directly.

### WOL Config (TOML, localStorage `wolDevicesToml`)
```toml
[[device]]
name = "3070"
mac = "2C:F0:5D:CE:4D:23"
ip = "192.168.2.2"
interface = "br-lan"
router_ip = "192.168.2.1"
agent_port = 32249
```

Fields: `name`, `mac`, `ip`, `interface`, `router_ip`, `agent_port`.  
`router_ip` was previously `wol_proxy` (renamed). No more separate `router_ip` field.

### i18n
Custom `tt(key)` function with inline dictionary in home.vue — NOT easytier-frontend-lib's i18n system.

### NAT Labels
`Open(1)→Open(green)`, `FullCone(3)→NAT1(green)`, `Restricted(4)→NAT2(green)`, `PortRestr(5)→NAT3(blue)`, `Sym(6)→NAT4(orange)`.

### Network Quality Colors
Red: `loss>10% || lat>300` | Orange: `loss>5% || lat>100` | Blue: `loss>3% || lat>50` | Green: normal.

### Peer Sorting
Router first (hostname matches `/route|wrt|路由|home|家/i`), then by IP. Servers by hostname alpha.

## Critical Gotchas
- **Never accidentally delete variable declarations** (e.g. `let wolPeriod`) when inserting new code
- `settings.json` set to `"default": "allow"` for build efficiency — revert when done
- CSS vars (`--md-*`) must be on `:root` not `.md-app` — Dialog renders outside app div
- `:root { font-size: 15px }` affects ALL pages including original app
- Dialog buttons are custom `<button>` not PrimeVue `<Button>`
- Network cards: P2P/RELAY tags are at **bottom-right** (before tcp/nat chips)
- Text selection context menu on Android — still unresolved
