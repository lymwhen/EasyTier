# EasyTier + WOLPlus 整合项目

Tauri v2 Android App，集成 EasyTier 组网与 WOL 电脑设备管理。

## 基本原则

1. **不改 easytier-core**：路由器使用官方二进制，跟随上游更新。
2. **easytier-gui 最小改动**：仅新增页面，保留原 `index.vue` 不变（路径 `/`），方便合并上游。
3. **luci-app-wolplus 可改**：作为 WOL 中间层，CGI API 部署在路由器 `/cgi-bin/wolplus-api`。
4. **配置驱动**：WOL 设备通过 TOML 配置文件管理（独立于网络 TOML），不提供图形化添加。
5. **Material Design 风格**：参考 WOLPlus 的卡片设计。

## 相对上游变更清单

> 以下为 `HEAD` 相对 `upstream/main` 的全部文件级变更及必要性说明。
> 每次对上游文件进行新增/修改后，**必须**同步更新此表格。
> 分支 `main` 的 merge base: `73bea01`，上游最新: `upstream/main`。

### 核心功能变更（必要）

| 文件 | 变更 | 必要性 |
|------|------|--------|
| `easytier-gui/src/pages/home.vue` | **[新增]** WOL/网络/路由器/设置四个 Tab 主界面（~1500行） | 必要 — 项目核心 |
| `easytier-gui/src/App.vue` | 启动 Splash 遮罩 + 后端就绪检测 + 自动跳转 `/home` | 必要 — 初始化流程 |
| `easytier-gui/src/composables/backend.ts` | 新增 `httpGet`/`httpPost`/`tcpPing`/`luciProxy*` IPC 封装 | 必要 — 前端 HTTP/代理调用 |
| `easytier-gui/src-tauri/src/lib.rs` | 新增 `http_get`/`http_post`/`tcp_ping` 命令 + LuCI HTTP 反向代理完整实现（~360行） | 必要 — 后端 HTTP/代理支持 |
| `easytier-gui/src-tauri/Cargo.toml` | 新增 `reqwest(socks)`/`hyper`/`http-body-util`/`hyper-util`/`bytes` + `tauri-plugin-opener`/`tauri-plugin-clipboard-manager` 依赖 | 必要 — Rust 侧 HTTP 客户端 + 代理服务器 + 跨平台剪贴板/浏览器 |
| `easytier-gui/index.html` | viewport 添加 `maximum-scale=1.0, user-scalable=no` | 必要 — 禁用 Android 双指缩放 |
| `easytier-gui/src-tauri/tauri.conf.json` | 插件权限配置：`shell.open` scope、clipboard-manager 权限 | 必要 — 跨平台浏览器打开 + 剪贴板 |
| `easytier-gui/src-tauri/capabilities/migrated.json` | 新增 `shell:allow-open`、`clipboard-manager:allow-read`、`clipboard-manager:allow-write` 权限 | 必要 — 插件能力声明 |

### Android 原生层（必要）

| 文件 | 变更 | 必要性 |
|------|------|--------|
| `MainActivity.kt` | 新增主题 Bridge（`ThemeBridge`，动态切换状态栏颜色）+ 禁用双指缩放 + 移除旧 `PasteBridge`（剪贴板已迁移至 `tauri-plugin-clipboard-manager`） | 必要 — Android 主题/缩放/剪贴板 |
| `AndroidManifest.xml` | `usesCleartextTraffic="true"` 硬编码 | 必要 — 局域网 HTTP 请求 |
| `app/build.gradle.kts` | release 构建添加 `manifestPlaceholders["usesCleartextTraffic"]="true"` | 必要 — 配合明文 HTTP |
| `themes.xml` / `values-night/themes.xml` | 状态栏/导航栏颜色（浅色 `#f5f5f5` / 深色 `#121212`） | 必要 — UI 适配 |
| 所有 `mipmap-*` 图标 | 替换为 EasyTier 品牌图标 | 必要 — 品牌 |
| `ic_launcher_foreground.xml` | 矢量图标改为 EasyTier 三层 V 形 | 必要 — 品牌 |
| `ic_launcher_background.xml` | 背景色改为白色，移除网格线 | 必要 — 品牌 |

