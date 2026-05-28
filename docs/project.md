# EasyTier + WOLPlus 整合项目

## 基本原则

1. **不改 easytier-core**：路由器用官方二进制，跟随上游更新。
2. **easytier-gui 最小改动**：仅新增页面，保留原 `index.vue` 不变（路径 `/`），方便合并上游。
3. **luci-app-wolplus 可改**：作为 WOL 中间层，已部署 CGI API 在路由器 `/cgi-bin/wolplus-api`。
4. **配置驱动**：WOL 设备通过 TOML 配置文件管理（独立于网络 TOML），不开发图形化添加设备。
5. **Material Design 风格**：参考 WOLPlus 的卡片设计。

## 涉及项目

| 项目 | 路径 | 角色 |
|------|------|------|
| EasyTier | `D:\projects\projectsAlpha\EasyTier` | 组网工具，修改目标为 `easytier-gui`（Tauri Android App） |
| luci-app-wolplus | `D:\projects\projectsOpenwrt\luci-app-wolplus` | OpenWrt LuCI 插件，作为 WOL 中间层 |
| wol-agent | `luci-app-wolplus/wol-agent/` | Go 编写的 Windows Agent，提供状态检测和关机 API |

## easytier-web 与本项目无关

EasyTier 项目包含三个主要组件：

| 组件 | 用途 | 本项目是否涉及 |
|------|------|---------------|
| `easytier` (easytier-core) | VPN 守护进程，核心组网能力 | ✅ 嵌入在 App 中使用；路由器运行官方版 |
| `easytier-gui` | Tauri v2 桌面/移动 App，内嵌 easytier-core | ✅ **唯一修改目标** |
| `easytier-web` | Web 配置服务器 + 管理面板 | ❌ **完全无关** |

**easytier-web 是什么**：
- 一个独立的配置服务器（Rust + axum），提供 REST API 和 Web 前端
- 用于**集中管理**多台 easytier-core 实例
- easytier-core 通过 `--config-server` 连接到 easytier-web，从服务器获取配置
- easytier-web 提供 Web UI（Vue 3 + PrimeVue）用于远程管理设备、编辑配置
- 内置用户认证（SQLite + 密码/OIDC）

**为什么与本项目无关**：
- 本项目使用 **Core 模式**（配置文件直连），不需要配置服务器
- App 内嵌 easytier-core，直接读取 TOML 配置组网，不需要登录
- 所有 UI 交互走 Tauri IPC（`invoke()`），不经过 easytier-web 的 REST API
- 早期曾误以为需要通过 easytier-web API 获取 Peer 数据、管理设备，后来确认 Core 模式下 Tauri IPC 直接调用 Core 的 RPC

**easytier-web vs Core 模式对比**：

| | Core 模式（本项目） | Web 模式（easytier-web） |
|------|------|------|
| 配置方式 | TOML 配置文件 | Web UI 或 API |
| 需要登录 | 否 | 是 |
| 管理界面 | 本项目自研 | easytier-web 自带 |
| 通信方式 | Tauri IPC → Core RPC | HTTP → easytier-web API → Core RPC |
| 适用场景 | 单设备组网 | 多设备集中管理 |

## 架构

```
手机 App (easytier-gui / Tauri v2)
  ├── 嵌入 easytier-core（组网能力）
  ├── Tab 1: WOL 页面 — 设备管理（独立于组网）
  ├── Tab 2: 组网页面 — EasyTier 网络管理
  ├── Tab 3: 设置页面 — 高级设置入口 + 语言切换
  └── 底部三 Tab 导航

路由器（OpenWrt/ImmortalWRT）
  ├── easytier-core（官方二进制）
  ├── luci-app-wolplus（含 CGI API /cgi-bin/wolplus-api）
  │     ├── action=wake&mac=...&iface=...     → etherwake
  │     ├── action=status&ip=...&port=...      → curl Agent
  │     └── action=shutdown&ip=...&port=...    → curl Agent POST
  ├── proxy_cidrs = ["192.168.2.0/24"]（已配置）
  └── etherwake + curl

PC
  └── wol-agent.exe (Go Agent)
        ├── GET  /api/v1/status  → {"online":true,"hostname":"...","os":"windows","uptime":...}
        └── POST /api/v1/shutdown → shutdown /s /t 5
```

