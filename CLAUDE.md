# EasyTier + WOLPlus — Quick Reference

## Project
Tauri v2 Android app (`easytier-gui`) — WOL device management + EasyTier VPN control.  
Detail: see `docs/project.md`（项目详细架构）。构建细节: `docs/本地Android构建.md`（本地） / `docs/CI全平台构建.md`（CI）。

## Documentation Index
All project documentation is in `docs/` directory:

| 文件 | 说明 |
|------|------|
| `docs/project.md` | 项目详细架构：功能清单、系统架构、产品设计、UI设计、重难点方案、配置规范 |
| `docs/本地Android构建.md` | 本地 Windows Android 构建：三步流程、.cargo/config.toml、指纹稳定性 |
| `docs/CI全平台构建.md` | CI 全平台构建：Tag 推送触发、构建矩阵、Release Draft 发布 |
| `docs/电脑性能监测功能.md` | PC 硬件性能监控：wol-agent API、前端图表渲染、Agent 部署指南 |

> ⚠️ **文档同步规则**：每次在 `docs/` 目录下新增、删除或重命名 `.md` 文档，必须同步更新本索引表格。

## Constraints
- **Never modify `easytier/` (easytier-core)** — router uses official binary
- **Minimal changes to `easytier-gui`** — preserve `index.vue` for upstream merge
- `luci-app-wolplus` is the WOL middleware on router (`/cgi-bin/wolplus-api`)
- **Cross-platform first**: 除平台特有功能适配（如 Android 通知栏），所有更改必须采用全平台方案；方案沟通时需说明各平台适配情况

## File Map
| Path | Role |
|------|------|
| `easytier-gui/src/pages/home.vue` | **Main dev target** — WOL / Network / Settings tabs |
| `easytier-gui/src/composables/backend.ts` | Tauri IPC wrappers (`httpGet`, `httpPost`, `tcpPing`, etc.) |
| `easytier-gui/src-tauri/src/lib.rs` | Rust Tauri commands (`http_get`, `http_post`, `tcp_ping`) |
| `easytier-gui/src-tauri/Cargo.toml` | Rust deps (reqwest with `socks` feature) |
| `easytier-gui/src-tauri/gen/android/.../MainActivity.kt` | Android activity (theme bridge for status bar) |
| `easytier-gui/src/composables/mobile_vpn.ts` | VPN service setup (routes, `disallowedApplications`) |
| `easytier-gui/src/composables/theme.ts` | Theme state machine (resolveTheme, applyTheme, AMOLED support) |

## Build

本地 Android 构建详见 [docs/本地Android构建.md](docs/本地Android构建.md)，本地构建 Android APK 时，必须严格按照此文档进行。
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

### Path Labels
- **ET-LAN** (blue): `netRunning` is true → route through EasyTier tunnel
- **LAN** (blue): otherwise → direct LAN connection

> Same logic applies to WOL: `getWolPathType()` returns `'tunnel' | 'lan'` based on `netRunning`

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
- Text selection context menu on Android — ✅ 已修复（main.ts 中 contextmenu 仅拦截非 INPUT/TEXTAREA 元素）

## Commit

- **格式**：`type: description`，type = `feat` / `fix` / `docs` / `chore` / `refactor`
- **禁止**以 `@` 开头，禁止在 commit message 两端使用 `@'...'@` 包裹
- **推送前**检查 `git log --oneline -1` 确认格式正确

## Upstream Tracking

- **一切对上游的改动必须补充到** `docs/project.md` 的「相对上游变更清单」表格中
- 每次新增/修改对上游文件的变更时，同步更新该表格：文件路径、变更说明、必要性（必要/Fork适配/附带）
