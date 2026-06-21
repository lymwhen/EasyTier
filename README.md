# EasyTier + WOLPlus

[![Github release](https://img.shields.io/github/v/tag/lymwhen/EasyTier)](https://github.com/lymwhen/EasyTier/releases)
[![GitHub](https://img.shields.io/github/license/lymwhen/EasyTier)](https://github.com/lymwhen/EasyTier/blob/main/LICENSE)
[![GitHub last commit](https://img.shields.io/github/last-commit/lymwhen/EasyTier)](https://github.com/lymwhen/EasyTier/commits/main)
[![GitHub issues](https://img.shields.io/github/issues/lymwhen/EasyTier)](https://github.com/lymwhen/EasyTier/issues)
[![GitHub Core Actions](https://github.com/lymwhen/EasyTier/actions/workflows/core.yml/badge.svg)](https://github.com/lymwhen/EasyTier/actions/workflows/core.yml)
[![GitHub GUI Actions](https://github.com/lymwhen/EasyTier/actions/workflows/gui.yml/badge.svg)](https://github.com/lymwhen/EasyTier/actions/workflows/gui.yml)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/lymwhen/EasyTier)

> ✨ 基于 Rust + Tauri v2 的跨平台异地组网与远程设备管理一体化应用
>
> EasyTier 去中心化组网引擎 + WOLPlus 电脑唤醒/关机管理 + LuCI 路由器反向代理

<p align="center">
  <img src="assets/image-wol.png" width="280" alt="电脑 - WOL 设备管理">
  <img src="assets/image-network.png" width="280" alt="组网 - EasyTier VPN">
  <img src="assets/image-routers.png" width="280" alt="路由器 - LuCI 管理">
  <img src="assets/image-settings.png" width="280" alt="设置">
</p>

## 项目简介

本项目是 [EasyTier](https://github.com/lymwhen/EasyTier) 的功能增强分支，在保留 EasyTier 全部组网能力的基础上，深度整合了 **WOL 电脑唤醒/关机管理**、**LuCI 路由器反向代理管理**、**PC 硬件性能实时监控**、**一键配置导入/导出** 等实用功能，并采用 Glassmorphism + Material You 混合风格进行了全面界面美化。

应用基于 **Tauri v2** 框架构建，支持 **Windows / macOS / Linux 桌面端** 和 **Android 移动端**，App 内嵌 easytier-core 实现去中心化组网，无需中心服务器。

---

## 四大功能模块

| Tab | 功能 | 说明 |
|-----|------|------|
| 💻 **电脑** | WOL 设备管理 | 远程唤醒/关机 PC，在线状态实时监控，展开查看 CPU/内存/GPU/磁盘IO/网络流量性能图表 |
| 🖥️ **路由器** | LuCI 管理面板 | HTTP 反向代理 iframe 嵌入路由器管理页面，多路由器切换，会话自动保持，路径持久化 |
| 🌐 **组网** | EasyTier 网络 | 一键连接组网，Peer 列表 + 实时速率图表，网络质量颜色分级，多网络切换管理 |
| ⚙️ **设置** | 配置与主题 | 浅色/深色/AMOLED 纯黑三主题，中英双语，一键配置导入导出，默认首页，调试模式，帮助 |

---

## 核心特性

### EasyTier 组网能力（原版保留）

- 🔒 **去中心化**：节点平等独立，无需中心化服务
- 🔐 **安全**：AES-GCM 或 WireGuard 加密，防止中间人攻击
- 🔌 **高效 NAT 穿透**：UDP / IPv6 穿透，NAT4-NAT4 也能打通
- 🌐 **子网代理**：节点可共享子网供其他节点访问
- ⚡ **高性能**：全链路零拷贝，支持 TCP / UDP / WSS / WG 协议
- 🌍 **跨平台**：Win / macOS / Linux / FreeBSD / Android，X86 / ARM / MIPS / RISC-V 全架构

### 新增增强功能

| 功能 | 说明 |
|------|------|
| 💻 WOL 设备管理 | 远程唤醒/关机 PC，30 秒自动轮询在线状态，状态机驱动（唤醒脉冲 → 在线 → 关机脉冲 → 离线） |
| 🖥️ LuCI 代理管理 | 通过 EasyTier 隧道反向代理路由器 LuCI 面板，多路由器切换、自动登录、Cookie 会话保持、刷新后恢复浏览路径 |
| 📊 PC 性能监控 | 展开设备卡片实时查看 CPU/内存/GPU 使用率、磁盘 IO 读写速度、网络上下行流量，SVG 面积渐变折线图，10 秒刷新 |
| 🌓 三主题系统 | 浅色 / 深色 / AMOLED 纯黑三个平等独立主题，`applyTheme()` 统一状态机管理，动态适配 Android 状态栏颜色 |
| 🌐 中英双语 | 自定义 i18n 系统，所有界面文字即时切换（不依赖 easytier-frontend-lib） |
| 📦 一键配置导入/导出 | WOL + LuCI + 网络配置打包为 JSON，deflate 压缩 + Base64URL 编码，剪贴板一键导入导出 |
| 📈 实时速率图表 | 60 点 / 3 分钟上下行速率面积渐变折线图，SVG 渲染 |
| 🎨 网络质量可视化 | 延迟/丢包率颜色分级（绿/蓝/橙/红），NAT 类型标签，P2P/Relay 隧道标签 |
| 🔄 路径模式自适应 | EasyTier 隧道（ET-LAN）与局域网直连（LAN）双模式自动识别 |
| 🎯 Material You 风格 | 毛玻璃标题栏/Tab 栏、Tonal 按钮、卡片入场顺次动画、水波涟漪连接按钮 |

---

## WOL 设备管理

> ⚠️ WOL 功能为**可选增强特性**，需要额外安装以下组件方可使用：
> - 路由器端：[luci-app-wolplus](https://github.com/lymwhen/luci-app-wolplus)（OpenWrt 路由器插件，需使用定制版以支持 WOL Proxy）
> - PC 端：`wol-agent`（运行在每台被管理的 Windows PC 上）

WOL 功能依赖路由器端 `luci-app-wolplus` CGI 中间层 + PC 端 `wol-agent`（Go 编写）协同工作，组成完整的 **唤醒 → 状态检测 → 关机** 操作闭环。

### 整体架构

```
手机 / 桌面 App (easytier-gui)
  ├── 唤醒 PC：HTTP GET → 路由器 CGI → etherwake 发送魔术包
  ├── 状态检测：HTTP GET → PC Agent (32249端口) → 返回在线状态
  └── 远程关机：HTTP POST → PC Agent (32249端口) → shutdown /s /t 5

路由器 (OpenWrt / ImmortalWRT)
  ├── easytier-core（组网节点，官方二进制）
  ├── luci-app-wolplus（CGI API: /cgi-bin/wolplus-api）
  └── etherwake + curl

PC (Windows)
  └── wol-agent.exe (Go Agent，监听 32249 端口)
        ├── GET  /api/v1/status   → {"online":true,"hostname":"MyPC",...}
        └── POST /api/v1/shutdown → shutdown /s /t 5
```

### 数据流路径

| 模式 | 条件 | 路径 |
|------|------|------|
| 局域网直连 | App 与路由器/PC 同网段 | App 直连路由器 LAN IP / PC IP |
| EasyTier 隧道 | 连接了 EasyTier 网络 | App → SOCKS5 代理 → P2P 隧道 → 远端路由器 → 本地转发 |

### 操作状态机

```
唤醒: idle → waking（橙色脉冲 5s）→ online（绿）/ offline（灰）
关机: online → shutting（红色脉冲）→ offline（灰）/ 回退 online（12次/60s 检测）
```

### 前提条件

- 路由器安装 `luci-app-wolplus` + `etherwake`
- PC 端运行 `wol-agent`（监听 32249 端口）
- EasyTier 隧道模式下，网络 TOML 需启用 SOCKS5 代理：`socks5_proxy = "socks5://0.0.0.0:32259"`

---

## 系统架构

```
App (Tauri v2 / easytier-gui) — Win / macOS / Linux / Android
  ├── 嵌入 easytier-core（组网引擎，Core 模式，去中心化）
  ├── Tab 1: 电脑 — WOL 唤醒/关机 + 在线检测 + PC 性能监控
  ├── Tab 2: 路由器 — LuCI 管理面板 (HTTP 反向代理 iframe)
  ├── Tab 3: 组网 — EasyTier 网络管理 + Peer 列表 + 速率图表
  ├── Tab 4: 设置 — 主题/语言/默认首页/调试/配置导入导出/帮助/关于
  └── 底部 Tab 导航栏

路由器 (OpenWrt / ImmortalWRT)
  ├── easytier-core（官方二进制，组网节点）
  ├── luci-app-wolplus（CGI API: /cgi-bin/wolplus-api）
  └── etherwake + curl + uhttpd

PC (Windows)
  └── wol-agent（Go Agent，32249 端口）
```

> **Core 模式 vs Web 模式**：本项目使用 Core 模式 — App 直接内嵌 easytier-core，通过 Tauri IPC 通信，无需登录、无需运行 easytier-web 服务。配置通过 TOML 文件管理，适合单设备组网场景。

---

## 配置说明

### WOL 设备配置

存储在 `localStorage['wolDevicesToml']`，通过「电脑」Tab 编辑按钮修改。

```toml
[[device]]
name = "我的PC"
mac = "AA:BB:CC:DD:EE:FF"
ip = "192.168.2.2"
interface = "br-lan"
router_ip = "192.168.2.1"
agent_port = 32249
```

| 字段 | 说明 |
|------|------|
| `name` | 设备显示名称 |
| `mac` | MAC 地址（用于 etherwake 发送 WOL 魔术包） |
| `ip` | PC IP 地址（状态检测和关机请求目标） |
| `interface` | 路由器上发送 WOL 包的网口 |
| `router_ip` | 路由器 IP（WOL 唤醒请求目标、路径模式判断依据） |
| `agent_port` | wol-agent 监听端口（默认 32249） |

### LuCI 路由器配置

存储在 `localStorage['luciRoutersToml']`，支持多路由器。

```toml
[[router]]
name = "主路由"
ip = "192.168.2.1"
username = "root"
password = ""
```

### EasyTier 网络配置

多个网络配置存储在 `localStorage['networkList']`。为支持 WOL 通过 EasyTier 隧道，需添加 SOCKS5 代理：

```toml
socks5_proxy = "socks5://0.0.0.0:32259"
```

### 一键导入/导出格式

导出时 JSON 序列化后经 deflate 压缩 + Base64URL 编码，以 `ETGC:2:` 前缀标识：

```json
{
  "v": 1,
  "wol": "<devices.toml 内容>",
  "luci": "<luci.toml 内容>",
  "net": ["<网络1 TOML>", "<网络2 TOML>"]
}
```

通过「设置 → 配置信息 → Export」写入剪贴板，Import 支持 JSON 校验 + 二次确认弹窗。

---

## 快速开始

### 安装

从 [GitHub Releases](https://github.com/lymwhen/EasyTier/releases) 下载对应平台的安装包：

| 平台 | 安装方式 |
|------|---------|
| 🪟 Windows | nsis `.exe` 安装包 |
| 🍎 macOS | `.dmg` 镜像 |
| 🐧 Linux | `.deb` / `.rpm` / `.AppImage` |
| 📱 Android | `.apk` 安装包 |

命令行安装：

```bash
# Linux（推荐）
curl -fsSL "https://github.com/lymwhen/EasyTier/blob/main/script/install.sh?raw=true" | sudo bash -s install

# macOS / Linux (Homebrew)
brew tap brewforge/chinese
brew install --cask easytier-gui

# Windows（管理员权限运行）
irm "https://github.com/lymwhen/EasyTier/blob/main/script/install.ps1?raw=true" | iex
```

### 基本使用流程

1. **安装路由器端**：在 OpenWrt 路由器上安装 `luci-app-wolplus` + `etherwake`
2. **安装 PC Agent**：在需要管理的 Windows PC 上运行 `wol-agent.exe`
3. **配置网络**：在 App「组网」Tab 中编辑网络 TOML（确保包含 `socks5_proxy`），点击 Connect 连接
4. **配置设备**：在「电脑」Tab 点击编辑按钮，添加 WOL 设备配置
5. **管理设备**：点击 🔔 唤醒设备，点击 ⏻ 远程关机，展开卡片查看性能监控
6. **管理路由器**：在「路由器」Tab 配置路由器信息，通过 iframe 直接操作 LuCI 管理面板

### EasyTier 快速组网

```bash
# 节点 A 和节点 B 使用相同的网络名和密钥
sudo easytier-core -d --network-name mynetwork --network-secret mysecret -p tcp://<共享节点IP>:11010
```

更多组网方式（WireGuard 集成、子网代理、自建共享节点等）参考 [EasyTier 官方文档](https://easytier.cn)。

---

## 构建

| 文档 | 说明 |
|------|------|
| [本地 Android 构建](docs/本地Android构建.md) | 三步构建流程、指纹稳定性、Cargo 增量编译缓存 |
| [CI 全平台构建](docs/CI全平台构建.md) | Tag 推送自动触发全平台构建 + GitHub Release 发布 |
| [项目详细架构](docs/project.md) | 功能清单、UI 设计、重难点技术方案、配置规范 |

### CI 构建矩阵

| 平台 | 产物 | 架构 |
|------|------|------|
| 🐧 Linux | deb / rpm / AppImage | x86_64, aarch64 |
| 🍎 macOS | dmg | x86_64, aarch64 |
| 🪟 Windows | nsis installer (.exe) | x86_64, i686, aarch64 |
| 📱 Android | APK | aarch64, armv7, i686, x86_64 |
| 🔧 CLI | 二进制 | 16 个目标 (含 MIPS/LoongArch/RISC-V/FreeBSD) |

---

## 许可证

本项目基于 EasyTier，在 [LGPL-3.0](https://github.com/lymwhen/EasyTier/blob/main/LICENSE) 许可下发布。新增的 WOLPlus 整合代码同样遵循 LGPL-3.0 许可。

## 致谢

- 🙏 **[EasyTier](https://github.com/EasyTier/EasyTier)** — 去中心化组网引擎，高性能 Rust 实现，稳定的 NAT 穿透能力，基于 Tauri v2 的 App 框架
- 🙏 **[luci-app-wolplus](https://github.com/animegasan/luci-app-wolplus)** — OpenWrt 路由器 WOL 插件，提供 CGI API 通过 etherwake 发送魔术包

---

<p align="center">
  <b>EasyTier + WOLPlus</b> — 让异地组网与远程设备管理，从未如此简单
</p>
