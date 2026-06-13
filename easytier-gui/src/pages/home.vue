<script setup lang="ts">
import { generateNetworkConfig, sendConfigs, collectNetworkInfo, updateNetworkConfigState, parseNetworkConfig, httpGet, httpPost, tcpPing, deleteNetworkInstance, luciProxyStart, luciProxyStop, luciGetLastPath } from '~/composables/backend'
import { loadLastNetworkInstanceId, saveLastNetworkInstanceId } from '~/composables/config'
import { Utils, I18nUtils } from 'easytier-frontend-lib'
import pkg from '~/../package.json'
import Button from 'primevue/button'
import Dialog from 'primevue/dialog'
import Textarea from 'primevue/textarea'
import { openUrl } from '@tauri-apps/plugin-opener'
import { readText, writeText } from '@tauri-apps/plugin-clipboard-manager'
import { resolveTheme, applyTheme as setTheme } from '~/composables/theme'
import type { NetworkTypes } from 'easytier-frontend-lib'

const router = useRouter()
const { locale } = useI18n()

// --- snackbar ---
const snackMsg = ref('')
const snackShow = ref(false)
let snackTimer: ReturnType<typeof setTimeout> | null = null
function showSnack(msg: string) {
  snackMsg.value = msg
  snackShow.value = true
  if (snackTimer) clearTimeout(snackTimer)
  snackTimer = setTimeout(() => { snackShow.value = false }, 2500)
}

// --- i18n ---
const T: Record<string, [string, string]> = {
  wolDevices: ['PC Devices', '电脑设备'],
  refresh: ['Refresh', '刷新'],
  editConfig: ['Edit config', '编辑配置'],
  noWolDevices: ['No PC devices', '无电脑设备'],
  tapToAdd: ['Tap ✏️ to add devices via TOML config', '点击 ✏️ 通过 TOML 配置添加设备'],
  online: ['Online', '在线'],
  offline: ['Offline', '离线'],
  routerOffline: ['Router Offline', '路由器离线'],
  detecting: ['Detecting...', '检测中...'],
  routerLabel: ['Router IP', '路由器 IP'],
  noIpConfigured: ['No IP configured', '未配置 IP'],
  wake: ['Wake', '唤醒'],
  shutdown: ['Shutdown', '关机'],
  macLabel: ['MAC', 'MAC 地址'],
  interfaceLabel: ['Interface', '接口'],
  agentPort: ['Agent Port', 'Agent 端口'],
  connecting: ['Connecting...', '连接中...'],
  connected: ['Connected', '已连接'],
  waitingForPeers: ['Waiting for peers...', '等待节点...'],
  discoveringPeers: ['Discovering peers...', '发现节点中...'],
  switchNetwork: ['Switch network', '切换网络'],
  addNetwork: ['Add Network', '添加网络'],
  unnamed: ['Unnamed', '未命名'],
  noNetwork: ['No Network', '无网络'],
  noNetworks: ['No networks configured', '未配置网络'],
  tapToSwitch: ['Tap ⚡ to switch and add networks', '点击 ⚡ 切换并添加网络'],
  connect: ['Connect', '连接'],
  disconnect: ['Disconnect', '断开'],
  servers: ['Servers', '服务器'],
  server: ['Server', '服务器'],
  networkDevices: ['Network Devices', '网络设备'],
  loadingPeers: ['Loading peers...', '加载节点中...'],
  noPeersConnected: ['No peers connected', '无节点连接'],
  settings: ['Settings', '设置'],
  advancedSettings: ['Advanced Settings', '高级设置'],
  advancedSettingsDesc: ['EasyTier engine native config panel: network mode, log level, etc.', 'EasyTier 引擎原生配置面板，含网络模式、日志级别等'],
  language: ['Language', '语言'],
  languageDesc: ['Switch interface language, takes effect instantly', '切换界面显示语言，即时生效'],
  about: ['About', '关于'],
  version: ['Version', '版本'],
  network: ['Network', '组网'],
  wol: ['PC', '电脑'],
  configSaved: ['Config saved', '配置已保存'],
  signalSent: ['Signal sent', '信号已发送'],
  saved: ['Saved', '已保存'],
  cancel: ['Cancel', '取消'],
  save: ['Save', '保存'],
  saveAndRun: ['Save & Run', '保存并运行'],
  editDevicesToml: ['Edit devices.toml', '编辑 devices.toml'],
  editConfigDialog: ['Edit Config', '编辑配置'],
  disconnectedToast: ['Disconnected', '已断开'],
  english: ['English', '英文'],
  chinese: ['中文', '中文'],
  delete: ['Delete', '删除'],
  deleteNetworkConfirm: ['Delete this network?', '确定删除此网络？'],
  deleted: ['Deleted', '已删除'],
  confirm: ['Confirm', '确定'],
  debug: ['Debug Mode', '调试模式'],
  debugDesc: ['Show advanced settings entry and diagnostic information', '开启后显示高级设置入口和诊断信息'],
  clear: ['Clear', '清空'],
  paste: ['Paste', '粘贴'],
  importFromClipboard: ['From clipboard', '从剪切板导入'],
  defaultHome: ['Default Home', '默认首页'],
  defaultHomeDesc: ['Default tab shown on app launch', '启动 App 后默认显示的标签页'],
  wolTab: ['PC', '电脑'],
  netTab: ['Network', '组网'],
  luciTab: ['Router', '路由器'],
  routers: ['Routers', '路由器'],
  noRouters: ['No routers configured', '未配置路由器'],
  addRouter: ['Add Router', '添加路由器'],
  editRoutersToml: ['Edit routers.toml', '编辑 routers.toml'],
  connectingRouter: ['Connecting...', '连接中...'],
  oneClickConfig: ['Configuration Info', '配置信息'],
  oneClickConfigDesc: ['One-tap export/import PC devices, routers, and network configs', '一键导出/导入电脑、路由器和网络配置'],
  exportLabel: ['Export', '导出'],
  importLabel: ['Import', '导入'],
  exportedToClipboard: ['Config exported to clipboard', '配置信息已导出到剪切板'],
  importConfig: ['Import Config', '导入配置'],
  importConfirmMsg: ['This will overwrite ALL PC, router, and network configs. Continue?', '本次操作将会覆盖电脑、路由器和组网所有的配置，确定继续？'],
  importSuccess: ['Config imported', '配置已导入'],
  importFailed: ['Invalid config format', '配置格式无效'],
  help: ['Help', '帮助'],
  helpDesc: ['View usage guide for all modules and config examples', '查看各模块使用指南与配置示例'],
  helpWolTitle: ['WOL — PC Wake-on-LAN', '电脑 — 网络唤醒'],
  helpWolDesc: ['Remotely wake, monitor, and shut down PCs. Works via router CGI (luci-app-wolplus) for wake-up and PC agent (Go, port 32249) for status/shutdown. Supports both direct LAN and EasyTier tunnel paths.', '远程唤醒、监控和关闭电脑。通过路由器 CGI（luci-app-wolplus）发送魔术包唤醒，通过 PC 端 agent（Go，32249 端口）检测在线状态和执行关机。支持局域网直连和 EasyTier 隧道两种路径。'],
  helpWolDep: ['Dependencies: luci-app-wolplus (install on OpenWrt router) + wol-agent (run on each PC)', '依赖：luci-app-wolplus（安装到 OpenWrt 路由器）+ wol-agent（每台电脑上运行）'],
  helpWolLink: ['Download Dependencies', '下载依赖组件'],
  helpNetTitle: ['EasyTier — Virtual Networking', '组网 — EasyTier 虚拟组网'],
  helpNetDesc: ['Connect your phone, PC, and router into a virtual LAN — access them as if they\'re on the same local network, no public IP required.', '将手机、电脑、路由器组成虚拟局域网，无论身在何处都能像在同一网络下互相访问。无需公网 IP，无需复杂配置。'],
  helpNetFeat: ['Auto P2P direct connection — low latency, high speed\nAuto relay fallback when P2P is unavailable — always connected\nReal-time latency, packet loss, and bandwidth charts\nMulti-network profile management with one-tap switching\nColor-coded connection quality at a glance', '自动 P2P 直连，延迟低速度快\n无法直连时自动中转，确保稳定连接\n实时显示延迟、丢包率和网速图表\n支持多网络配置，一键切换\n颜色标识连接质量，一目了然'],
  helpLuciTitle: ['Router — LuCI Router Management', '路由器 — LuCI 路由器管理'],
  helpLuciDesc: ['Manage your OpenWrt router directly in the app — no browser needed. Automatic login with full LuCI admin panel access.', '在 App 内直接管理 OpenWrt 路由器，无需打开浏览器。自动完成登录，提供完整 LuCI 后台面板。'],
  helpLuciFeat: ['Auto login — configure once, no repeated password entry\nMulti-router support — add and switch between routers anytime\nOver EasyTier tunnel — manage home router remotely\nBrowsing position memory — restores last page after refresh', '自动登录，首次配置后无需每次输密码\n支持多台路由器，随时切换管理\n通过 EasyTier 隧道也能远程管理家里路由器\n刷新后自动恢复上次浏览位置'],
  helpBackupTitle: ['Import & Export', '配置导入导出'],
  helpBackupDesc: ['In Settings > Configuration Info, you can export all WOL devices, network profiles, and router configs to the clipboard with one tap (compressed ETGC:2:...), or import from clipboard to restore. Perfect for switching phones or sharing configs.', '在设置页的「配置信息」中，可将 WOL 设备、网络配置、路由器配置一键导出到剪贴板（压缩编码 ETGC:2:...），也可从剪贴板导入恢复。更换手机或分享给他人时无需重新配置。'],
  helpCfgTitle: ['Config Example', '配置示例'],
  helpWolParams: ['name — Device display name, customize freely\nmac — MAC address for sending WOL magic packets\nip — PC LAN IP for status detection and shutdown\ninterface — Router network interface for sending WOL (e.g. br-lan)\nrouter_ip — Router LAN IP for wake requests and path detection\nagent_port — wol-agent listening port (default 32249)', 'name — 设备显示名称，可自由设置\nmac — MAC 地址，用于发送 WOL 魔术包\nip — 电脑局域网 IP，用于状态检测和关机\ninterface — 路由器上发送 WOL 包的网口（如 br-lan）\nrouter_ip — 路由器 IP，唤醒请求目标和路径模式判断\nagent_port — wol-agent 监听端口（默认 32249）'],
  helpWolSocks5: ['All WOL operations (wake, status check, shutdown) go through the SOCKS5 proxy when connected via EasyTier. Make sure your network config includes: socks5_proxy = "socks5://0.0.0.0:32259". The port can be customized but must match on both sides.', '通过 EasyTier 组网时，所有 WOL 操作（唤醒、状态检测、关机）均经过 SOCKS5 代理。请确保组网配置中包含：socks5_proxy = "socks5://0.0.0.0:32259"。端口可自行指定，但两端需保持一致。'],
  helpLuciParams: ['name — Router display name, customize freely\nip — Router LAN IP\nusername — OpenWrt login username (usually root)\npassword — OpenWrt login password', 'name — 路由器显示名称，可自由设置\nip — 路由器局域网 IP\nusername — OpenWrt 登录用户名（通常为 root）\npassword — OpenWrt 登录密码'],
  helpLuciSocks5: ['To access the router LuCI panel via EasyTier tunnel, make sure your network config includes: socks5_proxy = "socks5://0.0.0.0:32259". The port can be customized but must match on both sides.', '要通过 EasyTier 隧道访问路由器 LuCI 面板，请在组网配置中添加：socks5_proxy = "socks5://0.0.0.0:32259"。端口可自行指定，但两端需保持一致。'],
  helpNetParams: ['instance_name — Unique name for this network instance\nlisteners — Listening addresses and ports for peer connections\nnetwork_name — Shared identifier for all nodes in the same virtual network\nnetwork_secret — Shared secret key for encrypting communication\npeer.uri — Remote node address to connect to\nproxy_network.cidr — LAN subnet to proxy into the virtual network\nflags.enable_exit_node — Allow this node to act as an exit gateway\nsocks5_proxy — SOCKS5 proxy for WOL / LuCI tunnel access; port must match the router configuration', 'instance_name — 网络实例名称，用于区分多个网络\nlisteners — 监听地址和端口，接收其他节点连接\nnetwork_name — 同一虚拟网络所有节点共享的网络标识\nnetwork_secret — 加密通信的共享密钥\npeer.uri — 要连接的远程节点地址\nproxy_network.cidr — 代理到虚拟网络的局域网子网\nflags.enable_exit_node — 允许本节点作为出口网关\nsocks5_proxy — SOCKS5 代理端口，用于 WOL / LuCI 隧道访问；端口需与路由器配置一致'],
  theme: ['Theme', '主题'],
  themeAuto: ['Auto', '自动'],
  themeLight: ['Light', '浅色'],
  themeDark: ['Dark', '深色'],
  themeDesc: ['Switch between light, dark, or follow system', '切换浅色、深色外观或跟随系统'],
  amoled: ['AMOLED Black', 'AMOLED 纯黑模式'],
  amoledDesc: ['Pure black background for AMOLED screens, only effective in dark mode, saves power and reduces eye strain', 'AMOLED 屏幕纯黑背景，仅在深色模式下生效，更省电更护眼'],
  aboutSub: ['Project introduction, acknowledgments, and open source license', '项目简介、致谢与开源许可信息'],
  aboutDesc: ['EasyTier + WOLPlus is a cross-platform virtual networking and remote device management app built on Tauri v2. It retains all EasyTier networking capabilities while deeply integrating WOL wake-on-LAN (via luci-app-wolplus), remote status monitoring and shutdown (via wol-agent), LuCI router reverse proxy management, one-click config import/export, with comprehensive Material Design theming. Supports Windows, macOS, Linux desktop and Android mobile.', 'EasyTier + WOLPlus 是基于 Tauri v2 的跨平台异地组网与远程设备管理一体化应用。在保留 EasyTier 全部组网能力的基础上，深度整合了 WOL 网络唤醒（通过 luci-app-wolplus）、远程状态监控与关机（通过 wol-agent）、LuCI 路由器反向代理管理、一键配置导入/导出等功能，并进行了全面的 Material Design 界面美化。支持 Windows、macOS、Linux 桌面端及 Android 移动端。'],
  aboutThanks: ['Acknowledgments', '致谢'],
  aboutEasyTierDesc: ['Decentralized virtual networking engine — high-performance Rust implementation, stable NAT traversal, multi-protocol support (TCP/UDP/WSS/WG)', '去中心化异地组网引擎 — 高性能 Rust 实现、稳定的 NAT 穿透能力、丰富的协议支持（TCP/UDP/WSS/WG）'],
  aboutWolplusDesc: ['OpenWrt WOL LuCI plugin — CGI API for sending etherwake magic packets to wake up PCs on LAN', 'OpenWrt WOL LuCI 插件，提供 CGI API 通过 etherwake 发送魔术包唤醒局域网电脑'],
  aboutLinks: ['Project Links', '项目地址'],
  aboutThisProject: ['EasyTier + WOLPlus (This Project)', 'EasyTier + WOLPlus（本项目）'],
  aboutWolplusDep: ['luci-app-wolplus (Dependency)', 'luci-app-wolplus（依赖）'],
  close: ['Close', '关闭'],
  copyCode: ['Copy', '复制'],
  copiedCode: ['Copied', '已复制'],
  etLan: ['ET-LAN', '异地组网'],
  lanDirect: ['LAN', '局域网'],
  cpuModel: ['CPU', '处理器'],
  cores: ['cores', '核心'],
  memoryUsage: ['Memory', '内存'],
  gpuUsage: ['GPU', '显卡'],
  noGPU: ['No GPU detected', '未检测到 GPU'],
  diskIO: ['Disk IO', '磁盘 IO'],
  disk: ['Disk', '磁盘'],
  networkIO: ['Network', '网络'],
  recv: ['Recv', '接收'],
  sent: ['Sent', '发送'],
  loadingStats: ['Loading stats...', '加载中...'],
}
function tt(key: string): string {
  const entry = T[key]
  if (!entry) return key
  return locale.value === 'cn' ? entry[1] : entry[0]
}

// --- Tabs ---
type Tab = 'wol' | 'net' | 'luci' | 'settings'
const defaultTab = ref<Tab>((localStorage.getItem('defaultTab') as Tab) || 'wol')
const activeTab = ref<Tab>(defaultTab.value)
const showDebug = ref(false)

function switchTab(tab: Tab) {
  activeTab.value = tab
  if (tab === 'wol') { checkAllWolStatus(); manageStatsPoll() }
  else {
    if (statsTimer) { clearInterval(statsTimer); statsTimer = null }
  }
  if (tab === 'net' && netInstId.value) refreshNet()
  if (tab === 'luci') {
    if (!proxyUrl.value && luciRouters.value.length > 0 && !luciLoading.value) {
      luciIdx.value = luciIdx.value >= 0 ? luciIdx.value : 0
      startLuciProxy(luciRouters.value[luciIdx.value])
    } else if (proxyUrl.value) {
      // If path type changed (ET-LAN ↔ LAN), fully restart proxy
      const curType = getSocks5Proxy() ? 'tunnel' : 'lan'
      if (lastLuciPathType.value && lastLuciPathType.value !== curType) {
        refreshLuciIframe()
      }
    }
  }
}

async function toggleLang() {
  const next = locale.value === 'cn' ? 'en' : 'cn'
  await I18nUtils.loadLanguageAsync(next)
  locale.value = next
  showSnack(next === 'cn' ? tt('chinese') : tt('english'))
}

function setDefaultTab(tab: Tab) {
  defaultTab.value = tab
  localStorage.setItem('defaultTab', tab)
}

// ==================== WOL Tab ====================
interface WolDevice { name: string; mac: string; ip: string; interface: string; router_ip: string; agent_port: number }
const wolDevices = ref<WolDevice[]>([])
const wolToml = ref(localStorage.getItem('wolDevicesToml') || '')
const showWolEditor = ref(false)
const wolStatus = reactive<Record<string, { online: boolean; phase: 'idle' | 'waking' | 'shutting' }>>({})
const wolErrors = reactive<Record<string, string>>({})
const routerOnline = ref<undefined | boolean>(undefined)
const wolFirstCheck = ref(true)
const wolDebug = ref('')
const phaseTimers: Record<string, ReturnType<typeof setTimeout>> = {}
const checkingWol = ref(false)
const expandedDeviceIdx = ref<number | null>(null)
const statsHistory = reactive<Record<string, StatsSnapshot[]>>({})
const statsRaw = reactive<Record<string, StatsRaw | null>>({})
const statsError = reactive<Record<string, string>>({})
const statsLoading = reactive<Record<string, boolean>>({})
let statsTimer: ReturnType<typeof setInterval> | null = null
let wolPeriod: Utils.PeriodicTask | null = null

// --- Stats types ---
interface StatsRaw {
  cpu_pct: number
  mem_pct: number
  gpu_pct: number | null
  gpu_mem_pct: number | null
  disks: { name: string; read: number; write: number }[]
  nets: { name: string; recv: number; sent: number }[]
  _ts: number
}

interface StatsSnapshot {
  ts: number
  cpu: number
  mem: number
  gpu: number | null
  gpuMem: number | null
  disks: { name: string; readBps: number; writeBps: number }[]
  nets: { name: string; recvBps: number; sentBps: number }[]
}

const STATS_MAX_POINTS = 60  // 5 min at 5s interval

function ensureStatsKeys(ip: string) {
  if (!statsHistory[ip]) statsHistory[ip] = []
  if (!(ip in statsRaw)) statsRaw[ip] = null
  if (!(ip in statsLoading)) statsLoading[ip] = false
  if (!(ip in statsError)) statsError[ip] = ''
}

function toggleDeviceExpand(idx: number) {
  if (expandedDeviceIdx.value === idx) {
    expandedDeviceIdx.value = null
  } else {
    expandedDeviceIdx.value = idx
    const w = sortedWol.value[idx]
    if (w?.ip) {
      ensureStatsKeys(w.ip)
      if (statsHistory[w.ip].length === 0) fetchStats(w.ip, w.agent_port || 32249)
    }
  }
  manageStatsPoll()
}