### 构建/配置（必要，含副作用）

| 文件 | 变更 | 必要性 |
|------|------|--------|
| `easytier-gui/package.json` | build script 移除 `vue-tsc --noEmit`（跳过 TS 类型检查加速构建） | 必要 — 构建加速 |
| `pnpm-workspace.yaml` | 添加 `allowBuilds: esbuild, unrs-resolver, vue-demi` | 必要 — pnpm 原生构建许可 |
| `pnpm-lock.yaml` | 依赖锁文件更新（因新增依赖变更） | 必要 — 附带产物 |
| `Cargo.lock` | Rust 依赖锁更新（新增 reqwest/hyper + kcp-sys rev） | 必要 — 附带产物 |
| `easytier-gui/src/auto-imports.d.ts` | **[自动生成]** unplugin-auto-import 更新 | 附带 — 新增 IPC 函数自动导入声明 |
| `easytier-gui/src/typed-router.d.ts` | **[自动生成]** 新增 `/home` 路由声明 | 附带 — 新增页面自动路由注册 |
| `.gitignore` | 新增 1 行忽略规则 | 必要 — 忽略本地构建产物 |

### CI/CD（Fork 适配）

| 文件 | 变更 | 必要性 |
|------|------|--------|
| `.github/workflows/release-tag.yml` | **[新增]** 全平台构建 + GitHub Release 自动发布（455行） | 必要 — CI 全平台构建 |
| `.github/actions/prepare-pnpm/action.yml` | 添加 `--no-frozen-lockfile`（因 lockfile 与上游不一致） | Fork 适配 |
| `.github/workflows/docker.yml` | Docker 镜像标签通用化为 `${{ github.repository_owner }}` | Fork 适配 |

### 文档/资源

| 文件 | 变更 | 必要性 |
|------|------|--------|
| `CLAUDE.md` | **[新增]** Claude Code 项目指令 | 必要 — 开发规范 |
| `docs/project.md` | **[新增]** 结构化项目文档 | 必要 — 文档 |
| `docs/CI全平台构建.md` | **[新增]** CI 构建文档 | 必要 — 文档 |
| `docs/本地Android构建.md` | **[新增]** 本地 Android 构建文档 | 必要 — 文档 |
| `README.md` | 重写为中文 README + 功能清单表 + 截图 | 必要 — 文档 |
| `assets/image-*.png` | **[新增]** 4 张界面截图 | 必要 — 文档配图 |

### 已同步至上游（无差异）

以下文件此前与上游存在差异，已在 `16b7d7e` 合并 `upstream/main` 后归零：

| 文件 | 说明 |
|------|------|
| `easytier/Cargo.toml` | kcp-sys rev 跟随上游回退至 `d7427c22`（上游在 `bfa3383` 主动 revert） |
| `.github/workflows/Dockerfile` | HEALTHCHECK 跟随上游恢复 |
| `easytier-gui/.../BuildTask.kt` | 文件末尾空行移除，与上游一致 |
| `easytier-contrib/easytier-ohrs/Cargo.lock` | kcp-sys rev 跟随上游同步 |

## 功能清单（相对上游新增/优化）

> 以下梳理自 `a72a593`（首个 WOL 功能提交）至 `HEAD` 的所有变更，覆盖功能增强、UI 优化、CI 建设、平台适配等维度。