## 数据流

### WOL 状态检测
```
App (Rust reqwest) → HTTP GET http://<PC_IP>:32249/api/v1/status
  ├── 局域网：直连 PC Agent
  └── EasyTier 隧道：TcpProxy 拦截 192.168.2.x → P2P 隧道 → 路由器 → 本地转发 → PC Agent
```

### WOL 唤醒
```
App → HTTP GET http://<wol_proxy>/cgi-bin/wolplus-api?action=wake&mac=...&iface=...
  ├── 局域网：直连路由器 LAN IP
  └── 隧道：TcpProxy 拦截 → P2P → 路由器 → uhttpd → CGI → etherwake
```

### WOL 关机
```
App → HTTP POST http://<PC_IP>:32249/api/v1/shutdown
  ├── 局域网：直连 PC Agent
  └── 隧道：TcpProxy 拦截 → P2P → 路由器 → 本地转发 → PC Agent
```

### EasyTier 组网
```
App → Tauri invoke → easytier-core (嵌入式)
  ├── collectNetworkInfo → Peer 列表、路由、延迟、丢包率
  ├── sendConfigs → 加载 TOML 配置并运行网络
  └── 每 3 秒 PeriodicTask 自动刷新 Peer 数据
```

## 文件结构（easytier-gui）

| 文件 | 状态 | 说明 |
|------|------|------|
| `src/App.vue` | 修改 | Splash 遮罩 + 初始化检测 + 跳转 `/home` |
| `src/pages/home.vue` | **新增** | Tab 容器（WOL / 组网 / 设置），核心页面 |
| `src/pages/index.vue` | **不动** | 原页面，`/` 路由，设置中可跳转访问 |
| `src/composables/backend.ts` | 修改 | 新增 `httpGet`/`httpPost`（Rust 侧 reqwest 代理） |
| `src-tauri/src/lib.rs` | 修改 | 新增 `http_get`/`http_post` Tauri 命令（含 5s 超时） |
| `src-tauri/Cargo.toml` | 修改 | 新增 `reqwest` 依赖 |
| `src-tauri/gen/android/.../BuildTask.kt` | 修改 | .so 文件复制替代符号链接（Windows 限制） |
| `src-tauri/gen/android/.../AndroidManifest.xml` | 修改 | `usesCleartextTraffic="true"` 硬编码 |
| `src-tauri/gen/android/.../build.gradle.kts` | 修改 | release 块加 `manifestPlaceholders["usesCleartextTraffic"]` |
| `src-tauri/gen/android/.../themes.xml` | 修改 | 状态栏颜色（亮 `#f5f5f5` / 暗 `#121212`） |
| `src-tauri/gen/android/.../values-night/themes.xml` | 修改 | 同上 |

## UI 设计

### 底部 Tab 栏
- 三个 Tab：WOL / Network / Settings
- 默认打开 WOL Tab
- 设置 Tab 显示菜单列表（Advanced Settings 跳转原页面、Language 切换、About 版本号）

### WOL 页面
- 标题栏：设备数量标签 + 路径标签（ET / LAN）+ 刷新按钮 + 编辑按钮
- 设备卡片：状态指示灯 + 名称 + IP + 状态文字 + Wake/Shutdown 按钮
- 点击卡片展开：MAC、WOL Proxy、Interface、Agent Port
- 路径判断：`netRunning && router_ip 有值` → ET 标签，否则 LAN 标签
- 30 秒自动刷新 + 手动刷新按钮