async function fetchStats(ip: string, port: number) {
  ensureStatsKeys(ip)
  if (statsLoading[ip]) return
  const proxy = getSocks5Proxy()
  statsLoading[ip] = true
  try {
    const text = await httpGet(`http://${ip}:${port}/api/v1/stats`, proxy, 3)
    const data = JSON.parse(text)
    statsInfo[ip] = data
    const prev = statsRaw[ip]
    const now = Date.now()

    // CPU / Memory / GPU (instant values)
    const cpuPct = data.cpu?.usage_percent ?? 0
    const memPct = data.memory?.usage_percent ?? 0
    const gpuPct = data.gpu?.usage_percent ?? null
    const gpuMemPct = data.gpu?.vram_total_gb ? (gpuPct ?? 0) : null

    // Per-disk IO delta
    const curDisks: { name: string; read: number; write: number }[] = []
    if (data.disks) {
      for (const d of data.disks) {
        curDisks.push({ name: d.name || '', read: d.read_bytes || 0, write: d.write_bytes || 0 })
      }
    }

    // Per-NIC delta
    const curNets: { name: string; recv: number; sent: number }[] = []
    if (data.network) {
      for (const n of data.network) {
        curNets.push({ name: n.name || '', recv: n.recv_bytes || 0, sent: n.sent_bytes || 0 })
      }
    }

    // Compute per-disk rates
    const diskSnaps: StatsSnapshot['disks'] = []
    for (const cd of curDisks) {
      let readBps = 0, writeBps = 0
      if (prev) {
        const pd = prev.disks.find(d => d.name === cd.name)
        if (pd) {
          const elapsed = Math.max(0.1, (now - prev._ts) / 1000)
          readBps = Math.max(0, (cd.read - pd.read) / elapsed)
          writeBps = Math.max(0, (cd.write - pd.write) / elapsed)
        }
      }
      diskSnaps.push({ name: cd.name, readBps, writeBps })
    }

    // Compute per-NIC rates
    const netSnaps: StatsSnapshot['nets'] = []
    for (const cn of curNets) {
      let recvBps = 0, sentBps = 0
      if (prev) {
        const pn = prev.nets.find(n => n.name === cn.name)
        if (pn) {
          const elapsed = Math.max(0.1, (now - prev._ts) / 1000)
          recvBps = Math.max(0, (cn.recv - pn.recv) / elapsed)
          sentBps = Math.max(0, (cn.sent - pn.sent) / elapsed)
        }
      }
      netSnaps.push({ name: cn.name, recvBps, sentBps })
    }

    statsRaw[ip] = {
      cpu_pct: cpuPct, mem_pct: memPct, gpu_pct: gpuPct, gpu_mem_pct: gpuMemPct,
      disks: curDisks, nets: curNets,
      _ts: now,
    }

    const snap: StatsSnapshot = {
      ts: now, cpu: cpuPct, mem: memPct, gpu: gpuPct, gpuMem: gpuMemPct,
      disks: diskSnaps, nets: netSnaps,
    }
    statsHistory[ip].push(snap)
    while (statsHistory[ip].length > STATS_MAX_POINTS) statsHistory[ip].shift()
    statsError[ip] = ''
  } catch (e: any) {
    statsError[ip] = cleanErr(e)
  }
  statsLoading[ip] = false
}

// Full stats response cache for template display (model names, etc.)
const statsInfo = reactive<Record<string, any>>({})

function manageStatsPoll() {
  if (statsTimer) { clearInterval(statsTimer); statsTimer = null }
  if (activeTab.value === 'wol' && expandedDeviceIdx.value !== null) {
    const ip = sortedWol.value[expandedDeviceIdx.value]?.ip
    if (ip) {
      statsTimer = setInterval(() => {
        if (activeTab.value === 'wol' && expandedDeviceIdx.value !== null) {
          const dev = sortedWol.value[expandedDeviceIdx.value!]
          if (dev?.ip) fetchStats(dev.ip, dev.agent_port || 32249)
        } else {
          if (statsTimer) { clearInterval(statsTimer); statsTimer = null }
        }
      }, 5000)
    }
  }
}

// Chart helpers — same style as buildSmoothPath / formatSpeedY / formatXLabel
function buildStatsPath(data: number[], maxVal: number, chartH: number, chartW: number): string {
  if (data.length < 2) return ''
  const stepX = chartW / (data.length - 1)
  const scaleY = (v: number) => chartH - (v / maxVal) * (chartH - 4) - 2
  let d = `M 0,${scaleY(data[0])}`
  for (let i = 0; i < data.length - 1; i++) {
    const x1 = i * stepX; const y1 = scaleY(data[i])
    const x2 = (i + 1) * stepX; const y2 = scaleY(data[i + 1])
    const cx1 = x1 + stepX * 0.4; const cx2 = x2 - stepX * 0.4
    d += ` C ${cx1},${y1} ${cx2},${y2} ${x2},${y2}`
  }
  return d
}

function buildStatsAreaPath(data: number[], maxVal: number, chartH: number, chartW: number): string {
  const line = buildStatsPath(data, maxVal, chartH, chartW)
  if (!line) return ''
  const baseY = chartH
  return line + ` L ${chartW},${baseY} L 0,${baseY} Z`
}

function statsXLabel(dataLen: number, idx: number): string {
  // idx=0 → oldest (left), idx=last → newest "now" (right)
  const total = dataLen - 1
  const sec = (total - idx) * 5
  if (sec === 0) return 'now'
  if (sec < 60) return '-' + sec + 's'
  const m = sec / 60
  const r = Math.round(m * 2) / 2  // round to nearest 0.5, same as traffic chart
  const t = r.toFixed(1)
  return '-' + (t.endsWith('0') ? t.slice(0, -2) : t) + 'm'
}

function formatSpeedYStats(v: number): string {
  if (v < 1024) return v.toFixed(0) + ' B/s'
  if (v < 1048576) return (v / 1024).toFixed(0) + ' KB/s'
  if (v < 1073741824) return (v / 1048576).toFixed(0) + ' MB/s'
  return (v / 1073741824).toFixed(1) + ' GB/s'
}
function formatSpeedBps(v: number): string {
  const bps = v * 8
  if (bps < 1000) return bps.toFixed(0) + ' bps'
  if (bps < 1000000) return (bps / 1000).toFixed(0) + ' Kbps'
  if (bps < 1000000000) return (bps / 1000000).toFixed(1) + ' Mbps'
  return (bps / 1000000000).toFixed(1) + ' Gbps'
}

// Template helpers — read from cached statsInfo
function cpuModel(ip: string): string {
  return statsInfo[ip]?.cpu?.model || 'CPU'
}
function cpuTitle(ip: string): string {
  const model = statsInfo[ip]?.cpu?.model
  const cores = statsInfo[ip]?.cpu?.cores
  if (!model) return 'CPU'
  let s = model
    .replace(/\s*CPU\s*/gi, ' ')
    .replace(/\(R\)|\(TM\)/gi, '')
    .replace(/Core\s+i(\d)/gi, 'I$1')
    .replace(/@\s+([\d.]+)\s*GHz/gi, '@$1Ghz')
    .replace(/\s+/g, ' ')
    .trim()
  if (cores) s += ` × ${cores}`
  return s
}
function cpuCores(ip: string): number {
  return statsInfo[ip]?.cpu?.cores || 0
}
function memUsed(ip: string): string {
  return statsInfo[ip]?.memory?.used_gb?.toFixed(1) || '--'
}
function memTotal(ip: string): string {
  const v = statsInfo[ip]?.memory?.total_gb
  return v ? Math.round(v).toString() : '--'
}
function hasGPU(ip: string): boolean {
  return !!statsInfo[ip]?.gpu
}
function gpuModel(ip: string): string {
  return statsInfo[ip]?.gpu?.model || tt('gpuUsage')
}
function diskList(ip: string): any[] {
  const info = statsInfo[ip]
  if (!info?.disks) return []
  return info.disks.map((d: any) => {
    const parts = ((d.partitions || []) as any[]).map((p: any) => ({
      name: p.name,
      total: gbStr(p.total_gb),
      used: gbStr(p.used_gb),
      pct: p.total_gb > 0 ? Math.round(p.used_gb / p.total_gb * 100) : 0,
    })).sort((a: any, b: any) => a.name.localeCompare(b.name))
    // Sort key: min partition name (ASCII) → stable ordering
    const sortKey = parts.length ? parts.reduce((min: string, p: any) => p.name < min ? p.name : min, parts[0].name) : d.name
    return { name: d.name, model: d.model || '', total: gbStr(d.total_gb), used: gbStr(d.used_gb), parts, sortKey }
  }).sort((a, b) => a.sortKey.localeCompare(b.sortKey))
}
function netList(ip: string): any[] {
  const info = statsInfo[ip]
  if (!info?.network) return []
  return info.network.map((n: any) => ({
    name: n.name,
    desc: n.desc || n.name,
  }))
}
function gbStr(v: number | undefined): string {
  if (v == null) return '--'
  return v.toFixed(v >= 1000 ? 0 : 1)
}

// Per-disk / per-NIC chart data helpers
function diskHistory(ip: string, diskName: string, field: 'readBps' | 'writeBps'): number[] {
  return statsHistory[ip]?.map(s => {
    const d = s.disks?.find(d => d.name === diskName)
    return d ? d[field] : 0
  }) || []
}
function netHistory(ip: string, netName: string, field: 'recvBps' | 'sentBps'): number[] {
  return statsHistory[ip]?.map(s => {
    const n = s.nets?.find(n => n.name === netName)
    return n ? n[field] : 0
  }) || []
}

// Dynamic Y-axis max — pure data maximum, recalculates as points fall out of window
function cpuChartMax(ip: string): number {
  const arr = statsHistory[ip]
  if (!arr?.length) return 1
  return Math.max(1, ...arr.map(s => s.cpu))
}
function gpuChartMax(ip: string): number {
  const arr = statsHistory[ip]
  if (!arr?.length) return 1
  return Math.max(1, ...arr.map(s => s.gpu ?? 0))
}
function memChartMax(ip: string): number {
  const arr = statsHistory[ip]
  if (!arr?.length) return 1
  const totalGB = statsInfo[ip]?.memory?.total_gb || 1
  return Math.max(0.1, ...arr.map(s => s.mem / 100 * totalGB))
}
function memGBHistory(ip: string): number[] {
  const totalGB = statsInfo[ip]?.memory?.total_gb || 1
  return statsHistory[ip]?.map(s => s.mem / 100 * totalGB) || []
}
function diskChartMax(ip: string, diskName: string): number {
  let m = 0
  for (const s of (statsHistory[ip] || [])) {
    const d = s.disks?.find(d => d.name === diskName)
    if (d) {
      if (d.readBps > m) m = d.readBps
      if (d.writeBps > m) m = d.writeBps
    }
  }
  return Math.max(m, 1)
}
function netChartMax(ip: string, netName: string): number {
  let m = 0
  for (const s of (statsHistory[ip] || [])) {
    const n = s.nets?.find(n => n.name === netName)
    if (n) {
      if (n.recvBps > m) m = n.recvBps
      if (n.sentBps > m) m = n.sentBps
    }
  }
  return Math.max(m, 1)
}

// Get current speed values for legend display
function curDiskSpeed(ip: string, diskName: string): { r: number, w: number } {
  const h = statsHistory[ip]
  if (!h?.length) return { r: 0, w: 0 }
  const last = h[h.length - 1]
  const d = last.disks?.find(d => d.name === diskName)
  return { r: d?.readBps || 0, w: d?.writeBps || 0 }
}
function curNetSpeed(ip: string, netName: string): { r: number, s: number } {
  const h = statsHistory[ip]
  if (!h?.length) return { r: 0, s: 0 }
  const last = h[h.length - 1]
  const n = last.nets?.find(n => n.name === netName)
  return { r: n?.recvBps || 0, s: n?.sentBps || 0 }
}
function curCpuPct(ip: string): number {
  const h = statsHistory[ip]
  if (!h?.length) return 0
  return h[h.length - 1].cpu
}
function dkTotalPct(ip: string, dk: { name: string, used: string, total: string }): number {
  const info = statsInfo[ip]
  if (!info?.disks) return 0
  const raw = info.disks.find((d: any) => d.name === dk.name)
  if (!raw || !raw.total_gb) return 0
  return Math.round(raw.used_gb / raw.total_gb * 100)
}
function diskFillColor(pct: number): string {
  if (pct >= 90) return '#ef5350'
  if (pct >= 80) return '#ff9800'
  return '#42a5f5'
}
function curGpuPct(ip: string): number {
  const h = statsHistory[ip]
  if (!h?.length) return 0
  return h[h.length - 1].gpu ?? 0
}

function memLabel(ip: string): string {
  const m = statsInfo[ip]?.memory
  if (!m) return tt('memory')
  let label = ''
  if (m.ddr_type) label += m.ddr_type + ' '
  label += Math.round(m.total_gb || 0) + 'GB'
  if (m.speed_mhz) label += ' ' + m.speed_mhz + 'MHz'
  return label
}

function getSocks5Proxy(): string | undefined {
  if (!netRunning.value) return undefined
  const cfg = currentNet.value?.config as any
  if (cfg?.enable_socks5 && cfg?.socks5_port) return `socks5://127.0.0.1:${cfg.socks5_port}`
  return undefined
}

function parseWolToml(t: string): WolDevice[] {
  if (!t?.trim()) return []
  const devs: WolDevice[] = []; const lines = t.split('\n'); let cur: Record<string, any> | null = null
  for (const line of lines) {
    const tl = line.trim(); if (!tl || tl.startsWith('#')) continue
    if (tl === '[[device]]') { if (cur?.mac) devs.push(cur as any); cur = {}; continue }
    if (cur) { const m = tl.match(/^(\w+)\s*=\s*(.+)$/); if (m) { let v: any = m[2].trim(); if ((v.startsWith('"') && v.endsWith('"')) || (v.startsWith("'") && v.endsWith("'"))) v = v.slice(1, -1); if (/^\d+$/.test(v)) v = parseInt(v); cur[m[1]] = v } }
  }
  if (cur?.mac) devs.push(cur as any); return devs
}
function loadWolConfig() { wolDevices.value = parseWolToml(wolToml.value); for (const w of wolDevices.value) { if (!(w.ip in wolStatus)) { wolStatus[w.ip] = { online: false, phase: 'idle' }; wolErrors[w.ip] = '' } } }
function cleanErr(e: any): string {
  let s = String(e)
  s = s.replace(/^HTTP request failed: /, '')
  s = s.replace(/^error sending request for url \(https?:\/\/[^)]+\):?\s*/, '')
  return s || 'Connection failed'
}

async function checkRouterStatus(): Promise<boolean> {
  const r = wolDevices.value[0]?.router_ip
  if (!r) { wolDebug.value = 'No router_ip configured'; return false }
  if (netRunning.value) {
    const proxy = getSocks5Proxy()
    if (!proxy) { wolDebug.value = 'No SOCKS5 proxy (check socks5_proxy in TOML)'; return false }
    wolDebug.value = 'ET check via ' + proxy + ' → ' + r
    try { await httpGet(`http://${r}/cgi-bin/wolplus-api?action=status&ip=127.0.0.1`, proxy); wolDebug.value += ' OK'; return true }
    catch (e: any) { wolDebug.value += ' FAIL: ' + cleanErr(e); return false }
  }
  wolDebug.value = 'LAN check tcpPing → ' + r
  try { await tcpPing(r, 80); wolDebug.value += ' OK'; return true } catch (e: any) { wolDebug.value += ' FAIL: ' + cleanErr(e); return false }
}

function stopPhasePoll(w: WolDevice, phase: string) {
  const key = `${phase}_${w.ip}`
  if (phaseTimers[key]) { clearTimeout(phaseTimers[key]); delete phaseTimers[key] }
  wolStatus[w.ip] = { ...wolStatus[w.ip], phase: 'idle' }
}

async function checkAllWolStatus() {
  if (checkingWol.value || !wolDevices.value.length) return; checkingWol.value = true
  if (wolFirstCheck.value) routerOnline.value = undefined
  const reachable = await checkRouterStatus()
  routerOnline.value = reachable
  wolFirstCheck.value = false
  if (!reachable) {
    // Abort all in-progress phase polls
    for (const key of Object.keys(phaseTimers)) { clearTimeout(phaseTimers[key]); delete phaseTimers[key] }
    for (const w of wolDevices.value) {
      if (w.ip) wolStatus[w.ip] = { ...wolStatus[w.ip], online: false, phase: 'idle' }
    }
    checkingWol.value = false; return
  }
  const proxy = getSocks5Proxy()
  await Promise.all(wolDevices.value.filter(w => w.ip).map(async w => {
    try { const text = await httpGet(`http://${w.ip}:${w.agent_port || 32249}/api/v1/status`, proxy, 2); const data = JSON.parse(text); wolStatus[w.ip] = { ...wolStatus[w.ip], online: data.online === true }; wolErrors[w.ip] = '' }
    catch (e: any) { wolStatus[w.ip] = { ...wolStatus[w.ip], online: false }; wolErrors[w.ip] = cleanErr(e) }
  })); checkingWol.value = false
}
function openWolEditorFn() { wolToml.value = localStorage.getItem('wolDevicesToml') || ''; showWolEditor.value = true }
function saveWolConfig() { localStorage.setItem('wolDevicesToml', wolToml.value); loadWolConfig(); showWolEditor.value = false; checkAllWolStatus(); showSnack(tt('configSaved')) }

// --- Dialog textarea helpers ---
const wolTextareaRef = ref<any>(null)
const netTextareaRef = ref<any>(null)
async function readClipboard(): Promise<string> {
  try { return await readText() } catch { return '' }
}

async function importFromClipboard(refName: 'wol' | 'net' | 'luci') {
  const text = await readClipboard()
  if (!text) { showSnack('Clipboard is empty'); return }
  if (refName === 'wol') wolToml.value = text
  else if (refName === 'net') editingNetToml.value = text
  else luciToml.value = text
}

function getWolPhaseLabel(w: WolDevice & { status: { online: boolean; phase: string } }): string {
  if (!w.ip) return tt('noIpConfigured')
  if (routerOnline.value === undefined) return tt('detecting')
  if (!routerOnline.value) return tt('routerOffline')
  if (w.status.phase === 'waking') return tt('connecting')
  if (w.status.phase === 'shutting') return tt('connecting')
  return w.status.online ? tt('online') : tt('offline')
}

async function doWake(w: WolDevice) {
  wolStatus[w.ip] = { ...wolStatus[w.ip], phase: 'waking' }
  const proxy = getSocks5Proxy()
  try {
    const url = `http://${w.router_ip}/cgi-bin/wolplus-api?action=wake&mac=${encodeURIComponent(w.mac)}&iface=${encodeURIComponent(w.interface || 'br-lan')}`
    const text = await httpGet(url, proxy); const data = JSON.parse(text); wolErrors[w.ip] = ''
    showSnack(data.success ? tt('signalSent') : data.message)
  } catch (e: any) { wolErrors[w.ip] = cleanErr(e); showSnack(e.message); wolStatus[w.ip] = { ...wolStatus[w.ip], phase: 'idle' }; return }
  startPhasePoll(w, 'waking', 0)
}
async function doShutdown(w: WolDevice) {
  wolStatus[w.ip] = { ...wolStatus[w.ip], phase: 'shutting' }
  const proxy = getSocks5Proxy()
  try { await httpPost(`http://${w.ip}:${w.agent_port || 32249}/api/v1/shutdown`, proxy); wolErrors[w.ip] = '' } catch (e: any) { wolErrors[w.ip] = cleanErr(e); wolStatus[w.ip] = { ...wolStatus[w.ip], phase: 'idle' }; return }
  startPhasePoll(w, 'shutting', 0)
}
function startPhasePoll(w: WolDevice, phase: 'waking' | 'shutting', count: number) {
  if (count >= 12) { wolStatus[w.ip] = { ...wolStatus[w.ip], phase: 'idle' }; return }
  const key = `${phase}_${w.ip}`
  phaseTimers[key] = setTimeout(async () => {
    if (!routerOnline.value) { wolStatus[w.ip] = { ...wolStatus[w.ip], phase: 'idle' }; return }
    const proxy = getSocks5Proxy()
    try { const text = await httpGet(`http://${w.ip}:${w.agent_port || 32249}/api/v1/status`, proxy, 2); const data = JSON.parse(text); const online = data.online === true; wolErrors[w.ip] = ''; if ((phase === 'waking' && online) || (phase === 'shutting' && !online)) { wolStatus[w.ip] = { online, phase: 'idle' }; showSnack(w.name + ' ' + (phase === 'waking' ? tt('online') : tt('offline'))); return } } catch (e: any) {
      // Shutdown: connection refused → agent already down → success
      if (phase === 'shutting') { wolStatus[w.ip] = { online: false, phase: 'idle' }; wolErrors[w.ip] = ''; showSnack(w.name + ' ' + tt('offline')); return }
      wolErrors[w.ip] = cleanErr(e)
    }
    if (routerOnline.value) startPhasePoll(w, phase, count + 1)
    else { wolStatus[w.ip] = { ...wolStatus[w.ip], phase: 'idle' } }
  }, 3000)
}
function getWolPathType(): 'tunnel' | 'lan' { return netRunning.value ? 'tunnel' : 'lan' }
const sortedWol = computed(() => {
  const list = wolDevices.value.map(w => ({ ...w, status: wolStatus[w.ip] || { online: false, phase: 'idle' as const }, error: wolErrors[w.ip] || '' }))
  list.sort((a, b) => { if (a.status.online !== b.status.online) return a.status.online ? -1 : 1; return ipToNum(a.ip) - ipToNum(b.ip) })
  return list
})