<table>
<tr><th>模块</th><th>功能项</th><th>平台支持情况</th></tr>
<tr><td rowspan="8"><b>整体UI</b></td><td>Material Design 风格全面美化 — 自定义 CSS 变量体系（<code>:root</code> 级）、卡片 12px 圆角 + 阴影、三主题系统（浅色/深色/AMOLED 平等独立，<code>html[data-theme]</code> 属性驱动，<code>applyTheme()</code> 统一状态机，移除全部 <code>@media (prefers-color-scheme: dark)</code> 避免 JS 不可控）</td><td>全平台</td></tr>
<tr><td>底部 Tab 导航栏（电脑 / 组网 / LuCI / 设置），默认首页可配置（localStorage 持久化）</td><td>全平台</td></tr>
<tr><td>App 图标更换为 EasyTier 网络层叠图标 + Splash 遮罩优化（路由跳转完成后隐藏，最低 500ms）</td><td>全平台</td></tr>
<tr><td>全部操作图标统一更换 — 标题栏编辑/刷新/切换、底部 Tab、唤醒铃铛面型图标、关机电源面型图标、Connect 链式连接、断开断链 SVG</td><td>全平台</td></tr>
<tr><td>断开按钮颜色匹配网络质量最差红（<code>#ef5350</code>），编辑器关闭按钮隐藏、Textarea 间距优化</td><td>全平台</td></tr>
<tr><td>图表 SVG translate + CSS left 固定 20px Y 轴标签区 — 平板/窄屏适配，不再左侧过宽</td><td>全平台</td></tr>
<tr><td>自定义 Material snackbar 底部居中胶囊 Toast（替代 PrimeVue useToast）— 2.5s 滑入动画</td><td>全平台</td></tr>
<tr><td>中英双语 — 自定义 <code>tt(key)</code> 函数 + 内联 i18n 字典（不依赖 easytier-frontend-lib），用户面文本全部双语化</td><td>全平台</td></tr>
<tr><td rowspan="7"><b>电脑<br>(WOL)</b></td><td>设备卡片 — 在线状态指示灯（脉冲动画）+ 名称 + IP + 状态文字，点击展开详情（MAC / Router IP / Interface / Agent Port）</td><td>全平台</td></tr>
<tr><td>远程唤醒 — App → HTTP GET 路由器 CGI（<code>luci-app-wolplus</code>）→ <code>etherwake</code> 发送魔术包；局域网直连 / EasyTier SOCKS5 隧道双路径</td><td>全平台</td></tr>
<tr><td>在线状态检测 — App → HTTP GET PC Agent（Go，32249 端口，<code>/api/v1/status</code>）；30 秒自动轮询，路由器离线时中止并置灰卡片</td><td>全平台</td></tr>
<tr><td>远程关机 — App → HTTP POST PC Agent（<code>/api/v1/shutdown</code>），Agent 执行 <code>shutdown /s /t 5</code>；含关机失败回退 online 机制（12 次 / 60s 检测）</td><td>全平台</td></tr>
<tr><td>唤醒/关机状态机 — idle → waking（橙色脉冲 5s）→ online / offline；online → shutting（红色脉冲）→ offline / 回退 online</td><td>全平台</td></tr>
<tr><td>设备配置 TOML 文本编辑器（等宽字体 Dialog）— 支持从剪贴板导入；路由器离线时卡片半透明（opacity 0.55）、操作按钮隐藏</td><td>全平台</td></tr>
<tr><td>路径标签（ET-LAN / LAN 蓝色胶囊）— 根据 <code>netRunning</code> 与 <code>router_ip</code> 自动判断路径模式</td><td>全平台</td></tr>
<tr><td rowspan="7"><b>组网</b></td><td>Peer 卡片展示 — IP + 主机名 + 延迟/丢包率 + 上下行流量 + P2P/Relay 标签 + NAT 类型芯片 + 隧道协议标签；点击展开版本号</td><td>全平台</td></tr>
<tr><td>网络质量颜色分级 — 绿（正常）/ 蓝（loss>3% 或 lat>50）/ 橙（loss>5% 或 lat>100）/ 红（loss>10% 或 lat>300）；NAT 标签：绿/绿/绿/蓝/橙对应 Open/NAT1/NAT2/NAT3/NAT4</td><td>全平台</td></tr>
<tr><td>Peer 排序 — 路由器优先（hostname 匹配 <code>/route|wrt|路由|home|家/i</code>），其余按 IP 排序；服务器按 hostname 字母排序</td><td>全平台</td></tr>
<tr><td>实时速率图表 — SVG 平滑折线图，蓝色上行 / 柔红下行，3 分钟窗口（60 点），HTML 坐标轴标签覆盖 SVG 之上；连接重建时速率防负值 + 清空旧数据</td><td>全平台</td></tr>
<tr><td>多网络管理 — 切换/新增/删除网络，当前网络标记 ✓，底部 "Add Network" 入口；网络配置 TOML 编辑器（含剪贴板导入）</td><td>全平台</td></tr>
<tr><td>连接状态条 — 本机 IP + 设备名 + 版本号 + 红色断开按钮；连接中显示 "Connecting..."；netDiscovering 12 秒内空列表显示 "Waiting for peers..."</td><td>全平台</td></tr>
<tr><td>3 秒自动刷新 Peer 数据；连接时自动检测全部 WOL 设备状态</td><td>全平台</td></tr>
<tr><td rowspan="5"><b>LuCI</b></td><td>Hyper 本地 HTTP 反向代理 — 自动向路由器发起 LuCI 登录（POST <code>/cgi-bin/luci/</code>），提取 <code>sysauth</code> / <code>sysauth_http</code> cookie，后续请求自动注入</td><td>全平台</td></tr>
<tr><td>iframe 嵌入路由器管理页面 — 全屏 100%×100%；SOCKS5 代理支持 EasyTier 隧道访问远程路由器</td><td>全平台</td></tr>
<tr><td>会话过期自动重登（检测 302 重定向到 login），刷新保持当前 URL（<code>last_path</code> 仅追踪 <code>text/html</code> 响应，过滤 CSS/JS/API 资源请求）</td><td>全平台</td></tr>
<tr><td>多路由器管理 — TOML 配置路由器列表，支持切换/新增/删除；删除当前路由器自动切换至剩余第一个</td><td>全平台</td></tr>
<tr><td>路径持久化 — 刷新或路径模式切换（ET-LAN ↔ LAN）后，代理重启自动恢复上次浏览位置；v-show 切回 Tab 不重载 iframe</td><td>全平台</td></tr>
<tr><td rowspan="8"><b>设置</b></td><td>高级设置入口（默认隐藏，开启调试模式后显示）— 跳转原 <code>index.vue</code>（<code>/</code> 路由）访问 easytier-frontend-lib 自带高级面板</td><td>全平台</td></tr>
<tr><td>语言切换 — 地球图标 + 当前语言蓝色胶囊标签（中文 / English），点击 <code>toggleLang()</code> 即时切换所有 UI 文字</td><td>全平台</td></tr>
<tr><td>默认首页 — 原生 <code>&lt;select&gt;</code> 下拉（电脑 / 组网 / LuCI），<code>localStorage['defaultTab']</code> 持久化</td><td>全平台</td></tr>
<tr><td>调试模式 — toggle 开关，开启后在组网页底部显示诊断信息 + 解锁高级设置菜单项；副标题"显示高级选项和调试信息"</td><td>全平台</td></tr>
<tr><td>一键配置导入/导出 — WOL + LuCI + 网络配置打包为 JSON（<code>v:1</code> 结构），Export 写入剪贴板；Import 含 JSON 校验 + 二次确认弹窗防误操作覆盖</td><td>全平台</td></tr>
<tr><td>AMOLED 纯黑模式 — 开关 toggle（<code>localStorage['amoledMode']</code>），作为与浅色/深色平等的第三个独立主题由 <code>applyTheme()</code> 统一管理，纯黑背景色（<code>#000</code>）+ 适配 Dialog/卡片/代码块/标题栏/Tab 栏等全部组件</td><td>全平台</td></tr>
<tr><td>帮助对话框 — 展示 WOL、EasyTier VPN、LuCI 路由器、配置导入导出四大模块的功能说明和示例配置（含配置代码复制按钮）；WOL 部分提供 <code>luci-app-wolplus</code> 下载链接（通过 <code>tauri-plugin-opener</code> 打开浏览器）</td><td>全平台</td></tr>
<tr><td>关于 — 显示版本号（来自 <code>package.json</code> 的 <code>pkg.version</code>）</td><td>全平台</td></tr>
<tr><td rowspan="6"><b>Android<br>平台适配</b></td><td>跨平台剪贴板 — 从 <code>PasteBridge</code>（<code>@JavascriptInterface</code> + <code>CountDownLatch</code>，仅 Android）迁移至 <code>tauri-plugin-clipboard-manager</code>（<code>readText</code>/<code>writeText</code>），全平台可用</td><td>全平台</td></tr>
<tr><td>动态主题适配 — <code>MainActivity.kt</code> 新增 <code>ThemeBridge</code>（<code>@JavascriptInterface</code>），前端 <code>applyTheme()</code> 统一调用 <code>setStatusBarStyle(dark)</code> + <code>setAmoledMode(bool)</code> 动态切换 Android 状态栏/导航栏颜色（浅色 <code>#f5f5f5</code> / 深色 <code>#121212</code> / AMOLED <code>#000</code>）</td><td>Android</td></tr>
<tr><td>VPN 路由配置 — <code>mobile_vpn.ts</code> 设置 <code>disallowedApplications</code> 将 App 自身排除出 VPN 避免路由死循环；配置 <code>routes</code> 将虚拟网段路由至 TUN</td><td>Android</td></tr>
<tr><td><code>AndroidManifest.xml</code> 添加 <code>usesCleartextTraffic="true"</code> 允许 HTTP 明文请求（访问局域网路由器 CGI 和 PC Agent）</td><td>Android</td></tr>
<tr><td>文本选择上下文菜单 — 已调查（Chromium WebView 147 边界 bug，<code>SelectionPopupController</code> 不触发 ActionMode），不计划修复，已提供 JS 桥接 + 剪贴板按钮替代方案</td><td>Android</td></tr>
<tr><td>APK 四架构构建 — aarch64 / armv7 / i686 / x86_64</td><td>Android</td></tr>
<tr><td rowspan="4"><b>CI / 构建</b></td><td>Tag 推送自动触发全平台构建 + GitHub Release Draft — <code>release-tag.yml</code> (456 行)，矩阵含 Core CLI ×16、GUI 桌面 ×7、Android APK ×4、Magisk 模块</td><td>全平台（CI 产物）</td></tr>
<tr><td>CI 兼容适配 — <code>prepare-pnpm</code> 添加 <code>--no-frozen-lockfile</code>、<code>docker.yml</code> 仓库名通用化、<code>vue-tsc</code> 类型检查临时跳过、<code>BuildTask.kt</code> 还原上游跨平台版本</td><td>Linux CI Runner</td></tr>
<tr><td>Rust 代码 CI 合规 — <code>lib.rs</code> 通过 <code>cargo fmt</code> 格式检查 + <code>cargo clippy -D warnings</code>，确保上游 OHOS / Test workflow 通过</td><td>Linux CI Runner</td></tr>
<tr><td>Cargo 增量编译缓存修复 — CC/AR/BINDGEN/PROTOC/LIBCLANG_PATH 环境变量固化到 <code>.cargo/config.toml [env]</code>，构建加速 4 倍（~10min → ~2m30s）</td><td>本地开发（Windows + MSYS2）</td></tr>
<tr><td rowspan="2"><b>跨平台 GUI</b></td><td>桌面安装包构建 — Linux（deb/rpm/AppImage，x86_64 + aarch64）、macOS（dmg，x86_64 + aarch64）、Windows（nsis installer，x86_64 + i686 + aarch64）</td><td>Windows / macOS / Linux</td></tr>
<tr><td>Desktop GUI 基于 Tauri v2 — 内嵌 easytier-core，Core 模式直连（无需 easytier-web），TOML 配置文件驱动</td><td>Windows / macOS / Linux</td></tr>
</table>