### 组网页面
- 标题栏：网络名 + 状态（Connecting/Waiting/Connected）+ 编辑按钮 + 切换网络按钮
- 切换网络弹出菜单：Add Network + 已有网络列表（当前网络标记 ✓）
- 断开状态：中央大 Connect 按钮
- 连接中状态：连接按钮变灰 Connecting...
- 连接后：连接状态条（IP、设备名、版本、Disconnect 按钮）
- Servers 区域：Route Cost 标签 + 主机名 + 延迟/丢包/Tunnel 芯片
- Network Devices 区域：Route Cost 标签 + IP + 主机名 + 延迟/丢包 + Tunnel/NAT 芯片
- 卡片点击展开：Version

### 设置页面
- 菜单列表：Advanced Settings（→ `/`）、Language（点击切换中/英）、About（版本号）

### Material Design 样式
- `:root { font-size: 15px; font-family: ... }`
- CSS 自定义属性（`--md-primary`, `--md-card`, `--md-text`, etc.）
- `prefers-color-scheme: dark` 暗色模式支持
- 卡片：`border-radius: 12px`，`box-shadow`
- 状态点：绿色在线 / 灰色离线 / 橙色唤醒中 / 红色关机中（脉冲动画）
- 路由标签：Local(绿) / P2P(蓝) / Relay(橙)
- 标题栏：`min-height: 56px`

## 构建流程

详见 [项目构建方式.md](项目构建方式.md)。

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
- 已有 WOL 设备: `3070` (2C:F0:5D:CE:4D:23, 192.168.2.2), `wxypc` (40:B0:76:DC:1A:F4, 无 IP)

## WOL 配置格式 (devices.toml)

```toml
[[device]]
name = "3070"
mac = "2C:F0:5D:CE:4D:23"
ip = "192.168.2.2"
interface = "br-lan"
router_ip = "192.168.2.1"
agent_port = 32249

[[device]]
name = "wxypc"
mac = "40:B0:76:DC:1A:F4"
ip = ""
interface = "br-lan"
router_ip = "192.168.2.1"
agent_port = 32249
```

存储在 `localStorage['wolDevicesToml']`。

## 开发历程与已解决问题

### 1. WOL 通过 EasyTier 隧道不通 — 已解决 (SOCKS5 方案)

**根因**：Android App 在 `mobile_vpn.ts` 中设置了 `disallowedApplications: ['com.kkrainbow.easytier']`，导致 App 自身所有流量（包括 Rust reqwest）绕过 VPN TUN 走物理网络，无法到达 `192.168.2.x`。

**原因链**：
1. easytier-core 未调用 Android `VpnService.protect()` 保护 P2P 隧道 Socket
2. 为避免 VPN 路由死循环，只能将整个 App 排除出 VPN
3. 副作用：App 内所有 HTTP 请求绕过 TUN

**解决方案**：使用 easytier-core 内置的 SOCKS5 代理
- 网络 TOML 配置 `socks5_proxy = "socks5://0.0.0.0:32259"`
- `getSocks5Proxy()` 从 `NetworkConfig.enable_socks5` + `socks5_port` 读取端口
- WOL HTTP 请求经 `socks5://127.0.0.1:32259` → EasyTier 虚拟网 → 路由器 → PC

**曾尝试但否决的方案**：
- 移除 `disallowedApplications` → 会导致 VPN 死循环
- 前端 `fetch()` → WebView mixed content 限制 + 不支持 SOCKS5
- 从 `rawToml` 正则解析端口 → `rawToml` 可能为空（非 home.vue 编辑时）

### 2. 组网 Loading 状态 — 已解决
- `netConnecting` 控制中央按钮 loading
- `netRunning && !peers.length` → 标题栏 "Waiting for peers..."
- 加载中用 `netDiscovering` computed（连接后 12 秒内空列表视为发现中）
- `netConnecting && !netRunning` 条件防止周期性 `fetchNetInfo` 误触发中央按钮