// ==================== Network Tab ====================
interface StoredNet { config: NetworkTypes.NetworkConfig; source: string; rawToml: string }
const networks = ref<StoredNet[]>([])
const netIndex = ref(-1)
const netRunning = ref(false)
const netConnecting = ref(false)
const netConnectTime = ref(0)
const netDevName = ref('')
const netSelfIp = ref('')
const netSelfIpv6 = ref('')
const netSelfNat = ref<any>(null)
const netPeers = ref<any[]>([])
const netServers = ref<any[]>([])
const netLoading = ref(false)
const showNetEditor = ref(false)
const editingNetToml = ref('')
const showSwitchMenu = ref(false)
const savingNet = ref(false)
let netPeriod: Utils.PeriodicTask | null = null
const netExpanded = reactive<Record<string, boolean>>({})
const showDeleteConfirm = ref(false)
const deleteTargetIdx = ref(-1)

function loadNetworks() { const raw = localStorage.getItem('networkList'); const list: StoredNet[] = JSON.parse(raw || '[]').filter((e: any) => e.config?.instance_id); networks.value = list; const lastId = loadLastNetworkInstanceId(); let idx = list.findIndex(e => e.config.instance_id === lastId); if (idx < 0 && list.length > 0) idx = 0; netIndex.value = idx }
const currentNet = computed(() => netIndex.value >= 0 ? networks.value[netIndex.value] : null)
const netInstId = computed(() => currentNet.value?.config?.instance_id || '')
const netDiscovering = computed(() => netRunning.value && !netPeers.length && !netServers.length && netConnectTime.value > 0 && Date.now() - netConnectTime.value < 12000)

async function fetchNetInfo() { if (!netInstId.value) return; try { const resp = await collectNetworkInfo(netInstId.value); const infoMap = resp.info?.map || {}; const info = infoMap[netInstId.value] || Object.values(infoMap)[0]; if (info) { netRunning.value = info.running; netDevName.value = info.dev_name || ''; const my = info.my_node_info; netSelfIp.value = formatIp(my?.virtual_ipv4); netSelfIpv6.value = formatIpv6(my?.ips?.public_ipv6); netSelfNat.value = my?.stun_info || null; const all = (info.peer_route_pairs || []).filter((p: any) => p.route?.cost > 0); netPeers.value = all.filter((p: any) => !isServer(p)); netServers.value = all.filter((p: any) => isServer(p)) } } catch { /* */ } updateTrafficSpeed() }
async function refreshNet() { netLoading.value = true; await fetchNetInfo(); netLoading.value = false }

async function doConnect() {
  if (!currentNet.value || netConnecting.value) return; netConnecting.value = true; trafficHistory.splice(0, trafficHistory.length); lastTraffic = { up: 0, down: 0, ts: 0 }
  try {
    const ids = networks.value.map(e => e.config.instance_id)
    await sendConfigs(ids)
    saveLastNetworkInstanceId(currentNet.value.config.instance_id)
    await new Promise(r => setTimeout(r, 2000))
    await fetchNetInfo()
    if (netRunning.value) { netConnectTime.value = Date.now(); checkAllWolStatus() }
  } catch (e: any) { showSnack(String(e)) }
  finally { netConnecting.value = false }
}
async function doDisconnect() { try { await updateNetworkConfigState(netInstId.value, true); netRunning.value = false; netConnectTime.value = 0; showSnack(tt('disconnectedToast')) } catch (e: any) { showSnack(String(e)) } }

async function openNetEditor() {
  if (currentNet.value?.rawToml) { editingNetToml.value = currentNet.value.rawToml }
  else if (currentNet.value?.config) { try { editingNetToml.value = await parseNetworkConfig(currentNet.value.config) } catch { editingNetToml.value = '' } }
  else { editingNetToml.value = '' }
  showNetEditor.value = true
}
async function saveNetConfig() { if (!editingNetToml.value.trim()) return; savingNet.value = true; try { const config = await generateNetworkConfig(editingNetToml.value); const raw = localStorage.getItem('networkList'); const list: StoredNet[] = JSON.parse(raw || '[]'); const idx = list.findIndex(e => e.config?.instance_id === config.instance_id); const isNew = idx < 0; if (idx >= 0) list[idx] = { config, source: 'user', rawToml: editingNetToml.value }; else list.push({ config, source: 'user', rawToml: editingNetToml.value }); localStorage.setItem('networkList', JSON.stringify(list)); loadNetworks(); if (isNew) { const newIdx = networks.value.findIndex(e => e.config.instance_id === config.instance_id); if (newIdx >= 0) switchToNet(newIdx) } showNetEditor.value = false; showSnack(tt('saved')); } catch (e: any) { showSnack(String(e)) } finally { savingNet.value = false } }
function openAddNet() { editingNetToml.value = ''; showNetEditor.value = true; showSwitchMenu.value = false }
function switchToNet(idx: number) { netIndex.value = idx; netConnectTime.value = 0; saveLastNetworkInstanceId(networks.value[idx].config.instance_id); showSwitchMenu.value = false; refreshNet() }
function confirmDeleteNet(idx: number, e: MouseEvent) { e.stopPropagation(); deleteTargetIdx.value = idx; showDeleteConfirm.value = true; showSwitchMenu.value = false }
async function doDeleteNet() {
  const idx = deleteTargetIdx.value
  if (idx < 0 || idx >= networks.value.length) return
  const instId = networks.value[idx].config.instance_id
  try { await deleteNetworkInstance(instId) } catch { /* */ }
  const raw = localStorage.getItem('networkList')
  const list: StoredNet[] = JSON.parse(raw || '[]')
  const newList = list.filter(e => e.config?.instance_id !== instId)
  localStorage.setItem('networkList', JSON.stringify(newList))
  editingNetToml.value = ''
  loadNetworks()
  showDeleteConfirm.value = false
  showSnack(tt('deleted'))
}

// --- Helpers ---
function formatIp(ip: any): string { if (!ip) return ''; if (typeof ip === 'string') return ip; if (ip.address?.addr !== undefined) { const a = ip.address.addr; return ((a>>>24)&0xFF)+'.'+((a>>>16)&0xFF)+'.'+((a>>>8)&0xFF)+'.'+(a&0xFF) } return '' }
function formatIpv6(ip: any): string {
  if (!ip) return ''
  if (typeof ip === 'string') return ip
  const p = [ip.part1, ip.part2, ip.part3, ip.part4]
  if (p.some(x => x === undefined)) return ''
  return p.map(x => (x >>> 16).toString(16) + ':' + (x & 0xFFFF).toString(16)).join(':')
}
function ipToNum(ip: string): number { const p = ip.split('.'); return p.length === 4 ? (+p[0]<<24)+(+p[1]<<16)+(+p[2]<<8)+(+p[3]) : 0 }
function costLabel(c: number | undefined): string { return c === undefined || c === 0 ? 'Local' : c === 1 ? 'P2P' : 'Relay(' + c + ')' }
function costCls(c: number | undefined): string { if (c === undefined || c === 0) return 'md-rt-l'; if (c === 1) return 'md-rt-p'; return 'md-rt-r' }
function isServer(p: any): boolean { return p.route?.feature_flag?.is_public_server === true }
function peerKey(p: any): string { return 'pn-' + p.route?.peer_id }

function isRouterHost(hostname: string): boolean {
  if (!hostname) return false
  return /route|wrt|路由|home|家/i.test(hostname)
}

function formatBytes(b: number): string {
  if (!b || b < 0) return ''
  if (b < 1024) return b + 'B'
  if (b < 1048576) return (b / 1024).toFixed(0) + 'KB'
  if (b < 1073741824) return (b / 1048576).toFixed(1) + 'MB'
  return (b / 1073741824).toFixed(1) + 'GB'
}

function qualityText(lat: string, loss: string): string {
  if (!lat) return ''
  const l = parseFloat(loss) || 0
  if (l === 0) return lat
  return loss + ' · ' + lat
}

function qualityCls(lat: string, loss: string): string {
  const l = parseFloat(loss) || 0
  const d = parseFloat(lat) || 0
  if (l > 10 || d > 300) return 'md-q-bad'
  if (l > 5 || d > 100) return 'md-q-warn'
  if (l > 3 || d > 50) return 'md-q-info'
  return 'md-q-good'
}

function natLabel(s: any): { text: string; cls: string } {
  if (!s || s.udp_nat_type === undefined) return { text: '', cls: '' }
  const t = s.udp_nat_type
  if (t === 1) return { text: 'Open', cls: 'md-nat-g' }
  const m: Record<number, { n: number; cls: string }> = {
    3: { n: 1, cls: 'md-nat-g' },
    4: { n: 2, cls: 'md-nat-g' },
    5: { n: 3, cls: 'md-nat-b' },
    6: { n: 4, cls: 'md-nat-o' },
  }
  const e = m[t]
  if (!e) return { text: '', cls: '' }
  return { text: 'NAT' + e.n, cls: e.cls }
}

function buildDetail(p: any): any {
  const r = p.route; const pr = p.peer; const d: any = { cost: r?.cost, host: r?.hostname, ver: r?.version }
  if (pr?.conns?.length) {
    let lat = 0, loss = 0, up = 0, down = 0
    for (const c of pr.conns) {
      if (c.stats?.latency_us) lat += c.stats.latency_us
      loss += c.loss_rate || 0
      up += (c.stats?.tx_bytes || 0)
      down += (c.stats?.rx_bytes || 0)
    }
    const n = pr.conns.length
    if (lat > 0) { d.lat = Math.round(lat / 1000 / n) + 'ms'; d.latNum = Math.round(lat / 1000 / n) }
    d.loss = n > 0 ? Math.round(loss / n * 100) + '%' : ''
    if (up > 0) d.up = '↑' + formatBytes(up)
    if (down > 0) d.down = '↓' + formatBytes(down)
    d.tun = [...new Set(pr.conns.map((c: any) => c.tunnel?.tunnel_type || '?'))].join(', ')
  }
  return d
}

const sortedPeers = computed(() => {
  const list = [...netPeers.value]
  list.sort((a, b) => {
    const aRtr = isRouterHost(a.route?.hostname || '')
    const bRtr = isRouterHost(b.route?.hostname || '')
    if (aRtr !== bRtr) return aRtr ? -1 : 1
    return ipToNum(formatIp(a.route?.ipv4_addr)) - ipToNum(formatIp(b.route?.ipv4_addr))
  })
  return list
})

const sortedServers = computed(() => [...netServers.value].sort((a, b) => (a.route?.hostname || '').localeCompare(b.route?.hostname || '')))

// Traffic speed tracking
const netTrafficSpeed = reactive({ up: '', down: '', upBytes: 0, downBytes: 0, upTotal: '', downTotal: '' })
const trafficHistory = reactive<{ up: number; down: number }[]>([])
const TRAFFIC_MAX_POINTS = 60 // ~3 min at 3s interval
let lastTraffic = { up: 0, down: 0, ts: 0 }
const TRAFFIC_UP_COLOR = '#43a047'
const TRAFFIC_DOWN_COLOR = '#1e88e5'

const SETTINGS_COLORS = ['#81c784', '#e57373', '#64b5f6', '#ffcc80', '#b39ddb', '#80cbc4', '#f48fb1']
const settingsItemColors = computed(() => {
  const ids = ['defaultHome', 'theme', 'amoled', 'language', 'debug']
  if (showDebug.value) ids.push('advanced')
  ids.push('config', 'help', 'about')
  const map: Record<string, string> = {}
  ids.forEach((id, i) => { map[id] = SETTINGS_COLORS[i % SETTINGS_COLORS.length] })
  return map
})

function updateTrafficSpeed() {
  let up = 0, down = 0
  for (const p of [...netPeers.value, ...netServers.value]) {
    const pr = p.peer
    if (pr?.conns?.length) {
      for (const c of pr.conns) {
        up += c.stats?.tx_bytes || 0
        down += c.stats?.rx_bytes || 0
      }
    }
  }
  netTrafficSpeed.upTotal = formatBytes(up)
  netTrafficSpeed.downTotal = formatBytes(down)
  const now = Date.now()
  if (lastTraffic.ts > 0 && now > lastTraffic.ts) {
    const sec = (now - lastTraffic.ts) / 1000
    netTrafficSpeed.upBytes = Math.max(0, Math.round((up - lastTraffic.up) / sec))
    netTrafficSpeed.downBytes = Math.max(0, Math.round((down - lastTraffic.down) / sec))
    netTrafficSpeed.up = formatBytes(netTrafficSpeed.upBytes) + '/s'
    netTrafficSpeed.down = formatBytes(netTrafficSpeed.downBytes) + '/s'
    trafficHistory.push({ up: netTrafficSpeed.upBytes, down: netTrafficSpeed.downBytes })
    while (trafficHistory.length > TRAFFIC_MAX_POINTS) trafficHistory.shift()
  }
  lastTraffic = { up, down, ts: now }
}

function buildSmoothPath(data: number[], maxVal: number, height: number, width: number): string {
  if (data.length < 2) return ''
  const stepX = width / (data.length - 1)
  const scaleY = (v: number) => height - (v / maxVal) * (height - 4) - 2
  let d = `M 0,${scaleY(data[0])}`
  for (let i = 0; i < data.length - 1; i++) {
    const x1 = i * stepX; const y1 = scaleY(data[i])
    const x2 = (i + 1) * stepX; const y2 = scaleY(data[i + 1])
    const cx1 = x1 + stepX * 0.4; const cx2 = x2 - stepX * 0.4
    d += ` C ${cx1},${y1} ${cx2},${y2} ${x2},${y2}`
  }
  return d
}

function formatSpeedY(v: number): string {
  if (v < 1024) return v + 'B/s'
  if (v < 1048576) return (v / 1024).toFixed(0) + 'K'
  if (v < 1073741824) return (v / 1048576).toFixed(0) + 'M'
  return (v / 1073741824).toFixed(1) + 'G'
}
function formatXLabel(total: number, idx: number): string {
  const sec = (total - 1 - idx) * 3
  if (sec === 0) return 'now'
  if (sec < 60) return '-' + sec + 's'
  const min = sec / 60
  // Round to nearest 0.5 to avoid floating point quirks
  const r = Math.round(min * 2) / 2
  const t = r.toFixed(1)
  return '-' + (t.endsWith('0') ? t.slice(0, -2) : t) + 'm'
}

// ==================== Router Tab (LuCI proxy) ====================
interface LuciRouter { name: string; ip: string; username: string; password: string }
const luciToml = ref(localStorage.getItem('luciRoutersToml') || '')
const luciRouters = ref<LuciRouter[]>([])
const luciIdx = ref(-1)
const proxyUrl = ref('')
const luciIframeSrc = ref('')
const luciIframeKey = ref(0)
const luciLoading = ref(false)
const luciCurrentPath = ref('/cgi-bin/luci/admin/')
const showLuciEditor = ref(false)
const showLuciMenu = ref(false)
const lastLuciPathType = ref('')

function parseLuciToml(t: string): LuciRouter[] {
  if (!t?.trim()) return []
  const devs: LuciRouter[] = []; const lines = t.split('\n'); let cur: Record<string, any> | null = null
  for (const line of lines) {
    const tl = line.trim(); if (!tl || tl.startsWith('#')) continue
    if (tl === '[[router]]') { if (cur?.ip) devs.push(cur as any); cur = {}; continue }
    if (cur) { const m = tl.match(/^(\w+)\s*=\s*(.+)$/); if (m) { let v: any = m[2].trim(); if ((v.startsWith('"') && v.endsWith('"')) || (v.startsWith("'") && v.endsWith("'"))) v = v.slice(1, -1); cur[m[1]] = v } }
  }
  if (cur?.ip) devs.push(cur as any); return devs
}
function loadLuciConfig() {
  luciRouters.value = parseLuciToml(luciToml.value)
  if (luciIdx.value >= luciRouters.value.length) luciIdx.value = luciRouters.value.length - 1
}
const currentLuciRouter = computed(() => luciIdx.value >= 0 ? luciRouters.value[luciIdx.value] : null)

async function startLuciProxy(r: LuciRouter) {
  luciLoading.value = true
  try {
    await luciProxyStop()
    const socks5 = getSocks5Proxy()
    const url = await luciProxyStart(r.ip, r.username, r.password, socks5 || null)
    proxyUrl.value = url
    luciIframeSrc.value = url + '/cgi-bin/luci/admin/'
    luciCurrentPath.value = '/cgi-bin/luci/admin/'
    lastLuciPathType.value = socks5 ? 'tunnel' : 'lan'
    luciIframeKey.value++
    localStorage.setItem('lastLuciRouterIp', r.ip)
  } catch (e: any) {
    showSnack(String(e))
    proxyUrl.value = ''
  }
  luciLoading.value = false
}

async function stopLuciProxyFn() {
  try { await luciProxyStop() } catch { /* */ }
  proxyUrl.value = ''
}

async function refreshLuciIframe() {
  const r = currentLuciRouter.value
  if (!r) return
  luciLoading.value = true
  luciIframeSrc.value = ''  // hide iframe immediately, show spinner during restart
  try {
    const socks5 = getSocks5Proxy()
    const prevPath = await luciGetLastPath().catch(() => '/cgi-bin/luci/admin/')
    await luciProxyStop()
    const url = await luciProxyStart(r.ip, r.username, r.password, socks5 || null)
    proxyUrl.value = url
    luciCurrentPath.value = prevPath
    luciIframeSrc.value = url + prevPath
    lastLuciPathType.value = socks5 ? 'tunnel' : 'lan'
    luciIframeKey.value++
    localStorage.setItem('lastLuciRouterIp', r.ip)
  } catch (e: any) {
    showSnack(String(e))
    proxyUrl.value = ''
  }
  luciLoading.value = false
}

function deleteLuciRouter(idx: number, e: MouseEvent) {
  e.stopPropagation()
  const wasCurrent = idx === luciIdx.value
  luciRouters.value.splice(idx, 1)
  const lines = luciRouters.value.map(r => `[[router]]
name = "${r.name}"
ip = "${r.ip}"
username = "${r.username}"
password = "${r.password}"
`)
  luciToml.value = lines.join('\n')
  saveLuciConfigSilent()
  if (wasCurrent) {
    stopLuciProxyFn()
    if (luciRouters.value.length > 0) {
      luciIdx.value = Math.min(idx, luciRouters.value.length - 1)
      startLuciProxy(luciRouters.value[luciIdx.value])
    } else {
      luciIdx.value = -1
    }
  } else if (idx < luciIdx.value) {
    luciIdx.value--
  }
}

function saveLuciConfigSilent() {
  localStorage.setItem('luciRoutersToml', luciToml.value)
  loadLuciConfig()
}

async function switchLuciRouter(idx: number) {
  luciIdx.value = idx; showLuciMenu.value = false
  const r = luciRouters.value[idx]
  if (r) await startLuciProxy(r)
}

function openLuciEditorFn() {
  luciToml.value = localStorage.getItem('luciRoutersToml') || ''
  showLuciEditor.value = true
}
function saveLuciConfig() {
  localStorage.setItem('luciRoutersToml', luciToml.value)
  loadLuciConfig()
  showLuciEditor.value = false
  showSnack(tt('configSaved'))
  const cur = luciRouters.value[luciIdx.value] || luciRouters.value[0]
  if (cur && luciIdx.value >= 0) {
    luciIdx.value = luciRouters.value.findIndex(r => r.ip === cur.ip)
    startLuciProxy(cur)
  }
}

function onLuciDocClick(e: MouseEvent) {
  const t = e.target as HTMLElement
  if (!t.closest('.md-switch-menu') && !t.closest('.md-switch-btn')) showLuciMenu.value = false
}

// ==================== Help Dialog ====================
const showHelpDlg = ref(false)
const showAboutDlg = ref(false)
const HELP_WOL_URL = 'https://github.com/lymwhen/luci-app-wolplus/releases'
const ABOUT_EASYTIER_URL = 'https://github.com/EasyTier/EasyTier'
const ABOUT_WOLPLUS_URL = 'https://github.com/animegasan/luci-app-wolplus'
const ABOUT_THIS_URL = 'https://github.com/lymwhen/EasyTier'
const ABOUT_WOLPLUS_DEP_URL = 'https://github.com/lymwhen/luci-app-wolplus'

async function openHelpUrl(url: string) {
  try {
    await openUrl(url)
  } catch {
    window.open(url, '_system')
  }
}

const HELP_CFG_WOL = `[[device]]
name = "My PC"
mac = "AA:BB:CC:DD:EE:FF"
ip = "192.168.1.100"
interface = "br-lan"
router_ip = "192.168.1.1"
agent_port = 32249`

const HELP_CFG_NET = `instance_name = "easytier"
dhcp = true
listeners = [
    "tcp://0.0.0.0:11010",
    "udp://0.0.0.0:11010",
    "wg://0.0.0.0:11011",
]
rpc_portal = "0.0.0.0:0"

[network_identity]
network_name = "easytier"
network_secret = ""

[[peer]]
uri = "tcp://public.easytier.top:11010"

[[proxy_network]]
cidr = "192.168.1.0/24"

[flags]
enable_exit_node = true

socks5_proxy = "socks5://0.0.0.0:32259"`

const HELP_CFG_LUCI = `[[router]]
name = "Home Router"
ip = "192.168.1.1"
username = "root"
password = ""`

