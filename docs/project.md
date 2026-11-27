# EasyTier + WOLPlus 整合项目

Tauri v2 Android App，集成 EasyTier 组网与 WOL 电脑设备管理。

## 基本原则

1. **不改 easytier-core**：路由器使用官方二进制，跟随上游更新。
2. **easytier-gui 最小改动**：仅新增页面，原 `index.vue` 通过路由 redirect 跳转至 `/home`，方便合并上游。
3. **luci-app-wolplus 可改**：作为 WOL 中间层，CGI API 部署在路由器 `/cgi-bin/wolplus-api`。
4. **配置驱动**：WOL 设备通过 TOML 配置文件管理（独立于网络 TOML），不提供图形化添加。
5. **现有玻璃标题栏/Tab 栏 + n3 Material You 内容风格**：底部导航保留玻璃材质和现有图标，布局与状态胶囊参考 `D:\Downloads\n3.html`；总览卡、按钮、标签、组网设备列表按 n3 实现；卡片统一使用电脑列表的入场与展开动画。

## 相对上游变更清单

> 以下为 `HEAD` 相对 `upstream/releases/v2.7.1` 的全部文件级变更及必要性说明。
> 每次对上游文件进行新增/修改后，**必须**同步更新此表格。
> 当前跟进上游：`upstream/releases/v2.7.1`（`0d9c52b1`，tag `v2.7.1`）。

### 核心功能变更（必要）

| 文件 | 变更 | 必要性 |
|------|------|--------|
| `easytier-gui/src/pages/home.vue` | **[新增]** WOL/网络/路由器/设置四个 Tab 主界面；电脑卡片稳定 ID、居中的展开工作区、路由器传输状态提示；v2.7.1 适配：启动时先加载配置再做 RPC 初始化（try/catch 容错），Android 分支调用 `bootstrapAndroidStorageKeepLocal`，`safeLastNetworkId` 兜底 | 必要 — 项目核心 |
| `easytier-gui/src/composables/android_compat.ts` | **[新增]** Fork 适配层：复用上游 `bootstrapAndroidManagement` 完成 native 加密存储迁移与状态注册后，恢复 localStorage 明文键，维持 fork 以 localStorage 为配置真源的模型 | 必要 — v2.7.1 起 Android 的 `init_rpc_connection`/`loadLastNetworkInstanceId` 依赖该存储初始化，否则启动链抛错致配置不显示 |
| `easytier-gui/src/App.vue` | 主题初始化 + 窗口标题 + Toast + RouterView（Splash 已移除） | 必要 — 初始化流程 |
| `easytier-gui/src/composables/theme.ts` | `Theme` 收敛为浅色/深色，保留自动/浅色/深色切换解析；深色统一使用纯黑主题 | 必要 — 主题切换逻辑 |
| `easytier-gui/src/composables/backend.ts` | 新增 `httpGet`/`httpPost`/`tcpPing`/`luciProxy*` IPC 封装 | 必要 — 前端 HTTP/代理调用 |
| `easytier-gui/src/composables/event.ts` | `onSaveConfigs` 改为合并本地网络配置，避免覆盖用户配置；接入上游 `normalizeConfigSource`；v2.7.1 合并时叠加上游 android 早退守卫并保留上游新增 `VPN_SERVICE_CONFIG_CHANGED` 事件 | 必要 — 修复配置覆盖 |
| `easytier-gui/src-tauri/src/lib.rs` | 新增 `http_get`/`http_post`/`tcp_ping` 命令 + LuCI 流式反向代理、文件传输进度事件、长传输超时控制 | 必要 — 后端 HTTP/代理支持 |
| `easytier-gui/src-tauri/Cargo.toml` | 新增 `reqwest(socks,stream)`/`hyper`/`http-body-util`/`hyper-util`/`bytes`/`futures-util` + `tauri-plugin-opener`/`tauri-plugin-positioner` 依赖 | 必要 — Rust 侧流式 HTTP 客户端 + 代理服务器 + 跨平台浏览器/托盘定位 |
| `easytier-gui/index.html` | viewport 添加 `maximum-scale=1.0, user-scalable=no` | 必要 — 禁用 Android 双指缩放 |
| `easytier-gui/src-tauri/tauri.conf.json` | 插件权限配置：`shell.open` scope、clipboard-manager 权限 | 必要 — 跨平台浏览器打开 + 剪贴板 |
| `easytier-gui/src-tauri/capabilities/migrated.json` | 新增 `shell:allow-open`、`clipboard-manager:allow-read`、`clipboard-manager:allow-write` 权限 | 必要 — 插件能力声明 |

### Android 原生层（必要）

| 文件 | 变更 | 必要性 |
|------|------|--------|
| `MainActivity.kt` | 新增主题 Bridge、禁用双指缩放、`DownloadListener + DownloadManager` 下载、加固版文件选择 ChromeClient 包装器 | 必要 — Android 主题/文件传输兼容性 |
| `MainForegroundService.kt` | 主前台服务保留 `dataSync` 类型，避免 Android 10–13 因 `specialUse` 类型不匹配导致启动闪退 | 必要 — Android 启动兼容性 |
| `AndroidManifest.xml` | `usesCleartextTraffic="true"` 硬编码 | 必要 — 局域网 HTTP 请求 |
| `app/build.gradle.kts` | release 构建添加 `manifestPlaceholders["usesCleartextTraffic"]="true"`；补充 `androidx.lifecycle:lifecycle-process:2.6.2` | 必要 — 配合明文 HTTP + Wry 生命周期桥 |
| `themes.xml` / `values-night/themes.xml` | 状态栏/导航栏颜色（浅色 `#FCFCFC` / 深色 `#000000` 纯黑） | 必要 — UI 适配 |
| 所有 `mipmap-*` 图标 | 替换为 EasyTier 品牌图标 | 必要 — 品牌 |
| `ic_launcher_foreground.xml` | 矢量图标改为 EasyTier 三层 V 形 | 必要 — 品牌 |
| `ic_launcher_background.xml` | 背景色改为白色，移除网格线 | 必要 — 品牌 |

### 构建/配置（必要，含副作用）

| 文件 | 变更 | 必要性 |
|------|------|--------|
| `easytier-gui/package.json` | `dev`/`build` 保持定制精简命令，移除 frontend-lib 预构建与 `vue-tsc --noEmit`；新增 `@tauri-apps/plugin-opener` | 必要 — 保持定制构建流程/浏览器打开能力 |
| `pnpm-workspace.yaml` | 保留 native build 白名单和依赖版本约束 | Fork 适配 — 保证本地/CI 安装一致 |
| `pnpm-lock.yaml` | 依赖锁文件更新（因新增依赖变更） | 必要 — 附带产物 |
| `Cargo.lock` | Rust 依赖锁更新（新增流式传输依赖 + kcp-sys rev） | 必要 — 附带产物 |
| `easytier-gui/src/auto-imports.d.ts` | **[自动生成]** unplugin-auto-import 更新 | 附带 — 新增 IPC 函数自动导入声明 |
| `easytier-gui/src/typed-router.d.ts` | **[自动生成]** 新增 `/home` 路由声明 | 附带 — 新增页面自动路由注册 |
| `.gitignore` | 新增 Tauri Android 构建产物、`.claude/`、`.mimocode/` 忽略规则 | 必要 — 忽略本地构建/工具产物 |

### CI/CD（Fork 适配）

| 文件 | 变更 | 必要性 |
|------|------|--------|
| `.github/workflows/release-tag.yml` | **[新增]** 全平台构建 + GitHub Release 自动发布（455行） | 必要 — CI 全平台构建 |
| `.github/actions/prepare-pnpm/action.yml` | 固定使用 `pnpm -r install --frozen-lockfile` | Fork 适配 — CI 可复现安装 |
| `.github/workflows/docker.yml` | Docker 镜像标签通用化为 `${{ github.repository_owner }}` | Fork 适配 |

### 文档/资源

| 文件 | 变更 | 必要性 |
|------|------|--------|
| `CLAUDE.md` | **[新增]** Claude Code 项目指令 | 必要 — 开发规范 |
| `docs/project.md` | **[新增]** 结构化项目文档 | 必要 — 文档 |
| `docs/电脑性能检测功能.md` | **[新增]** PC 硬件性能检测功能全文档 | 必要 — 文档 |
| `docs/CI全平台构建.md` | **[新增]** CI 构建文档 | 必要 — 文档 |
| `docs/本地Android构建.md` | **[新增]** 本地 Android 构建文档 | 必要 — 文档 |
| `README.md` | 重写为中文 README + 功能清单表 + 截图 | 必要 — 文档 |
| `assets/image-*.png` | **[新增]** 4 张界面截图 | 必要 — 文档配图 |