> **平台说明**："全平台"指 Windows / macOS / Linux 桌面端 + Android 移动端均支持。标记为特定平台的项仅在对应平台上生效。

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
| `src-tauri/gen/android/.../MainActivity.kt` | 修改 | 主题桥接（`ThemeBridge`），已移除旧 `PasteBridge` |
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

提供 8 个菜单项，每项含图标、标题和副标题：

**Advanced Settings（高级设置）**
齿轮图标，副标题 "Mode, logging, config server, language"。**默认隐藏**，仅在开启调试模式后显示。点击跳转到原始页面 `/`（easytier-frontend-lib 自带的高级设置面板），排在调试模式开关下方。

**Language（语言）**
地球图标，副标题显示当前语言和 "点击切换语言" 提示，右侧蓝色胶囊标签显示 `中文` 或 `English`。点击调用 `toggleLang()`，切换 `locale` ref 并写入 `localStorage['easytierLang']`，UI 所有文本即时更新。

**Default Tab（默认首页）**
九宫格图标，副标题显示当前默认首页名称。右侧原生 `<select>` 下拉，可选电脑 / 路由器 / 组网。选中值存入 `localStorage['defaultTab']`，仅首次加载时影响 `activeTab` 初始值。

**Debug Mode（调试模式）**
齿轮图标，副标题 "显示高级选项和调试信息"，右侧开关 toggle。开启后生效两处：1) 组网页底部显示 Debug 区域（本机 IP、端口、raw TOML 文本等诊断信息）；2) 解锁高级设置菜单项（显示在调试模式下方）。

