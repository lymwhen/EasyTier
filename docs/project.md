# EasyTier + WOLPlus 整合项目

Tauri v2 Android App，集成 EasyTier 组网与 WOL 电脑设备管理。

## 基本原则

1. **不改 easytier-core**：路由器使用官方二进制，跟随上游更新。
2. **easytier-gui 最小改动**：仅新增页面，保留原 `index.vue` 不变（路径 `/`），方便合并上游。
3. **luci-app-wolplus 可改**：作为 WOL 中间层，CGI API 部署在路由器 `/cgi-bin/wolplus-api`。
4. **配置驱动**：WOL 设备通过 TOML 配置文件管理（独立于网络 TOML），不提供图形化添加。
5. **Material Design 风格**：参考 WOLPlus 的卡片设计。

## 系统架构

### 涉及项目

| 项目 | 路径 | 角色 |
|------|------|------|
| EasyTier | `D:\projects\projectsAlpha\EasyTier` | 组网工具，修改目标为 `easytier-gui` |
| luci-app-wolplus | `D:\projects\projectsOpenwrt\luci-app-wolplus` | OpenWrt LuCI 插件，WOL 中间层 |
| wol-agent | `luci-app-wolplus/wol-agent/` | Go 编写的 Windows Agent，状态检测和关机 API |

### 为什么不用 easytier-web

| | Core 模式（本项目） | Web 模式（easytier-web） |
|------|------|------|
| 配置方式 | TOML 配置文件 | Web UI 或 API |
| 需要登录 | 否 | 是 |
| 通信方式 | Tauri IPC → Core RPC | HTTP → easytier-web API → Core RPC |
| 适用场景 | 单设备组网 | 多设备集中管理 |

本项目使用 Core 模式：App 内嵌 easytier-core，直接读取 TOML 配置组网，所有 UI 交互走 Tauri IPC。

### 系统拓扑

```
手机 App (easytier-gui / Tauri v2)
  ├── 嵌入 easytier-core（组网能力）
  ├── Tab 1: 电脑 — 设备管理（独立于组网）
  ├── Tab 2: 组网 — EasyTier 网络管理
  ├── Tab 3: 设置 — 高级设置 + 语言切换
  ├── Tab 4: LuCI — 路由器管理面板
  └── 底部 Tab 导航

路由器（OpenWrt/ImmortalWRT）
  ├── easytier-core（官方二进制）
  ├── luci-app-wolplus（CGI API /cgi-bin/wolplus-api）
  │     ├── action=wake&mac=...&iface=...     → etherwake
  │     ├── action=status&ip=...&port=...      → curl Agent
  │     └── action=shutdown&ip=...&port=...    → curl Agent POST
  └── etherwake + curl

PC
  └── wol-agent.exe (Go Agent)
        ├── GET  /api/v1/status  → {"online":true,"hostname":"...","os":"windows","uptime":...}
        └── POST /api/v1/shutdown → shutdown /s /t 5
```

### 数据流

**WOL 状态检测**：App (Rust reqwest) → HTTP GET `http://<PC_IP>:32249/api/v1/status`
- 局域网：直连 PC Agent
- EasyTier 隧道：TcpProxy 拦截 192.168.2.x → P2P 隧道 → 路由器 → 本地转发 → PC Agent

**WOL 唤醒**：App → HTTP GET `http://<router_ip>/cgi-bin/wolplus-api?action=wake&mac=...&iface=...`
- 局域网：直连路由器 LAN IP
- 隧道：TcpProxy 拦截 → P2P → 路由器 → uhttpd → CGI → etherwake

**WOL 关机**：App → HTTP POST `http://<PC_IP>:32249/api/v1/shutdown`
- 局域网：直连 PC Agent
- 隧道：TcpProxy 拦截 → P2P → 路由器 → 本地转发 → PC Agent

**EasyTier 组网**：App → Tauri invoke → easytier-core (嵌入式)
- `collectNetworkInfo` → Peer 列表、路由、延迟、丢包率
- `sendConfigs` → 加载 TOML 配置并运行网络
- 每 3 秒自动刷新 Peer 数据