### 已同步至上游（无差异）

以下文件此前与上游存在差异，已在 `16b7d7e` 合并 `upstream/main` 及 v2.7.1 合并后归零：

| 文件 | 说明 |
|------|------|
| `easytier/Cargo.toml` | kcp-sys rev 跟随上游回退至 `d7427c22`（上游在 `bfa3383` 主动 revert） |
| `.github/workflows/Dockerfile` | HEALTHCHECK 跟随上游恢复 |
| `easytier-gui/.../BuildTask.kt` | 文件末尾空行移除，与上游一致 |
| `easytier-contrib/easytier-ohrs/Cargo.lock` | kcp-sys rev 跟随上游同步 |
| `.github/workflows/test.yml` | fork 曾将 check/pre-test 的 `pnpm` 安装置 false；上游 v2.7.1 重构为 `full_build` 动态开关后效果等价，已完全跟随上游 |

## 功能清单（相对上游新增/优化）

> 以下梳理自 `a72a593`（首个 WOL 功能提交）至 `HEAD` 的所有变更，覆盖功能增强、UI 优化、CI 建设、平台适配等维度。

<table>
<tr><th>模块</th><th>功能项</th><th>平台支持情况</th></tr>
<tr><td rowspan="12"><b>整体UI</b></td><td>现有玻璃标题栏/Tab 栏 + n3 Material You 内容风格 — 标题栏和底部 Tab 保留玻璃模糊材质；底部导航采用 n3 的图标胶囊、文字排布和 active 状态；总览卡使用 n3 28px 圆角、三段统计卡和粉/绿/蓝渐变（深色下渐变整体压暗一档、更接近纯黑）；按钮与标签统一使用 n3 tonal container 色；深色模式下所有列表项（电脑页设备卡、设置页列表项、组网流量图卡、网络设备/服务器卡片）统一使用组网页设备列表项背景 #1c1f24，电脑列表展开区子卡片保持纯黑；浅色模式下组网页与设置页整页底色沿用电脑页背景（--pc-bg，#f9f9ff）；双主题系统（浅色/深色，深色统一使用纯黑背景）</td><td>全平台</td></tr>
<tr><td>底部 Tab 导航栏（电脑 / 路由器 / 组网 / 设置），默认首页可配置（localStorage 持久化）</td><td>全平台</td></tr>
<tr><td>App 图标更换为圆润 Lucide layers 风格网络层叠图标，PNG 按 Android 官方 108dp 画布尺寸降采样（14.7MB → 1.2MB）；Splash 页面已移除，路由 redirect 直接跳转 `/home`</td><td>全平台</td></tr>
<tr><td>全部操作图标统一更换 — 标题栏编辑/刷新/切换、底部 Tab、唤醒铃铛面型图标、关机电源面型图标、Connect 链式连接、断开断链 SVG</td><td>全平台</td></tr>
<tr><td>卡片入场顺次动画 — 总览卡、统计卡、电脑列表、组网图表、网络设备/服务器和设置项统一使用电脑卡片的 `14px` 上移 + 淡入；由 `page-active` 控制，每次切换到对应 Tab 时重新播放，组内错开 60ms</td><td>全平台</td></tr>
<tr><td>图表 SVG — 无左侧 Y 轴预留空间，最大值标签叠于图表左上角，y=0 由横轴线隐式表达，横坐标左右贴边（`left:2px` / `right:2px`）</td><td>全平台</td></tr>
<tr><td>自定义 Material snackbar 底部居中胶囊 Toast（替代 PrimeVue useToast）— 2.5s 滑入动画</td><td>全平台</td></tr>
<tr><td>中英双语 — 自定义 `tt(key)` 函数 + 内联 i18n 字典（不依赖 easytier-frontend-lib），用户面文本全部双语化</td><td>全平台</td></tr>
<tr><td>页面内容区域隐藏滚动条 — MD 风格：`.overflow-y-auto` 通过 `scrollbar-width:none` / `::-webkit-scrollbar { display:none }` 隐藏滚动条，保留滚动功能</td><td>全平台</td></tr>
<tr><td>Tonal 按钮统一风格 — 唤醒/关机/保存等操作按钮使用 `color-mix(in srgb, accent 15%, panel-strong)` 背景，无内边框，active 态 `scale(0.96)`；变体：white/green/red/primary/export；参考设计来源 patch-3</td><td>全平台</td></tr>
<tr><td>连接按钮水波涟漪动画 — 双层 `::before`/`::after` 伪元素扩散淡出（`scale(1→1.25)` + `opacity(0.45→0)`），accent 色半透明背景；点击弹跳反馈（`connect-bounce` scale 1→0.88→1）；状态过渡动画（`background`/`color`/`box-shadow` 0.4s 平滑切换）</td><td>全平台</td></tr>
<tr><td>图表颜色 CSS 变量化 — `--chart-cpu`/`--chart-mem`/`--chart-gpu`/`--chart-disk-r`/`--chart-disk-w`/`--chart-net-up`/`--chart-net-down`，模板 SVG 统一引用变量</td><td>全平台</td></tr>
<tr><td rowspan="10"><b>电脑<br>(WOL)</b></td><td>设备卡片 — 粉红渐变总览卡片（电脑在线数/路由器/通道三统计）+ 设备卡片（在线状态指示灯脉冲动画 + 名称 + IP + 状态文字），点击展开详情查看 PC 性能监控图表</td><td>全平台</td></tr>
<tr><td>远程唤醒 — App → HTTP GET 路由器 CGI（<code>luci-app-wolplus</code>）→ <code>etherwake</code> 发送魔术包；局域网直连 / EasyTier SOCKS5 隧道双路径</td><td>全平台</td></tr>
<tr><td>在线状态检测 — App → HTTP GET PC Agent（Go，32249 端口，<code>/api/v1/status</code>）；30 秒自动轮询，按每台设备自己的 <code>router_ip</code> 独立检查路由器；首次探测显示“检测中”，连续 2 次失败才确认离线，单次超时不再造成在线列表闪烁</td><td>全平台</td></tr>
<tr><td>远程关机 — App → HTTP POST PC Agent（<code>/api/v1/shutdown</code>），Agent 执行 <code>shutdown /s /t 5</code>；含关机失败回退 online 机制（12 次 / 60s 检测）</td><td>全平台</td></tr>
<tr><td>唤醒/关机状态机 — idle → waking（橙色脉冲 5s）→ online / offline；online → shutting（红色脉冲）→ offline / 回退 online</td><td>全平台</td></tr>
<tr><td>设备配置 TOML 文本编辑器（等宽字体 Dialog）— 支持从剪贴板导入；路由器离线时卡片半透明（opacity 0.55）、操作按钮隐藏</td><td>全平台</td></tr>
<tr><td>展开底部收起按钮 — 设备展开后底部显示向上箭头 + "收起/Collapse" 文字，点击即可折叠，无需滚动回父卡片</td><td>全平台</td></tr>
<tr><td>设备信息卡片仅调试模式显示 — 底部 MAC · 路由器 IP · 接口 · 端口摘要行仅在调试模式开启时展示，非调试模式不显示</td><td>全平台</td></tr>
<tr><td>PC 性能监控 — 实时采集 CPU/内存/GPU 使用率、磁盘 IO 读写速度、网络上下行流量，SVG 面积渐变折线图展示（60 点 / 5 分钟窗口）；硬件信息（CPU 型号、内存 DDR 类型与频率、GPU 型号）；5 秒自动刷新；颜色标识健康状态</td><td>全平台</td></tr>
<tr><td>路径标签（ET-LAN / LAN 蓝色胶囊）— 根据 <code>netRunning</code> 与 <code>router_ip</code> 自动判断路径模式</td><td>全平台</td></tr>
<tr><td rowspan="9"><b>组网</b></td><td>总览信息卡片 — n3 绿色渐变 + 虚拟IP副标题 + NAT标签 + 三统计（节点/服务器/累计流量）；接口IPv4/IPv6 地址仅在调试模式显示</td><td>全平台</td></tr>
<tr><td>Peer / 服务器卡片展示 — n3 ListItem：头像 + 名称 + 延迟，IP/流量与 P2P、协议、NAT 标签分两行；无 hover；点击整卡按电脑页 grid 动画展开，展开区只显示版本号</td><td>全平台</td></tr>
<tr><td>网络质量颜色分级 — 绿（正常）/ 蓝（loss>3% 或 lat>50）/ 橙（loss>5% 或 lat>100）/ 红（loss>10% 或 lat>300）；NAT 标签：绿/绿/绿/蓝/橙对应 Open/NAT1/NAT2/NAT3/NAT4</td><td>全平台</td></tr>
<tr><td>Peer 排序 — 路由器优先（hostname 匹配 <code>/route|wrt|路由|home|家/i</code>），其余按 IP 排序；服务器按 hostname 字母排序</td><td>全平台</td></tr>
<tr><td>实时速率图表 — 复用电脑页 `ComputerChart.vue`，仅绘制折线不绘制面积；上下行颜色与设备列表流量一致，2 分钟窗口（40 点 / 3 秒采样），不采用 n3 图表样式；连接重建时速率防负值 + 清空旧数据</td><td>全平台</td></tr>
<tr><td>多网络管理 — 切换/新增/删除网络，当前网络标记 ✓，底部 "Add Network" 入口；网络配置 TOML 编辑器（含剪贴板导入）</td><td>全平台</td></tr>
<tr><td>连接状态条 — 本机 IP + 设备名 + 版本号 + 红色断开按钮；连接中显示 "Connecting..."；netDiscovering 12 秒内空列表显示 "Waiting for peers..."</td><td>全平台</td></tr>
<tr><td>3 秒自动刷新 Peer 数据；连接时自动检测全部 WOL 设备状态</td><td>全平台</td></tr>
<tr><td>多网络管理 Bug 修复 — 新增配置仅保存不自动连接（用户手动 Connect）；删除网络后编辑器清空旧内容；删除/导入确认按钮文案中文化（<code>tt('confirm')</code>）</td><td>全平台</td></tr>
<tr><td rowspan="5"><b>LuCI</b></td><td>Hyper 本地 HTTP 反向代理 — 自动向路由器发起 LuCI 登录（POST <code>/cgi-bin/luci/</code>），提取 <code>sysauth</code> / <code>sysauth_http</code> cookie，后续请求自动注入</td><td>全平台</td></tr>
<tr><td>iframe 嵌入路由器管理页面 — 全屏 100%×100%；SOCKS5 代理支持 EasyTier 隧道访问远程路由器</td><td>全平台</td></tr>
<tr><td>会话过期自动重登（检测 302 重定向到 login），刷新保持当前 URL（<code>last_path</code> 仅追踪 <code>text/html</code> 响应，过滤 CSS/JS/API 资源请求）</td><td>全平台</td></tr>
<tr><td>多路由器管理 — TOML 配置路由器列表，支持切换/新增/删除；删除当前路由器自动切换至剩余第一个</td><td>全平台</td></tr>
<tr><td>路径持久化 — 刷新或路径模式切换（ET-LAN ↔ LAN）后，代理重启自动恢复上次浏览位置；v-show 切回 Tab 不重载 iframe</td><td>全平台</td></tr>
<tr><td rowspan="11"><b>设置</b></td><td>设置菜单图标彩色方案 — SETTINGS_PALETTE 9 色调色板，通过 settingsIconStyle(key) 函数按可见项索引动态分配，隐藏项（高级设置）不占用颜色槽位，浅色/深色通用</td><td>全平台</td></tr>
<tr><td>高级设置入口（默认隐藏，开启调试模式后显示）— 跳转原 <code>index.vue</code>（<code>/</code> 路由）访问 easytier-frontend-lib 自带高级面板</td><td>全平台</td></tr>
<tr><td>语言切换 — 地球图标 + 当前语言蓝色胶囊标签（中文 / English），点击 <code>toggleLang()</code> 即时切换所有 UI 文字</td><td>全平台</td></tr>
<tr><td>默认首页 — 原生 <code>&lt;select&gt;</code> 下拉（电脑 / 路由器 / 组网），<code>localStorage['defaultTab']</code> 持久化</td><td>全平台</td></tr>
<tr><td>主题切换（Theme）— 浅色/深色两种模式切换，右侧显示当前模式蓝色胶囊标签，支持自动跟随系统主题（<code>prefers-color-scheme</code>），<code>localStorage['themeMode']</code> 持久化，<code>applyTheme()</code> 统一状态机管理；深色模式统一使用纯黑背景</td><td>全平台</td></tr>
<tr><td>调试模式 — Material You 风格 toggle 开关（关闭态小圆点 + 2px 轨道描边，开启态圆点放大并以 accent 填充），开启后在组网页底部显示诊断信息 + 解锁高级设置菜单项；副标题"显示高级选项和调试信息"</td><td>全平台</td></tr>
<tr><td>一键配置导入/导出 — WOL + LuCI + 网络配置打包为 JSON（<code>v:1</code> 结构），经 deflate 压缩 + Base64URL 编码为 <code>ETGC:2:...</code> 格式写入剪贴板（压缩后约为原始 JSON 的 30-40%）；Import 校验前缀 + 解压 + 解码 + 校验 + 二次确认弹窗防误操作覆盖</td><td>全平台</td></tr>
<tr><td>深色主题（Dark）— 深色模式统一使用纯黑背景色（<code>#000</code>）并适配 Dialog/卡片/代码块/标题栏/Tab 栏等全部组件</td><td>全平台</td></tr>
<tr><td>帮助对话框 — 展示 WOL、PC 性能监控、EasyTier VPN、LuCI 路由器、配置导入导出五大模块的功能说明和示例配置（含配置代码复制按钮）；WOL 部分提供 <code>luci-app-wolplus</code> 下载链接（通过 <code>tauri-plugin-opener</code> 打开浏览器）</td><td>全平台</td></tr>
<tr><td>关于 — 显示版本号（来自 <code>package.json</code> 的 <code>pkg.version</code>），项目简介、致谢与开源许可信息</td><td>全平台</td></tr>
<tr><td>设置页布局对齐 — 设置卡片 <code>.md-settings-card</code> 水平外边距归零（<code>margin:8px 0</code>），容器添加 <code>px-3</code> 与电脑/路由器页边距一致；确认提示框按钮右对齐（<code>justify-content:flex-end</code>）</td><td>全平台</td></tr>
<tr><td rowspan="6"><b>Android<br>平台适配</b></td><td>跨平台剪贴板 — 从 <code>PasteBridge</code>（<code>@JavascriptInterface</code> + <code>CountDownLatch</code>，仅 Android）迁移至 <code>tauri-plugin-clipboard-manager</code>（<code>readText</code>/<code>writeText</code>），全平台可用</td><td>全平台</td></tr>
<tr><td>动态主题适配 — <code>MainActivity.kt</code> 新增 <code>ThemeBridge</code>（<code>@JavascriptInterface</code>），前端 <code>applyTheme()</code> 统一调用 <code>setStatusBarStyle(dark)</code> 动态切换 Android 状态栏/导航栏颜色（浅色 <code>#FCFCFC</code> / 深色 <code>#000000</code>）</td><td>Android</td></tr>
<tr><td>VPN 路由配置 — <code>mobile_vpn.ts</code> 设置 <code>disallowedApplications</code> 将 App 自身排除出 VPN 避免路由死循环；配置 <code>routes</code> 将虚拟网段路由至 TUN</td><td>Android</td></tr>
<tr><td><code>AndroidManifest.xml</code> 添加 <code>usesCleartextTraffic="true"</code> 允许 HTTP 明文请求（访问局域网路由器 CGI 和 PC Agent）</td><td>Android</td></tr>
<tr><td>文本选择上下文菜单 — 上游 <code>main.ts</code> 中 <code>contextmenu</code> 事件 <code>preventDefault()</code> 拦截了所有长按/右键菜单；修改为仅对非 INPUT/TEXTAREA 元素拦截，输入框保留系统菜单，桌面端普通页面右键仍被阻止</td><td>Android</td></tr>
<tr><td>APK 四架构构建 — aarch64 / armv7 / i686 / x86_64</td><td>Android</td></tr>
<tr><td rowspan="4"><b>CI / 构建</b></td><td>Tag 推送自动触发全平台构建 + GitHub Release Draft — <code>release-tag.yml</code> (455 行)，矩阵含 Core CLI ×16、GUI 桌面 ×7、Android APK ×4、Magisk 模块</td><td>全平台（CI 产物）</td></tr>
<tr><td>CI 兼容适配 — <code>prepare-pnpm</code> 使用 frozen lockfile、GUI/Mobile 仅构建 <code>easytier-gui...</code> 依赖闭包、Android cmdline-tools 使用稳定构建号、macOS 签名凭据缺失时回退未签名构建、<code>docker.yml</code> 仓库名通用化</td><td>Linux CI Runner</td></tr>
<tr><td>Rust 代码 CI 合规 — <code>lib.rs</code> 通过 <code>cargo fmt</code> 格式检查 + <code>cargo clippy -D warnings</code>，确保上游 OHOS / Test workflow 通过</td><td>Linux CI Runner</td></tr>
<tr><td>Cargo 环境隔离 — CC/AR/BINDGEN/PROTOC/LIBCLANG_PATH/GIT_CEILING_DIRECTORIES 由 <code>/mnt/d/tools/EasyTierLinux/env.sh</code> 注入，不写入 <code>.cargo/config.toml</code>，避免宿主机路径污染 CI</td><td>本地开发（Ubuntu / WSL）</td></tr>
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
  ├── Tab 1: 电脑 — WOL 设备管理 + 性能监控
  ├── Tab 2: 路由器 — LuCI 管理面板
  ├── Tab 3: 组网 — EasyTier 网络管理
  ├── Tab 4: 设置 — 主题/语言/默认首页/调试/帮助/关于
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
| `src/App.vue` | 修改 | 主题初始化 + 窗口标题 + Toast + RouterView（Splash 已移除） |
| `src/pages/home.vue` | **新增** | Tab 容器（电脑/路由器/组网/设置），核心页面（~2900行） |
| `src/pages/index.vue` | **不动** | 原页面，`/` 路由 |
| `src/composables/backend.ts` | 修改 | 新增 `httpGet`/`httpPost`/`tcpPing`/`luciProxy*` IPC 封装 |
| `src-tauri/src/lib.rs` | 修改 | 新增 `http_get`/`http_post`/`tcp_ping` Tauri 命令 + LuCI HTTP 反向代理 |
| `src-tauri/Cargo.toml` | 修改 | 新增 `reqwest`（含 socks 特性）/`hyper`/`hyper-util` 等依赖 |
| `src/composables/mobile_vpn.ts` | 修改 | VPN 路由配置，`disallowedApplications` |
| `src-tauri/gen/android/.../MainActivity.kt` | 修改 | 主题桥接（`ThemeBridge`），已移除旧 `PasteBridge` |
| `src-tauri/gen/android/.../BuildTask.kt` | 修改 | .so 文件复制替代符号链接 |
| `src-tauri/gen/android/.../AndroidManifest.xml` | 修改 | `usesCleartextTraffic="true"` |