**Configuration Info（配置信息）**
云下载图标，副标题 "全部数据导入与导出" / "Import/Export all data"，右侧两个操作按钮：
- **Export（导出）**：绿色文字按钮。将 WOL 配置（`wolDevicesToml`）、LuCI 配置（`luciRoutersToml`）、网络配置列表（`networkList` 中各网络的 `rawToml`）打包为 JSON 对象，通过 <code>tauri-plugin-clipboard-manager</code> 的 <code>writeText()</code> 写入剪贴板，底部 snackbar 提示 "配置信息已导出到剪切板"。
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

**Help（帮助）**
帮助圆环图标，副标题 "模块说明"。点击打开帮助 Dialog，展示四个模块的功能说明、示例配置代码（含一键复制按钮）：
- **WOL**：luci-app-wolplus + wolplus-agent 安装说明，提供下载链接（通过 `tauri-plugin-opener` 的 `openUrl()` 在浏览器打开）
- **EasyTier VPN**：P2P 虚拟组网原理和功能特性说明
- **LuCI 路由器**：OpenWrt 管理面板接入说明（LAN / VPN 双模式）
- **配置导入导出**：TOML 配置文件结构和导入导出操作说明

**AMOLED Black（AMOLED 纯黑）**
深色半圆图标，副标题 "AMOLED 屏幕纯黑色模式"。右侧开关 toggle（`localStorage['amoledMode']`）。开启后 AMOLED 作为与浅色/深色平等的第三个独立主题，由 `applyTheme()` 统一状态机管理。纯黑背景色（`#000`），卡片/标题栏/Tab 栏/代码块/分隔线等全部组件独立适配，不再作为深色的子类。