### 3. 中文语言切换 — 已解决
- 自定义 `tt(key)` 函数 + 内联字典，不依赖 easytier-frontend-lib 的 i18n 系统
- 所有 home.vue 用户可见文字均已双语化

### 4. Gradle 构建缓存问题 — 已解决
- **根因**：bash (MSYS2) 文件操作与 Windows NTFS 文件锁不兼容
- **解决**：Gradle 必须用 **PowerShell** 执行，并设置 `$env:GRADLE_USER_HOME = "D:/tmp/gradle-tmp"`

### 5. 前端改动后 APK 不更新 — 已解决
- **根因**：Tauri build.rs 将 `dist/` 嵌入 `.so`，只重编前端不重编 Rust 导致 APK 加载旧前端
- **解决**：任何改动都必须三步全跑。纯前端改动可只删 easytier-gui 构建产物，秒级重链

### 6. Dialog 设计问题 — 已解决
- PrimeVue Button 太大且不美观 → 改用自定义 `<button class="md-dlg-btn">` Material 风格小按钮
- Footer 新增"清空"按钮
- 标题字号从 `1.05rem` 降为 `0.95rem`
- 关闭按钮去掉圆形边框
- 背景双层圆角修复
- 打开编辑器时重新读取配置（`openWolEditorFn` 从 localStorage 读取）

### 7. 连接状态条重构
- 断开按钮从连接条移至标题栏红色图标（插头+斜杠）
- 连接状态下隐藏编辑和切换网络按钮
- IP 行距最小化（`line-height: 1.3`）
- IPv6 和 IPv4 在同一 div 中用 `\n` + `white-space: pre-line` 显示
- 有内容时才显示连接条（`v-if="netSelfIp || natLabel(...).text"`）

### 8. 网络设备卡片布局重构
- P2P/RELAY 标签移至右下角（tcp/nat 前）
- 主机名移至第一行 IP 后（浅色小字）
- 第二行仅剩流量信息
- 网络质量块右上角（`align-self: flex-start`）
- NAT 类型映射：Open→Open, FullCone→NAT1, Restricted→NAT2, PortRestr→NAT3, Sym→NAT4

### 9. 路由器检测状态机
- `routerOnline`: `undefined → true/false`，仅首次加载置 `undefined`
- `wolFirstCheck` 标志控制：首次显示"检测中..."，后续保持上次状态
- 路由器离线时卡片 `opacity: 0.55`，所有按钮隐藏
- phase poll 中检测路由器离线时立即中止动画
- ET 模式通过 SOCKS5 检测路由器（`httpGet(router_ip/cgi, proxy)`）
- LAN 模式通过 `tcpPing` 直连检测

### 10. Toast 替换
- 移除 PrimeVue `useToast`，用自定义 Material snackbar（`showSnack(msg)`）
- 底部居中胶囊，2.5s 自动消失，滑入动画
- 连接成功不再弹 toast

### 11. WOL 配置字段重命名
- `wol_proxy` → `router_ip`（更直观）
- 移除旧 `router_ip` 字段
- 展开卡片显示 "Router IP: xxx" 而非 "WOL Proxy: xxx"

## 当前仍待解决的问题

### 文本选择上下文菜单（未解决，根因已定位，现阶段不考虑开发）
- **现象**：Dialog 中 Textarea 可以选择文字，但 Android 原生复制/粘贴工具栏不弹出
- **排查结论**：
  - CSS/JS 无拦截（无 `user-select: none`、无 `oncontextmenu` 拦截）
  - wry 无 ActionMode 覆盖（源码审计：RustWebView、WryActivity、RustWebChromeClient、RustWebViewClient 均无相关代码）
  - AppCompat 1.6.1 正确透传 TYPE_FLOATING
  - 无 FLAG_SECURE、无悬浮窗、无 startActionMode 重写
  - 手动 `webView.startActionMode(callback, TYPE_FLOATING)` 能正常弹出（诊断验证通过）
  - `setCustomSelectionActionModeCallback` 在 SDK 34 stubs 中隐藏，反射调用亦无效