const copiedIdx = ref(-1)
async function copyHelpCfg(text: string, idx: number) {
  try {
    await writeText(text)
    copiedIdx.value = idx
    setTimeout(() => { copiedIdx.value = -1 }, 2000)
  } catch {
    /* ignore */
  }
}

// ==================== Theme State Machine ====================
const themeMode = ref(localStorage.getItem('themeMode') || 'auto')
const amoledMode = ref(localStorage.getItem('amoledMode') === '1')

function applyTheme() {
  setTheme(resolveTheme(themeMode.value, amoledMode.value))
}

watch([themeMode, amoledMode], () => {
  localStorage.setItem('themeMode', themeMode.value)
  localStorage.setItem('amoledMode', amoledMode.value ? '1' : '0')
  applyTheme()
})

// Follow system dark mode changes
if (typeof window !== 'undefined') {
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (themeMode.value === 'auto') applyTheme()
  })
}

// ==================== One-Click Config (Export/Import) ====================
const showImportDlg = ref(false)
const showImportConfirm = ref(false)
const importText = ref('')
const importPendingData = ref<any>(null)
const importTextareaRef = ref<any>(null)

function toBase64Url(bytes: Uint8Array): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_'
  let result = ''
  const len = bytes.length
  for (let i = 0; i < len; i += 3) {
    const b0 = bytes[i], b1 = i + 1 < len ? bytes[i + 1] : 0, b2 = i + 2 < len ? bytes[i + 2] : 0
    result += chars[(b0 >> 2) & 0x3F]
    result += chars[((b0 & 0x3) << 4) | ((b1 >> 4) & 0xF)]
    if (i + 1 < len) result += chars[((b1 & 0xF) << 2) | ((b2 >> 6) & 0x3)]
    if (i + 2 < len) result += chars[b2 & 0x3F]
  }
  return result
}

function fromBase64Url(str: string): Uint8Array {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_'
  const lookup = new Uint8Array(128)
  for (let i = 0; i < 64; i++) lookup[chars.charCodeAt(i)] = i
  const len = str.length
  const bytes = new Uint8Array(Math.floor(len * 3 / 4))
  let j = 0
  for (let i = 0; i < len; i += 4) {
    const c0 = lookup[str.charCodeAt(i)], c1 = i + 1 < len ? lookup[str.charCodeAt(i + 1)] : 0
    const c2 = i + 2 < len ? lookup[str.charCodeAt(i + 2)] : 0, c3 = i + 3 < len ? lookup[str.charCodeAt(i + 3)] : 0
    bytes[j++] = (c0 << 2) | (c1 >> 4)
    if (i + 2 < len) bytes[j++] = ((c1 & 0xF) << 4) | (c2 >> 2)
    if (i + 3 < len) bytes[j++] = ((c2 & 0x3) << 6) | c3
  }
  return j < bytes.length ? bytes.slice(0, j) : bytes
}

async function compressDeflate(text: string): Promise<Uint8Array> {
  const blob = new Blob([text])
  const cs = new CompressionStream('deflate')
  const stream = blob.stream().pipeThrough(cs)
  const reader = stream.getReader()
  const chunks: Uint8Array[] = []
  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    chunks.push(value)
  }
  const total = chunks.reduce((s, c) => s + c.length, 0)
  const result = new Uint8Array(total)
  let offset = 0
  for (const c of chunks) { result.set(c, offset); offset += c.length }
  return result
}

async function decompressDeflate(bytes: Uint8Array): Promise<string> {
  const blob = new Blob([bytes])
  const ds = new DecompressionStream('deflate')
  const stream = blob.stream().pipeThrough(ds)
  const reader = stream.getReader()
  const chunks: Uint8Array[] = []
  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    chunks.push(value)
  }
  const total = chunks.reduce((s, c) => s + c.length, 0)
  const result = new Uint8Array(total)
  let offset = 0
  for (const c of chunks) { result.set(c, offset); offset += c.length }
  return new TextDecoder().decode(result)
}

async function decodeExportData(raw: string): Promise<any> {
  const trimmed = raw.trim()
  if (trimmed.startsWith('ETGC:2:')) {
    const bytes = fromBase64Url(trimmed.slice(7))
    const json = await decompressDeflate(bytes)
    return JSON.parse(json)
  }
  throw new Error('Invalid format')
}

async function doExport() {
  const nets: StoredNet[] = JSON.parse(localStorage.getItem('networkList') || '[]')
  const netTomls: string[] = []
  for (const n of nets) {
    if (n.rawToml) { netTomls.push(n.rawToml); continue }
    if (n.config) { try { netTomls.push(await parseNetworkConfig(n.config)) } catch { /* skip */ } }
  }
  const data = {
    v: 1,
    wol: wolToml.value,
    luci: luciToml.value,
    net: netTomls,
  }
  const json = JSON.stringify(data)
  const compressed = await compressDeflate(json)
  const encoded = 'ETGC:2:' + toBase64Url(compressed)
  try { await writeText(encoded) } catch { /* ignore */ }
  showSnack(tt('exportedToClipboard'))
}

function openImportDlg() {
  importText.value = ''
  showImportDlg.value = true
}

async function doImportFromClipboard() {
  const text = await readClipboard()
  if (!text) { showSnack('Clipboard is empty'); return }
  importText.value = text
}

async function confirmImport() {
  let data: any
  try { data = await decodeExportData(importText.value) } catch { showSnack(tt('importFailed')); return }
  if (!data || typeof data.v !== 'number' || typeof data.wol !== 'string' || typeof data.luci !== 'string' || !Array.isArray(data.net) || !data.net.every((s: any) => typeof s === 'string')) {
    showSnack(tt('importFailed')); return
  }
  importPendingData.value = data
  showImportConfirm.value = true
}

async function executeImport() {
  const data = importPendingData.value
  if (!data) return
  showImportConfirm.value = false
  showImportDlg.value = false

  if (netRunning.value) await doDisconnect()

  localStorage.setItem('wolDevicesToml', data.wol)
  wolToml.value = data.wol
  loadWolConfig()

  localStorage.setItem('luciRoutersToml', data.luci)
  luciToml.value = data.luci
  loadLuciConfig()
  if (activeTab.value === 'luci' && luciRouters.value.length > 0) {
    luciIdx.value = 0
    startLuciProxy(luciRouters.value[0])
  }

  // Delete all old network instances
  const oldList: StoredNet[] = JSON.parse(localStorage.getItem('networkList') || '[]')
  for (const n of oldList) {
    try { await deleteNetworkInstance(n.config.instance_id) } catch { /* */ }
  }
  // Generate new network configs from TOML
  const newList: StoredNet[] = []
  for (const toml of data.net) {
    if (!toml.trim()) continue
    try {
      const config = await generateNetworkConfig(toml)
      newList.push({ config, source: 'user', rawToml: toml })
    } catch { /* skip invalid */ }
  }
  localStorage.setItem('networkList', JSON.stringify(newList))
  loadNetworks()

  importPendingData.value = null
  showSnack(tt('importSuccess'))
}

function netName(): string { const n = currentNet.value; return n ? (n.config.instance_name || n.config.network_name || tt('unnamed')) : tt('noNetwork') }

function onDocClick(e: MouseEvent) { const t = e.target as HTMLElement; if (!t.closest('.md-switch-menu') && !t.closest('.md-switch-btn')) showSwitchMenu.value = false }

onMounted(() => {
  applyTheme()
  loadWolConfig(); checkAllWolStatus(); wolPeriod = new Utils.PeriodicTask(checkAllWolStatus, 30000); wolPeriod.start()
  loadNetworks(); if (netInstId.value) refreshNet(); netPeriod = new Utils.PeriodicTask(fetchNetInfo, 3000); netPeriod.start()
  loadLuciConfig()
  if (activeTab.value === 'luci' && luciRouters.value.length > 0) {
    const lastIp = localStorage.getItem('lastLuciRouterIp')
    const idx = lastIp ? luciRouters.value.findIndex(r => r.ip === lastIp) : -1
    luciIdx.value = idx >= 0 ? idx : 0
    startLuciProxy(luciRouters.value[luciIdx.value])
  }
  document.addEventListener('click', onDocClick)
  document.addEventListener('click', onLuciDocClick)
})
onUnmounted(() => { wolPeriod?.stop(); netPeriod?.stop(); if (statsTimer) clearInterval(statsTimer); stopLuciProxyFn(); for (const k of Object.keys(phaseTimers)) clearTimeout(phaseTimers[k]); if (snackTimer) clearTimeout(snackTimer); document.removeEventListener('click', onDocClick); document.removeEventListener('click', onLuciDocClick) })
</script>