## 产品设计

### 页面结构

四个 Tab，底部导航切换：

| Tab | 图标 | 功能 |
|-----|------|------|
| 电脑 | desktop | WOL 设备管理：唤醒、关机、状态监控、PC 性能监控 |
| 路由器 | router | 路由器管理面板（HTTP 反向代理 iframe） |
| 组网 | network | EasyTier 网络：连接、Peer 列表、速率图表 |
| 设置 | settings | 配置导入导出、语言切换、默认首页、Debug、帮助、关于 |

设置中可通过 "Advanced Settings" 跳转到原始页面（`/` 路由）。

### 电脑页面

**标题栏**：编辑按钮（白色 Tonal 按钮）。

**总览信息卡片**（Hero Card，左列标题+副标题，右列刷新按钮）：副标题 "远程唤醒与设备管理"；三个统计项：电脑（在线数/总数）、路由器（在线/总数）、通道（局域网/组网）。

**设备卡片**：
- 状态指示灯（脉冲动画）+ 名称 + IP + 状态文字
- 唤醒按钮（铃铛图标）、关机按钮（电源图标）
- 点击展开详情：MAC、Router IP、Interface、Agent Port
- 展开底部收起按钮 — 设备展开后底部显示向上箭头 + "收起/Collapse" 文字，点击即可折叠，无需滚动回父卡片
- 设备信息摘要行（MAC · 路由器 IP · 接口 · 端口）仅在调试模式开启时展示
- 子卡片展开方向 — `transform-origin: center bottom`，子卡片紧贴父卡片底部向下展开，视觉连贯
- 路由器离线时卡片半透明（`opacity: 0.55`），所有操作按钮隐藏
- 卡片入场顺次动画（每组内从 0ms 开始错开）