**About（关于）**
信息图标，副标题显示版本号（来自 `package.json` 的 `pkg.version`）。

### LuCI 路由器管理

#### 架构

通过 Rust 侧 Hyper 反向代理，在 iframe 中嵌入路由器 LuCI Web 管理面板。核心流程：

```
Tauri IPC (luci_proxy_start)
  → Rust: reqwest 向路由器发起 LuCI 登录请求（POST /cgi-bin/luci/）
    → 从响应 Set-Cookie 中提取 sysauth 或 sysauth_http cookie
  → Rust: TcpListener 绑定 127.0.0.1 随机端口，启动 Hyper HTTP 代理服务
    → 代理在每次转发请求时自动注入登录 cookie
    → 检测 session 过期（302 重定向到 login），自动重新登录并重试
  → 返回代理 URL: http://127.0.0.1:<port>
  → 前端 iframe src 指向代理 URL + /cgi-bin/luci/admin/
```

**路径模式**：与 WOL 设备检测一致，ET 模式下 `luciProxyStart` 接收可选 `socks5Proxy` 参数，reqwest 经 SOCKS5 代理连接路由器。LAN 模式下直连。路径模式改变时（ET-LAN ↔ LAN），代理自动完全重启（`refreshLuciIframe`）。

**路径持久化**：代理通过 `last_path` 记录用户当前浏览的 LuCI 页面路径。刷新或路径模式切换时，代理重启后自动恢复到上次浏览位置，而非回到首页。