<template>
  <div class="md-app flex flex-col h-screen">

    <!-- ==================== WOL Tab ==================== -->
    <template v-if="activeTab === 'wol'">
      <div class="md-hdr">
        <span class="md-hdr-title">{{ tt('wolDevices') }}</span>
        <span v-if="sortedWol.length && getWolPathType() === 'tunnel'" class="md-path-badge">{{ tt('etLan') }}</span>
        <span v-else-if="sortedWol.length && getWolPathType() === 'lan'" class="md-path-badge md-path-lan">{{ tt('lanDirect') }}</span>
        <div class="flex-1" />
        <button class="md-hdr-btn" @click="openWolEditorFn" :title="tt('editConfig')">
          <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M714.965 128l-85.333 85.333H213.333v597.334h597.334V394.368L896 309.035v544.298A42.667 42.667 0 0 1 853.333 896H170.667A42.667 42.667 0 0 1 128 853.333V170.667A42.667 42.667 0 0 1 170.667 128h544.298z m159.062-38.4l60.373 60.416-392.192 392.192-60.245 0.128-0.086-60.459L874.027 89.6z"/></svg>
        </button>
        <button class="md-hdr-btn" :class="{ 'animate-spin': checkingWol }" @click="checkAllWolStatus" :title="tt('refresh')">
          <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M128 170.666667l92.330667 92.330666A382.378667 382.378667 0 0 1 512 128c211.754667 0 384 172.288 384 384h-85.333333c0-164.693333-134.016-298.666667-298.666667-298.666667a297.557333 297.557333 0 0 0-231.125333 110.208L384 426.666667H128V170.666667z m768 682.666666v-256h-256l103.125333 103.125334A297.514667 297.514667 0 0 1 512 810.666667c-164.650667 0-298.666667-134.016-298.666667-298.666667H128c0 211.754667 172.245333 384 384 384a382.378667 382.378667 0 0 0 291.669333-134.997333L896 853.333333z"/></svg>
        </button>
      </div>
      <div v-if="showDebug && wolDebug && sortedWol.length" class="px-4 py-1 text-xs opacity-60" style="color:var(--md-secondary);font-family:monospace;word-break:break-all">{{ wolDebug }}</div>
      <div class="flex-1 overflow-y-auto px-3 pt-3 pb-2">
        <div v-if="!sortedWol.length" class="flex flex-col items-center justify-center h-full gap-3 opacity-50">
          <svg width="48" height="48" viewBox="0 0 1024 1024" fill="none" stroke="var(--md-muted)" stroke-width="20"><path d="M864 752H624v64h136c12.8 0 24 11.2 24 24s-11.2 24-24 24h-480c-12.8 0-24-11.2-24-24s11.2-24 24-24H400v-64H160c-17.6 0-32-14.4-32-32V192c0-17.6 14.4-32 32-32h704c17.6 0 32 14.4 32 32v528c0 17.6-14.4 32-32 32z m-16-544H176v464h672V208z m-48 416H224V256h576v368z"/></svg>
          <div class="text-base" style="color:var(--md-muted)">{{ tt('noWolDevices') }}</div>
          <div class="text-sm text-center px-4" style="color:var(--md-muted)">{{ tt('tapToAdd') }}</div>
        </div>
        <div v-for="(w, i) in sortedWol" :key="w.ip" class="md-card" :class="{ 'md-card-dim': routerOnline === false }">
          <div class="md-row" @click="toggleDeviceExpand(i)">
            <span class="md-dot" :class="{
              'md-dot-on': w.status.online && w.status.phase==='idle' && routerOnline === true,
              'md-dot-off': (!w.status.online || routerOnline === false) && w.status.phase==='idle',
              'md-dot-ph md-dot-waking': w.status.phase==='waking' && routerOnline !== false,
              'md-dot-ph md-dot-shutting': w.status.phase==='shutting' && routerOnline !== false,
              'md-dot-off animate-pulse': routerOnline === undefined && w.status.phase==='idle'
            }" />
            <div class="flex-1 min-w-0">
              <div class="md-name">{{ w.name }}</div>
              <div class="md-sub">
                <template v-if="w.ip">{{ w.ip }} &middot; {{ getWolPhaseLabel(w) }}</template>
                <template v-else>{{ tt('noIpConfigured') }}</template>
              </div>
            </div>
            <svg v-if="w.status.online" class="md-chevron" :class="{ 'md-chevron-open': expandedDeviceIdx === i }" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M6 9l6 6 6-6"/></svg>
            <button v-if="w.ip && w.status.phase==='idle' && routerOnline === true && !w.status.online" class="md-wake-btn" @click.stop="doWake(w)">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 22c1.1 0 2-.9 2-2h-4c0 1.1.89 2 2 2zm6-6v-5c0-3.07-1.64-5.64-4.5-6.32V4c0-.83-.67-1.5-1.5-1.5s-1.5.67-1.5 1.5v.68C7.63 5.36 6 7.92 6 11v5l-2 2v1h16v-1l-2-2z"/></svg>
              {{ tt('wake') }}
            </button>
            <button v-if="w.ip && w.status.phase==='idle' && routerOnline === true && w.status.online" class="md-shutdown-btn" @click.stop="doShutdown(w)">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M13 3h-2v10h2V3zm4.83 2.17l-1.42 1.42C17.99 7.86 19 9.81 19 12c0 3.87-3.13 7-7 7s-7-3.13-7-7c0-2.19 1.01-4.14 2.59-5.42L6.17 5.17C4.23 6.82 3 9.26 3 12c0 4.97 4.03 9 9 9s9-4.03 9-9c0-2.74-1.23-5.18-3.17-6.83z"/></svg>
              {{ tt('shutdown') }}
            </button>
            <button v-if="!w.ip && routerOnline === true" class="md-wake-btn" @click.stop="doWake(w)">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12 22c1.1 0 2-.9 2-2h-4c0 1.1.89 2 2 2zm6-6v-5c0-3.07-1.64-5.64-4.5-6.32V4c0-.83-.67-1.5-1.5-1.5s-1.5.67-1.5 1.5v.68C7.63 5.36 6 7.92 6 11v5l-2 2v1h16v-1l-2-2z"/></svg>
              {{ tt('wake') }}
            </button>
          </div>
          <!-- Expanded Stats -->
          <div v-if="expandedDeviceIdx === i && w.status.online && routerOnline === true" class="md-stats-area">
            <!-- Loading -->
            <div v-if="statsHistory[w.ip].length === 0" class="md-stats-loading">
              <i class="pi pi-spin pi-spinner" style="font-size:1.2rem;color:var(--md-muted)" />
              <span>{{ tt('loadingStats') }}</span>
            </div>
            <template v-else-if="statsHistory[w.ip].length > 0">
              <template v-if="statsRaw[w.ip]">
                <!-- Section 1: CPU -->
                <div class="md-stats-section">
                  <div class="md-stats-hdr">
                    <span class="md-stats-label">{{ cpuTitle(w.ip) }}</span>
                    <span class="md-stats-legends">
                      <span class="md-stats-legend" style="color:#1976d2">&#9679; CPU {{ curCpuPct(w.ip).toFixed(0) }}%</span>
                    </span>
                  </div>
                  <div class="md-stats-chart-wrap">
                    <svg class="md-stats-svg" viewBox="0 0 320 108" preserveAspectRatio="none">
                      <defs>
                        <linearGradient :id="'g-cpu-'+w.ip" x1="0" y1="0" x2="0" y2="1">
                          <stop offset="0%" stop-color="#1976d2" stop-opacity="0.30"/>
                          <stop offset="100%" stop-color="#1976d2" stop-opacity="0.02"/>
                        </linearGradient>
                      </defs>
                      <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsAreaPath(statsHistory[w.ip].map(s => s.cpu), cpuChartMax(w.ip), 88, 320)" :fill="'url(#g-cpu-'+w.ip+')'"/>
                      <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsPath(statsHistory[w.ip].map(s => s.cpu), cpuChartMax(w.ip), 88, 320)" fill="none" stroke="#1976d2" stroke-width="1.5"/>
                      <line x1="0" y1="88" x2="320" y2="88" stroke="var(--md-border)" stroke-width="0.5"/>
                    </svg>
                    <span class="md-chart-label md-chart-y-max">{{ cpuChartMax(w.ip).toFixed(0) }}%</span>

                    <span class="md-chart-label md-chart-x-start">{{ statsXLabel(statsHistory[w.ip].length, 0) }}</span>
                    <span class="md-chart-label md-chart-x-mid">{{ statsXLabel(statsHistory[w.ip].length, Math.floor((statsHistory[w.ip].length-1)/2)) }}</span>
                    <span class="md-chart-label md-chart-x-end">{{ statsXLabel(statsHistory[w.ip].length, statsHistory[w.ip].length-1) }}</span>
                  </div>
                </div>

                <!-- Section 2: Memory -->
                <div class="md-stats-section">
                  <div class="md-stats-hdr">
                    <span class="md-stats-label">{{ memLabel(w.ip) }}</span>
                    <span class="md-stats-legends">
                      <span class="md-stats-legend" style="color:#e57373">&#9679; MEM {{ memUsed(w.ip) }}G / {{ memTotal(w.ip) }}G</span>
                    </span>
                  </div>
                  <div class="md-stats-chart-wrap">
                    <svg class="md-stats-svg" viewBox="0 0 320 108" preserveAspectRatio="none">
                      <defs>
                        <linearGradient :id="'g-mem-'+w.ip" x1="0" y1="0" x2="0" y2="1">
                          <stop offset="0%" stop-color="#e57373" stop-opacity="0.30"/>
                          <stop offset="100%" stop-color="#e57373" stop-opacity="0.02"/>
                        </linearGradient>
                      </defs>
                      <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsAreaPath(memGBHistory(w.ip), memChartMax(w.ip), 88, 320)" :fill="'url(#g-mem-'+w.ip+')'"/>
                      <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsPath(memGBHistory(w.ip), memChartMax(w.ip), 88, 320)" fill="none" stroke="#e57373" stroke-width="1.5"/>
                      <line x1="0" y1="88" x2="320" y2="88" stroke="var(--md-border)" stroke-width="0.5"/>
                    </svg>
                    <span class="md-chart-label md-chart-y-max">{{ memChartMax(w.ip).toFixed(1) }}G</span>

                    <span class="md-chart-label md-chart-x-start">{{ statsXLabel(statsHistory[w.ip].length, 0) }}</span>
                    <span class="md-chart-label md-chart-x-mid">{{ statsXLabel(statsHistory[w.ip].length, Math.floor((statsHistory[w.ip].length-1)/2)) }}</span>
                    <span class="md-chart-label md-chart-x-end">{{ statsXLabel(statsHistory[w.ip].length, statsHistory[w.ip].length-1) }}</span>
                  </div>
                </div>

                <!-- Section 3: GPU -->
                <div v-if="hasGPU(w.ip)" class="md-stats-section">
                  <div class="md-stats-hdr">
                    <span class="md-stats-label">{{ gpuModel(w.ip) }}</span>
                    <span class="md-stats-legends">
                      <span class="md-stats-legend" style="color:#7c4dff">&#9679; GPU {{ curGpuPct(w.ip).toFixed(0) }}%</span>
                    </span>
                  </div>
                  <div class="md-stats-chart-wrap">
                    <svg class="md-stats-svg" viewBox="0 0 320 108" preserveAspectRatio="none">
                      <defs>
                        <linearGradient :id="'g-gpu-'+w.ip" x1="0" y1="0" x2="0" y2="1">
                          <stop offset="0%" stop-color="#7c4dff" stop-opacity="0.30"/>
                          <stop offset="100%" stop-color="#7c4dff" stop-opacity="0.02"/>
                        </linearGradient>
                      </defs>
                      <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsAreaPath(statsHistory[w.ip].map(s => s.gpu ?? 0), gpuChartMax(w.ip), 88, 320)" :fill="'url(#g-gpu-'+w.ip+')'"/>
                      <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsPath(statsHistory[w.ip].map(s => s.gpu ?? 0), gpuChartMax(w.ip), 88, 320)" fill="none" stroke="#7c4dff" stroke-width="1.5"/>
                      <line x1="0" y1="88" x2="320" y2="88" stroke="var(--md-border)" stroke-width="0.5"/>
                    </svg>
                    <span class="md-chart-label md-chart-y-max">{{ gpuChartMax(w.ip).toFixed(0) }}%</span>

                    <span class="md-chart-label md-chart-x-start">{{ statsXLabel(statsHistory[w.ip].length, 0) }}</span>
                    <span class="md-chart-label md-chart-x-mid">{{ statsXLabel(statsHistory[w.ip].length, Math.floor((statsHistory[w.ip].length-1)/2)) }}</span>
                    <span class="md-chart-label md-chart-x-end">{{ statsXLabel(statsHistory[w.ip].length, statsHistory[w.ip].length-1) }}</span>
                  </div>
                </div>

                <!-- Section 4: Disks -->
                <template v-if="diskList(w.ip).length">
                  <div v-for="(dk, di) in diskList(w.ip)" :key="'dk'+di" class="md-stats-section">
                    <div class="md-stats-hdr">
                      <span class="md-stats-label"><template v-if="dk.model">{{ dk.model }}</template><template v-else>{{ dk.name }}</template></span>
                      <span class="md-stats-legends">
                        <span class="md-stats-legend" style="color:#43a047">&#9679; R {{ formatSpeedYStats(curDiskSpeed(w.ip, dk.name).r) }}</span>
                        <span class="md-stats-legend" style="color:#ff9800">&#9679; W {{ formatSpeedYStats(curDiskSpeed(w.ip, dk.name).w) }}</span>
                      </span>
                    </div>
                    <div class="md-stats-chart-wrap">
                      <svg class="md-stats-svg" viewBox="0 0 320 108" preserveAspectRatio="none">
                        <defs>
                          <linearGradient :id="'g-dr-'+w.ip+'-'+di" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0%" stop-color="#43a047" stop-opacity="0.30"/>
                            <stop offset="100%" stop-color="#43a047" stop-opacity="0.02"/>
                          </linearGradient>
                          <linearGradient :id="'g-dw-'+w.ip+'-'+di" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0%" stop-color="#ff9800" stop-opacity="0.30"/>
                            <stop offset="100%" stop-color="#ff9800" stop-opacity="0.02"/>
                          </linearGradient>
                        </defs>
                        <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsAreaPath(diskHistory(w.ip, dk.name, 'readBps'), diskChartMax(w.ip, dk.name), 88, 320)" :fill="'url(#g-dr-'+w.ip+'-'+di+')'"/>
                        <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsAreaPath(diskHistory(w.ip, dk.name, 'writeBps'), diskChartMax(w.ip, dk.name), 88, 320)" :fill="'url(#g-dw-'+w.ip+'-'+di+')'"/>
                        <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsPath(diskHistory(w.ip, dk.name, 'readBps'), diskChartMax(w.ip, dk.name), 88, 320)" fill="none" stroke="#43a047" stroke-width="1.5"/>
                        <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsPath(diskHistory(w.ip, dk.name, 'writeBps'), diskChartMax(w.ip, dk.name), 88, 320)" fill="none" stroke="#ff9800" stroke-width="1.5"/>
                        <line x1="0" y1="88" x2="320" y2="88" stroke="var(--md-border)" stroke-width="0.5"/>
                        </svg>
                      <span class="md-chart-label md-chart-y-max">{{ formatSpeedYStats(diskChartMax(w.ip, dk.name)) }}</span>
  
                      <span class="md-chart-label md-chart-x-start">{{ statsXLabel(statsHistory[w.ip].length, 0) }}</span>
                      <span class="md-chart-label md-chart-x-mid">{{ statsXLabel(statsHistory[w.ip].length, Math.floor((statsHistory[w.ip].length-1)/2)) }}</span>
                      <span class="md-chart-label md-chart-x-end">{{ statsXLabel(statsHistory[w.ip].length, statsHistory[w.ip].length-1) }}</span>
                    </div>
                    <!-- Disk total row (only when multiple partitions) -->
                    <div v-if="dk.parts.length > 1" class="md-disk-total">
                      <div class="md-part-fill-bg" :style="{ width: dkTotalPct(w.ip, dk) + '%', background: diskFillColor(dkTotalPct(w.ip, dk)) }" />
                      <span class="md-part-label">{{ tt('disk') }}</span>
                      <span class="md-disk-total-usage">{{ dk.used }}GB / {{ dk.total }}GB</span>
                    </div>
                    <!-- Partition rows with background progress -->
                    <div v-for="p in dk.parts" :key="p.name" class="md-part-row">
                      <div class="md-part-fill-bg" :style="{ width: p.pct + '%', background: diskFillColor(p.pct) }" />
                      <span class="md-part-label">{{ p.name }}</span>
                      <span class="md-part-usage">{{ p.used }}GB / {{ p.total }}GB</span>
                    </div>
                  </div>
                </template>

                <!-- Section 5: Network -->
                <template v-if="netList(w.ip).length">
                  <div v-for="(nc, ni) in netList(w.ip)" :key="'nc'+ni" class="md-stats-section">
                    <div class="md-stats-hdr">
                      <span class="md-stats-label">{{ nc.desc }}</span>
                      <span class="md-stats-legends">
                        <span class="md-stats-legend" style="color:#43a047">&#9679; &#8593; {{ formatSpeedBps(curNetSpeed(w.ip, nc.name).s) }}</span>
                        <span class="md-stats-legend" style="color:#1e88e5">&#9679; &#8595; {{ formatSpeedBps(curNetSpeed(w.ip, nc.name).r) }}</span>
                      </span>
                    </div>
                    <div class="md-stats-chart-wrap">
                      <svg class="md-stats-svg" viewBox="0 0 320 108" preserveAspectRatio="none">
                        <defs>
                          <linearGradient :id="'g-nr-'+w.ip+'-'+ni" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0%" stop-color="#1e88e5" stop-opacity="0.30"/>
                            <stop offset="100%" stop-color="#1e88e5" stop-opacity="0.02"/>
                          </linearGradient>
                          <linearGradient :id="'g-ns-'+w.ip+'-'+ni" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="0%" stop-color="#43a047" stop-opacity="0.30"/>
                            <stop offset="100%" stop-color="#43a047" stop-opacity="0.02"/>
                          </linearGradient>
                        </defs>
                        <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsAreaPath(netHistory(w.ip, nc.name, 'recvBps'), netChartMax(w.ip, nc.name), 88, 320)" :fill="'url(#g-nr-'+w.ip+'-'+ni+')'"/>
                        <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsAreaPath(netHistory(w.ip, nc.name, 'sentBps'), netChartMax(w.ip, nc.name), 88, 320)" :fill="'url(#g-ns-'+w.ip+'-'+ni+')'"/>
                        <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsPath(netHistory(w.ip, nc.name, 'recvBps'), netChartMax(w.ip, nc.name), 88, 320)" fill="none" stroke="#1e88e5" stroke-width="1.5"/>
                        <path v-if="statsHistory[w.ip].length > 1" :d="buildStatsPath(netHistory(w.ip, nc.name, 'sentBps'), netChartMax(w.ip, nc.name), 88, 320)" fill="none" stroke="#43a047" stroke-width="1.5"/>
                        <line x1="0" y1="88" x2="320" y2="88" stroke="var(--md-border)" stroke-width="0.5"/>
                        </svg>
                      <span class="md-chart-label md-chart-y-max">{{ formatSpeedBps(netChartMax(w.ip, nc.name)) }}</span>
  
                      <span class="md-chart-label md-chart-x-start">{{ statsXLabel(statsHistory[w.ip].length, 0) }}</span>
                      <span class="md-chart-label md-chart-x-mid">{{ statsXLabel(statsHistory[w.ip].length, Math.floor((statsHistory[w.ip].length-1)/2)) }}</span>
                      <span class="md-chart-label md-chart-x-end">{{ statsXLabel(statsHistory[w.ip].length, statsHistory[w.ip].length-1) }}</span>
                    </div>
                  </div>
                </template>
              </template>
            </template>
            <!-- Device info row (bottom) -->
            <div class="md-stats-info">{{ w.mac }} &middot; {{ w.router_ip }} &middot; {{ w.interface || 'br-lan' }} &middot; {{ w.agent_port || 32249 }}</div>
          </div>
          <!-- Error message when not expanded -->
          <div v-if="w.error && expandedDeviceIdx !== i" class="md-extra">
            <div v-if="w.error" class="mt-1" style="color:#ea4335;word-break:break-all">{{ w.error }}</div>
          </div>
        </div>
      </div>
    </template>

    <!-- ==================== Network Tab ==================== -->
    <template v-if="activeTab === 'net'">
      <div class="md-hdr">
        <div class="flex-1 flex items-center gap-2 min-w-0">
          <span class="md-hdr-title truncate">{{ netName() }}</span>
          <span v-if="netConnecting" class="md-dot md-dot-on flex-shrink-0 ml-2 animate-pulse" />
          <span v-if="netConnecting" class="md-connected-text">{{ tt('connecting') }}</span>
          <template v-else-if="netRunning && (netPeers.length || netServers.length)">
            <span class="md-dot md-dot-on flex-shrink-0 ml-2" />
            <span class="md-connected-text">{{ tt('connected') }}</span>
          </template>
          <template v-else-if="netRunning">
            <span class="md-dot md-dot-on flex-shrink-0 ml-2 animate-pulse" />
            <span class="md-connected-text">{{ netDiscovering ? tt('discoveringPeers') : tt('waitingForPeers') }}</span>
          </template>
        </div>
        <button v-if="!netRunning" class="md-hdr-btn" @click="openNetEditor" :title="tt('editConfig')">
          <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M714.965 128l-85.333 85.333H213.333v597.334h597.334V394.368L896 309.035v544.298A42.667 42.667 0 0 1 853.333 896H170.667A42.667 42.667 0 0 1 128 853.333V170.667A42.667 42.667 0 0 1 170.667 128h544.298z m159.062-38.4l60.373 60.416-392.192 392.192-60.245 0.128-0.086-60.459L874.027 89.6z"/></svg>
        </button>
        <div style="position:relative">
          <button v-if="netRunning" class="md-hdr-btn md-hdr-btn-off" @click="doDisconnect" :title="tt('disconnect')">
            <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M797.248 111.936a210.944 210.944 0 0 1 114.816 114.816c10.624 25.728 16 53.312 15.936 81.216a210.624 210.624 0 0 1-61.888 150.016L744.32 580.48a11.264 11.264 0 0 1-8.256 3.328 11.2 11.2 0 0 1-8.256-3.328l-56.768-56.768a11.264 11.264 0 0 1-3.392-8.32 11.264 11.264 0 0 1 3.392-8.192l122.24-122.24a109.12 109.12 0 0 0-118.848-177.792 108.288 108.288 0 0 0-35.392 23.68L517.12 353.088a11.392 11.392 0 0 1-8.256 3.328 11.52 11.52 0 0 1-8.256-3.328l-56.768-56.832a11.456 11.456 0 0 1 0-16.128l122.176-122.24a210.88 210.88 0 0 1 150.016-61.888c27.84-0.128 55.488 5.312 81.28 15.936zM299.584 217.664a31.04 31.04 0 0 0-44.224 0l-36.224 36.096a31.168 31.168 0 0 0-6.784 34.24 31.36 31.36 0 0 0 6.784 10.24l488.384 488.32a30.976 30.976 0 0 0 34.24 6.848 30.912 30.912 0 0 0 10.176-6.848l36.16-36.16a31.36 31.36 0 0 0 0-44.416l-488.512-488.32z m224 451.136a11.2 11.2 0 0 0-8.32-3.392 11.328 11.328 0 0 0-8.192 3.392l-122.24 122.24a108.352 108.352 0 0 1-76.992 31.808 108.16 108.16 0 0 1-77.12-31.872 109.184 109.184 0 0 1 0-154.176l122.688-122.048a11.392 11.392 0 0 0 3.328-8.32 11.52 11.52 0 0 0-3.328-8.192L296.576 441.6a11.52 11.52 0 0 0-8.256-3.328 11.392 11.392 0 0 0-8.192 3.328l-122.24 122.24a210.752 210.752 0 0 0-61.888 150.016 210.496 210.496 0 0 0 61.888 150.016 210.368 210.368 0 0 0 150.016 61.888 210.752 210.752 0 0 0 150.08-61.888l122.176-122.24a11.52 11.52 0 0 0 0-16.192l-56.576-56.704z"/></svg>
          </button>
          <button v-if="!netRunning" class="md-hdr-btn md-switch-btn" @click.stop="showSwitchMenu = !showSwitchMenu" :title="tt('switchNetwork')">
            <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M900.4 424.87c19.47 0 37.03-11.73 44.49-29.73 7.46-17.98 3.33-38.7-10.43-52.48L713.97 122.19c-7.3-7.3-19.12-7.3-26.42 0l-41.69 41.69c-7.3 7.3-7.3 19.13 0 26.42l138.28 138.27H86.32c-10.19 0-18.46 8.26-18.46 18.46v59.39c0 10.19 8.26 18.46 18.46 18.46H900.4zM937.65 598.72H123.8c-19.47 0-37.03 11.73-44.49 29.73-7.46 17.98-3.33 38.7 10.43 52.48l220.49 220.48c7.3 7.3 19.12 7.3 26.42 0l41.69-41.69c7.3-7.3 7.3-19.13 0-26.42L240.06 695.02h697.59c10.32 0 18.68-8.37 18.68-18.68v-58.93c0-10.32-8.36-18.69-18.68-18.69z"/></svg>
          </button>
          <div v-if="showSwitchMenu" class="md-switch-menu md-card" @click.stop>
            <div class="md-sw-item" @click="openAddNet">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="var(--md-primary)"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/></svg>
              <span style="color:var(--md-primary);font-weight:500">{{ tt('addNetwork') }}</span>
            </div>
            <div class="md-sw-divider" />
            <div v-for="(n, i) in networks" :key="n.config.instance_id" class="md-sw-item" @click="switchToNet(i)">
              <span v-if="i === netIndex" class="md-sw-check">&#10003;</span>
              <span v-else style="width:18px;display:inline-block" />
              <span class="flex-1 truncate">{{ n.config.instance_name || n.config.network_name || tt('unnamed') }}</span>
              <button class="md-sw-del" @click="confirmDeleteNet(i, $event)">&times;</button>
            </div>
          </div>
        </div>
      </div>

      <!-- No networks configured -->
      <div v-if="!networks.length && !netRunning && !netConnecting" class="flex-1 flex items-center justify-center">
        <div class="flex flex-col items-center gap-3 opacity-50">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="var(--md-muted)" stroke-width="1.2"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>
          <div class="text-base" style="color:var(--md-muted)">{{ tt('noNetworks') }}</div>
          <div class="text-sm text-center px-4" style="color:var(--md-muted)">{{ tt('tapToSwitch') }}</div>
        </div>
      </div>

      <!-- Disconnected (has networks but not running) -->
      <div v-if="networks.length && !netRunning && !netConnecting" class="flex-1 flex items-center justify-center">
        <button class="md-connect-btn" @click="doConnect" :disabled="netConnecting">
          <svg width="32" height="32" viewBox="0 0 1024 1024" fill="currentColor"><path d="M594.6 716.6a10.68 10.68 0 0 0-15 0l-154.9 154.9c-71.75 71.67-192.79 79.35-272 0-79.35-79.35-71.67-200.25 0-272l154.9-154.9a10.68 10.68 0 0 0 0-15.07l-53.02-53.1a10.68 10.68 0 0 0-15.07 0L84.62 531.41a288.23 288.23 0 0 0 0 407.96 288.3 288.3 0 0 0 407.96 0l154.9-154.9a10.68 10.68 0 0 0 0-15.07l-52.8-52.8z m344.84-631.97a288.16 288.16 0 0 0-407.96 0l-155.05 154.9a10.68 10.68 0 0 0 0 15.07l52.95 52.95a10.68 10.68 0 0 0 15 0l154.97-154.98c71.67-71.67 192.79-79.28 271.92 0 79.35 79.35 71.75 200.32 0 272.07L716.37 579.46a10.68 10.68 0 0 0 0 15.07l53.1 53.02a10.68 10.68 0 0 0 15.07 0l154.9-154.9a288.67 288.67 0 0 0 0-408.1zM642.8 325.82a10.68 10.68 0 0 0-15.07 0L325.75 627.66a10.68 10.68 0 0 0 0 15.07l52.8 52.8a10.68 10.68 0 0 0 15.07 0l301.84-301.9a10.68 10.68 0 0 0 0-15.07l-52.66-52.73z"/></svg>
          <span>{{ tt('connect') }}</span>
        </button>
      </div>

      <!-- Connecting -->
      <div v-if="netConnecting && !netRunning" class="flex-1 flex items-center justify-center">
        <button class="md-connect-btn md-connect-btn-loading" disabled>
          <svg class="animate-spin" width="32" height="32" viewBox="0 0 24 24" fill="currentColor"><path d="M12 4V2A10 10 0 0 0 2 12h2a8 8 0 0 1 8-8z"/></svg>
          <span>{{ tt('connecting') }}</span>
        </button>
      </div>

      <!-- Connected -->
      <template v-if="netRunning">
        <div v-if="netSelfIp || natLabel(netSelfNat).text" class="md-conn md-conn-on">
          <div class="flex-1 md-conn-ips">
            <div>{{ netSelfIpv6 ? netSelfIp + '\n' + netSelfIpv6 : netSelfIp }}</div>
          </div>
          <span v-if="natLabel(netSelfNat).text" class="md-chip md-chip-nat" :class="natLabel(netSelfNat).cls">{{ natLabel(netSelfNat).text }}</span>
        </div>
        <div class="flex-1 overflow-y-auto px-3 pt-3 pb-2">
          <!-- Traffic Chart -->
          <div v-if="trafficHistory.length > 1" class="md-card md-traffic-card">
            <div class="md-traffic-hdr">
              <div class="flex-1">
                <span class="md-traffic-label" :style="{color:TRAFFIC_UP_COLOR}">&#8593; {{ netTrafficSpeed.up || '--' }}</span>
                <span class="md-traffic-label" :style="{color:TRAFFIC_DOWN_COLOR}">&#8595; {{ netTrafficSpeed.down || '--' }}</span>
              </div>
              <div class="md-traffic-total">
                <span class="md-traffic-label" :style="{color:TRAFFIC_UP_COLOR}">&#8593; {{ netTrafficSpeed.upTotal }}</span>
                <span class="md-traffic-label" :style="{color:TRAFFIC_DOWN_COLOR}">&#8595; {{ netTrafficSpeed.downTotal }}</span>
              </div>
            </div>
            <div class="md-traffic-chart-wrap">
              <svg class="md-traffic-svg" viewBox="0 0 320 108" preserveAspectRatio="none">
                <defs>
                  <linearGradient id="g-traffic-up" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="#43a047" stop-opacity="0.30"/>
                    <stop offset="100%" stop-color="#43a047" stop-opacity="0.02"/>
                  </linearGradient>
                  <linearGradient id="g-traffic-down" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="0%" stop-color="#1e88e5" stop-opacity="0.30"/>
                    <stop offset="100%" stop-color="#1e88e5" stop-opacity="0.02"/>
                  </linearGradient>
                </defs>
                <!-- Up area -->
                <path v-if="trafficHistory.length > 1" :d="buildStatsAreaPath(trafficHistory.map(p => p.up), Math.max(1, ...trafficHistory.map(p => Math.max(p.up, p.down))), 88, 320)" fill="url(#g-traffic-up)"/>
                <!-- Down area -->
                <path v-if="trafficHistory.length > 1" :d="buildStatsAreaPath(trafficHistory.map(p => p.down), Math.max(1, ...trafficHistory.map(p => Math.max(p.up, p.down))), 88, 320)" fill="url(#g-traffic-down)"/>
                <!-- Up line -->
                <path v-if="trafficHistory.length > 1" :d="buildStatsPath(trafficHistory.map(p => p.up), Math.max(1, ...trafficHistory.map(p => Math.max(p.up, p.down))), 88, 320)" fill="none" :stroke="TRAFFIC_UP_COLOR" stroke-width="1.5"/>
                <!-- Down line -->
                <path v-if="trafficHistory.length > 1" :d="buildStatsPath(trafficHistory.map(p => p.down), Math.max(1, ...trafficHistory.map(p => Math.max(p.up, p.down))), 88, 320)" fill="none" :stroke="TRAFFIC_DOWN_COLOR" stroke-width="1.5"/>
                <line x1="0" y1="88" x2="320" y2="88" stroke="var(--md-border)" stroke-width="0.5"/>
              </svg>
              <span class="md-chart-label md-chart-y-max">{{ (() => { const m = Math.max(1, ...trafficHistory.map(p => Math.max(p.up, p.down))); return formatSpeedY(m); })() }}</span>
              <span class="md-chart-label md-chart-x-start">{{ formatXLabel(trafficHistory.length, 0) }}</span>
              <span class="md-chart-label md-chart-x-mid">{{ formatXLabel(trafficHistory.length, Math.floor(trafficHistory.length/2)) }}</span>
              <span class="md-chart-label md-chart-x-end">{{ formatXLabel(trafficHistory.length, trafficHistory.length-1) }}</span>
            </div>
          </div>

          <!-- Network Devices -->
          <div v-if="netPeers.length" class="md-section">
            <div class="md-sec-hdr"><span class="md-sec-title">{{ tt('networkDevices') }}</span><span class="md-badge">{{ netPeers.length }}</span></div>
            <div v-for="p in sortedPeers" :key="peerKey(p)" class="md-card md-card-item" @click="netExpanded[peerKey(p)]=!netExpanded[peerKey(p)]">
              <div class="md-row md-row-top">
                <div class="flex-1 min-w-0 flex items-center gap-2">
                  <span class="md-name">{{ formatIp(p.route?.ipv4_addr) }}</span>
                  <span v-if="p.route?.hostname" class="md-sub" style="margin-top:0">{{ p.route.hostname }}</span>
                </div>
                <span v-if="buildDetail(p).lat" class="md-quality" :class="qualityCls(buildDetail(p).lat, buildDetail(p).loss)">{{ qualityText(buildDetail(p).lat, buildDetail(p).loss) }}</span>
              </div>
              <div class="md-row md-row-bot">
                <div class="md-sub flex-1">
                  <template v-if="buildDetail(p).up || buildDetail(p).down">
                    <span v-if="buildDetail(p).up" class="md-traffic-up-sub">{{ buildDetail(p).up }}</span><template v-if="buildDetail(p).up && buildDetail(p).down"> &middot; </template><span v-if="buildDetail(p).down" class="md-traffic-down-sub">{{ buildDetail(p).down }}</span>
                  </template>
                </div>
                <div class="flex items-center gap-1 flex-shrink-0">
                  <span class="md-rt" :class="costCls(p.route?.cost)">{{ costLabel(p.route?.cost) }}</span>
                  <span v-if="buildDetail(p).tun" class="md-chip">{{ buildDetail(p).tun }}</span>
                  <span v-if="natLabel(p.route?.stun_info).text" class="md-chip md-chip-nat" :class="natLabel(p.route?.stun_info).cls">{{ natLabel(p.route?.stun_info).text }}</span>
                </div>
              </div>
              <div v-if="netExpanded[peerKey(p)]" class="md-extra"><template v-if="buildDetail(p).ver"><div>{{ tt('version') }}: {{ buildDetail(p).ver }}</div></template></div>
            </div>
          </div>

          <!-- Servers -->
          <div v-if="netServers.length" class="md-section">
            <div class="md-sec-hdr"><span class="md-sec-title">{{ tt('servers') }}</span><span class="md-badge">{{ netServers.length }}</span></div>
            <div v-for="p in sortedServers" :key="peerKey(p)" class="md-card md-card-item" @click="netExpanded[peerKey(p)]=!netExpanded[peerKey(p)]">
              <div class="md-row md-row-top">
                <div class="flex-1 min-w-0 flex items-center gap-2">
                  <span class="md-name truncate">{{ p.route?.hostname || tt('server') }}</span>
                </div>
                <span v-if="buildDetail(p).lat" class="md-quality" :class="qualityCls(buildDetail(p).lat, buildDetail(p).loss)">{{ qualityText(buildDetail(p).lat, buildDetail(p).loss) }}</span>
              </div>
              <div class="md-row md-row-bot">
                <div class="md-sub flex-1">
                  <template v-if="buildDetail(p).up"><span class="md-traffic-up-sub">{{ buildDetail(p).up }}</span></template>
                  <template v-if="buildDetail(p).down"> &middot; <span class="md-traffic-down-sub">{{ buildDetail(p).down }}</span></template>
                </div>
                <div class="flex items-center gap-1 flex-shrink-0">
                  <span class="md-rt" :class="costCls(p.route?.cost)">{{ costLabel(p.route?.cost) }}</span>
                  <span v-if="buildDetail(p).tun" class="md-chip">{{ buildDetail(p).tun }}</span>
                  <span v-if="natLabel(p.route?.stun_info).text" class="md-chip md-chip-nat" :class="natLabel(p.route?.stun_info).cls">{{ natLabel(p.route?.stun_info).text }}</span>
                </div>
              </div>
              <div v-if="netExpanded[peerKey(p)]" class="md-extra"><template v-if="buildDetail(p).ver"><div>{{ tt('version') }}: {{ buildDetail(p).ver }}</div></template></div>
            </div>
          </div>

          <div v-if="!netPeers.length && !netServers.length" class="flex flex-col items-center justify-center py-12 gap-3 opacity-50">
            <template v-if="netDiscovering || netLoading"><i class="pi pi-spin pi-spinner text-xl" style="color:var(--md-muted)" /><div class="text-base" style="color:var(--md-muted)">{{ tt('discoveringPeers') }}</div></template>
            <template v-else><div class="text-base" style="color:var(--md-muted)">{{ tt('noPeersConnected') }}</div></template>
          </div>
        </div>
      </template>
    </template>

    <!-- ==================== Router Tab ==================== -->
    <div v-show="activeTab === 'luci'" class="flex flex-col flex-1 min-h-0">
      <div class="md-hdr">
        <span class="md-hdr-title">{{ currentLuciRouter?.ip || tt('routers') }}</span>
        <span v-if="currentLuciRouter && getWolPathType() === 'tunnel'" class="md-path-badge">{{ tt('etLan') }}</span>
        <span v-else-if="currentLuciRouter && getWolPathType() === 'lan'" class="md-path-badge md-path-lan">{{ tt('lanDirect') }}</span>
        <div class="flex-1" />
        <button class="md-hdr-btn" @click="openLuciEditorFn" :title="tt('editConfig')">
          <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M714.965 128l-85.333 85.333H213.333v597.334h597.334V394.368L896 309.035v544.298A42.667 42.667 0 0 1 853.333 896H170.667A42.667 42.667 0 0 1 128 853.333V170.667A42.667 42.667 0 0 1 170.667 128h544.298z m159.062-38.4l60.373 60.416-392.192 392.192-60.245 0.128-0.086-60.459L874.027 89.6z"/></svg>
        </button>
        <button v-if="proxyUrl" class="md-hdr-btn" @click="refreshLuciIframe" :title="tt('refresh')">
          <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M128 170.666667l92.330667 92.330666A382.378667 382.378667 0 0 1 512 128c211.754667 0 384 172.288 384 384h-85.333333c0-164.693333-134.016-298.666667-298.666667-298.666667a297.557333 297.557333 0 0 0-231.125333 110.208L384 426.666667H128V170.666667z m768 682.666666v-256h-256l103.125333 103.125334A297.514667 297.514667 0 0 1 512 810.666667c-164.650667 0-298.666667-134.016-298.666667-298.666667H128c0 211.754667 172.245333 384 384 384a382.378667 382.378667 0 0 0 291.669333-134.997333L896 853.333333z"/></svg>
        </button>
        <div v-if="luciRouters.length" style="position:relative">
          <button class="md-hdr-btn md-switch-btn" @click.stop="showLuciMenu = !showLuciMenu" :title="tt('switchNetwork')">
            <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M900.4 424.87c19.47 0 37.03-11.73 44.49-29.73 7.46-17.98 3.33-38.7-10.43-52.48L713.97 122.19c-7.3-7.3-19.12-7.3-26.42 0l-41.69 41.69c-7.3 7.3-7.3 19.13 0 26.42l138.28 138.27H86.32c-10.19 0-18.46 8.26-18.46 18.46v59.39c0 10.19 8.26 18.46 18.46 18.46H900.4zM937.65 598.72H123.8c-19.47 0-37.03 11.73-44.49 29.73-7.46 17.98-3.33 38.7 10.43 52.48l220.49 220.48c7.3 7.3 19.12 7.3 26.42 0l41.69-41.69c7.3-7.3 7.3-19.13 0-26.42L240.06 695.02h697.59c10.32 0 18.68-8.37 18.68-18.68v-58.93c0-10.32-8.36-18.69-18.68-18.69z"/></svg>
          </button>
          <div v-if="showLuciMenu" class="md-switch-menu md-card" @click.stop>
            <div class="md-sw-item" @click="openLuciEditorFn">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="var(--md-primary)"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/></svg>
              <span style="color:var(--md-primary);font-weight:500">{{ tt('addRouter') }}</span>
            </div>
            <div class="md-sw-divider" />
            <div v-for="(r, i) in luciRouters" :key="r.ip" class="md-sw-item" @click="switchLuciRouter(i)">
              <span v-if="i === luciIdx" class="md-sw-check">&#10003;</span>
              <span v-else style="width:18px;display:inline-block" />
              <span class="flex-1 truncate" :style="i === luciIdx ? 'color:var(--md-primary);font-weight:500' : ''">{{ r.ip }}</span>
              <button class="md-sw-del" @click="deleteLuciRouter(i, $event)">&times;</button>
            </div>
          </div>
        </div>
      </div>
      <div class="flex-1 overflow-hidden">
        <div v-if="!luciRouters.length" class="flex flex-col items-center justify-center h-full gap-3 opacity-50">
          <svg width="48" height="48" viewBox="0 0 1024 1024" fill="none" stroke="var(--md-muted)" stroke-width="20"><path d="M881.777778 597.333333h-56.888889V199.111111c0-17.066667-11.377778-28.444444-28.444445-28.444444s-28.444444 11.377778-28.444444 28.444444v398.222222h-227.555556V199.111111c0-17.066667-11.377778-28.444444-28.444444-28.444444s-28.444444 11.377778-28.444444 28.444444v398.222222H256V199.111111c0-17.066667-11.377778-28.444444-28.444444-28.444444s-28.444444 11.377778-28.444445 28.444444v398.222222H142.222222c-31.288889 0-56.888889 25.6-56.888889 56.888889v142.222222c0 31.288889 25.6 56.888889 56.888889 56.888889h739.555556c31.288889 0 56.888889-25.6 56.888889-56.888889v-142.222222c0-31.288889-25.6-56.888889-56.888889-56.888889z"/></svg>
          <div class="text-base" style="color:var(--md-muted)">{{ tt('noRouters') }}</div>
          <div class="text-sm text-center px-4" style="color:var(--md-muted)">{{ tt('tapToAdd') }}</div>
        </div>
        <iframe v-else-if="luciIframeSrc" :src="luciIframeSrc" :key="luciIframeKey" class="luci-iframe" />
        <div v-else class="flex flex-col items-center justify-center h-full gap-3">
          <svg class="animate-spin" width="32" height="32" viewBox="0 0 24 24" fill="var(--md-muted)"><path d="M12 4V2A10 10 0 0 0 2 12h2a8 8 0 0 1 8-8z"/></svg>
          <div class="text-base" style="color:var(--md-muted)">{{ tt('connectingRouter') }}</div>
        </div>
      </div>
    </div>

    <!-- ==================== Settings Tab ==================== -->
    <template v-if="activeTab === 'settings'">
      <div class="md-hdr">
        <span class="md-hdr-title">{{ tt('settings') }}</span>
        <div class="flex-1" />
        <div style="width:36px" />
      </div>
      <div class="flex-1 overflow-y-auto px-3 py-1">
        <div class="md-card md-settings-card">
          <div class="md-row">
            <svg width="22" height="22" viewBox="0 0 24 24" :fill="settingsItemColors.defaultHome"><path d="M3 13h8V3H3v10zm0 8h8v-6H3v6zm10 0h8V11h-8v10zm0-18v6h8V3h-8z"/></svg>
            <div class="flex-1">
              <div class="md-name">{{ tt('defaultHome') }}</div>
              <div class="md-sub">{{ tt('defaultHomeDesc') }}</div>
            </div>
            <select class="md-select" :value="defaultTab" @change="setDefaultTab(($event.target as HTMLSelectElement).value as Tab)" @click.stop>
              <option value="wol">{{ tt('wolTab') }}</option>
              <option value="luci">{{ tt('luciTab') }}</option>
              <option value="net">{{ tt('netTab') }}</option>
            </select>
          </div>
        </div>
        <div class="md-card md-settings-card">
          <div class="md-row">
            <svg width="22" height="22" viewBox="0 0 1024 1024" :fill="settingsItemColors.theme"><g transform="translate(100,100) scale(0.8)"><path d="M1012.286427 953.863217l-171.572479-174.157892a41.091581 41.091581 0 0 0-29.209678-12.321974 40.321458 40.321458 0 0 0-28.714599 11.771886 41.036572 41.036572 0 0 0-12.266964 29.209678 41.476643 41.476643 0 0 0 11.771885 29.209678l171.627487 174.157892a41.091581 41.091581 0 0 0 29.209678 12.266965 40.321458 40.321458 0 0 0 28.65959-11.771885 41.091581 41.091581 0 0 0 12.321974-29.209679 42.356784 42.356784 0 0 0-11.826894-29.154669z"/><path d="M935.494128 618.309476a93.019898 93.019898 0 0 0 50.223043-65.570501 90.214448 90.214448 0 0 0-27.174352-75.802141c-48.132709-53.248528-127.015342-136.256822-159.800593-171.077399-6.656066-37.90107-27.504405-157.765267-42.026731-231.532081a90.379475 90.379475 0 0 0-47.637629-65.570502 80.807942 80.807942 0 0 0-37.90107-8.691392 96.650479 96.650479 0 0 0-42.191758 9.681551c-67.605827 31.740083-178.283553 86.033779-213.10413 102.976492-38.946237-4.62074-162.881086-19.473119-237.142979-27.174352a49.507929 49.507929 0 0 0-8.691392-0.495079 85.703726 85.703726 0 0 0-60.949762 25.634105 90.764537 90.764537 0 0 0-26.129185 73.766815c8.691392 74.811982 24.093859 198.746831 28.65959 238.188147-16.887705 35.370665-70.191242 148.523787-101.436245 218.219949a91.864713 91.864713 0 0 0-1.540247 80.91796 83.338347 83.338347 0 0 0 58.914435 43.016889c71.511453 13.807211 197.206584 35.865744 237.14298 43.01689 28.164511 27.174352 117.828871 112.162964 172.617646 162.386007a91.97473 91.97473 0 0 0 62.490008 27.504405 83.723409 83.723409 0 0 0 11.001762-1.045168 86.143796 86.143796 0 0 0 61.994929-48.132708c36.855903-65.570502 97.310585-178.283553 116.288625-213.59921 33.830418-18.978039 140.327474-78.387554 205.897975-116.783703z"/><path d="M331.497394 484.638068a41.091581 41.091581 0 0 0-29.209678-12.321974 42.356784 42.356784 0 0 0-29.209678 11.771886 41.036572 41.036572 0 0 0-12.266965 29.209678 41.476643 41.476643 0 0 0 11.771885 29.209678l25.634106 25.634105a41.091581 41.091581 0 0 0 29.209678 12.321974 42.356784 42.356784 0 0 0 29.209678-11.771885 41.036572 41.036572 0 0 0 12.266965-29.209679 41.476643 41.476643 0 0 0-11.771886-29.209678z"/><path d="M504.11504 362.188457a41.091581 41.091581 0 0 0 29.209678 12.266964 42.411793 42.411793 0 0 0 29.209678-11.771885 41.14659 41.14659 0 0 0 0-57.869268l-25.634106-25.634106a41.036572 41.036572 0 0 0-29.209678-12.321973 42.356784 42.356784 0 0 0-29.209678 11.771885 41.036572 41.036572 0 0 0-12.321973 29.209678 41.476643 41.476643 0 0 0 11.771885 29.209679z"/><path d="M578.927021 547.128075a50.38807 50.38807 0 0 0-8.691392 1.045167 40.596502 40.596502 0 0 0-31.245004 48.682797 52.203361 52.203361 0 0 1-11.001762 42.026731 41.696678 41.696678 0 0 1-31.245004 11.001762 20.628304 20.628304 0 0 1-9.24148-1.045167 56.108986 56.108986 0 0 0-9.24148-1.045168 40.596502 40.596502 0 0 0-39.936396 31.740084 41.036572 41.036572 0 0 0 30.749925 49.177876 106.497056 106.497056 0 0 0 27.174352 3.080493 125.970175 125.970175 0 0 0 90.159439-35.865744 133.726417 133.726417 0 0 0 33.280331-117.278783 42.741845 42.741845 0 0 0-40.486485-31.740083z"/></g></svg>
            <div class="flex-1">
              <div class="md-name">{{ tt('theme') }}</div>
              <div class="md-sub">{{ tt('themeDesc') }}</div>
            </div>
            <select class="md-select" :value="themeMode" @change="themeMode = ($event.target as HTMLSelectElement).value" @click.stop>
              <option value="auto">{{ tt('themeAuto') }}</option>
              <option value="light">{{ tt('themeLight') }}</option>
              <option value="dark">{{ tt('themeDark') }}</option>
            </select>
          </div>
        </div>
        <div class="md-card md-settings-card" @click="amoledMode = !amoledMode">
          <div class="md-row">
            <svg width="22" height="22" viewBox="0 0 24 24" :fill="settingsItemColors.amoled"><path d="M9.37 5.51A7.35 7.35 0 0 0 9.1 7.5c0 4.08 3.32 7.4 7.4 7.4.68 0 1.35-.09 1.99-.27A7.014 7.014 0 0 1 12 19c-3.86 0-7-3.14-7-7 0-2.93 1.81-5.45 4.37-6.49zM12 3a9 9 0 1 0 9 9c0-.46-.04-.92-.1-1.36a5.389 5.389 0 0 1-4.4 2.26 5.403 5.403 0 0 1-3.14-9.8c-.44-.06-.9-.1-1.36-.1z"/></svg>
            <div class="flex-1">
              <div class="md-name">{{ tt('amoled') }}</div>
              <div class="md-sub">{{ tt('amoledDesc') }}</div>
            </div>
            <span class="md-toggle" :class="{ 'md-toggle-on': amoledMode }" />
          </div>
        </div>
        <div class="md-card md-settings-card" @click="toggleLang">
          <div class="md-row">
            <svg width="22" height="22" viewBox="0 0 1024 1024" :fill="settingsItemColors.language"><g transform="translate(100,100) scale(0.9)"><path d="M873.984 102.912c-26.112 0-47.104 20.992-47.104 47.104 0 5.632 1.024 10.752 2.56 15.872h12.8c8.704 0 15.872 7.168 15.872 15.872v12.8c5.12 1.536 10.24 2.56 15.872 2.56 26.112 0 47.104-20.992 47.104-47.104s-20.992-47.104-47.104-47.104z"/><path d="M889.856 102.912H258.56c-17.408 0-31.744 14.336-31.744 31.744s14.336 31.744 31.744 31.744h599.552v599.552c0 17.408 14.336 31.744 31.744 31.744s31.744-14.336 31.744-31.744V134.656c0-17.408-14.336-31.744-31.744-31.744zM394.752 621.568h107.52L448.512 470.528l-53.76 151.04z"/><path d="M747.52 226.816H152.064a49.152 49.152 0 0 0-49.152 49.152v595.456c0 27.136 22.016 49.152 49.152 49.152H747.52c27.136 0 49.152-22.016 49.152-49.152V276.48a49.152 49.152 0 0 0-49.152-49.152z m-184.832 567.808l-31.744-91.136H366.08l-33.28 91.136H252.928l156.16-441.856h78.336l155.648 441.856H563.2z"/></g></svg>
            <div class="flex-1">
              <div class="md-name">{{ tt('language') }}</div>
              <div class="md-sub">{{ tt('languageDesc') }}</div>
            </div>
            <span class="md-lang-badge">{{ locale === 'cn' ? '中文' : 'English' }}</span>
          </div>
        </div>
        <div class="md-card md-settings-card" @click="showDebug = !showDebug">
          <div class="md-row">
            <svg width="22" height="22" viewBox="0 0 24 24" :fill="settingsItemColors.debug"><path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 00.12-.61l-1.92-3.32a.488.488 0 00-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 00-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.07.62-.07.94s.02.64.07.94l-2.03 1.58a.49.49 0 00-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/></svg>
            <div class="flex-1">
              <div class="md-name">{{ tt('debug') }}</div>
              <div class="md-sub">{{ tt('debugDesc') }}</div>
            </div>
            <span class="md-toggle" :class="{ 'md-toggle-on': showDebug }" />
          </div>
        </div>
        <div v-if="showDebug" class="md-card md-settings-card" @click="router.push('/')">
          <div class="md-row">
            <svg width="22" height="22" viewBox="0 0 24 24" :fill="settingsItemColors.advanced"><path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 00.12-.61l-1.92-3.32a.488.488 0 00-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 00-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.07.62-.07.94s.02.64.07.94l-2.03 1.58a.49.49 0 00-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/></svg>
            <div class="flex-1">
              <div class="md-name">{{ tt('advancedSettings') }}</div>
              <div class="md-sub">{{ tt('advancedSettingsDesc') }}</div>
            </div>
            <svg width="20" height="20" viewBox="0 0 24 24" fill="var(--md-muted)" opacity="0.5"><path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6z"/></svg>
          </div>
        </div>
        <div class="md-card md-settings-card">
          <div class="md-row">
            <svg width="22" height="22" viewBox="0 0 24 24" :fill="settingsItemColors.config"><path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM17 13l-5 5-5-5h3V9h4v4h3z"/></svg>
            <div class="flex-1">
              <div class="md-name">{{ tt('oneClickConfig') }}</div>
              <div class="md-sub">{{ tt('oneClickConfigDesc') }}</div>
            </div>
            <button class="md-settings-btn md-settings-btn-export" @click.stop="doExport">{{ tt('exportLabel') }}</button>
            <button class="md-settings-btn md-settings-btn-primary" @click.stop="openImportDlg">{{ tt('importLabel') }}</button>
          </div>
        </div>
        <div class="md-card md-settings-card" @click="showHelpDlg = true">
          <div class="md-row">
            <svg width="22" height="22" viewBox="0 0 24 24" :fill="settingsItemColors.help"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 17h-2v-2h2v2zm2.07-7.75l-.9.92C13.45 12.9 13 13.5 13 15h-2v-.5c0-1.1.45-2.1 1.17-2.83l1.24-1.26c.37-.36.59-.86.59-1.41 0-1.1-.9-2-2-2s-2 .9-2 2H8c0-2.21 1.79-4 4-4s4 1.79 4 4c0 .88-.36 1.68-.93 2.25z"/></svg>
            <div class="flex-1">
              <div class="md-name">{{ tt('help') }}</div>
              <div class="md-sub">{{ tt('helpDesc') }}</div>
            </div>
          </div>
        </div>
        <div class="md-card md-settings-card" @click="showAboutDlg = true">
          <div class="md-row">
            <svg width="22" height="22" viewBox="0 0 24 24" :fill="settingsItemColors.about"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/></svg>
            <div class="flex-1">
              <div class="md-name">{{ tt('about') }}</div>
              <div class="md-sub">{{ tt('aboutSub') }}</div>
            </div>
            <span class="md-ver-badge">v{{ pkg.version }}</span>
          </div>
        </div>
      </div>
    </template>

    <!-- ==================== Bottom Tab Bar ==================== -->
    <div class="md-tabs">
      <button class="md-tab" :class="{ 'md-tab-active': activeTab === 'wol' }" @click="switchTab('wol')">
        <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M864 752H624v64h136c12.8 0 24 11.2 24 24s-11.2 24-24 24h-480c-12.8 0-24-11.2-24-24s11.2-24 24-24H400v-64H160c-17.6 0-32-14.4-32-32V192c0-17.6 14.4-32 32-32h704c17.6 0 32 14.4 32 32v528c0 17.6-14.4 32-32 32z m-16-544H176v464h672V208z m-48 416H224V256h576v368z"/></svg>
        <span class="md-tab-label">{{ tt('wol') }}</span>
      </button>
      <button class="md-tab" :class="{ 'md-tab-active': activeTab === 'luci' }" @click="switchTab('luci')">
        <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M881.777778 597.333333h-56.888889V199.111111c0-17.066667-11.377778-28.444444-28.444445-28.444444s-28.444444 11.377778-28.444444 28.444444v398.222222h-227.555556V199.111111c0-17.066667-11.377778-28.444444-28.444444-28.444444s-28.444444 11.377778-28.444444 28.444444v398.222222H256V199.111111c0-17.066667-11.377778-28.444444-28.444444-28.444444s-28.444444 11.377778-28.444445 28.444444v398.222222H142.222222c-31.288889 0-56.888889 25.6-56.888889 56.888889v142.222222c0 31.288889 25.6 56.888889 56.888889 56.888889h739.555556c31.288889 0 56.888889-25.6 56.888889-56.888889v-142.222222c0-31.288889-25.6-56.888889-56.888889-56.888889z m-327.111111 170.666667c-22.755556 0-42.666667-19.911111-42.666667-42.666667s19.911111-42.666667 42.666667-42.666666 42.666667 19.911111 42.666666 42.666666-19.911111 42.666667-42.666666 42.666667z m142.222222 0c-22.755556 0-42.666667-19.911111-42.666667-42.666667s19.911111-42.666667 42.666667-42.666666 42.666667 19.911111 42.666667 42.666666-19.911111 42.666667-42.666667 42.666667z m142.222222 0c-22.755556 0-42.666667-19.911111-42.666667-42.666667s19.911111-42.666667 42.666667-42.666666 42.666667 19.911111 42.666667 42.666666-19.911111 42.666667-42.666667 42.666667z"/></svg>
        <span class="md-tab-label">{{ tt('routers') }}</span>
      </button>
      <button class="md-tab" :class="{ 'md-tab-active': activeTab === 'net' }" @click="switchTab('net')">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>
        <span class="md-tab-label">{{ tt('network') }}</span>
      </button>
      <button class="md-tab" :class="{ 'md-tab-active': activeTab === 'settings' }" @click="switchTab('settings')">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 00.12-.61l-1.92-3.32a.488.488 0 00-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 00-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.07.62-.07.94s.02.64.07.94l-2.03 1.58a.49.49 0 00-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/></svg>
        <span class="md-tab-label">{{ tt('settings') }}</span>
      </button>
    </div>

    <!-- ==================== Dialogs ==================== -->
    <Dialog v-model:visible="showWolEditor" modal :header="tt('editDevicesToml')" :style="{ width: '90vw', maxWidth: '520px' }" class="md-dialog" :pt="{ closeButton: { style: 'display:none' }, header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }" @show="wolTextareaRef?.$el?.querySelector('textarea')?.focus()">
      <Textarea ref="wolTextareaRef" v-model="wolToml" rows="14" class="w-full" style="font-family:'Courier New',monospace;font-size:0.9rem;line-height:1.5"
        placeholder='[[device]]&#10;name = "My PC"&#10;mac = "AA:BB:CC:DD:EE:FF"&#10;ip = "192.168.2.2"&#10;interface = "br-lan"&#10;router_ip = "192.168.2.1"&#10;agent_port = 32249' />
      <template #footer>
        <div class="md-dlg-ft">
          <button class="md-dlg-btn" @click="importFromClipboard('wol')">{{ tt('importFromClipboard') }}</button>
          <div class="flex-1" />
          <button class="md-dlg-btn" @click="showWolEditor = false">{{ tt('cancel') }}</button>
          <button class="md-dlg-btn md-dlg-btn-save" @click="saveWolConfig">{{ tt('save') }}</button>
        </div>
      </template>
    </Dialog>

    <Dialog v-model:visible="showNetEditor" modal :header="tt('editConfigDialog')" :style="{ width: '90vw', maxWidth: '560px' }" class="md-dialog" :pt="{ closeButton: { style: 'display:none' }, header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }" @show="netTextareaRef?.$el?.querySelector('textarea')?.focus()">
      <Textarea ref="netTextareaRef" v-model="editingNetToml" rows="16" class="w-full" style="font-family:'Courier New',monospace;font-size:0.9rem;line-height:1.5" placeholder='instance_name = "my-network"&#10;...' />
      <template #footer>
        <div class="md-dlg-ft">
          <button class="md-dlg-btn" @click="importFromClipboard('net')">{{ tt('importFromClipboard') }}</button>
          <div class="flex-1" />
          <button class="md-dlg-btn" @click="showNetEditor = false">{{ tt('cancel') }}</button>
          <button class="md-dlg-btn md-dlg-btn-save" @click="saveNetConfig" :disabled="savingNet">{{ savingNet ? '...' : tt('save') }}</button>
        </div>
      </template>
    </Dialog>

    <!-- LuCI Router Config Editor -->
    <Dialog v-model:visible="showLuciEditor" modal :header="tt('editRoutersToml')" :style="{ width: '90vw', maxWidth: '520px' }" class="md-dialog" :pt="{ closeButton: { style: 'display:none' }, header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }">
      <Textarea v-model="luciToml" rows="12" class="w-full" style="font-family:'Courier New',monospace;font-size:0.9rem;line-height:1.5"
        placeholder='[[router]]&#10;name = "Home Router"&#10;ip = "192.168.2.1"&#10;username = "root"&#10;password = "admin"' />
      <template #footer>
        <div class="md-dlg-ft">
          <button class="md-dlg-btn" @click="importFromClipboard('luci')">{{ tt('importFromClipboard') }}</button>
          <div class="flex-1" />
          <button class="md-dlg-btn" @click="showLuciEditor = false">{{ tt('cancel') }}</button>
          <button class="md-dlg-btn md-dlg-btn-save" @click="saveLuciConfig">{{ tt('save') }}</button>
        </div>
      </template>
    </Dialog>

    <!-- Delete Confirm Dialog -->
    <Dialog v-model:visible="showDeleteConfirm" modal :header="tt('delete')" :style="{ width: '85vw', maxWidth: '380px' }" class="md-dialog" :pt="{ header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }">
      <p class="text-base py-2" style="color:var(--md-text)">{{ tt('deleteNetworkConfirm') }}</p>
      <template #footer>
        <div class="md-dlg-ft" style="justify-content:flex-end;gap:8px">
          <button class="md-dlg-btn" @click="showDeleteConfirm = false">{{ tt('cancel') }}</button>
          <button class="md-dlg-btn md-dlg-btn-del" @click="doDeleteNet">{{ tt('confirm') }}</button>
        </div>
      </template>
    </Dialog>

    <!-- Import Config Dialog -->
    <Dialog v-model:visible="showImportDlg" modal :header="tt('importConfig')" :style="{ width: '90vw', maxWidth: '520px' }" class="md-dialog" :pt="{ closeButton: { style: 'display:none' }, header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }" @show="importTextareaRef?.$el?.querySelector('textarea')?.focus()">
      <Textarea ref="importTextareaRef" v-model="importText" rows="16" class="w-full" style="font-family:'Courier New',monospace;font-size:0.9rem;line-height:1.5" placeholder='{ "v": 1, "wol": "...", "luci": "...", "net": ["..."] }' />
      <template #footer>
        <div class="md-dlg-ft">
          <button class="md-dlg-btn" @click="doImportFromClipboard">{{ tt('importFromClipboard') }}</button>
          <div class="flex-1" />
          <button class="md-dlg-btn" @click="showImportDlg = false">{{ tt('cancel') }}</button>
          <button class="md-dlg-btn md-dlg-btn-save" @click="confirmImport">{{ tt('save') }}</button>
        </div>
      </template>
    </Dialog>

    <!-- Import Confirm Dialog -->
    <Dialog v-model:visible="showImportConfirm" modal :header="tt('importConfig')" :style="{ width: '85vw', maxWidth: '400px' }" class="md-dialog" :pt="{ header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }">
      <p class="text-base py-2" style="color:var(--md-text)">{{ tt('importConfirmMsg') }}</p>
      <template #footer>
        <div class="md-dlg-ft" style="justify-content:flex-end;gap:8px">
          <button class="md-dlg-btn" @click="showImportConfirm = false">{{ tt('cancel') }}</button>
          <button class="md-dlg-btn md-dlg-btn-save" @click="executeImport">{{ tt('confirm') }}</button>
        </div>
      </template>
    </Dialog>

    <!-- Help Dialog -->
    <Dialog v-model:visible="showHelpDlg" modal :header="tt('help')" :style="{ width: '90vw', maxWidth: '540px' }" class="md-dialog" :pt="{ header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }">
      <!-- WOL -->
      <div class="md-help-section">
        <div class="md-help-title">{{ tt('helpWolTitle') }}</div>
        <div class="md-help-desc">{{ tt('helpWolDesc') }}</div>
        <div class="md-help-dep">{{ tt('helpWolDep') }}</div>
        <a class="md-help-link" @click.prevent="openHelpUrl(HELP_WOL_URL)">{{ tt('helpWolLink') }}</a>
        <div class="md-help-cfg-label">{{ tt('helpCfgTitle') }}</div>
        <div class="md-help-cfg-wrap">
          <pre class="md-help-code">{{ HELP_CFG_WOL }}</pre>
          <button class="md-help-copy" @click="copyHelpCfg(HELP_CFG_WOL, 0)">{{ copiedIdx === 0 ? tt('copiedCode') : tt('copyCode') }}</button>
        </div>
        <div v-for="(p, i) in tt('helpWolParams').split('\n')" :key="i" class="md-help-param">
          <code>{{ p.split(' — ')[0] }}</code> — {{ p.split(' — ').slice(1).join(' — ') }}
        </div>
        <div class="md-help-note">{{ tt('helpWolSocks5') }}</div>
      </div>
      <!-- LuCI -->
      <div class="md-help-section">
        <div class="md-help-title">{{ tt('helpLuciTitle') }}</div>
        <div class="md-help-desc">{{ tt('helpLuciDesc') }}</div>
        <div class="md-help-desc" style="white-space:pre-line;margin-bottom:8px">{{ tt('helpLuciFeat') }}</div>
        <div class="md-help-cfg-label">{{ tt('helpCfgTitle') }}</div>
        <div class="md-help-cfg-wrap">
          <pre class="md-help-code">{{ HELP_CFG_LUCI }}</pre>
          <button class="md-help-copy" @click="copyHelpCfg(HELP_CFG_LUCI, 1)">{{ copiedIdx === 1 ? tt('copiedCode') : tt('copyCode') }}</button>
        </div>
        <div v-for="(p, i) in tt('helpLuciParams').split('\n')" :key="i" class="md-help-param">
          <code>{{ p.split(' — ')[0] }}</code> — {{ p.split(' — ').slice(1).join(' — ') }}
        </div>
        <div class="md-help-note">{{ tt('helpLuciSocks5') }}</div>
      </div>
      <!-- EasyTier -->
      <div class="md-help-section">
        <div class="md-help-title">{{ tt('helpNetTitle') }}</div>
        <div class="md-help-desc">{{ tt('helpNetDesc') }}</div>
        <div class="md-help-desc" style="white-space:pre-line;margin-bottom:8px">{{ tt('helpNetFeat') }}</div>
        <div class="md-help-cfg-label">{{ tt('helpCfgTitle') }}</div>
        <div class="md-help-cfg-wrap">
          <pre class="md-help-code">{{ HELP_CFG_NET }}</pre>
          <button class="md-help-copy" @click="copyHelpCfg(HELP_CFG_NET, 2)">{{ copiedIdx === 2 ? tt('copiedCode') : tt('copyCode') }}</button>
        </div>
        <div v-for="(p, i) in tt('helpNetParams').split('\n')" :key="i" class="md-help-param">
          <code>{{ p.split(' — ')[0] }}</code> — {{ p.split(' — ').slice(1).join(' — ') }}
        </div>
      </div>
      <!-- Import & Export -->
      <div class="md-help-section">
        <div class="md-help-title">{{ tt('helpBackupTitle') }}</div>
        <div class="md-help-desc">{{ tt('helpBackupDesc') }}</div>
      </div>
      <template #footer>
        <div class="md-dlg-ft" style="justify-content:flex-end">
          <button class="md-dlg-btn" @click="showHelpDlg = false">{{ tt('close') }}</button>
        </div>
      </template>
    </Dialog>

    <!-- About Dialog -->
    <Dialog v-model:visible="showAboutDlg" modal :header="tt('about')" :style="{ width: '90vw', maxWidth: '540px' }" class="md-dialog md-about-dlg" :pt="{ header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }">
      <div class="md-help-section">
        <div class="md-help-title">EasyTier + WOLPlus <span class="md-about-ver">v{{ pkg.version }}</span></div>
        <div class="md-help-desc">{{ tt('aboutDesc') }}</div>
      </div>
      <div class="md-help-section">
        <div class="md-help-title">{{ tt('aboutLinks') }}</div>
        <div class="md-help-desc" style="margin-bottom:4px">{{ tt('aboutThisProject') }}</div>
        <a class="md-help-link" @click.prevent="openHelpUrl(ABOUT_THIS_URL)">github.com/lymwhen/EasyTier</a>
        <div class="md-help-desc" style="margin-top:10px;margin-bottom:4px">{{ tt('aboutWolplusDep') }}</div>
        <a class="md-help-link" @click.prevent="openHelpUrl(ABOUT_WOLPLUS_DEP_URL)">github.com/lymwhen/luci-app-wolplus</a>
      </div>
      <hr class="md-help-divider">
      <div class="md-help-section">
        <div class="md-help-title">{{ tt('aboutThanks') }}</div>
        <div class="md-help-desc" style="margin-bottom:4px"><b>EasyTier</b> — {{ tt('aboutEasyTierDesc') }}</div>
        <a class="md-help-link" @click.prevent="openHelpUrl(ABOUT_EASYTIER_URL)">github.com/EasyTier/EasyTier</a>
        <div class="md-help-desc" style="margin-top:10px;margin-bottom:4px"><b>luci-app-wolplus</b> — {{ tt('aboutWolplusDesc') }}</div>
        <a class="md-help-link" @click.prevent="openHelpUrl(ABOUT_WOLPLUS_URL)">github.com/animegasan/luci-app-wolplus</a>
      </div>
      <template #footer>
        <div class="md-dlg-ft" style="justify-content:flex-end">
          <button class="md-dlg-btn" @click="showAboutDlg = false">{{ tt('close') }}</button>
        </div>
      </template>
    </Dialog>

    <!-- Snackbar -->
    <div v-if="snackShow" class="md-snackbar">{{ snackMsg }}</div>
  </div>
