# EasyTier + WOLPlus — Quick Reference

## Project
Tauri v2 Android app (`easytier-gui`) — WOL device management + EasyTier VPN control.  
Detail: see `docs/project.md`。构建细节: `docs/本地Android构建.md`（本地） / `docs/CI全平台构建.md`（CI）。

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

## Build

本地 Android 构建详见 [docs/本地Android构建.md](docs/本地Android构建.md)。
CI 全平台构建详见 [docs/CI全平台构建.md](docs/CI全平台构建.md)。

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