- **根因**：Chromium WebView 147 内部 `SelectionPopupController` 在此配置下不触发 ActionMode（Chromium 边缘 bug）
- **可行方案**：JS `selectionchange` 桥接 + 反射 `startActionMode` + `evaluateJavascript` 执行 `document.execCommand`——已验证菜单可弹出、复制功能正常。粘贴因 `execCommand('paste')` 被安全策略禁用而失效，后未继续

### 配置编辑框粘贴功能（已解决）
- **最终方案**：后台定时轮询 + 纯 v-model 赋值
- **错误历程**（重要教训）：
  1. 试图操作 DOM（`el.value = text` + `dispatchEvent`）——失败，因为 `ref="wolTextareaRef"` 拿到的是 PrimeVue Textarea **组件实例**而非原生 `<textarea>`，`dispatchEvent('input')` 触发 v-model 从 DOM 读回旧值覆盖
  2. 试图绕过焦点问题（span 替代 button、touchstart.prevent 等）——均无效
  3. CountDownLatch 桥接 UI 线程读 ClipboardManager——正确且必要，但赋值环节的自相冲突掩盖了正确性
- **正确做法**：`wolToml.value = cachedClipboard.value` ——仅通过 Vue 响应式系统更新，不动 DOM、不发事件

## 剪贴板架构

```
MainActivity.kt:
  PasteBridge.readClipboard() [@JavascriptInterface, JavaBridge线程]
    → CountDownLatch(500ms)
      → mainHandler.post { ClipboardManager.getPrimaryClip() } [UI线程]
    → return text

home.vue:
  readClipboard(): _easytier_paste.readClipboard?.() || ''
  importFromClipboard(): wolToml.value = readClipboard()  // 每次点击时直接调用
```

- 剪贴板必须在 UI 线程读取（JavaBridge 线程读可能失败）
- 每次点击「从剪切板导入」时直接调用 `readClipboard()`，不再使用定时轮询
- v-model 绑定后不要手动操作 DOM——Vue 响应式系统自己处理更新

### 插头图标已替换
- 三个图标已用用户提供的 SVG 替换：切换网络（左右箭头）、连接（链式）、断开（断链）

## 已解决问题（本会话新增）

### 12. Dialog 样式修复
- 消除双层背景：给 PrimeVue 中间层 `.p-dialog-header/content/footer` 全设 `background: transparent`
- 标题栏缩小：padding `18px 18px 2px` → `12px 16px 0`
- 关闭按钮 focus 圆圈：`:focus { outline:none; box-shadow:none }`
- 底部按钮留白减半：`padding: 4px 8px 12px` → `2px 4px 6px`
- 按钮居左问题：PrimeVue Dialog footer 默认 `justify-content: flex-end`，需给 `.md-dlg-ft` 加 `width: 100%`

### 13. 图标替换
- 切换网络：插头 → 左右箭头交换（`viewBox="0 0 1024 1024"`）
- Connect 按钮：服务器 → 链式连接
- 断开按钮：插头+斜杠 → 断链

### 14. 列表项布局优化
- 网络质量块：移除 `align-self: flex-start; margin-top: 3px`，垂直自然居中
- Hostname 扩展到质量块：服务器和网络设备均用 `flex-1 min-w-0` 容器，不再受底部 P2P/TCP/NAT 标签约束
- 路径标签（ET-LAN/LAN）改为蓝色（NAT 标签同款）

### 15. 关机呼吸灯修复
- **原因**：`startPhasePoll` catch 块中连接失败时 `online` 未设为 `false`，条件 `phase==='shutting' && !online` 永远不满足，循环持续 12 次（60s）
- **修复**：catch 中判断 `phase==='shutting'` → 连接失败即判定 Agent 已关闭 → 立即设 `online: false, phase: 'idle'`