`last_path` 仅在响应 `Content-Type: text/html` 时更新，过滤掉 CSS/JS/图片/API 等资源请求，避免资源路径污染页面追踪（详见重难点 5）。

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
  --md-card: #fff;
  --md-text: #212121;
  /* 浅色主题默认值 */
}
html[data-theme="dark"] {
  --md-primary: #42a5f5;
  --md-card: #1e1e1e;
  --md-text: #e0e0e0;
  /* 深色主题覆写 */
}
html[data-theme="amoled"] {
  --md-primary: #42a5f5;
  --md-card: #0f0f0f;
  --md-text: #e0e0e0;
  --md-shadow: none;
  background: #000;
  /* AMOLED 独立主题覆写 */
}
```

- 卡片：`border-radius: 12px`，`box-shadow` 阴影
- 标题栏：`min-height: 56px`
- CSS 变量须在 `:root` 而非 `.md-app`（PrimeVue Dialog 渲染在 app div 外）
- 三主题系统：浅色/深色/AMOLED 为三个平等独立的主题，通过 `html[data-theme="light|dark|amoled"]` 属性驱动，`applyTheme()` 统一状态机管理（综合 `themeMode` + `amoledMode` + 系统 `prefers-color-scheme`），已移除全部 `@media (prefers-color-scheme: dark)` 避免 CSS 优先级冲突；`.app-dark` class 仅保留供 PrimeVue `darkModeSelector` 使用
- **`:deep()` 在 unscoped `<style>` 中无效**：Vue 3 SFC 编译器仅在 scoped 块中处理 `:deep()`，unscoped 块中直接透传至浏览器。浏览器将 `:deep()` 视为无效 CSS 伪类，静默丢弃整条规则。修复：unscoped 块移除所有 `:deep()`，改用原生后代/复合选择器
- **PrimeVue Dialog Teleport 导致 `.md-dialog .p-dialog` 无法匹配**：`class="md-dialog"` 与 `.p-dialog` 在 Teleport 渲染的同一根元素上，后代选择器（空格）匹配不到。改用复合选择器 `.md-dialog.p-dialog`（无空格）匹配同一元素上的两个类

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

### 3. 跨平台剪贴板（tauri-plugin-clipboard-manager）

**架构**：

```
home.vue:
  import { readText, writeText } from 'tauri-plugin-clipboard-manager'
  importFromClipboard(): text = await readText()
  exportToClipboard(): await writeText(json)
```

**迁移原因**：旧的 `PasteBridge`（`@JavascriptInterface` + `CountDownLatch` + `mainHandler.post`）仅在 Android 上可用，且实现复杂的跨线程同步。`tauri-plugin-clipboard-manager` 通过 Tauri 原生插件提供全平台一致的剪贴板 API，桌面端和 Android 共享同一套 TypeScript 调用接口。

**要点**：
- `readText()` / `writeText()` 均为 async 函数
- 每次点击「从剪切板导入」时按需调用，不使用定时轮询
- v-model 绑定后仅通过 Vue 响应式系统更新值，不动 DOM

### 4. 文本选择上下文菜单（已知限制）

**现象**：Dialog 中 Textarea 可选择文字，但 Android 原生复制/粘贴工具栏不弹出。

**根因**：Chromium WebView 147 内部 `SelectionPopupController` 在此配置下不触发 ActionMode（Chromium 边缘 bug，非本项目代码问题）。

**排查结论**：CSS/JS 无拦截、wry 无 ActionMode 覆盖、AppCompat 正确透传、手动 `startActionMode` 可弹出。反射调用 `setCustomSelectionActionModeCallback` 在 SDK 34 stubs 中隐藏无效。

**状态**：不计划修复。JS 桥接 + 反射方案可弹出菜单和复制，但粘贴被安全策略禁用。

### 5. LuCI 刷新后 iframe 白屏（last_path 资源污染）

**现象**：LuCI 页面正常浏览后点击标题栏刷新按钮，iframe 变为空白，且 spinner 未显示。首次加载正常，仅刷新触发。

**根因链**：

1. `proxy_request` 对**每个代理请求**都更新 `last_path`，包括页面导航请求（如 `/cgi-bin/luci/admin/status/overview`）和资源请求（如 `/cgi-bin/luci-static/resources/...js`、`/cgi-bin/luci-static/resources/...css`、API 端点）
2. LuCI 页面加载后会触发大量资源请求（JS/CSS/图标/轮询 API）
3. 用户点击刷新时，`luciGetLastPath()` 返回的极可能是最后一次资源请求的路径（如 `.../xhr.js`），而非当前 HTML 页面路径
4. `luciIframeSrc = url + prevPath` 将资源路径作为 iframe src → WebView 尝试将 JS/CSS 文本作为 HTML 渲染 → **白屏**（无可见内容）
5. 此时 iframe 元素存在且 `luciIframeSrc` 非空，template 走 `v-else-if="luciIframeSrc"` 分支渲染 iframe，不会回退到 loading spinner 分支

**关键代码路径**：

```
proxy_request (Rust)
  → 旧: 无条件 *last_path = path  // 每个请求都更新
  → 新: 仅在 is_html(&resp) 时更新  // 只追踪 HTML 页面