**PC 性能监控**：展开设备详情后显示 CPU/内存/GPU 使用率图表、磁盘 IO 读写速度、网络上下行流量，10 秒自动刷新。

**自动刷新**：30 秒轮询设备在线状态。


### 电脑页面升级计划（已实施）

> 本节记录已实施的电脑页面升级：n2 风格设备列表与性能图表、稳定设备 ID、多设备展开，以及 App 端自定义远程命令。保留现有 WOL、路由器可达性、LAN/EasyTier 访问路径和性能监控，不修改 `easytier-core`。

#### 实现范围

1. 电脑列表项、展开交互和性能图表按 `D:\Downloads\n2.html` 实现。
2. 在线状态继续由现有 `wolStatus`、`routerOnline` 和唤醒/关机阶段驱动，不采用 n2 的模拟在线逻辑。
3. 展开状态、状态数据、性能历史、命令和运行记录均使用稳定设备 ID。
4. 支持多张设备卡片同时展开。
5. 命令定义保存在 App 本地；`wol-agent` 不维护预置用户命令。
6. Agent 以 `program + args[]` 直接执行程序，不启用 Shell。

#### n2 设备列表

设备卡片结构：

```text
[头像] [名称]                         [状态] [电源] [更多] [展开]
       [IP · OS]
```

关键规格：

- 卡片圆角 `24px`，列表间距 `12px`
- 卡片展开前后颜色保持一致：浅色模式白底、深色模式 `#1c1f24`（与组网页列表项背景统一，取 `--pc-card-bg`/`--n3-surface`），无边框并复用电脑页顶部总览卡的 `--md-shadow`；列表卡片不响应 hover
- 所有卡片统一无边框；需要阴影的卡片必须使用电脑列表项的 `--md-shadow`。电脑列表项内部展开的性能卡、命令输出卡和状态卡不添加阴影
- 指标、状态和命令输出等子卡片使用独立背景：浅色 `#f5f6f9`，深色纯黑 `#000`
- CPU、GPU、内存、磁盘和网络图标底色与父卡片一致：浅色白色，深色 `#111214`
- 头像桌面端 `48px`、移动端 `40px`
- 电脑列表头像使用灰色 `secondary-container` 背景和深色图标；图标与标题列间距为 `14px`（移动端 `10px`）
- 头像左右保留额外留白（桌面端 `8px`、移动端 `4px`），避免与卡片边缘和名称区域过近
- 电源、更多、展开按钮为 `40px` 圆形按钮
- 状态标签由“圆点 + 文字”组成，颜色不单独承载含义
- 离线显示绿色开机按钮，在线显示红色关机按钮，阶段进行中或路由器不可达时禁用
- 点击头部非按钮区域、箭头均可展开/收起
- 支持 `⋮`、桌面端右键和移动端长按打开命令菜单
- 展开使用 `grid-template-rows: 0fr → 1fr`，面板淡入并上移
- 展开面板不使用树形连接线；信息标签、指标卡和命令输出区左右等距居中
- 展开底部“收起”按钮水平居中

响应式规则：

| 宽度 | 行为 |
|------|------|
| `> 720px` | 性能指标两列 |
| `<= 720px` | 性能指标单列 |
| `<= 600px` | 状态移至第二行，隐藏 OS，头像缩小，列表提示隐藏 |
| `<= 380px` | 图标按钮缩小至 `36px`，长文本和状态信息继续收缩 |

所有网格列均使用 `minmax(0, 1fr)`，图表宽度为 `100%`，避免窄屏溢出。

#### 在线状态逻辑

统一有效状态。每台设备使用自己的路由器状态，不再共享第一台设备的全局状态：

```text
首次且路由器尚未确认       → checking
设备未配置 router_ip       → routerOffline
router.state = unreachable → routerOffline
phase = waking             → waking
phase = shutting           → shutting
尚无首次 Agent 探测结果    → checking
online = true              → online
otherwise                  → offline
```