### 文件结构

| 文件 | 状态 | 说明 |
|------|------|------|
| `src/App.vue` | 修改 | Splash 遮罩 + 初始化检测 + 跳转 `/home` |
| `src/pages/home.vue` | **新增** | Tab 容器（电脑/组网/设置/LuCI），核心页面 |
| `src/pages/index.vue` | **不动** | 原页面，`/` 路由 |
| `src/composables/backend.ts` | 修改 | 新增 `httpGet`/`httpPost`（Rust 侧 reqwest） |
| `src-tauri/src/lib.rs` | 修改 | 新增 `http_get`/`http_post` Tauri 命令 |
| `src-tauri/Cargo.toml` | 修改 | 新增 `reqwest`（含 socks 特性） |
| `src/composables/mobile_vpn.ts` | 修改 | VPN 路由配置，`disallowedApplications` |
| `src-tauri/gen/android/.../MainActivity.kt` | 修改 | 剪贴板桥接（`PasteBridge`） |
| `src-tauri/gen/android/.../BuildTask.kt` | 修改 | .so 文件复制替代符号链接 |
| `src-tauri/gen/android/.../AndroidManifest.xml` | 修改 | `usesCleartextTraffic="true"` |

## 产品设计

### 页面结构

四个 Tab，底部导航切换：

| Tab | 图标 | 功能 |
|-----|------|------|
| 电脑 | desktop | WOL 设备管理：唤醒、关机、状态监控 |
| 组网 | network | EasyTier 网络：连接、Peer 列表、速率图表 |
| LuCI | router | 路由器管理面板（HTTP 反向代理 iframe） |
| 设置 | settings | 配置导入导出、语言切换、默认首页、Debug、关于 |

设置中可通过 "Advanced Settings" 跳转到原始页面（`/` 路由）。

### 电脑页面

**标题栏**：设备数量标签 + 路径标签（ET-LAN / LAN，蓝色胶囊）+ 刷新按钮 + 编辑按钮。

**设备卡片**：
- 状态指示灯（脉冲动画）+ 名称 + IP + 状态文字
- 唤醒按钮（铃铛图标）、关机按钮（电源图标）
- 点击展开详情：MAC、Router IP、Interface、Agent Port
- 路由器离线时卡片半透明（`opacity: 0.55`），所有操作按钮隐藏

**自动刷新**：30 秒轮询设备在线状态。

### 组网页面

**未连接态**：中央大 Connect 按钮。

**连接中**：Connect 按钮变灰显示 "Connecting..."。

**已连接态**：
- 顶部连接状态条：本机 IP、设备名、版本号、断开按钮（红色）
- 连接状态下隐藏编辑和切换网络按钮
- 断开按钮在标题栏以红色断链图标展示

**速率图表**：实时上下行速率折线图（60 点 / 3 分钟），HTML 坐标轴标签覆盖在 SVG 之上。左侧 20px 固定宽度留给 Y 轴标签，图表线随卡片宽度拉伸。

**网络设备列表**：Peer 卡片，含 IP + 主机名、延迟/丢包率、流量信息、P2P/Relay 标签、NAT 类型芯片。点击展开显示版本号。

**服务器列表**：同设备卡片布局，按主机名字母排序。

**Peer 排序**：路由器优先（hostname 匹配 `/route|wrt|路由|home|家/i`），其余按 IP 排序。

**切换网络**：弹出菜单列出所有已配置网络（当前网络标记 ✓），底部 "Add Network" 入口。

### 设置页面

提供 6 个菜单项，每项含图标、标题和副标题：

**Advanced Settings（高级设置）**
齿轮图标，副标题 "Mode, logging, config server, language"。点击跳转到原始页面 `/`（easytier-frontend-lib 自带的高级设置面板）。

**Language（语言）**
地球图标，副标题显示当前语言和 "点击切换语言" 提示，右侧蓝色胶囊标签显示 `中文` 或 `English`。点击调用 `toggleLang()`，切换 `locale` ref 并写入 `localStorage['easytierLang']`，UI 所有文本即时更新。