refreshLuciIframe (JS)
  → luciIframeSrc = ''             // 立即隐藏 iframe，显示 spinner
  → luciGetLastPath()              // 读取 last_path（现在只含 HTML 页面路径）
  → luciProxyStop()                // 显式停旧代理
  → luciProxyStart()               // 启新代理
  → luciIframeSrc = url + prevPath // 恢复正确的页面路径
```

**方案**：

1. **Rust 侧**：新增 `is_html()` 辅助函数，检查响应的 `Content-Type` 是否包含 `text/html`。`last_path` 仅在 HTML 响应时更新，过滤掉所有资源请求
2. **Rust 侧**：`luci_proxy_start` 中将 `stop_proxy_inner()` 移到 `luci_login` + `TcpListener::bind` 成功之后。新代理一切就绪才杀旧代理，避免登录失败导致旧代理提前死亡
3. **JS 侧**：`refreshLuciIframe` 开头立即设 `luciIframeSrc = ''`（隐藏 iframe 显示 spinner），然后 `luciGetLastPath` → `luciProxyStop` → `luciProxyStart` → 恢复 `luciIframeSrc`。失败时清除 `proxyUrl`，回退到 spinner

**为何首次加载不触发**：`startLuciProxy` 使用硬编码 `/cgi-bin/luci/admin/`（始终是合法 HTML 页面），不依赖 `last_path`。

**教训**：
- HTTP 代理的路径追踪必须区分页面导航和资源请求，仅靠 URL 模式不可靠（不同 Web 应用的路径规范不同），响应 `Content-Type` 是更通用的判断依据
- 刷新类操作应先隐藏当前内容（显示加载态），等新资源就绪后再展示，避免残留旧 URL 指向已销毁的代理
- Rust 侧 proxy 生命周期管理应与 JS 侧的显式控制（`luciProxyStop` → `luciProxyStart`）保持一致的双向确认模式

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

- **本地 Android 构建** — 详见 [本地Android构建.md](本地Android构建.md)
- **CI 全平台构建** — 详见 [CI全平台构建.md](CI全平台构建.md)

### 构建关键点

**Tauri 前端嵌入机制**：`tauri_build::build()` 在 Cargo 编译阶段将 `dist/` 目录（Vite 构建产物）以 BROTLI 压缩格式嵌入 `.so`。只改前端必须重新编译 `.so`，仅跑 Vite 不会更新 APK。

**`-x rustBuildArm64Release` 原因**：Gradle 的 `rustBuildArm64Release` 任务内部调用 `pnpm tauri android android-studio-script`，该 Tauri CLI 子命令通过 `%TEMP%\com.kkrainbow.easytier-server-addr` 文件连接到 Tauri 主进程的 TCP 服务端获取 CLI 配置。此服务端仅 Android Studio 调用 Gradle 时启动，纯命令行构建中不存在。**解决**：Step 1 用 `cargo build --lib` 直接编译 `.so`，Step 2 手动复制到 `jniLibs/` 并清理 Gradle 中间产物（`merged_jni_libs` / `merged_native_libs` / `stripped_native_libs`），然后用 `-x rustBuildArm64Release` 跳过 Gradle 的 Rust 任务。

**Vite + Cargo 同一 bash 调用**：MSYS2 跨会话环境漂移会导致 PROTOC/LIBCLANG_PATH 路径值波动，触发依赖 crate 指纹失效 → easytier 全量重编译（~8min）。同一调用内环境快照一致 → 增量编译（~1.5min）。