- `checking`：灰色脉冲
- `routerOffline`：卡片降透明度，操作禁用
- `waking` / `shutting`：黄色旋转状态
- `online`：绿色圆点和光晕
- `offline`：灰色圆点
- 命令只有在设备在线、路由器路径可用且命令凭据存在时可执行
- 路由器探测确认 WOLPlus CGI 可响应，而不仅是 TCP 端口可连接
- App 启动后网络实例状态由 LAN 校正为 EasyTier 时，会立即使旧路径结果失效并重新探测，不再等待下一个 30 秒周期
- 路由器和设备均允许 1 次瞬时失败；连续 2 次失败才切换为离线
- 设备列表按 IP/名称稳定排序，在线状态变化不会导致卡片上下跳动
- 旧结果按设备探测令牌丢弃，30 秒轮询与唤醒/关机阶段轮询不会互相覆盖

Agent `status` 增加 `system_uptime_secs`，App 用其显示“已运行”；旧 `uptime` 仍表示 Agent 进程运行时间，保持兼容。

#### 展开和状态模型

```ts
interface SortedWolDevice extends WolDevice {
  deviceId: string
  status: WolDeviceState
  error: string
}

const expandedDeviceIds = reactive(new Set<string>())
```

- `deviceId = mac || ip:agent_port`
- 状态和错误按设备 ID 保存，不再使用 IP 作为唯一键
- 性能轮询只请求已展开且在线的设备
- 收起设备时停止对应轮询
- 配置删除时同步清理展开、状态、图表和命令状态

#### 性能图表

新增 `ComputerChart.vue`，统一实现 CPU、GPU、内存、磁盘和网络图表：

- SVG `viewBox="0 0 100 100"`，`preserveAspectRatio="none"`
- 图表高度 `96px`
- 水平网格位于 `4%`、`50%`、`96%`
- 折线 `2px`，非缩放描边
- 单序列面积透明度 `0.14`
- 主序列 `#2a78d6`，次序列 `#eb6834`；深色模式使用 `#3987e5` / `#d95926`
- Y 轴显示最大值和中间值，X 轴左右贴边
- 鼠标、触摸显示十字准线、数据点和提示框
- 键盘支持 `←`、`→`、`Home`、`End`
- 无数据保留为空隙，不错误显示为零
- 图表按已有样本数动态铺满 X 轴，与组网页图表一致；只有一个样本时将该值复制到左右端点，形成一条表示当前值的水平线
- `null` 或请求失败区间断开折线，不用直线跨接中断时段
- 中英文提示、无数据文案和键盘读屏文本随界面语言切换

采集保持现有 5 秒频率，保存 60 点，因此实际窗口为 5 分钟；X 轴根据数据点显示“N 秒前 / N 分钟前”，而不是固定显示 60 秒。

指标卡采用 n2 布局：

- CPU、GPU、内存使用率图
- 指标子卡片和命令输出卡片不使用边框，以背景和阴影区分层级
- 每个磁盘独立卡片，包含容量、进度条和读写双线图
- 即使磁盘只有一个分区，也显示分区名称、容量和占用进度
- 每个物理网卡独立卡片，包含上下行双线图
- Agent 未提供频率、温度等数据时不虚构显示
- CPU/GPU 卡片没有“已用/可用”等副信息行时，图表仍与标题行保持固定间距
- 总览卡三个统计值统一使用组网页设备列表项名称的 `15px / 22px / 500` 字号、行高与字重
- 三个统计卡整体与 Hero 标题区间距为 `12px`，卡内上下 padding 为 `10px / 8px`，统计卡不添加阴影；深色模式统计卡背景保持 `#111214`（hero 渐变压暗后仍保持层次）

#### App 端自定义命令

命令保存在 App：

```ts
interface AppCommand {
  id: string
  name: string
  program: string
  args: string[]
  timeoutSecs: number
  cancelable: boolean
  risk: 'normal' | 'high'
}
```

存储键：

```text
computerCustomCommandsV1   // 命令，按 deviceId 分组
wolAgentCommandTokensV1    // 每台设备的命令 Token
```

Token 不写入可导出的 WOL TOML，也不进入配置导入导出内容。

命令入口：

- 设备卡片 `⋮`
- 桌面端右键卡片
- 移动端长按卡片

展开区域不再显示“性能 / 命令”分段切换。命令从菜单和确认 Dialog 发起，执行后进入命令输出视图，并提供“返回性能”按钮。菜单和确认 Dialog 采用 n2 视觉，但显示的是 App 自定义命令。执行时只提交结构化 `program + args[]`，不提交 Shell 文本。

#### wol-agent 命令 API

```http
GET    /api/v1/command-policy
POST   /api/v1/command-runs
GET    /api/v1/command-runs?active=true
GET    /api/v1/command-runs/{run_id}
GET    /api/v1/command-runs/{run_id}/output?after_seq=0
POST   /api/v1/command-runs/{run_id}/cancel
```

提交示例：

```json
{
  "request_id": "3c259c3c-5cb1-4d32-a48d-72e6f221a4f4",
  "command": {
    "name": "刷新 DNS 缓存",
    "program": "C:\\Windows\\System32\\ipconfig.exe",
    "args": ["/flushdns"],
    "timeout_secs": 30,
    "cancelable": true,
    "risk": "normal"
  }
}
```

运行状态：

| 状态 | 含义 |
|------|------|
| `queued` | 已接收，等待执行 |
| `running` | 正在执行 |
| `succeeded` | 正常结束且退出码为 0 |
| `failed` | 启动失败或退出码非 0 |
| `timed_out` | 超过命令超时 |
| `canceled` | 用户主动取消 |
| `interrupted` | Agent 重启导致旧运行无法恢复 |
| `unknown` | 无法可靠确认 |

输出使用全局递增序列号：

```json
{
  "next_seq": 3,
  "eof": true,
  "truncated": false,
  "chunks": [
    { "seq": 1, "stream": "stdout", "text": "..." },
    { "seq": 2, "stream": "stderr", "text": "..." }
  ]
}
```

App 通过 `after_seq` 增量读取；活动运行期间每 1–2 秒轮询，断开后继续使用游标。

#### Agent 安全策略

命令接口必须配置 Bearer Token 和允许目录：

```bash
wol-agent.exe ^
  --port 32249 ^
  --command-token "<device-token>" ^
  --command-allowed-dir "C:\\Tools"
```

安全规则：

- Token 为空或允许目录为空时命令 API 禁用
- 程序必须是允许目录内的绝对路径
- 默认禁止 `cmd.exe`、PowerShell、脚本解释器和常用系统绕过程序
- Windows 只允许 `.exe` / `.com`
- 不允许 App 提供 Shell、环境变量或工作目录
- `request_id` 提供幂等保护
- 默认每台 Agent 同时只允许一个活动运行
- 限制超时、请求频率和单次运行输出量
- Windows 使用 Job Object、Unix 使用进程组终止完整进程树
- 状态和输出保存在内存；第一版不承诺跨 Agent 重启恢复结果

现有 `status`、`stats` 和 `shutdown` 的认证兼容策略保持独立；新增命令接口不复用无认证接口。

#### Tauri HTTP Bridge

新增通用请求命令：

```ts
httpRequest({
  method,
  url,
  proxy,
  timeoutSecs,
  headers,
  body
})
```

返回：

```ts
interface HttpRequestResult {
  status: number
  headers: Record<string, string>
  body: string
}
```

支持 GET/POST、JSON body、Authorization 请求头、非 2xx 响应、SOCKS5 代理和独立超时。

#### 文件改造点

| 文件 | 改动 |
|------|------|
| `easytier-gui/src/components/ComputerChart.vue` | 新增 n2 响应式图表组件 |
| `easytier-gui/src/composables/wol_status.ts` | 纯状态转换、首次探测、失败宽限和多路由器语义 |
| `easytier-gui/src/composables/wol_status.test.ts` | 在线状态转换单测 |
| `easytier-gui/src/composables/computer_commands.ts` | App 命令存储、Token 和 Agent API 封装 |
| `easytier-gui/src/composables/backend.ts` | 通用 `httpRequest` |
| `easytier-gui/src-tauri/src/lib.rs` | 通用 HTTP Bridge |
| `easytier-gui/src/pages/home.vue` | n2 列表、状态重构、多展开、图表和命令 UI |
| `wol-agent/commands.go` | 命令策略、异步运行、输出游标和认证 |
| `wol-agent/process_windows.go` | Windows Job Object 进程树取消 |
| `wol-agent/process_unix.go` | Unix 进程组取消 |
| `wol-agent/main.go` | 注册命令路由和系统 uptime |

#### 验收标准