**Default Tab（默认首页）**
九宫格图标，副标题显示当前默认首页名称。右侧原生 `<select>` 下拉，可选电脑 / 路由器 / 组网。选中值存入 `localStorage['defaultTab']`，仅首次加载时影响 `activeTab` 初始值。

**Debug Info（调试信息）**
齿轮图标，副标题 "显示调试信息"，右侧开关 toggle。开启后在组网页底部显示 Debug 区域：本机 IP、端口、raw TOML 文本等诊断信息。

**Configuration Info（配置信息）**
云下载图标，副标题 "全部数据导入与导出" / "Import/Export all data"，右侧两个操作按钮：
- **Export（导出）**：绿色文字按钮。将 WOL 配置（`wolDevicesToml`）、LuCI 配置（`luciRoutersToml`）、网络配置列表（`networkList` 中各网络的 `rawToml`）打包为 JSON 对象，通过 `PasteBridge.writeClipboard()` 写入剪贴板，底部 snackbar 提示 "配置信息已导出到剪切板"。
- **Import（导入）**：蓝色主色按钮。打开导入 Dialog（`showImportDlg`）。

导入 Dialog 流程：
1. 弹出 textarea 输入框（等宽字体），支持手动粘贴 JSON 或点击 "从剪切板导入" 按钮调用 `readClipboard()` 填入
2. 点击 "确认" → `confirmImport()` 校验 JSON 结构（`{ v: 1, wol: "...", luci: "...", net: ["..."] }`）
3. 校验通过后弹出二次确认 Dialog："本次操作将会覆盖电脑、路由器和组网所有的配置，确定继续？"，按钮右对齐
4. 确认 → `executeImport()`：
   - 如当前网络运行中，先 `doDisconnect()` 断开
   - 写入 `wolDevicesToml` → `loadWolConfig()`
   - 写入 `luciRoutersToml` → `loadLuciConfig()`；如当前在 LuCI Tab，自动连接第一个路由器
   - 删除全部旧网络实例，重新 `generateNetworkConfig()` 生成新网络列表
   - snackbar 提示 "配置已导入"

**About（关于）**
信息图标，副标题显示版本号（来自 `package.json` 的 `pkg.version`）。

### LuCI 路由器管理

#### 架构

通过 Rust 侧 Hyper 反向代理，在 iframe 中嵌入路由器 LuCI Web 管理面板。核心流程：

```
Tauri IPC (luci_proxy_start)
  → Rust: reqwest 向路由器发起 LuCI 登录请求（POST /cgi-bin/luci/admin/login）
    → 获取 session cookie
  → Rust: TcpListener 绑定 127.0.0.1 随机端口，启动 Hyper HTTP 代理服务
    → 代理服务在每次转发请求时自动注入登录 cookie
  → 返回代理 URL: http://127.0.0.1:<port>
  → 前端 iframe src 指向代理 URL + /cgi-bin/luci/admin/
```

**路径模式**：与 WOL 设备检测一致，ET 模式下 `luciProxyStart` 接收可选 `socks5Proxy` 参数，reqwest 经 SOCKS5 代理连接路由器。LAN 模式下直连。路径模式改变时（ET-LAN ↔ LAN），代理自动完全重启（`refreshLuciIframe`）。

**路由持久化**：代理记录 `last_path`（最后一次请求的 LuCI 路径），切换路由器时或路径模式变更时恢复上次浏览位置。

#### 多路由器管理

支持配置多个路由器，通过 TOML 格式编辑。标题栏右侧「切换」按钮弹出菜单（与组网切换网络同款）：

- **添加路由器**：蓝色 "+" 入口，打开编辑 Dialog
- **切换路由器**：点击列表项，`switchLuciRouter(i)` 停止旧代理 → 启动新代理 → iframe 重新加载
- **删除路由器**：点击右侧 × 按钮，如删除当前路由器则切换到剩余列表中的第一个
- **编辑配置**：打开 Dialog，Textarea 编辑 TOML，支持「从剪切板导入」按钮