</template>

<style>
:root {
  font-size: 15px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --md-primary: #1976d2;
  --md-card: #fff;
  --md-text: #212121;
  --md-secondary: #616161;
  --md-muted: #9e9e9e;
  --md-border: #e0e0e0;
  --md-divider: #eeeeee;
  --md-shadow: 0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.08);
  --md-radius: 12px;
}

/* ===== Dark Theme ===== */
html[data-theme="dark"] {
  --md-primary: #42a5f5;
  --md-card: #1e1e1e;
  --md-text: #e0e0e0;
  --md-secondary: #9e9e9e;
  --md-muted: #757575;
  --md-border: #333;
  --md-divider: #2a2a2a;
  --md-shadow: 0 1px 3px rgba(0,0,0,0.4);
}

/* ===== AMOLED Theme ===== */
html[data-theme="amoled"] {
  --md-primary: #42a5f5;
  --md-card: #0f0f0f;
  --md-text: #e0e0e0;
  --md-secondary: #9e9e9e;
  --md-muted: #757575;
  --md-border: #141414;
  --md-divider: #111;
  --md-shadow: none;
  background: #000;
}

.md-app {
  color: var(--md-text);
  -webkit-text-size-adjust: 100%;
}

.md-dialog {
  --p-dialog-border-radius: 16px;
  --p-dialog-background: var(--md-card);
  --p-dialog-color: var(--md-text);
}
.md-dialog.p-dialog { border-radius:16px; overflow:hidden; background:var(--md-card); }
.overflow-y-auto { scrollbar-width:none; -ms-overflow-style:none; }
.overflow-y-auto::-webkit-scrollbar { display:none; }
.md-dialog .p-dialog-header { background:transparent !important; }
.md-dialog .p-dialog-content { background:transparent !important; }
.md-dialog .p-dialog-footer { background:transparent !important; }
.md-dialog .p-dialog-header-icon:focus,
.md-dialog .p-dialog-close-button:focus,
.md-dialog .p-dialog-header-close:focus { outline:none !important; box-shadow:none !important; }
.md-dialog-hdr { font-size:0.95rem !important; font-weight:600 !important; padding:12px 16px 0 !important; color:var(--md-text) !important; background:transparent !important; border:none !important; }
.md-dialog-content { padding:8px 16px 0 !important; background:transparent !important; }
.md-dialog-ft { padding:0 !important; background:transparent !important; border:none !important; }
/* Remove PrimeVue Textarea bottom margin inside dialogs */
.md-dialog .p-textarea { margin-bottom:0 !important; }
.md-dialog textarea { border:1px solid var(--md-border) !important; border-radius:10px !important; padding:10px 12px !important; background:var(--md-card) !important; color:var(--md-text) !important; font-size:0.88rem !important; outline:none !important; box-shadow:none !important; resize:vertical; }
.md-dialog textarea:focus { border-color:var(--md-primary) !important; }