### 16. 配置编辑框「从剪切板导入」按钮
- 替换清空+粘贴图标按钮为单个文字按钮「从剪切板导入」
- 居左、取消/保存居右
- 仅调 `wolToml.value = cachedClipboard.value`，不动 DOM

### 17. 剪贴板架构重构：定时轮询 → 按需调用
- 移除 `setInterval` 2 秒轮询 `cachedClipboard`
- `importFromClipboard()` 每次点击时直接调用 `readClipboard()`（JS bridge）
- 简化代码，减少不必要的后台读取

### 18. Dialog Textarea 底部间距修复
- **原因**：PrimeVue Textarea 有默认 `margin-bottom`，且 content padding-bottom 8px 叠加产生过大空隙
- **修复**：content `padding: 8px 16px` → `padding: 8px 16px 0`；新增 `.md-dialog :deep(.p-textarea) { margin-bottom: 0 !important }`

### 19. 断开按钮颜色微调
- `#e57373`（浅红）→ `#ef5350`（标准红），视觉上更明显

### 20. 默认首页设置
- 设置页新增「默认首页」选项（位于语言下方），右侧下拉选择 WOL / 组网
- 选择存入 `localStorage['defaultTab']`，仅首次加载时影响 `activeTab` 初始值
- 新增 `setDefaultTab()` 方法，下拉用原生 `<select>` 元素（`.md-select` 样式）

### 21. 语言标签样式更新
- 灰色（`.md-lang-badge`）→ 蓝色（ET-LAN 同款 `#e3f2fd` / `#1565c0`）
- 文字从缩写 `中` / `EN` → 完整 `中文` / `English`

### 22. Cargo 增量编译缓存修复（构建加速 4 倍）

详见 [项目构建方式.md](项目构建方式.md) 第三章（历史构建问题及根因）。

## 开发教训

1. **有 v-model 绑定就不要手动操作 DOM**：`el.value = text` + `dispatchEvent('input')` 和 `wolToml.value = text` 三件事相互冲突。v-model 下只需设 ref 值，Vue 自己渲染到 DOM
2. **`ref` 在组件上拿到的是组件实例，不是原生元素**：PrimeVue `<Textarea ref="xxx">` 的 ref 不是 `<textarea>` DOM
3. **Android ClipboardManager 必须在 UI 线程读**：JavaBridge 线程可能失败
4. **点击事件中读剪贴板可能受阻**：定时后台轮询 + 缓存可避让

## 设计规范速查

### 颜色体系
- NAT: 绿(NAT1/2/Open) / 蓝(NAT3) / 橙(NAT4)
- 网络质量: 绿(正常) / 蓝(loss>3%||lat>50) / 橙(loss>5%||lat>100) / 红(loss>10%||lat>300)
- P2P/Local: 绿色 / Relay: 橙色
- 断开按钮: `#e57373`
- 路径标签(ET-LAN/LAN): 蓝色 `#e3f2fd` / `#1565c0`（与 NAT1 标签同款）

### 卡片布局
```
网络设备: [IP] [hostname]        [质量块]
          [↑↓流量]    [P2P] [tcp] [NAT]

服务器:   [hostname]             [质量块]
          [↑↓流量]    [P2P] [tcp] [NAT]
```

### 构建后拷贝
每次构建完成自动 `Copy-Item` 到 `F:\tmp\app-arm64-release.apk`。
TODO: 将 powershell 构建部分实现自动化，设置settings.json后，不再为此浪费 token

## 后续方向

1. ~~文本选择上下文菜单~~（仅记录，现阶段不考虑开发）
2. 测试局域网 WiFi 场景下 WOL
3. 完善 Debug 模式功能
4. 处理其他 UI 细节