#### 状态流转

```
Tab 切换到 LuCI:
  → 无路由器配置: 显示空状态（"未配置路由器"）
  → 有配置但未启动: 自动调用 startLuciProxy(第一个路由器)
    → 加载中: 显示旋转动画 + "连接中..."
    → 成功: 显示 iframe（全屏 100%×100%）
    → 失败: 保持加载提示

会话恢复:
  onMounted 时读取 lastLuciRouterIp → 定位上次使用的路由器 → 自动连接
```

#### 标题栏

- 显示当前路由器 IP + 路径标签（ET-LAN / LAN）
- 编辑按钮（齿轮图标）
- 刷新按钮（仅在已连接时显示）：完整重启代理，恢复上次浏览路径
- 切换路由器按钮（仅在有多台路由器时显示）

#### 配置编辑 Dialog

与 WOL 配置编辑器同款 Material 风格 Dialog：
- Textarea 等宽字体编辑 TOML
- 「从剪切板导入」按钮（居左）、取消/保存（居右）
- 保存时 `saveLuciConfig()` → 写 localStorage → `loadLuciConfig()` → 自动连接当前路由器（如有变更则重启代理）

### 路由器检测与路径切换

```
routerOnline: undefined → true/false
  首次加载: undefined（显示"检测中..."）
  后续刷新: 保持上次状态，避免闪烁

检测方式:
  ET 模式: httpGet(router_ip/cgi-bin/wolplus-api, proxy) → 通过 SOCKS5
  LAN 模式: tcpPing(router_ip, 80) → 直连

路径标签:
  netRunning && router_ip 有值 → 蓝色 ET-LAN 标签
  否则 → 蓝色 LAN 标签
```

### 设备操作状态机

```
唤醒流程:
  idle → click Wake → waking（橙色脉冲，5s 超时）
    ├── Agent 在线 → online（绿色）
    └── 超时 → offline（灰色）

关机流程:
  online → click Shutdown → shutting（红色脉冲）
    ├── Agent 确认关机后不可达 → offline（灰色）
    └── 持续可达（12 次/60s）→ 回退 online

Phase poll 期间路由器离线 → 立即中止，设备置为 offline
```

### 组网连接流程

```
点击 Connect:
  1. 清空 trafficHistory + 重置 lastTraffic（在 await 之前，确保图表立即重置）
  2. sendConfigs(ids) → 加载 TOML 配置
  3. 等待 2s
  4. fetchNetInfo() → 获取 Peer 数据
  5. 成功 → netConnectTime = Date.now()
  6. 自动 checkAllWolStatus()

Loading 状态:
  netConnecting = true → 中央按钮显示 loading
  netRunning && peers.length === 0 → 标题栏 "Waiting for peers..."
  netDiscovering = 连接后 12 秒内空列表

断开:
  stopNetInstance() → 停止 easytier-core 实例 → 清空状态
```

### 配置编辑 Dialog

- 点击编辑按钮打开 Dialog，每次打开时从 localStorage 重新读取最新配置
- 底部按钮：「从剪切板导入」（居左）、取消/保存（居右）
- 导入确认弹窗：按钮右对齐，警告文字提示将覆盖所有配置
- 保存时写回 localStorage

## UI 设计

### 设计语言

Material Design 风格，自定义 CSS 变量体系。

```css
:root {
  font-size: 15px;  /* 影响所有页面，包括原始 app */
  --md-primary: #1976d2;
  --md-card: #ffffff;
  --md-text: #212121;
  /* ... etc */
}
```

- 卡片：`border-radius: 12px`，`box-shadow` 阴影
- 标题栏：`min-height: 56px`
- CSS 变量须在 `:root` 而非 `.md-app`（PrimeVue Dialog 渲染在 app div 外）
- 暗色模式：`@media (prefers-color-scheme: dark)`

### 颜色体系