- n2 列表在桌面、窄屏和移动宽度下无水平溢出
- 多设备同时展开时状态、图表和命令互不串扰
- 排序或配置变化不会导致展开状态跳到其他设备
- 在线状态继续遵循路由器和唤醒/关机业务逻辑
- 首次启动和 LAN → EasyTier 路径切换不会误报路由器离线
- 单次探测超时保持最后确认状态，连续 2 次失败才显示离线
- 图表固定 60 槽，断点正确分段，中英文提示一致
- App 只能执行本地保存的 `program + args[]`
- Agent 无预置用户命令，未配置 Token 或允许目录时 API 禁用
- 网络重试不会因相同 `request_id` 重复执行
- stdout/stderr 按全局序列增量读取
- 超时、取消、失败和中断具有明确终态
- 取消后终止完整进程树
- Vue 模板编译、Vite 构建、Go vet 和 Android 构建通过

### 组网页面

**标题栏**：未连接时显示网络名称 + 磨砂玻璃编辑按钮 + 切换按钮；已连接时标题切换为"组网" + 红色断开按钮。

**未连接态**：中央大 Connect 按钮（accent 蓝色背景；深色模式改用设置页关于按钮同款 tonal 底色 `--n3-primary-container`）。无网络配置时显示空状态提示（图标 + "未配置网络" + "点击 ⚡ 切换并添加网络"），不显示 Connect 按钮。

**连接中**：按钮背景色减淡过渡（0.4s），文字保持白色（深色模式文字/图标取 `--n3-on-primary-container`），涟漪同步换成 tonal 色，spinner 替代链接图标。

**已连接态**：
- 总览信息卡片：n3 绿色渐变 + 网络名称 + 虚拟IP副标题 + NAT标签 + 已连接状态 + 三统计项（节点数/服务器数/累计流量）
- 速率图表：复用电脑页 `ComputerChart.vue`，40 点 / 2 分钟，支持单样本水平线和历史回溯
- 网络设备列表：n3 ListItem + 无 hover + 点击展开版本号
- 服务器列表：同设备卡片布局，按主机名字母排序
- 连接状态下隐藏编辑和切换网络按钮
- 断开按钮在标题栏以红色断链图标展示

**速率图表**：复用电脑页 `ComputerChart.vue` 的网格、十字准线、提示框和动态 X 轴，不采用 n3 的图表样式。图表上方左侧显示累计流量、右侧显示实时速度；上下行颜色与设备列表流量一致，只画折线不画面积。

**网络设备列表**：n3 ListItem，显示头像、名称、延迟、IP、流量和 P2P/协议/NAT 标签；点击整卡展开版本号，无 hover。

**服务器列表**：同设备卡片布局，隐藏 IP 行，按主机名字母排序；底部流量与 P2P/协议/NAT 标签水平居中对齐。

**Peer 排序**：路由器优先（hostname 匹配 `/route|wrt|路由|home|家/i`），其余按 IP 排序。

**切换网络**：弹出菜单列出所有已配置网络（当前网络标记 ✓），底部 "Add Network" 入口。

### 设置页面

提供 8 个菜单项（高级设置仅调试模式下显示），每项含彩色圆角方块图标（46×46，`border-radius: 16px`）、标题和副标题。图标颜色由 SETTINGS_PALETTE 按可见项索引动态分配。菜单项顺序：Default Home → Theme → Language → Debug → [Advanced Settings] → Configuration Info → Help → About。

**设置页总览信息卡片**：

蓝色渐变背景，标题 "EasyTier" + 副标题 "异地组网和远程设备管理" + 右侧 About 按钮（版本号显示，紧凑 34px Tonal 样式）。下方三个统计指标：电脑数 / 路由器数 / 网络数。

**Default Tab（默认首页）**
九宫格图标，副标题显示当前默认首页名称。右侧原生 `<select>` 下拉（紧凑 34px 样式），可选电脑 / 路由器 / 组网。选中值存入 `localStorage['defaultTab']`，仅首次加载时影响 `activeTab` 初始值。

**Theme（主题）**
月亮图标，副标题显示当前主题模式 + "点击切换主题" 提示，右侧蓝色胶囊标签显示 `浅色` / `深色` / `跟随系统`。通过下拉选择自动 / 浅色 / 深色，写入 `localStorage['themeMode']`，由 `applyTheme()` 统一状态机管理（综合 `themeMode` + 系统 `prefers-color-scheme`），同步调用 Android `ThemeBridge` 动态切换状态栏/导航栏颜色；深色模式统一使用纯黑背景。

**Language（语言）**
地球图标，副标题显示当前语言和 "点击切换语言" 提示，右侧蓝色胶囊标签（紧凑样式：主标签 11px、副标签 10px）显示 `中文` 或 `English`。点击调用 `toggleLang()`，切换 `locale` ref 并写入 `localStorage['easytierLang']`，UI 所有文本即时更新。

**Debug Mode（调试模式）**
齿轮图标，副标题 "显示高级选项和调试信息"，右侧开关 toggle（Material You 两态样式）。开启后生效两处：1) 组网页底部显示 Debug 区域（本机 IP、端口、raw TOML 文本等诊断信息）；2) 解锁高级设置菜单项（显示在调试模式下方）；3) 电脑页设备卡片底部显示设备信息摘要行（MAC · 路由器 IP · 接口 · 端口）。

**Advanced Settings（高级设置）**
齿轮图标，副标题 "Mode, logging, config server, language"。**默认隐藏**，仅在开启调试模式后显示。点击跳转到原始页面 `/`（easytier-frontend-lib 自带的高级设置面板），排在调试模式开关下方。

**Configuration Info（配置信息）**
云下载图标，副标题 "全部数据导入与导出" / "Import/Export all data"，右侧两个紧凑操作按钮（34px 高）：
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
帮助圆环图标，副标题 "查看各模块功能说明与配置示例"。点击打开帮助 Dialog，展示五个模块的功能说明、示例配置代码（含一键复制按钮）：
- **WOL**：luci-app-wolplus + wolplus-agent 安装说明，提供下载链接（通过 `tauri-plugin-opener` 的 `openUrl()` 在浏览器打开）
- **PC 性能监控**：CPU/内存/GPU/磁盘IO/网络流量实时监控功能说明
- **EasyTier VPN**：P2P 虚拟组网原理和功能特性说明
- **LuCI 路由器**：OpenWrt 管理面板接入说明（LAN / VPN 双模式）
- **配置导入导出**：TOML 配置文件结构和导入导出操作说明

**About（关于）**
信息图标，副标题显示版本号（来自 `package.json` 的 `pkg.version`）。点击打开关于 Dialog，包含项目简介（提及 PC 性能监控）、项目地址链接、致谢（EasyTier + luci-app-wolplus）。

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

- 显示当前路由器 IP + 路径标签（组网 / 局域网）
- 刷新按钮（仅在已连接时显示）：完整重启代理，恢复上次浏览路径
- 切换路由器按钮（仅在有多台路由器时显示）
- 切换菜单第一项为“编辑路由器”，不在路由器项后显示删除 X

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
  2. runNetworkInstance(config, true) → 直接路径加载配置并运行（与旧版 RemoteManagement 一致）
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

Glassmorphism + Material You 混合风格，自定义 CSS 变量体系。

**毛玻璃系统**：标题栏和 Tab 栏使用 `backdrop-filter: blur(22px)` + `::before` 伪元素渐变叠加，形成半透明毛玻璃效果。面板背景色 `--panel` 和 `--panel-strong` 分别用于弱/强不透明度场景。

**Tonal 按钮**：按钮使用 n3 的 container 色和内容色，浅色/深色分别由 `--n3-*-container` 令牌控制。文字胶囊按钮高 `40px`、圆角 `20px`，active 态 `scale(.95)`；图标按钮为 `40px` 圆形，active 态 `scale(.94)`。

```css
:root {
  font-size: 15px;
  --bg: #f1f1f0;
  --ink: #0d1117;
  --muted: #6f716f;
  --panel: rgba(252, 252, 250, 0.2);
  --panel-strong: rgba(255, 255, 255, 0.82);
  --accent: #0a84ff;
  --accent-soft: rgba(10, 132, 255, 0.14);
  --green: #30d158;
  --chart-cpu: #1976d2;
  --chart-mem: #e57373;
  --chart-gpu: #7c4dff;
  --chart-disk-r: #43a047;
  --chart-disk-w: #ff9800;
  --chart-net-up: #43a047;
  --chart-net-down: #1e88e5;
}
```