/* Dialog custom footer */
.md-dlg-ft { display:flex; align-items:center; gap:6px; padding:4px 12px 10px; width:100%; }
.md-dlg-btn { padding:8px 12px; border:none; border-radius:6px; font-size:0.85rem; font-weight:500; cursor:pointer; background:transparent; color:var(--md-primary); min-width:64px; letter-spacing:0.3px; }
.md-dlg-btn:active { background:rgba(0,0,0,0.08); }
.md-dlg-btn-save { color:var(--md-primary); font-weight:600; }
.md-dlg-btn-del { color:#c62828; }
html[data-theme="dark"] .md-dlg-btn:active { background:rgba(255,255,255,0.08); }
html[data-theme="dark"] .md-dialog.p-dialog { border: 1px solid var(--md-border) !important; }
html[data-theme="amoled"] .md-dlg-btn:active { background:rgba(255,255,255,0.08); }
html[data-theme="amoled"] .md-dialog.p-dialog { border: 1px solid var(--md-border) !important; }
html[data-theme="amoled"] .p-dialog { background: #000 !important; }
html[data-theme="amoled"] .p-dialog-header,
html[data-theme="amoled"] .p-dialog-content,
html[data-theme="amoled"] .p-dialog-footer { background: transparent !important; }

html[data-theme="amoled"] .md-app { background: #000; }
</style>

<style scoped>
/* Header */
.md-hdr { display:flex; align-items:center; gap:8px; padding:0 16px; min-height:56px; flex-shrink:0; border-bottom:1px solid var(--md-border); }
.md-hdr-title { font-size:1rem; font-weight:600; }
.md-connected-text { font-size:0.78rem; color:#2e7d32; font-weight:500; white-space:nowrap; }
.md-hdr-btn { display:flex; align-items:center; border:none; background:none; padding:8px; border-radius:50%; cursor:pointer; color:var(--md-secondary); }
.md-hdr-btn-off { color:#c62828; }

/* Connection bar */
.md-conn { display:flex; align-items:center; gap:10px; padding:8px 16px; font-size:0.78rem; flex-shrink:0; min-height:32px; }
.md-conn-on { background:#e8f5e9; color:#2e7d32; }
.md-conn-ips { display:flex; flex-direction:column; justify-content:center; white-space:pre-line; line-height:1.3; }
.md-conn-ipv6 { font-size:0.7rem; color:var(--md-muted); }

/* Material buttons */
.md-wake-btn, .md-shutdown-btn {
  display:inline-flex; align-items:center; gap:4px;
  padding:5px 12px; border:none; border-radius:20px;
  font-size:0.75rem; font-weight:600; cursor:pointer;
  transition:background 0.15s; flex-shrink:0;
}
.md-wake-btn { background:#e3f2fd; color:#1565c0; }
.md-wake-btn:active { background:#bbdefb; }
.md-shutdown-btn { background:#fbe9e7; color:#c62828; }
.md-shutdown-btn:active { background:#ffccbc; }

/* Chips */
.md-chip { display:inline-flex; padding:1px 7px; border-radius:8px; font-size:0.65rem; font-weight:500; background:#f5f5f5; color:#616161; }

/* NAT colors */
.md-nat-g { background:#e8f5e9; color:#2e7d32; }
.md-nat-b { background:#e3f2fd; color:#1565c0; }
.md-nat-o { background:#fff3e0; color:#e65100; }

/* Quality */
.md-quality { font-size:0.72rem; font-weight:500; flex-shrink:0; white-space:nowrap; }
.md-q-good { color:#2e7d32; }
.md-q-info { color:#1565c0; }
.md-q-warn { color:#e65100; }
.md-q-bad { color:#c62828; }

.md-connect-btn {
  display:flex; flex-direction:column; align-items:center; gap:10px;
  padding:30px 44px; border:none; border-radius:24px; min-width:140px;
  background:var(--md-primary); color:#fff;
  font-size:1.05rem; font-weight:600; cursor:pointer;
  box-shadow:0 4px 16px rgba(0,0,0,0.2); transition:transform 0.15s;
}
.md-connect-btn:active { transform:scale(0.96); }
.md-connect-btn-loading { background:#78909c; cursor:not-allowed; }
.md-connect-btn-loading:active { transform:none; }

/* Section */
.md-section { margin-bottom:4px; }
.md-sec-hdr { display:flex; align-items:center; gap:8px; padding:14px 4px 8px; }
.md-sec-title { font-size:0.75rem; font-weight:600; color:var(--md-secondary); text-transform:uppercase; letter-spacing:0.5px; flex:1; }
.md-badge { font-size:0.7rem; font-weight:600; padding:2px 8px; border-radius:10px; background:var(--md-primary); color:#fff; line-height:1.2; }

/* Path badge */
.md-path-badge { font-size:0.65rem; font-weight:600; padding:2px 6px; border-radius:8px; background:#e3f2fd; color:#1565c0; }

/* Card */
.md-card { background:var(--md-card); border-radius:var(--md-radius); box-shadow:var(--md-shadow); margin-bottom:8px; cursor:pointer; transition:opacity 0.3s; }
.md-card-dim { opacity:0.55; }
.md-card-item { margin-bottom:8px; }
.md-settings-card { margin:8px 0; }
.md-row { display:flex; align-items:center; gap:12px; padding:12px 18px; }
.md-row-top { padding-bottom:1px; align-items:center; }
.md-row-bot { padding-top:1px; }
.md-name { font-size:0.9rem; font-weight:500; }
.md-sub { font-size:0.76rem; color:var(--md-muted); margin-top:1px; line-height:1.3; }
.md-chevron { width:18px; height:18px; opacity:0.55; transition:transform 0.2s; flex-shrink:0; color:var(--md-muted); }
.md-chevron-open { transform:rotate(180deg); }
.md-extra { margin:0 10px; padding:0 4px 12px 30px; font-size:0.76rem; color:var(--md-secondary); line-height:1.6; border-top:1px solid var(--md-divider); padding-top:7px; }

/* Dot */
.md-dot { width:11px; height:11px; border-radius:50%; flex-shrink:0; }
.md-dot-on { background:#4caf50; }
.md-dot-off { background:#bdbdbd; }
.md-dot-ph { animation:md-pulse 1s infinite; }
.md-dot-waking { background:#f9ab00; }
.md-dot-shutting { background:#ea4335; }
@keyframes md-pulse { 0%,100%{opacity:1} 50%{opacity:0.3} }
@keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.4} }
.animate-pulse { animation:pulse 1.5s ease-in-out infinite; }

/* Route tag */
.md-rt { display:inline-flex; padding:1px 7px; border-radius:8px; font-size:0.65rem; font-weight:600; text-transform:uppercase; letter-spacing:0.3px; flex-shrink:0; }
.md-rt-l { background:#e8f5e9; color:#2e7d32; }
.md-rt-p { background:#e8f5e9; color:#2e7d32; }
.md-rt-r { background:#fff3e0; color:#e65100; }

/* Switch menu */
.md-switch-menu { position:absolute; right:0; top:40px; z-index:200; min-width:200px; max-width:260px; padding:4px 0; overflow:hidden; }
.md-sw-item { display:flex; align-items:center; gap:8px; padding:10px 14px; font-size:0.95rem; cursor:pointer; white-space:nowrap; }
.md-sw-item:active { background:rgba(0,0,0,0.06); }
.md-sw-del:active { background:rgba(0,0,0,0.08); color:#c62828; }
.md-sw-divider { height:1px; background:var(--md-border); margin:4px 0; }
.md-sw-check { color:var(--md-primary); font-weight:700; width:18px; display:inline-block; }
.md-sw-del { display:flex; align-items:center; justify-content:center; width:24px; height:24px; border:none; background:none; cursor:pointer; color:var(--md-muted); font-size:1.1rem; border-radius:50%; flex-shrink:0; }

/* Language badge */
.md-lang-badge { font-size:0.7rem; font-weight:600; padding:2px 8px; border-radius:8px; background:#e3f2fd; color:#1565c0; flex-shrink:0; }
.md-ver-badge { font-size:0.75rem; font-weight:500; color:var(--md-muted); flex-shrink:0; }
.md-about-ver { font-size:0.75rem; font-weight:400; color:var(--md-muted); margin-left:8px; }

/* Toggle switch — Material Design 3 style */
.md-toggle { width:44px; height:24px; border-radius:12px; background:#bdbdbd; flex-shrink:0; position:relative; transition:background 0.2s; }
.md-toggle::after { content:''; position:absolute; top:3px; left:3px; width:18px; height:18px; border-radius:50%; background:#fff; transition:transform 0.2s; box-shadow:0 1px 3px rgba(0,0,0,0.2); }
.md-toggle-on { background:var(--md-primary); }
.md-toggle-on::after { transform:translateX(20px); }

/* Select dropdown */
.md-select {
  padding:6px 28px 6px 10px; border-radius:8px; border:1px solid var(--md-border);
  font-size:0.82rem; font-weight:500; background:var(--md-card); color:var(--md-text);
  cursor:pointer; appearance:none; -webkit-appearance:none;
  background-image:url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='%239e9e9e'%3E%3Cpath d='M7 10l5 5 5-5z'/%3E%3C/svg%3E");
  background-repeat:no-repeat; background-position:right 8px center; flex-shrink:0;
}
.md-select:focus { outline:none; border-color:var(--md-primary); }

/* Bottom tabs */
.md-tabs { display:flex; border-top:1px solid var(--md-border); flex-shrink:0; background:var(--md-card); }
.md-tab { flex:1; display:flex; flex-direction:column; align-items:center; gap:2px; padding:10px 0 8px; border:none; background:none; cursor:pointer; color:var(--md-muted); transition:color 0.15s; }
.md-tab-active { color:var(--md-primary); }
.md-tab-label { font-size:0.72rem; font-weight:500; }

@keyframes spin { from{transform:rotate(0deg)} to{transform:rotate(360deg)} }
.animate-spin { animation:spin 1s linear infinite; }

/* Snackbar */
.md-snackbar {
  position:fixed; bottom:80px; left:50%; transform:translateX(-50%);
  padding:10px 20px; border-radius:20px; font-size:0.88rem; font-weight:500;
  background:var(--md-text); color:var(--md-card);
  box-shadow:0 4px 12px rgba(0,0,0,0.25); z-index:999;
  animation:md-snack-in 0.25s ease-out;
  max-width:85vw; text-align:center;
}
@keyframes md-snack-in { from{opacity:0;transform:translateX(-50%) translateY(12px)} to{opacity:1;transform:translateX(-50%) translateY(0)} }

/* Traffic speed chart */
.md-traffic-card { padding:10px 14px 4px; margin-top:6px; margin-bottom:8px; cursor:default; }
.md-traffic-hdr { display:flex; justify-content:space-between; font-size:0.7rem; font-weight:500; margin-bottom:2px; }
.md-traffic-label { margin-right:12px; }
.md-traffic-total { text-align:right; font-size:0.68rem; color:var(--md-muted); flex-shrink:0; }
.md-traffic-up-sub { color:#43a047; font-size:0.7rem; }
.md-traffic-down-sub { color:#1e88e5; font-size:0.7rem; }
.md-traffic-svg { width:100%; height:90px; display:block; }
.md-traffic-chart-wrap { position:relative; }

/* Stats expanded area */
.md-stats-area { margin:0 10px; padding:0 4px 10px 8px; padding-top:8px; cursor:default; }
.md-stats-info { font-size:0.68rem; color:var(--md-secondary); opacity:0.85; margin-top:2px; padding-top:2px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
.md-stats-loading { display:flex; align-items:center; justify-content:center; gap:8px; padding:24px 0; color:var(--md-muted); font-size:0.82rem; }
.md-stats-section { margin-bottom:10px; }
.md-stats-hdr { display:flex; align-items:baseline; flex-wrap:wrap; gap:4px 8px; margin-bottom:2px; }
	.md-stats-legends { margin-left:auto; display:flex; flex-wrap:wrap; justify-content:flex-end; gap:2px 10px; }
.md-stats-label { font-size:0.76rem; font-weight:600; color:var(--md-text); overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
.md-stats-label-c { font-size:0.7rem; color:var(--md-muted); }
.md-stats-val { font-size:0.72rem; color:var(--md-secondary); flex-shrink:0; }
.md-stats-svg { width:100%; height:90px; display:block; }
.md-stats-chart-wrap { position:relative; }
.md-stats-legend { font-size:0.68rem; font-weight:500; flex-shrink:0; }
	/* Disk rows — identical style for total & partition, no left indent, light gray track */
	.md-disk-total, .md-part-row { position:relative; height:20px; border-radius:3px; background:rgba(127,127,127,0.06); overflow:hidden; margin:2px 0; display:flex; align-items:center; padding:0 6px; }
	.md-part-fill-bg { position:absolute; left:0; top:0; height:100%; border-radius:3px; background:#1565c0; transition:width 0.3s; opacity:0.22; }
.md-disk-total-usage, .md-part-usage { position:relative; z-index:1; font-size:0.64rem; color:var(--md-text); flex-shrink:0; margin-left:auto; opacity:0.7; }
.md-part-label { position:relative; z-index:1; font-size:0.68rem; color:var(--md-secondary); flex-shrink:0; }


.md-chart-label { position:absolute; font-size:0.5rem; color:var(--md-muted); font-family:sans-serif; pointer-events:none; white-space:nowrap; }
.md-chart-y-max { left:2px; top:1px; }
.md-chart-y-zero { left:2px; bottom:14px; }
.md-chart-x-start { left:2px; bottom:0; }
.md-chart-x-mid { left:50%; transform:translateX(-50%); bottom:0; }
.md-chart-x-end { right:2px; bottom:0; }

/* LuCI iframe */
.luci-iframe { width:100%; height:100%; border:none; }

/* Settings action buttons */
.md-settings-btn {
  display:inline-flex; align-items:center; gap:4px;
  padding:5px 12px; border:none; border-radius:16px;
  font-size:0.75rem; font-weight:500; cursor:pointer;
  transition:background 0.15s; flex-shrink:0;
  background:#f5f5f5; color:var(--md-secondary);
}
.md-settings-btn:active { background:#e0e0e0; }
.md-settings-btn-export {
  background:#e8f5e9; color:#2e7d32; font-weight:600;
}
.md-settings-btn-export:active { background:#c8e6c9; }
.md-settings-btn-primary {
  background:#e3f2fd; color:#1565c0; font-weight:600;
}
.md-settings-btn-primary:active { background:#bbdefb; }
.md-help-section { padding: 12px 0; }
.md-help-section + .md-help-section { border-top: 1px solid var(--md-divider); }
.md-about-dlg .md-help-section + .md-help-section { border-top: none; }
.md-help-divider { border: none; border-top: 1px solid var(--md-divider); margin: 0; }
.md-help-title { font-size: 0.92rem; font-weight: 600; color: var(--md-text); margin-bottom: 4px; }
.md-help-desc { font-size: 0.82rem; color: var(--md-secondary); line-height: 1.5; margin-bottom: 4px; }
.md-help-dep { font-size: 0.78rem; color: var(--md-muted); line-height: 1.4; margin-bottom: 8px; }
.md-help-link { display: inline-flex; align-items: center; gap: 6px; padding: 8px 16px; border: none; border-radius: 20px; font-size: 0.8rem; font-weight: 600; cursor: pointer; background: var(--md-primary); color: #fff; text-decoration: none; margin-bottom: 10px; }
.md-help-link:active { opacity: 0.85; }
.md-help-cfg-label { font-size: 0.75rem; font-weight: 600; color: var(--md-muted); margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.5px; }
.md-help-cfg-wrap { position: relative; }
.md-help-code { font-size: 0.76rem; line-height: 1.55; padding: 10px 12px; margin: 0 0 4px; border-radius: 8px; background: #f0f0f0; color: var(--md-text); overflow-x: auto; white-space: pre; font-family: 'Cascadia Code', 'Fira Code', 'JetBrains Mono', 'SF Mono', 'Consolas', monospace; }
.md-help-copy { position: absolute; top: 6px; right: 8px; padding: 3px 10px; border: none; border-radius: 4px; font-size: 0.72rem; font-weight: 600; cursor: pointer; background: var(--md-secondary); color: #fff; opacity: 0.7; transition: opacity 0.15s; }
.md-help-copy:hover { opacity: 1; }
.md-help-param { font-size:0.78rem; color:var(--md-secondary); line-height:1.6; margin-bottom:2px; }
.md-help-param code { font-size:0.75rem; font-weight:600; color:var(--md-text); margin-right:8px; }
.md-help-note { padding:10px 12px; border-radius:8px; background:var(--md-divider); font-size:0.78rem; color:var(--md-secondary); line-height:1.5; margin-top:8px; }
.md-help-note code { font-size:0.75rem; color:var(--md-text); }

/* ===== Dark Theme ===== */
html[data-theme="dark"] .md-connected-text { color:#66bb6a; }
html[data-theme="dark"] .md-hdr-btn:active { background:rgba(255,255,255,0.08); }
html[data-theme="dark"] .md-conn-on { background:#1b3a1b; color:#66bb6a; }
html[data-theme="dark"] .md-wake-btn { background:#1a3050; color:#64b5f6; }
html[data-theme="dark"] .md-wake-btn:active { background:#234a7e; }
html[data-theme="dark"] .md-shutdown-btn { background:#3e1a1a; color:#ef9a9a; }
html[data-theme="dark"] .md-shutdown-btn:active { background:#5e2828; }
html[data-theme="dark"] .md-chip { background:#333; color:#9e9e9e; }
html[data-theme="dark"] .md-nat-g { background:#1b3a1b; color:#66bb6a; }
html[data-theme="dark"] .md-nat-b { background:#1a3050; color:#64b5f6; }
html[data-theme="dark"] .md-nat-o { background:#3e2723; color:#ff9800; }
html[data-theme="dark"] .md-q-good { color:#66bb6a; }
html[data-theme="dark"] .md-q-info { color:#64b5f6; }
html[data-theme="dark"] .md-q-warn { color:#ff9800; }
html[data-theme="dark"] .md-q-bad { color:#ef9a9a; }
html[data-theme="dark"] .md-path-badge { background:#1a3050; color:#64b5f6; }
html[data-theme="dark"] .md-rt-l { background:#1b3a1b; color:#66bb6a; }
html[data-theme="dark"] .md-rt-p { background:#1b3a1b; color:#66bb6a; }
html[data-theme="dark"] .md-rt-r { background:#3e2723; color:#ff9800; }
html[data-theme="dark"] .md-sw-item:active { background:rgba(255,255,255,0.06); }
html[data-theme="dark"] .md-sw-del:active { background:rgba(255,255,255,0.08); color:#c62828; }
html[data-theme="dark"] .md-lang-badge { background:#1a3050; color:#64b5f6; }
html[data-theme="dark"] .md-toggle { background:#555; }
html[data-theme="dark"] .md-settings-btn { background:#333; color:var(--md-muted); }
html[data-theme="dark"] .md-settings-btn:active { background:#444; }
html[data-theme="dark"] .md-settings-btn-export { background:#1b3a1b; color:#66bb6a; }
html[data-theme="dark"] .md-settings-btn-export:active { background:#2a4a2a; }
html[data-theme="dark"] .md-settings-btn-primary { background:#1a3050; color:#64b5f6; }
html[data-theme="dark"] .md-settings-btn-primary:active { background:#234a7e; }
html[data-theme="dark"] .md-help-code { background:#1e1e1e; }

/* ===== AMOLED Theme ===== */
html[data-theme="amoled"] .md-hdr { background:#000; }
html[data-theme="amoled"] .md-tabs { background:#000; }
html[data-theme="amoled"] .md-connected-text { color:#66bb6a; }
html[data-theme="amoled"] .md-hdr-btn:active { background:rgba(255,255,255,0.08); }
html[data-theme="amoled"] .md-conn-on { background:#0a1a0a !important; color:#66bb6a; }
html[data-theme="amoled"] .md-wake-btn { background:#0d0d0d; color:#64b5f6; }
html[data-theme="amoled"] .md-wake-btn:active { background:#1a1a1a; }
html[data-theme="amoled"] .md-shutdown-btn { background:#0d0d0d; color:#ef9a9a; }
html[data-theme="amoled"] .md-shutdown-btn:active { background:#1a1a1a; }
html[data-theme="amoled"] .md-chip { background:#0d0d0d; color:#9e9e9e; }
html[data-theme="amoled"] .md-nat-g { background:#0a1a0a; color:#66bb6a; }
html[data-theme="amoled"] .md-nat-b { background:#0a1528; color:#64b5f6; }
html[data-theme="amoled"] .md-nat-o { background:#1a1010; color:#ff9800; }
html[data-theme="amoled"] .md-q-good { color:#66bb6a; }
html[data-theme="amoled"] .md-q-info { color:#64b5f6; }
html[data-theme="amoled"] .md-q-warn { color:#ff9800; }
html[data-theme="amoled"] .md-q-bad { color:#ef9a9a; }
html[data-theme="amoled"] .md-path-badge { background:#0a1528; color:#64b5f6; }
html[data-theme="amoled"] .md-rt-l { background:#0a1a0a; color:#66bb6a; }
html[data-theme="amoled"] .md-rt-p { background:#0a1a0a; color:#66bb6a; }
html[data-theme="amoled"] .md-rt-r { background:#1a1010; color:#ff9800; }
html[data-theme="amoled"] .md-sw-item:active { background:rgba(255,255,255,0.06); }
html[data-theme="amoled"] .md-sw-del:active { background:rgba(255,255,255,0.08); color:#c62828; }
html[data-theme="amoled"] .md-lang-badge { background:#0a1528; color:#64b5f6; }
html[data-theme="amoled"] .md-toggle { background:#333; }
html[data-theme="amoled"] .md-toggle-on { background:var(--md-primary); }
html[data-theme="dark"] .md-toggle-on { background:var(--md-primary); }
html[data-theme="amoled"] .md-settings-btn { background:#0d0d0d; color:var(--md-muted); }
html[data-theme="amoled"] .md-settings-btn:active { background:#1a1a1a; }
html[data-theme="amoled"] .md-settings-btn-export { background:#0a1a0a; color:#66bb6a; }
html[data-theme="amoled"] .md-settings-btn-export:active { background:#0d240d; }
html[data-theme="amoled"] .md-settings-btn-primary { background:#0a1528; color:#64b5f6; }
html[data-theme="amoled"] .md-settings-btn-primary:active { background:#0f1f3a; }
html[data-theme="amoled"] .md-help-code { background:#0d0d0d; }
html[data-theme="amoled"] .md-help-copy { background:#333; }
html[data-theme="amoled"] .md-select { background-color:#0d0d0d; }

</style>