| 场景 | 颜色 | 条件 |
|------|------|------|
| NAT 类型 | 绿 / 绿 / 绿 / 蓝 / 橙 | NAT1 / NAT2 / Open / NAT3 / NAT4 |
| 网络质量 | 绿 / 蓝 / 橙 / 红 | 正常 / loss>3% 或 lat>50 / loss>5% 或 lat>100 / loss>10% 或 lat>300 |
| 隧道类型 | 绿 / 蓝 / 橙 | Local / P2P / Relay |
| 路径标签 | 蓝 `#e3f2fd` / `#1565c0` | ET-LAN / LAN |
| 断开按钮 | `#ef5350` | 标准红 |
| 状态指示灯 | 绿 / 灰 / 橙（脉冲）/ 红（脉冲） | 在线 / 离线 / 唤醒中 / 关机中 |

### 卡片布局

```
网络设备: [IP] [hostname]                [质量块]
          [↑↓流量]            [P2P] [tcp] [NAT]

服务器:   [hostname]                     [质量块]
          [↑↓流量]            [P2P] [tcp] [NAT]
```

P2P/RELAY 标签位于右下角（tcp/nat 芯片之前）。网络质量块在右上角。

### 图标

| 功能 | 图标 | 类型 |
|------|------|------|
| 唤醒 | 铃铛 `M12 22c...` | Material 面型 |
| 关机 | 电源 `M13 3h-2v10h2V3z...` | Material 面型 |
| 切换网络 | 左右箭头交换 | 自定义 SVG |
| Connect | 链式连接 | 自定义 SVG |
| 断开 | 断链 | 自定义 SVG |

### Toast

自定义 Material snackbar（替代 PrimeVue `useToast`）：底部居中胶囊，2.5s 自动消失，滑入动画。

## 重难点技术方案

### 1. WOL 通过 EasyTier 隧道（SOCKS5 方案）

**问题**：Android App 在 `mobile_vpn.ts` 中设置了 `disallowedApplications: ['com.kkrainbow.easytier']`，将 App 自身排除出 VPN，避免路由死循环。副作用是 App 内所有 HTTP 请求绕过 TUN，无法到达 `192.168.2.x`。

**根因链**：
1. easytier-core 未调用 Android `VpnService.protect()` 保护 P2P 隧道 Socket
2. 为避免 VPN 路由死循环，只能将整个 App 排除出 VPN
3. App 内 Rust reqwest 发出的 HTTP 请求走物理网络，不通虚拟网段

**方案**：使用 easytier-core 内置 SOCKS5 代理。
- 网络 TOML 配置 `socks5_proxy = "socks5://0.0.0.0:32259"`
- `getSocks5Proxy()` 从 `NetworkConfig.enable_socks5` + `socks5_port` 读取端口
- WOL HTTP 请求经 `socks5://127.0.0.1:32259` → EasyTier 虚拟网 → 路由器 → PC

**否决方案**：移除 `disallowedApplications`（死循环）、前端 `fetch()`（mixed content + 不支持 SOCKS5）、正则解析 `rawToml`（可能为空）。

### 2. 多语言切换（i18n）

自定义 `tt(key)` 函数 + 内联字典，不依赖 easytier-frontend-lib 的 i18n 系统。

```js
const lang = ref<'en'|'zh'>(localStorage.getItem('easytierLang') === 'zh' ? 'zh' : 'en')
function tt(key: string): string {
  const idx = lang.value === 'zh' ? 1 : 0
  return i18nMap[key]?.[idx] ?? key
}
```

所有 home.vue 用户可见文字均已中英双语化，语言选择存入 `localStorage['easytierLang']`。

### 3. Android 剪贴板读取

**架构**：

```
MainActivity.kt:
  PasteBridge.readClipboard() [@JavascriptInterface, JavaBridge 线程]
    → CountDownLatch(500ms)
      → mainHandler.post { ClipboardManager.getPrimaryClip() } [UI 线程]
    → return text

home.vue:
  readClipboard(): _easytier_paste.readClipboard?.() || ''
  importFromClipboard(): wolToml.value = readClipboard()
```