- 卡片：`border-radius: 20px`（大卡片）/ `12px`（列表项），`box-shadow` 阴影 + `inset 0 0 0 1px` 内边框
- 标题栏：毛玻璃背景 + `backdrop-filter: saturate(260%) contrast(1.12) blur(22px)`
- CSS 变量须在 `:root` 而非 `.md-app`（PrimeVue Dialog 渲染在 app div 外）
- 双主题系统：浅色/深色两个主题，通过 `html[data-theme="light|dark"]` 属性驱动，`applyTheme()` 统一状态机管理（综合 `themeMode` + 系统 `prefers-color-scheme`），已移除全部 `@media (prefers-color-scheme: dark)` 避免 CSS 优先级冲突；`.app-dark` class 仅保留供 PrimeVue `darkModeSelector` 使用
- **`:deep()` 在 unscoped `<style>` 中无效**：Vue 3 SFC 编译器仅在 scoped 块中处理 `:deep()`，unscoped 块中直接透传至浏览器。浏览器将 `:deep()` 视为无效 CSS 伪类，静默丢弃整条规则。修复：unscoped 块移除所有 `:deep()`，改用原生后代/复合选择器
- **PrimeVue Dialog Teleport 导致 `.md-dialog .p-dialog` 无法匹配**：`class="md-dialog"` 与 `.p-dialog` 在 Teleport 渲染的同一根元素上，后代选择器（空格）匹配不到。改用复合选择器 `.md-dialog.p-dialog`（无空格）匹配同一元素上的两个类
- **状态栏/导航栏颜色与渐变层同步**：标题栏和 Tab 栏使用 `::before` 伪元素做毛玻璃渐变过渡，渐变起止色必须与 Android 系统栏颜色一致（见下表）。这些颜色**不是** `--bg`，因为毛玻璃叠加背景后视觉色值会偏移。四处必须同步修改：`MainActivity.kt`（动态切换）、`themes.xml` / `values-night/themes.xml`（启动初始值）、`home.vue` CSS `::before` 渐变色、`App.vue` 启动页背景。启动页背景必须与状态栏颜色一致，避免页面切换时颜色跳变。深色模式下 `--bg` 已是 `#000`，无需额外处理

| 主题 | Android 状态栏/导航栏 | 启动页背景 | 标题栏 `::before` 渐变起点 | Tab 栏 `::before` 渐变终点 | `--bg`（不同！） |
|------|----------------------|-----------|--------------------------|--------------------------|-----------------|
| 浅色 | `#FCFCFC` | `#FCFCFC` | `#FCFCFC` → transparent | transparent → `#FCFCFC` | `#F1F1F0` |
| 深色（纯黑） | `#000000` | `#000` | `var(--bg)` → transparent | transparent → `var(--bg)` | `#000` |

### 卡片入场动画

卡片使用电脑列表的 `@keyframes pc-card-in` 入场动画：从下方 `14px` 滑入并淡入。

**分组独立延迟**：每个卡片通过 `--card-i` 计数，组内顺次错开 60ms。电脑卡片沿用 `--pc-i`。

**Tab 切换重播**：四个页面在 `activeTab` 对应时添加 `page-active` 类，动画选择器只在类存在时生效。因此首次打开 App 和每次切换 Tab 都会重新播放；隐藏页面不会提前消耗动画。

**展开动画**：电脑卡片和组网设备/服务器均使用 `grid-template-rows: 0fr → 1fr`，内容淡入并上移；组网展开区仅显示版本号。

```css
.md-page.page-active .md-hero,
.md-page.page-active .md-hero-metric,
.md-page.page-active .md-settings-card,
.md-page.page-active .net-peer-card,
.md-page.page-active .pc-card {
  animation: pc-card-in .35s ease-out both;
  animation-delay: calc(var(--card-i, 0) * 60ms);
}
```

**底部 Tab 栏**：保留 `backdrop-filter` 玻璃材质、渐变过渡和现有图标；布局改用 n3 的 56×32 active pill、12px 标签和 primary container 状态。栏高收紧为 `76px`，上下 padding 分别为 `6px / 8px`。

### 颜色体系

| 场景 | 颜色 | 条件 |
|------|------|------|
| NAT 类型 | 绿 / 绿 / 绿 / 蓝 / 橙 | Open(NAT1) / NAT2 / NAT3 / NAT4 |
| 网络质量 | primary 文本 | n3 组网列表延迟样式，不再按质量变色 |
| 隧道类型 | 蓝(Local) / 绿(P2P) / 灰(Relay) | Local / P2P / Relay |
| 路径标签 | `primary-container` tonal 标签 | ET-LAN / LAN |
| 断开按钮 | `error-container` tonal 按钮 | 标准红 |
| 状态指示灯 | 绿 / 灰 / 橙（脉冲）/ 红（脉冲） | 在线 / 离线 / 唤醒中 / 关机中 |
| Tonal 按钮 | n3 `*-container` | 操作按钮（secondary/good/error/primary 变体） |
| 图表 CPU | `--chart-cpu` `#1976d2` | CPU 使用率 |
| 图表内存 | `--chart-mem` `#e57373` | 内存使用率 |
| 图表 GPU | `--chart-gpu` `#7c4dff` | GPU 使用率 |
| 图表磁盘读 | `--chart-disk-r` `#43a047` | 磁盘读取速度 |
| 图表磁盘写 | `--chart-disk-w` `#ff9800` | 磁盘写入速度 |
| 图表网络上行 | `--chart-net-up` `#43a047` | 网络发送流量 |
| 图表网络下行 | `--chart-net-down` `#1e88e5` | 网络接收流量 |

### 卡片布局

**总览信息卡片（Hero Card）左右列布局**：

电脑、组网和设置三个页面的总览信息卡片使用 n3 Hero 布局；路由器页无总览卡：

```
┌─────────────────────────────────────────┐
│ [左列 flex-1 min-w-0]    [右列 shrink-0] │
│  标题                           按钮/标签 │
│  副标题                                  │
├─────────────────────────────────────────┤
│           统计指标区域（metrics）          │
└─────────────────────────────────────────┘
```

- 左列：`flex-1 min-w-0`，标题和副标题垂直排列，标题过长时自动换行
- 右列：`flex-shrink:0`，优先保证宽度不被压缩，不换行（`white-space:nowrap`）
- 右列内容根据场景不同：电脑页为刷新按钮（绿色 Tonal，紧凑 34px）、组网页为 NAT 标签 + 已连接状态、设置页为 About 按钮（紧凑 34px）
- `.md-hero-head` 不使用 `flex-wrap:wrap`，确保右列始终在同行显示

**按钮与标签的样式区分**：

| 样式 | CSS 类 | 特征 | 用途 |
|------|--------|------|------|
| **Tonal 按钮** | `md-settings-btn` | n3 container 背景，无内边框，默认 `40px` 高、`20px` 圆角；统计卡片与设置列表行内为紧凑变体 `34px` 高、`17px` 圆角、图标 `16px` | 操作触发（导入/导出/刷新/关于） |
| **标签/胶囊** | `md-chip` / `md-rt` | n3 primary/good/tertiary/secondary container，`8px` 圆角，无内边框 | 信息展示（NAT/路径/状态/P2P/协议） |

区分要点：按钮和标签都使用 n3 tonal container，均无内边框；按钮用于触发操作并带按压缩放，标签只用于信息展示。

**网络设备/服务器卡片**（n3 ListItem）：

```
第一行: [头像] [名称 · IPv4]                  [延迟]
第二行:        [流量]              [P2P] [TCP] [NAT]
展开:          [版本]                         [v2.7.1]
```

- 左侧图标桌面/移动均为 `40px` 圆角头像；网络设备使用路由器图标，服务器使用服务器图标
- 名称优先显示 hostname，网络设备无名称时回退到 IPv4；服务器去掉 `PublicServer_` 前缀
- 网络设备 IPv4 紧随第一行名称，中间用 `·` 分隔（小圆点两侧间距 4px）；名称保持 `15px / 500`、IP 保持 `12px` 次要色，横向空间不足时仅缩略名称（ellipsis）；第二行只显示上下行流量，与右侧标签同行垂直居中，布局与服务器列表项一致
- 除 `40px` 图标外，组网列表项字号整体小于 n3 默认稿：名称 `15px`、延迟/行内 IP `12px`、流量与标签 `11px`，流量使用粗体
- 右侧：P2P/Relay、协议摘要、NAT 标签按 n3 chip 样式展示；协议过长时压缩，窄屏不覆盖左侧流量
- 版本号只在展开区显示，格式为 `v2.7.1`（小写 v 前缀）
- 协议摘要提取公共前缀：空间足够时 `TXT-TCP, TXT-UDP` → `TXT:TCP/UDP`；右侧空间不足时压缩为 `TXT:2` / `TXT:3`
- 卡片无 hover；点击整卡或聚焦后按 Enter/Space 展开
- 展开区只显示版本号，不显示协议全文
- 版本号使用与电脑页展开后“系统”信息一致的标签样式（圆角 8px、secondary-container 背景）
- 服务器名去掉公共前缀 `PublicServer_`
- 设置页列表项名称使用 `16px / 400`，取消粗体