**要点**：
- 剪贴板必须在 Android UI 线程读取（JavaBridge 线程可能失败）
- 每次点击「从剪切板导入」时按需调用，不使用定时轮询
- v-model 绑定后仅通过 Vue 响应式系统更新值，不动 DOM

### 4. 文本选择上下文菜单（已知限制）

**现象**：Dialog 中 Textarea 可选择文字，但 Android 原生复制/粘贴工具栏不弹出。

**根因**：Chromium WebView 147 内部 `SelectionPopupController` 在此配置下不触发 ActionMode（Chromium 边缘 bug，非本项目代码问题）。

**排查结论**：CSS/JS 无拦截、wry 无 ActionMode 覆盖、AppCompat 正确透传、手动 `startActionMode` 可弹出。反射调用 `setCustomSelectionActionModeCallback` 在 SDK 34 stubs 中隐藏无效。

**状态**：不计划修复。JS 桥接 + 反射方案可弹出菜单和复制，但粘贴被安全策略禁用。

## 配置规范

### WOL 设备配置 (devices.toml)

存储在 `localStorage['wolDevicesToml']`。

```toml
[[device]]
name = "3070"
mac = "AA:BB:CC:DD:EE:FF"
ip = "192.168.2.2"
interface = "br-lan"
router_ip = "192.168.2.1"
agent_port = 32249
```

| 字段 | 说明 |
|------|------|
| `name` | 设备显示名称 |
| `mac` | MAC 地址（用于 etherwake） |
| `ip` | PC IP 地址（用于状态检测和关机） |
| `interface` | 路由器上发送 WOL 包的网口 |
| `router_ip` | 路由器 IP（WOL 唤醒请求目标、路径判断依据） |
| `agent_port` | wol-agent 监听端口（默认 32249） |

### EasyTier 网络配置

多个网络配置存储在 `localStorage['networkList']`（JSON 数组，每项含 `config`、`source`、`rawToml`）。每个网络 TOML 需包含 `socks5_proxy` 以支持 WOL over EasyTier：

```toml
socks5_proxy = "socks5://0.0.0.0:32259"
```

`instance_id` 用于关联当前运行的网络实例与配置。

### LuCI 路由器配置 (luci.toml)

存储在 `localStorage['luciRoutersToml']`。

```toml
[[router]]
name = "Home Router"
ip = "192.168.2.1"
username = "root"
password = ""
```

| 字段 | 说明 |
|------|------|
| `name` | 路由器显示名称 |
| `ip` | 路由器 IP（用于反向代理连接） |
| `username` | LuCI 登录用户名 |
| `password` | LuCI 登录密码 |

### 一键配置导入/导出格式

导出 JSON 结构（写入剪贴板）：

```json
{
  "v": 1,
  "wol": "<devices.toml 完整内容>",
  "luci": "<luci.toml 完整内容>",
  "net": ["<网络1 TOML>", "<网络2 TOML>"]
}
```

导入时校验规则：`v` 为数字 1，`wol`/`luci` 为字符串，`net` 为字符串数组。

## 开发环境

| 工具 | 路径/版本 |
|------|----------|
| Android SDK | `D:\tools\Android\Sdk` |
| NDK | `27.0.12077973` |
| JDK 17 | `D:\tools\Java\jdk-17.0.12` |
| Rust | `stable-x86_64-pc-windows-msvc` 1.95 |
| pnpm | `$HOME/.npm-global` |
| MinGW | `$HOME/mingw64/mingw64` |
| protoc | `$HOME/protoc` |
| libclang | `$HOME/llvm-dll` |

## 路由器信息

- IP: `192.168.2.1`
- SSH: `plink -ssh -P 22 -l root -pw ... 192.168.2.1`
- 系统: ImmortalWRT SNAPSHOT
- luci-app-wolplus 已部署，CGI API: `/cgi-bin/wolplus-api`
- EasyTier 配置: `dhcp = true`, `proxy_network cidr = "192.168.2.0/24"`

## 构建流程

详见 [项目构建方式.md](项目构建方式.md)。