**总览信息卡片**（组网页）：

```
标题行: [网络名称]                    [NAT标签] [已连接]
副标题: [虚拟IP v4]
接口IP: [调试模式下显示接口 IPv4 / IPv6]
统计行: [节点数] [服务器数] [累计流量]
```

- 标题和副标题在左侧列，标签在右侧，标题过长时自动换行
- 接口 IP 区域：每个地址独立一行，无前缀标签，无分割线，v4 在前 v6 在后，各自 ASCII 升序排列
- 第三个统计项为「流量」，显示本机累计上传+下载总字节数（非瞬时速率）

### 图标

| 功能 | 图标 | 类型 |
|------|------|------|
| 唤醒 | 铃铛 `M12 22c...` | Material 面型 |
| 关机 | 电源 `M13 3h-2v10h2V3z...` | Material 面型 |
| 切换网络 | 左右箭头交换 | 自定义 SVG |
| Connect | 链式连接 | 自定义 SVG |
| 断开 | 断链 | 自定义 SVG |

### Toast

自定义 Material snackbar（替代 PrimeVue `useToast`）：底部居中胶囊，滑入动画。

- 按 `info / progress / success / error` 四种状态显示不同颜色和指示器。
- 普通提示默认 2.5 秒；上传下载的开始、进度、完成和失败使用独立时长。
- 上传/下载提示显示文件名、传输方向、文件大小、进度百分比、耗时和错误摘要。
- 全部提示接入应用中英文语言配置，切换语言后即时使用对应文案。
- Android 原生下载结果通过 WebView 事件回传到应用 Toast，不再显示硬编码英文提示。


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

### 4. 文本选择上下文菜单（已修复）

**现象**：Dialog 中 Textarea 可选择文字，但 Android 原生复制/粘贴工具栏不弹出。

**根因**：上游 `easytier-frontend-lib` 的 `main.ts` 中 `document.addEventListener('contextmenu', event => event.preventDefault())` 拦截了所有 contextmenu 事件。Android 长按文本触发 contextmenu 事件，被 preventDefault 后系统菜单不弹出。LuCI iframe 正常是因为独立浏览上下文不受父文档事件监听器影响。

**修复**：保留 contextmenu 事件监听器，但添加条件判断——仅对非 INPUT/TEXTAREA 元素执行 `preventDefault()`。输入框/文本域保留系统菜单（复制/粘贴/全选），桌面端普通页面右键仍被阻止（保留原有行为）。

**状态**：已修复。

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


### 6. 路由器 WebView 文件上传/下载（已实施）

路由器 iframe 不依赖 LuCI 页面自行保存文件，而是由 App 的流式反向代理和平台文件能力共同完成传输。

**Rust LuCI 代理**

- 上传请求体使用 `Incoming::into_data_stream()` 流式转发，不再一次性读入内存。
- 下载响应使用 `bytes_stream()` 流式返回，不再通过 `resp.bytes()` 缓冲整个文件。
- 移除 3 秒总超时；改为连接超时、发送响应头超时和单块 60 秒空闲超时。
- 保留 `Content-Length`、`Content-Type`、`Content-Disposition`、`Content-Range` 等传输相关响应头。
- 移除会阻止 iframe 加载的 `X-Frame-Options`、CSP 和跨源隔离响应头，并将路由器绝对跳转地址重写回本地代理路径。
- 本地 LuCI Listener 只启动一次，并维护多个按 `router_id` 区分的路由器上下文；切换路由器只更新活动上下文和 iframe URL，不再执行 `start/stop`。
- 路由器身份使用稳定 `router_id`，每个上下文独立保存登录 Cookie、LAN/ET 客户端、`last_path` 和下载缓存。
- 刷新、切换网络路径或重连时复用同一个本地 Listener，避免 WebView origin 随机变化。
- 上传、下载按 500ms 或 1MiB 发出 `router-transfer` 进度事件，并在开始、完成、失败时发出终态事件。
- 非幂等上传失败后不自动重试；登录过期导致的上传需要用户重新提交。

**Android**

- `MainActivity` 注册 `DownloadListener`，不再依赖 Wry Android 缺失的下载回调。
- 下载响应先由 Rust 代理写入应用缓存，再通过内部 `/__easytier_download__` 回放接口交给 Android 保存；这样即使原下载由 POST 表单产生，也不会被错误地重新提交为 GET。
- `127.0.0.1` 本地 LuCI 代理下载由 App 进程流式写入 MediaStore Downloads，避免系统 DownloadManager 无法访问应用 loopback 服务。
- 外部 HTTP/HTTPS 下载仍交给系统 `DownloadManager`，保存到公共 Downloads 并显示系统下载通知。
- 文件上传由加固版 `WebChromeClient` 处理，支持单选、多选、MIME/扩展名过滤，并保证选择取消或异常时正确回调 WebView。
- 相机采集等特殊文件选择仍委托给 Wry 原客户端。

**桌面端**

- Windows WebView2、macOS WKWebView、Linux WebKitGTK 使用 Wry 自带下载能力，下载到系统默认 Downloads 目录。
- 大文件是否成功主要由流式 LuCI 代理保证；桌面端暂不额外弹出保存位置对话框。
- macOS 下载完成事件可能不返回最终路径，这是 Wry/WebKit API 限制。

**边界**

- HTTP/HTTPS 文件是第一支持范围。
- `blob:`、JavaScript 动态生成文件、断点续传和自定义保存目录暂不保证。
- Android 公共 Downloads 写入失败时回退到应用外部 Downloads 目录，并保留下载通知。


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

导出时 JSON 序列化后经 deflate 压缩 + Base64URL 编码，以 `ETGC:2:` 前缀标识（EasyTier-Gui Custom v2，写入剪贴板）：

```
ETGC:2:<deflate压缩后的Base64URL编码>
```

内部 JSON 结构：

```json
{
  "v": 1,
  "wol": "<devices.toml 完整内容>",
  "luci": "<luci.toml 完整内容>",
  "net": ["<网络1 TOML>", "<网络2 TOML>"]
}
```

导入时校验 `ETGC:2:` 前缀 → Base64URL 解码 → deflate 解压 → JSON 解析。校验规则：`v` 为数字 1，`wol`/`luci` 为字符串，`net` 为字符串数组。

## 开发环境

| 工具 | 路径/版本 |
|------|----------|
| Android SDK | `D:\tools\Android\Sdk` |
| NDK | `27.0.12077973` |
| JDK 17 | `D:\tools\Java\jdk-17.0.12` |
| Rust 桌面默认 | `stable-x86_64-pc-windows-msvc` 1.95 |
| Rust Android 构建 | `stable-x86_64-pc-windows-gnu` + `aarch64-linux-android` target（无 VS Build Tools 时使用） |
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

Windows 本地 arm64 APK 快速构建：

```powershell
cd D:\projects\projectsAlpha\EasyTier
.\script\build-android-windows.ps1
```

只验证工具链和 Gradle 路径：

```powershell
.\script\build-android-windows.ps1 -ValidateOnly
```

脚本会自动处理以下 Windows 特有问题：

- 从 `Cargo.lock` 解析 Tauri/插件版本，重建含 Windows 绝对路径的 `tauri.settings.gradle`
- 使用 MinGW GNU Rust toolchain，避免缺少 MSVC `link.exe`
- 使用 NDK 27 Windows Clang 交叉编译 arm64
- 同步 `dist` 与 `libapp_lib.so`，清理 Gradle 旧 Native 中间目录
- 验证 APK 签名、前端资源哈希和 Native 库哈希，输出 SHA-256

不要在 Windows 构建中复用 WSL 生成的 `tauri.settings.gradle`，也不要让 Windows 与 WSL 共用同一个 `node_modules`。构建原理、平台隔离和完整踩坑记录详见 [本地Android构建.md](本地Android构建.md)。
