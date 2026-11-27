<script setup lang="ts">
import type { NetworkTypes } from 'easytier-frontend-lib'
import { listen } from '@tauri-apps/api/event'
import { readText, writeText } from '@tauri-apps/plugin-clipboard-manager'
import { openUrl } from '@tauri-apps/plugin-opener'
import { type } from '@tauri-apps/plugin-os'
import { I18nUtils, Utils } from 'easytier-frontend-lib'
import Dialog from 'primevue/dialog'
import Textarea from 'primevue/textarea'
import pkg from '~/../package.json'
import { bootstrapAndroidStorageKeepLocal } from '~/composables/android_compat'
import { collectNetworkInfo, deleteNetworkInstance, generateNetworkConfig, httpCancel, httpGet, httpPost, initRpcConnection, luciGetLastPath, luciProxyStart, luciProxyStop, parseNetworkConfig, runNetworkInstance, updateNetworkConfigState } from '~/composables/backend'
import {
  type AgentTask,
  type AppCommand,
  commandKeyFor,
  type CommandMode,
  createUuid,
  executeAgentCommand,
  getAgentCommandStatus,
  listAgentTasks,
  parseCommandsInline,
  type RawTomlCommand,
  setDeviceCommandsInToml,
  stopAgentTask,
} from '~/composables/computer_commands'
import { loadLastNetworkInstanceId, saveLastNetworkInstanceId } from '~/composables/config'
import { listenGlobalEvents } from '~/composables/event'
import { initMobileVpnService } from '~/composables/mobile_vpn'
import { resolveTheme, applyTheme as setTheme } from '~/composables/theme'
import {
  applyWolFailure,
  applyWolRouterFailure,
  applyWolRouterSuccess,
  applyWolSuccess,
  createWolDeviceStatus,
  createWolRouterStatus,
  effectiveWolState,
  forceWolOffline,
  markWolChecking,
  type WolDeviceStatus,
  type WolEffectiveState,
  type WolRouterStatus,
  type WolStatusResponse,
} from '~/composables/wol_status'

const router = useRouter()
const { locale } = useI18n()

// --- snackbar ---
type SnackType = 'info' | 'progress' | 'success' | 'error'
const snackMsg = ref('')
const snackShow = ref(false)
const snackType = ref<SnackType>('info')
let snackTimer: ReturnType<typeof setTimeout> | null = null
function showSnack(msg: string, type: SnackType = 'info', duration = 2500) {
  snackMsg.value = msg
  snackType.value = type
  snackShow.value = true
  if (snackTimer)
    clearTimeout(snackTimer)
  snackTimer = setTimeout(() => { snackShow.value = false }, duration)
}

// --- i18n ---
const T: Record<string, [string, string]> = {
  pcManagement: ['PC management', '电脑管理'],
  wolDevices: ['PC Devices', '电脑设备'],
  wolDesc: ['Remote Wake & Device Management', '远程唤醒与设备管理'],
  pcs: ['PCs', '电脑'],
  channel: ['Channel', '通道'],
  refresh: ['Refresh', '刷新'],
  editConfig: ['Edit config', '编辑配置'],
  emptyWol: ['Add a PC to get started', '添加电脑以开始'],
  online: ['Online', '在线'],
  offline: ['Offline', '离线'],
  routerOffline: ['Router Offline', '路由器离线'],
  noRouterConfigured: ['No router configured', '未配置路由器'],
  detecting: ['Detecting...', '检测中...'],
  waking: ['Starting...', '正在开机'],
  shutting: ['Shutting down...', '正在关机'],
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
  emptyNet: ['Configure a network to get started', '配置组网以开始'],
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
  emptyRouter: ['Configure a router to get started', '配置路由器以开始'],
  editRouter: ['Edit Router', '编辑路由器'],
  editRoutersToml: ['Edit routers.toml', '编辑 routers.toml'],
  connectingRouter: ['Connecting...', '连接中...'],
  oneClickConfig: ['Configuration', '配置信息'],
  oneClickConfigDesc: ['Export/import all configs', '一键导出/导入电脑、路由器和网络配置'],
  exportLabel: ['Export', '导出'],
  importLabel: ['Import', '导入'],
  exportedToClipboard: ['Config exported to clipboard', '配置信息已导出到剪切板'],
  importConfig: ['Import Config', '导入配置'],
  importConfirmMsg: ['This will overwrite ALL PC, router, and network configs. Continue?', '本次操作将会覆盖电脑、路由器和组网所有的配置，确定继续？'],
  importSuccess: ['Config imported', '配置已导入'],
  importFailed: ['Invalid config format', '配置格式无效'],
  help: ['Help', '帮助'],
  helpDesc: ['View usage guide for all modules, features, and config examples', '查看各模块功能说明与配置示例'],
  helpWolTitle: ['WOL — PC Wake-on-LAN', '电脑 — 网络唤醒'],
  helpWolDesc: ['Remotely wake, monitor, and shut down PCs. Works via router CGI (luci-app-wolplus) for wake-up and PC agent (Go, port 32249) for status/shutdown. Supports both direct LAN and EasyTier tunnel paths.', '远程唤醒、监控和关闭电脑。通过路由器 CGI（luci-app-wolplus）发送魔术包唤醒，通过 PC 端 agent（Go，32249 端口）检测在线状态和执行关机。支持局域网直连和 EasyTier 隧道两种路径。'],
  helpWolDep: ['Dependencies: luci-app-wolplus (install on OpenWrt router) + wol-agent (run on each PC)', '依赖：luci-app-wolplus（安装到 OpenWrt 路由器）+ wol-agent（每台电脑上运行）'],
  helpWolLink: ['Download Dependencies', '下载依赖组件'],
  helpPcTitle: ['PC — Performance Monitoring', '电脑 — 性能监控'],
  helpPcDesc: ['Real-time monitoring of connected PCs — CPU, memory, GPU usage, disk IO, and network traffic. Data is collected by the wol-agent running on each PC and displayed as live charts with historical trend.', '实时监控已连接电脑的 CPU、内存、GPU 使用率、磁盘 IO 和网络流量。数据由每台电脑上运行的 wol-agent 采集，以实时图表和历史趋势展示。'],
  helpPcFeat: ['CPU / Memory / GPU usage with live percentage charts\nDisk IO read/write speed trend\nNetwork upload/download traffic with bandwidth stats\nHardware info — CPU model, memory DDR type and frequency\nAuto-refresh every 10 seconds\nColor-coded quality indicators for quick health check', 'CPU / 内存 / GPU 使用率实时百分比图表\n磁盘读写速度趋势\n网络上下行流量与带宽统计\n硬件信息 — CPU 型号、内存 DDR 类型与频率\n每 10 秒自动刷新\n颜色标识健康状态，一目了然'],
  helpNetTitle: ['EasyTier — Virtual Networking', '组网 — EasyTier 虚拟组网'],
  helpNetDesc: ['Connect your phone, PC, and router into a virtual LAN — access them as if they\'re on the same local network, no public IP required.', '将手机、电脑、路由器组成虚拟局域网，无论身在何处都能像在同一网络下互相访问。无需公网 IP，无需复杂配置。'],
  helpNetFeat: ['Auto P2P direct connection — low latency, high speed\nAuto relay fallback when P2P is unavailable — always connected\nReal-time latency, packet loss, and bandwidth charts\nMulti-network profile management with one-tap switching\nColor-coded connection quality at a glance', '自动 P2P 直连，延迟低速度快\n无法直连时自动中转，确保稳定连接\n实时显示延迟、丢包率和网速图表\n支持多网络配置，一键切换\n颜色标识连接质量，一目了然'],
  helpLuciTitle: ['Router — LuCI Router Management', '路由器 — LuCI 路由器管理'],
  helpLuciDesc: ['Manage your OpenWrt router directly in the app — no browser needed. Automatic login with full LuCI admin panel access.', '在 App 内直接管理 OpenWrt 路由器，无需打开浏览器。自动完成登录，提供完整 LuCI 后台面板。'],
  helpLuciFeat: ['Auto login — configure once, no repeated password entry\nMulti-router support — add and switch between routers anytime\nOver EasyTier tunnel — manage home router remotely\nBrowsing position memory — restores last page after refresh', '自动登录，首次配置后无需每次输密码\n支持多台路由器，随时切换管理\n通过 EasyTier 隧道也能远程管理家里路由器\n刷新后自动恢复上次浏览位置'],
  helpBackupTitle: ['Import & Export', '配置导入导出'],
  helpBackupDesc: ['In Settings > Configuration Info, you can export all WOL devices, network profiles, and router configs to the clipboard with one tap (compressed ETGC:2:...), or import from clipboard to restore. Perfect for switching phones or sharing configs.', '在设置页的「配置信息」中，可将 WOL 设备、网络配置、路由器配置一键导出到剪贴板（压缩编码 ETGC:2:...），也可从剪贴板导入恢复。更换手机或分享给他人时无需重新配置。'],
  helpCfgTitle: ['Config Example', '配置示例'],
  helpWolParams: ['name — Device display name, customize freely\nmac — MAC address for sending WOL magic packets\nip — PC LAN IP for status detection and shutdown\ninterface — Router network interface for sending WOL (e.g. br-lan)\nrouter_ip — Router LAN IP for wake requests and path detection\nagent_port — wol-agent listening port (default 32249)\\ncommand_token — Bearer token for remote command execution, must match the agent config.toml\\ncommands — optional inline-table array of custom commands for this device, e.g. commands = [ { name = "Ping", command = "ping -n 4 127.0.0.1", mode = "INSTANT" } ] (mode INSTANT or LONG_RUNNING)', 'name — 设备显示名称，可自由设置\nmac — MAC 地址，用于发送 WOL 魔术包\nip — 电脑局域网 IP，用于状态检测和关机\ninterface — 路由器上发送 WOL 包的网口（如 br-lan）\nrouter_ip — 路由器 IP，唤醒请求目标和路径模式判断\nagent_port — wol-agent 监听端口（默认 32249）\\ncommand_token — 远程命令接口的 Bearer 令牌，需与电脑端 Agent 的 config.toml 一致\\ncommands — 可选，本设备的自定义命令内联表数组，如 commands = [ { name = "Ping", command = "ping -n 4 127.0.0.1", mode = "INSTANT" } ]（mode 为 INSTANT 或 LONG_RUNNING）'],
  helpWolSocks5: ['All WOL operations (wake, status check, shutdown) go through the SOCKS5 proxy when connected via EasyTier. Make sure your network config includes: socks5_proxy = "socks5://0.0.0.0:32259". The port can be customized but must match on both sides.', '通过 EasyTier 组网时，所有 WOL 操作（唤醒、状态检测、关机）均经过 SOCKS5 代理。请确保组网配置中包含：socks5_proxy = "socks5://0.0.0.0:32259"。端口可自行指定，但两端需保持一致。'],
  helpLuciParams: ['name — Router display name, customize freely\nip — Router LAN IP\nusername — OpenWrt login username (usually root)\npassword — OpenWrt login password', 'name — 路由器显示名称，可自由设置\nip — 路由器局域网 IP\nusername — OpenWrt 登录用户名（通常为 root）\npassword — OpenWrt 登录密码'],
  helpLuciSocks5: ['To access the router LuCI panel via EasyTier tunnel, make sure your network config includes: socks5_proxy = "socks5://0.0.0.0:32259". The port can be customized but must match on both sides.', '要通过 EasyTier 隧道访问路由器 LuCI 面板，请在组网配置中添加：socks5_proxy = "socks5://0.0.0.0:32259"。端口可自行指定，但两端需保持一致。'],
  helpNetParams: ['instance_name — Unique name for this network instance\nlisteners — Listening addresses and ports for peer connections\nnetwork_name — Shared identifier for all nodes in the same virtual network\nnetwork_secret — Shared secret key for encrypting communication\npeer.uri — Remote node address to connect to\nproxy_network.cidr — LAN subnet to proxy into the virtual network\nflags.enable_exit_node — Allow this node to act as an exit gateway\nsocks5_proxy — SOCKS5 proxy for WOL / LuCI tunnel access; port must match the router configuration', 'instance_name — 网络实例名称，用于区分多个网络\nlisteners — 监听地址和端口，接收其他节点连接\nnetwork_name — 同一虚拟网络所有节点共享的网络标识\nnetwork_secret — 加密通信的共享密钥\npeer.uri — 要连接的远程节点地址\nproxy_network.cidr — 代理到虚拟网络的局域网子网\nflags.enable_exit_node — 允许本节点作为出口网关\nsocks5_proxy — SOCKS5 代理端口，用于 WOL / LuCI 隧道访问；端口需与路由器配置一致'],
  theme: ['Theme', '主题'],
  themeAuto: ['Auto', '自动'],
  themeLight: ['Light', '浅色'],
  themeDark: ['Dark', '深色'],
  themeDesc: ['Switch between light, dark, or follow system', '切换浅色、深色外观或跟随系统'],
  aboutSub: ['Project introduction, acknowledgments, and open source license', '项目简介、致谢与开源许可信息'],
  aboutDesc: ['EasyTier + WOLPlus is a cross-platform virtual networking and remote device management app built on Tauri v2. It retains all EasyTier networking capabilities while deeply integrating WOL wake-on-LAN (via luci-app-wolplus), remote PC performance monitoring and shutdown (via wol-agent), LuCI router reverse proxy management, one-click config import/export, with comprehensive Material Design theming. Supports Windows, macOS, Linux desktop and Android mobile.', 'EasyTier + WOLPlus 是基于 Tauri v2 的跨平台异地组网与远程设备管理一体化应用。在保留 EasyTier 全部组网能力的基础上，深度整合了 WOL 网络唤醒（通过 luci-app-wolplus）、远程电脑性能监控与关机（通过 wol-agent）、LuCI 路由器反向代理管理、一键配置导入/导出等功能，并进行了全面的 Material Design 界面美化。支持 Windows、macOS、Linux 桌面端及 Android 移动端。'],
  aboutThanks: ['Acknowledgments', '致谢'],
  aboutEasyTierDesc: ['Decentralized virtual networking engine — high-performance Rust implementation, stable NAT traversal, multi-protocol support (TCP/UDP/WSS/WG)', '去中心化异地组网引擎 — 高性能 Rust 实现、稳定的 NAT 穿透能力、丰富的协议支持（TCP/UDP/WSS/WG）'],
  aboutWolplusDesc: ['OpenWrt WOL LuCI plugin — CGI API for sending etherwake magic packets to wake up PCs on LAN', 'OpenWrt WOL LuCI 插件，提供 CGI API 通过 etherwake 发送魔术包唤醒局域网电脑'],
  aboutLinks: ['Project Links', '项目地址'],
  aboutThisProject: ['EasyTier + WOLPlus (This Project)', 'EasyTier + WOLPlus（本项目）'],
  aboutWolplusDep: ['luci-app-wolplus (Dependency)', 'luci-app-wolplus（依赖）'],
  close: ['Close', '关闭'],
  copyCode: ['Copy', '复制'],
  copiedCode: ['Copied', '已复制'],
  etLan: ['Network', '组网'],
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
  protocols: ['Protocols', '协议'],
  loadingStats: ['Loading stats...', '加载中...'],
  retry: ['Retry', '重试'],
  copied: ['Copied', '已复制'],
  commandTokenRequired: ['Command credential not configured', '未配置命令凭据'],
  commandAuthFailed: ['Authentication failed. Check the token on both ends', '认证失败，请检查两端配置'],
  clipboardEmpty: ['Clipboard is empty', '剪贴板为空'],
  routerConnectFailed: ['Cannot connect to router {ip}', '无法连接到路由器 {ip}'],
  transferUpload: ['Upload', '上传'],
  transferDownload: ['Download', '下载'],
  transferUnknownFile: ['File', '文件'],
  transferStart: ['{action} started · {name}', '{action}开始 · {name}'],
  transferProgress: ['{action} · {name} · {value}', '{action} · {name} · {value}'],
  transferSuccess: ['{action} completed · {name} · {summary}', '{action}完成 · {name} · {summary}'],
  transferFailure: ['{action} failed · {name} · {error}', '{action}失败 · {name} · {error}'],
  transferSaved: ['Saved · {name}', '已保存 · {name}'],
  transferExternalStarted: ['System download started · {name}', '系统下载已开始 · {name}'],
  transferTimeout: ['Timed out', '操作超时'],
  transferHttpError: ['HTTP {code}', 'HTTP {code}'],
  transferUnavailable: ['File is no longer available', '文件已不可用'],
}
function tt(key: string): string {
  const entry = T[key]
  if (!entry)
    return key
  return locale.value === 'cn' ? entry[1] : entry[0]
}

function ttx(key: string, vars: Record<string, string | number>): string {
  return tt(key).replace(/\{(\w+)\}/g, (_, name: string) => String(vars[name] ?? `{${name}}`))
}

// --- Tabs ---
type Tab = 'wol' | 'net' | 'luci' | 'settings'
const defaultTab = ref<Tab>((localStorage.getItem('defaultTab') as Tab) || 'wol')
const activeTab = ref<Tab>(defaultTab.value)
const showDebug = ref(false)

const SETTINGS_PALETTE = [
  { tint: 'rgba(100,181,255,0.14)', tone: '#8ab4f8' },
  { tint: 'rgba(191,90,242,0.14)', tone: '#bf5af2' },
  { tint: 'rgba(255,159,10,0.14)', tone: '#ff9f0a' },
  { tint: 'rgba(100,181,255,0.14)', tone: '#8ab4f8' },
  { tint: 'rgba(48,209,88,0.14)', tone: '#30d158' },
  { tint: 'rgba(239,83,80,0.14)', tone: '#ef5350' },
  { tint: 'rgba(100,181,255,0.14)', tone: '#8ab4f8' },
  { tint: 'rgba(255,159,10,0.14)', tone: '#ff9f0a' },
]
const visibleSettingsKeys = computed(() => {
  const keys = ['home', 'theme', 'lang', 'debug']
  if (showDebug.value)
    keys.push('advanced')
  keys.push('config', 'help')
  return keys
})
function settingsIconStyle(key: string): string {
  const idx = visibleSettingsKeys.value.indexOf(key)
  const c = SETTINGS_PALETTE[idx] || SETTINGS_PALETTE[0]
  return `--tint:${c.tint};--tone:${c.tone}`
}

function switchTab(tab: Tab) {
  activeTab.value = tab
  if (tab === 'wol') { checkAllWolStatus(); manageStatsPoll() }
  else {
    if (statsTimer) { clearInterval(statsTimer); statsTimer = null }
  }
  if (tab === 'net' && netInstId.value)
    refreshNet()
  if (tab === 'luci') {
    if (!proxyUrl.value && luciRouters.value.length > 0 && !luciLoading.value) {
      luciIdx.value = luciIdx.value >= 0 ? luciIdx.value : 0
      startLuciProxy(luciRouters.value[luciIdx.value])
    }
    else if (proxyUrl.value) {
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
interface WolDevice { name: string, mac: string, ip: string, interface: string, router_ip: string, agent_port: number, command_token?: string, commands?: AppCommand[] }
interface SortedWolDevice extends WolDevice {
  deviceId: string
  status: WolDeviceStatus
  error: string
}

const wolDevices = ref<WolDevice[]>([])
const wolToml = ref(localStorage.getItem('wolDevicesToml') || '')
const showWolEditor = ref(false)
const wolStatus = reactive<Record<string, WolDeviceStatus>>({})
const wolErrors = reactive<Record<string, string>>({})
const wolRouters = reactive<Record<string, WolRouterStatus>>({})
const wolDebug = ref('')
const phaseTimers: Record<string, ReturnType<typeof setTimeout>> = {}
const deviceProbeTokens: Record<string, number> = {}
let nextDeviceProbeToken = 0
const checkingWol = ref(false)
let wolRecheckPending = false
const expandedDeviceIds = reactive(new Set<string>())
const statsHistory = reactive<Record<string, StatsSnapshot[]>>({})
const statsRaw = reactive<Record<string, StatsRaw | null>>({})
const statsError = reactive<Record<string, string>>({})
const statsLoading = reactive<Record<string, boolean>>({})
let statsTimer: ReturnType<typeof setInterval> | null = null
let wolPeriod: Utils.PeriodicTask | null = null
const WOL_STATUS_POLL_MS = 30_000
const WOL_STATUS_TIMEOUT_SECS = 3
const WOL_ROUTER_TIMEOUT_SECS = 4
const STATS_POLL_MS = 5_000
const STATS_FETCH_TIMEOUT_SECS = 8

function wolDeviceKey(w: Pick<WolDevice, 'mac' | 'ip' | 'agent_port'>) {
  return w.mac?.trim() || `${w.ip}:${w.agent_port || 32249}`
}

function ensureWolStatus(deviceId: string): WolDeviceStatus {
  if (!wolStatus[deviceId]) {
    wolStatus[deviceId] = createWolDeviceStatus()
    wolErrors[deviceId] = ''
  }
  return wolStatus[deviceId]
}

function ensureWolRouter(routerIp: string): WolRouterStatus {
  if (!wolRouters[routerIp])
    wolRouters[routerIp] = createWolRouterStatus()
  return wolRouters[routerIp]
}

function invalidateWolRouterChecks() {
  for (const routerIp of Object.keys(wolRouters)) {
    wolRouters[routerIp] = {
      ...wolRouters[routerIp],
      state: 'checking',
      consecutiveFailures: 0,
    }
  }
  manageStatsPoll()
}

function beginDeviceProbe(deviceId: string) {
  const token = ++nextDeviceProbeToken
  deviceProbeTokens[deviceId] = token
  wolStatus[deviceId] = markWolChecking(ensureWolStatus(deviceId))
  return token
}

function isLatestDeviceProbe(deviceId: string, token: number) {
  return deviceProbeTokens[deviceId] === token
}

// --- Stats types ---
interface StatsRaw {
  cpu_pct: number
  mem_pct: number
  gpu_pct: number | null
  gpu_mem_pct: number | null
  disks: { name: string, read: number, write: number }[]
  nets: { name: string, recv: number, sent: number }[]
  _ts: number
}

interface StatsSnapshot {
  ts: number
  cpu: number
  mem: number
  gpu: number | null
  gpuMem: number | null
  disks: { name: string, readBps: number, writeBps: number }[]
  nets: { name: string, recvBps: number, sentBps: number }[]
}

const STATS_MAX_POINTS = 60

function numericStatValue(value: unknown): number {
  if (typeof value === 'number')
    return Number.isFinite(value) ? value : 0
  if (typeof value === 'string' && value.trim() !== '') {
    const parsed = Number(value)
    return Number.isFinite(parsed) ? parsed : 0
  }
  return 0
}

function ensureStatsKeys(deviceId: string) {
  if (!statsHistory[deviceId])
    statsHistory[deviceId] = []
  if (!(deviceId in statsRaw))
    statsRaw[deviceId] = null
  if (!(deviceId in statsLoading))
    statsLoading[deviceId] = false
  if (!(deviceId in statsError))
    statsError[deviceId] = ''
}

function toggleDeviceExpand(w: WolDevice) {
  const deviceId = wolDeviceKey(w)
  if (expandedDeviceIds.has(deviceId)) {
    expandedDeviceIds.delete(deviceId)
  }
  else {
    expandedDeviceIds.add(deviceId)
    const status = ensureWolStatus(deviceId)
    const router = wolRouters[w.router_ip]
    if (w.ip && status.online && router?.state === 'reachable') {
      ensureStatsKeys(deviceId)
      if (statsHistory[deviceId].length === 0)
        fetchStats(deviceId, w.ip, w.agent_port || 32249)
    }
  }
  manageStatsPoll()
}

async function fetchStats(deviceId: string, ip: string, port: number) {
  ensureStatsKeys(deviceId)
  if (statsLoading[deviceId])
    return
  const proxy = getSocks5Proxy()
  statsLoading[deviceId] = true
  try {
    const text = await httpGet(`http://${ip}:${port}/api/v1/stats`, proxy, STATS_FETCH_TIMEOUT_SECS)
    const data = JSON.parse(text)
    statsInfo[deviceId] = data
    const prev = statsRaw[deviceId]
    const now = Date.now()

    const cpuPct = data.cpu?.usage_percent ?? 0
    const memPct = data.memory?.usage_percent ?? 0
    const gpuPct = data.gpu?.usage_percent ?? null
    const gpuMemPct = data.gpu?.vram_total_gb ? (gpuPct ?? 0) : null

    const curDisks: { name: string, read: number, write: number }[] = []
    if (data.disks) {
      for (const d of data.disks) {
        curDisks.push({
          name: d.name || '',
          read: numericStatValue(d.read_bytes),
          write: numericStatValue(d.write_bytes),
        })
      }
    }

    const curNets: { name: string, recv: number, sent: number }[] = []
    if (data.network) {
      for (const n of data.network) {
        curNets.push({
          name: n.name || '',
          recv: numericStatValue(n.recv_bytes),
          sent: numericStatValue(n.sent_bytes),
        })
      }
    }

    const diskSnaps: StatsSnapshot['disks'] = []
    for (const cd of curDisks) {
      let readBps = 0
      let writeBps = 0
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

    const netSnaps: StatsSnapshot['nets'] = []
    for (const cn of curNets) {
      let recvBps = 0
      let sentBps = 0
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

    statsRaw[deviceId] = {
      cpu_pct: cpuPct,
      mem_pct: memPct,
      gpu_pct: gpuPct,
      gpu_mem_pct: gpuMemPct,
      disks: curDisks,
      nets: curNets,
      _ts: now,
    }

    const snap: StatsSnapshot = {
      ts: now,
      cpu: cpuPct,
      mem: memPct,
      gpu: gpuPct,
      gpuMem: gpuMemPct,
      disks: diskSnaps,
      nets: netSnaps,
    }
    statsHistory[deviceId].push(snap)
    while (statsHistory[deviceId].length > STATS_MAX_POINTS) statsHistory[deviceId].shift()
    statsError[deviceId] = ''
  }
  catch (e: any) {
    statsError[deviceId] = cleanErr(e)
  }
  statsLoading[deviceId] = false
}

const statsInfo = reactive<Record<string, any>>({})

function expandedOnlineDevices(): SortedWolDevice[] {
  if (activeTab.value !== 'wol')
    return []
  return sortedWol.value.filter(w =>
    expandedDeviceIds.has(w.deviceId)
    && getWolEffectiveState(w) === 'online'
    && !!w.ip,
  )
}

function manageStatsPoll() {
  if (statsTimer) {
    clearInterval(statsTimer)
    statsTimer = null
  }
  const devices = expandedOnlineDevices()
  if (!devices.length)
    return

  for (const device of devices) {
    ensureStatsKeys(device.deviceId)
    if (statsHistory[device.deviceId].length === 0) {
      void fetchStats(device.deviceId, device.ip!, device.agent_port || 32249)
    }
  }

  statsTimer = setInterval(() => {
    const currentDevices = expandedOnlineDevices()
    if (!currentDevices.length) {
      if (statsTimer)
        clearInterval(statsTimer)
      statsTimer = null
      return
    }
    for (const device of currentDevices) {
      void fetchStats(device.deviceId, device.ip!, device.agent_port || 32249)
    }
  }, STATS_POLL_MS)
}

function resetDeviceStats(deviceId: string) {
  ensureStatsKeys(deviceId)
  statsHistory[deviceId].splice(0)
  statsRaw[deviceId] = null
  statsError[deviceId] = ''
}

function formatSpeedBps(v: number): string {
  const bps = v * 8
  if (bps < 1000)
    return `${bps.toFixed(0)} bps`
  if (bps < 1000000)
    return `${(bps / 1000).toFixed(0)} Kbps`
  if (bps < 1000000000)
    return `${(bps / 1000000).toFixed(1)} Mbps`
  return `${(bps / 1000000000).toFixed(1)} Gbps`
}

// Template helpers — read from cached statsInfo
function cpuModel(deviceId: string): string {
  return statsInfo[deviceId]?.cpu?.model || 'CPU'
}
function cpuCores(deviceId: string): number {
  return statsInfo[deviceId]?.cpu?.cores || 0
}
function memUsed(deviceId: string): string {
  return statsInfo[deviceId]?.memory?.used_gb?.toFixed(1) || '--'
}
function memTotal(deviceId: string): string {
  const v = statsInfo[deviceId]?.memory?.total_gb
  return v ? Math.round(v).toString() : '--'
}
function hasGPU(deviceId: string): boolean {
  return !!statsInfo[deviceId]?.gpu
}
function gpuModel(deviceId: string): string {
  return statsInfo[deviceId]?.gpu?.model || tt('gpuUsage')
}
function diskList(deviceId: string): any[] {
  const info = statsInfo[deviceId]
  if (!info?.disks)
    return []
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
  }).sort((a: any, b: any) => a.sortKey.localeCompare(b.sortKey))
}
function netList(deviceId: string): any[] {
  const info = statsInfo[deviceId]
  if (!info?.network)
    return []
  return info.network.map((n: any) => ({
    name: n.name,
    desc: n.desc || n.name,
  }))
}
function gbStr(v: number | undefined): string {
  if (v == null)
    return '--'
  return v.toFixed(v >= 1000 ? 0 : 1)
}

// Per-disk / per-NIC chart data helpers
// Dynamic Y-axis max — pure data maximum, recalculates as points fall out of window
function memGBHistory(deviceId: string): number[] {
  const totalGB = statsInfo[deviceId]?.memory?.total_gb || 1
  return statsHistory[deviceId]?.map(s => s.mem / 100 * totalGB) || []
}
function diskChartMax(deviceId: string, diskName: string): number {
  let m = 0
  for (const s of (statsHistory[deviceId] || [])) {
    const d = s.disks?.find(d => d.name === diskName)
    if (d) {
      if (d.readBps > m)
        m = d.readBps
      if (d.writeBps > m)
        m = d.writeBps
    }
  }
  return Math.max(m, 1)
}
function netChartMax(deviceId: string, netName: string): number {
  let m = 0
  for (const s of (statsHistory[deviceId] || [])) {
    const n = s.nets?.find(n => n.name === netName)
    if (n) {
      if (n.recvBps > m)
        m = n.recvBps
      if (n.sentBps > m)
        m = n.sentBps
    }
  }
  return Math.max(m, 1)
}

// Get current speed values for legend display
function curDiskSpeed(deviceId: string, diskName: string): { r: number, w: number } {
  const h = statsHistory[deviceId]
  if (!h?.length)
    return { r: 0, w: 0 }
  const last = h[h.length - 1]
  const d = last.disks?.find(d => d.name === diskName)
  return { r: d?.readBps || 0, w: d?.writeBps || 0 }
}
function curNetSpeed(deviceId: string, netName: string): { r: number, s: number } {
  const h = statsHistory[deviceId]
  if (!h?.length)
    return { r: 0, s: 0 }
  const last = h[h.length - 1]
  const n = last.nets?.find(n => n.name === netName)
  return { r: n?.recvBps || 0, s: n?.sentBps || 0 }
}
function curCpuPct(deviceId: string): number {
  const h = statsHistory[deviceId]
  if (!h?.length)
    return 0
  return h[h.length - 1].cpu
}

function curMemPct(deviceId: string): number {
  const h = statsHistory[deviceId]
  if (!h?.length)
    return 0
  return h[h.length - 1].mem
}
function dkTotalPct(deviceId: string, dk: { name: string, used: string, total: string }): number {
  const info = statsInfo[deviceId]
  if (!info?.disks)
    return 0
  const raw = info.disks.find((d: any) => d.name === dk.name)
  if (!raw || !raw.total_gb)
    return 0
  return Math.round(raw.used_gb / raw.total_gb * 100)
}
function diskFillColor(pct: number): string {
  if (pct >= 90)
    return '#ef5350'
  if (pct >= 80)
    return '#ff9800'
  return '#42a5f5'
}
function curGpuPct(deviceId: string): number {
  const h = statsHistory[deviceId]
  if (!h?.length)
    return 0
  return h[h.length - 1].gpu ?? 0
}

function memLabel(deviceId: string): string {
  const m = statsInfo[deviceId]?.memory
  if (!m)
    return tt('memory')
  let label = ''
  if (m.ddr_type)
    label += `${m.ddr_type} `
  label += `${Math.round(m.total_gb || 0)}GB`
  if (m.speed_mhz)
    label += ` ${m.speed_mhz}MHz`
  return label
}

function historyStartLabel(deviceId: string): string {
  const length = statsHistory[deviceId]?.length || 0
  if (length === 0)
    return locale.value === 'cn' ? '暂无数据' : 'No data'
  if (length === 1)
    return locale.value === 'cn' ? '现在' : 'Now'
  const seconds = (length - 1) * (STATS_POLL_MS / 1000)
  if (seconds < 60)
    return locale.value === 'cn' ? `${seconds} 秒前` : `${seconds}s ago`
  const minutes = Math.round(seconds / 60)
  return locale.value === 'cn' ? `${minutes} 分钟前` : `${minutes}m ago`
}

function formatChartPercent(value: number) {
  return `${Math.round(value)}%`
}

function formatChartMemory(value: number) {
  return `${value.toFixed(1)} GB`
}

function cpuChartSeries(deviceId: string) {
  return [{
    values: (statsHistory[deviceId] || []).map(item => item.cpu),
    color: 'var(--pc-series-1)',
  }]
}

function gpuChartSeries(deviceId: string) {
  return [{
    values: (statsHistory[deviceId] || []).map(item => item.gpu),
    color: 'var(--pc-series-1)',
  }]
}

function memGBChartSeries(deviceId: string) {
  return [{
    values: memGBHistory(deviceId),
    color: 'var(--pc-series-1)',
  }]
}

function diskChartSeries(deviceId: string, diskName: string) {
  const history = statsHistory[deviceId] || []
  return [
    {
      values: history.map(item => item.disks?.find(disk => disk.name === diskName)?.readBps ?? null),
      color: 'var(--pc-series-1)',
      label: locale.value === 'cn' ? '读取' : 'Read',
    },
    {
      values: history.map(item => item.disks?.find(disk => disk.name === diskName)?.writeBps ?? null),
      color: 'var(--pc-series-2)',
      label: locale.value === 'cn' ? '写入' : 'Write',
    },
  ]
}

function netChartSeries(deviceId: string, netName: string) {
  const history = statsHistory[deviceId] || []
  return [
    {
      values: history.map(item => item.nets?.find(net => net.name === netName)?.recvBps ?? null),
      color: 'var(--pc-series-1)',
      label: locale.value === 'cn' ? '下行' : 'Down',
    },
    {
      values: history.map(item => item.nets?.find(net => net.name === netName)?.sentBps ?? null),
      color: 'var(--pc-series-2)',
      label: locale.value === 'cn' ? '上行' : 'Up',
    },
  ]
}

function getSocks5Proxy(): string | undefined {
  if (!netRunning.value)
    return undefined
  const cfg = currentNet.value?.config as any
  if (cfg?.enable_socks5 && cfg?.socks5_port)
    return `socks5://127.0.0.1:${cfg.socks5_port}`
  return undefined
}

function parseWolToml(t: string): WolDevice[] {
  if (!t?.trim())
    return []
  const devs: WolDevice[] = []
  const lines = t.split('\n')
  let cur: Record<string, any> | null = null
  const finish = () => {
    if (cur?.mac)
      devs.push(cur as any); cur = null
  }
  for (let i = 0; i < lines.length; i++) {
    const tl = lines[i].trim()
    if (!tl || tl.startsWith('#'))
      continue
    if (tl === '[[device]]') { finish(); cur = {}; continue }
    if (cur && /^\[.+\]$/.test(tl)) { finish(); continue }
    if (!cur)
      continue
    const m = tl.match(/^(\w+)\s*=\s*(.+)$/)
    if (!m)
      continue
    if (m[1] === 'commands' && m[2].trim().startsWith('[')) {
      // gather the (possibly multi-line) inline-table array value
      let text = ''
      let depth = 0
      let quote: string | null = null
      let end = i
      for (let j = i; j < lines.length; j++) {
        const seg = lines[j]
        text += (j > i ? '\n' : '') + seg
        for (let k = 0; k < seg.length; k++) {
          const c = seg[k]
          if (quote) {
            if (quote === '"' && c === '\\') { k++; continue }
            if (c === quote)
              quote = null
            continue
          }
          if (c === '"' || c === '\'') { quote = c; continue }
          if (c === '[') {
            depth++
          }
          else if (c === ']') {
            depth--
            if (depth === 0) { end = j; break }
          }
        }
        if (depth === 0)
          break
      }
      const open = text.indexOf('[')
      const close = text.lastIndexOf(']')
      const entries = parseCommandsInline(open !== -1 && close > open ? text.slice(open, close + 1) : '')
      cur.commands = entries.map((entry, idx) => ({ ...entry, id: commandKeyFor(String(cur?.mac ?? ''), idx) }))
      i = end
      continue
    }
    let v: any = m[2].trim()
    if ((v.startsWith('"') && v.endsWith('"')) || (v.startsWith('\'') && v.endsWith('\'')))
      v = v.slice(1, -1)
    if (/^\d+$/.test(v))
      v = Number.parseInt(v)
    cur[m[1]] = v
  }
  finish()
  return devs
}

function loadWolConfig() {
  wolDevices.value = parseWolToml(wolToml.value)
  const validIds = new Set(wolDevices.value.map(wolDeviceKey))
  const validRouterIps = new Set(
    wolDevices.value.map(w => w.router_ip?.trim()).filter(Boolean),
  )
  for (const w of wolDevices.value) {
    const deviceId = wolDeviceKey(w)
    ensureWolStatus(deviceId)
    if (w.router_ip)
      ensureWolRouter(w.router_ip)
  }
  for (const id of [...expandedDeviceIds]) {
    if (!validIds.has(id))
      expandedDeviceIds.delete(id)
  }
  for (const id of Object.keys(wolStatus)) {
    if (validIds.has(id))
      continue
    delete wolStatus[id]
    delete wolErrors[id]
    delete deviceProbeTokens[id]
    delete statsHistory[id]
    delete statsRaw[id]
    delete statsInfo[id]
    delete statsError[id]
    delete statsLoading[id]
  }
  for (const routerIp of Object.keys(wolRouters)) {
    if (!validRouterIps.has(routerIp))
      delete wolRouters[routerIp]
  }
  manageStatsPoll()
}
function cleanErr(e: any): string {
  let s = String(e)
  s = s.replace(/^HTTP request failed: /, '')
  s = s.replace(/^error sending request for url \(https?:\/\/[^)]+\):?\s*/, '')
  return s || 'Connection failed'
}

async function checkRouterStatus(routerIp: string): Promise<boolean> {
  const proxy = netRunning.value ? getSocks5Proxy() : undefined
  if (netRunning.value && !proxy) {
    wolDebug.value = `${routerIp}: EasyTier SOCKS5 proxy is not configured`
    return false
  }

  const via = proxy ? `ET ${proxy}` : 'LAN'
  try {
    const text = await httpGet(
      `http://${routerIp}/cgi-bin/wolplus-api?action=status&ip=127.0.0.1`,
      proxy,
      WOL_ROUTER_TIMEOUT_SECS,
    )
    const payload = JSON.parse(text)
    if (!payload || typeof payload !== 'object' || (!('online' in payload) && !('error' in payload)))
      throw new Error('WOLPlus API returned an unexpected response')
    wolDebug.value = `${via} ${routerIp}: OK`
    return true
  }
  catch (e: any) {
    wolDebug.value = `${via} ${routerIp}: ${cleanErr(e)}`
    return false
  }
}

async function probeWolDevice(w: WolDevice, proxy?: string): Promise<boolean | null> {
  if (!w.ip)
    return null

  const deviceId = wolDeviceKey(w)
  const token = beginDeviceProbe(deviceId)
  try {
    const text = await httpGet(
      `http://${w.ip}:${w.agent_port || 32249}/api/v1/status`,
      proxy,
      WOL_STATUS_TIMEOUT_SECS,
    )
    const data = JSON.parse(text)
    if (!data || typeof data !== 'object' || !('online' in data))
      throw new Error('Agent returned an invalid status response')
    if (!isLatestDeviceProbe(deviceId, token))
      return null

    const previous = ensureWolStatus(deviceId)
    const next = applyWolSuccess(previous, data as WolStatusResponse, Date.now())
    wolStatus[deviceId] = next
    wolErrors[deviceId] = ''
    if (previous.online && !next.online)
      resetDeviceStats(deviceId)
    return next.online
  }
  catch (e: any) {
    if (!isLatestDeviceProbe(deviceId, token))
      return null
    const previous = ensureWolStatus(deviceId)
    const next = applyWolFailure(previous, Date.now())
    wolStatus[deviceId] = next
    wolErrors[deviceId] = cleanErr(e)
    if (previous.online && !next.online)
      resetDeviceStats(deviceId)
    return next.online
  }
}

async function checkAllWolStatus() {
  if (!wolDevices.value.length)
    return
  if (checkingWol.value) {
    wolRecheckPending = true
    return
  }

  checkingWol.value = true
  try {
    do {
      wolRecheckPending = false
      const routerIps = [...new Set(
        wolDevices.value.map(w => w.router_ip?.trim()).filter(Boolean),
      )]
      const routerResults = await Promise.all(routerIps.map(async (routerIp) => {
        const reachable = await checkRouterStatus(routerIp)
        const previous = ensureWolRouter(routerIp)
        wolRouters[routerIp] = reachable
          ? applyWolRouterSuccess(previous, Date.now())
          : applyWolRouterFailure(previous, Date.now())
        return [routerIp, reachable] as const
      }))
      const reachableRouters = new Set(
        routerResults.filter(([, reachable]) => reachable).map(([routerIp]) => routerIp),
      )
      const proxy = getSocks5Proxy()

      await Promise.all(wolDevices.value.map(async (w) => {
        if (!w.router_ip || !reachableRouters.has(w.router_ip))
          return
        await probeWolDevice(w, proxy)
      }))
      manageStatsPoll()
    } while (wolRecheckPending)
  }
  finally {
    checkingWol.value = false
    wolRecheckPending = false
  }
}
function openWolEditorFn() { wolToml.value = localStorage.getItem('wolDevicesToml') || ''; showWolEditor.value = true }
function saveWolConfig() { localStorage.setItem('wolDevicesToml', wolToml.value); loadWolConfig(); showWolEditor.value = false; checkAllWolStatus(); showSnack(tt('configSaved')) }

// --- Dialog textarea helpers ---
const wolTextareaRef = ref<any>(null)
const netTextareaRef = ref<any>(null)
async function readClipboard(): Promise<string> {
  try { return await readText() }
  catch { return '' }
}

async function importFromClipboard(refName: 'wol' | 'net' | 'luci') {
  const text = await readClipboard()
  if (!text) { showSnack(tt('clipboardEmpty')); return }
  if (refName === 'wol')
    wolToml.value = text
  else if (refName === 'net')
    editingNetToml.value = text
  else luciToml.value = text
}

function getWolEffectiveState(w: SortedWolDevice): WolEffectiveState {
  return effectiveWolState(
    w.router_ip ? wolRouters[w.router_ip] : undefined,
    w.status,
    !!w.router_ip,
    !!w.ip,
  )
}

function getWolPhaseLabel(w: SortedWolDevice): string {
  if (!w.router_ip)
    return tt('noRouterConfigured')
  switch (getWolEffectiveState(w)) {
    case 'checking':
      return tt('detecting')
    case 'routerOffline':
      return tt('routerOffline')
    case 'waking':
      return tt('waking')
    case 'shutting':
      return tt('shutting')
    case 'online':
      return tt('online')
    default:
      return w.ip ? tt('offline') : tt('noIpConfigured')
  }
}

function isWolBusyState(w: SortedWolDevice) {
  const state = getWolEffectiveState(w)
  return state === 'checking' || state === 'waking' || state === 'shutting'
}

function canWakeDevice(w: SortedWolDevice) {
  return getWolEffectiveState(w) === 'offline'
    && !!w.mac
}

function canShutdownDevice(w: SortedWolDevice) {
  return getWolEffectiveState(w) === 'online'
    && !!w.ip
}

function formatSystemUptime(seconds?: number) {
  if (!seconds || seconds <= 0)
    return '--'
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  if (locale.value === 'cn') {
    if (days)
      return `${days} 天 ${hours} 小时`
    if (hours)
      return `${hours} 小时 ${minutes} 分`
    return `${minutes} 分钟`
  }
  if (days)
    return `${days}d ${hours}h`
  if (hours)
    return `${hours}h ${minutes}m`
  return `${minutes}m`
}

async function doWake(w: WolDevice) {
  const deviceId = wolDeviceKey(w)
  resetDeviceStats(deviceId)
  wolStatus[deviceId] = { ...ensureWolStatus(deviceId), phase: 'waking' }
  const proxy = getSocks5Proxy()
  try {
    const url = `http://${w.router_ip}/cgi-bin/wolplus-api?action=wake&mac=${encodeURIComponent(w.mac)}&iface=${encodeURIComponent(w.interface || 'br-lan')}`
    const text = await httpGet(url, proxy)
    const data = JSON.parse(text)
    wolErrors[deviceId] = ''
    if (data.success !== true) {
      wolStatus[deviceId] = { ...ensureWolStatus(deviceId), phase: 'idle' }
      showSnack(
        data.message || (locale.value === 'cn' ? '唤醒请求失败' : 'Wake request failed'),
        'error',
      )
      return
    }
    showSnack(tt('signalSent'))
  }
  catch (e: any) {
    wolErrors[deviceId] = cleanErr(e)
    showSnack(e.message)
    wolStatus[deviceId] = { ...ensureWolStatus(deviceId), phase: 'idle' }
    return
  }
  if (w.ip)
    startPhasePoll(w, 'waking', 0)
  else
    wolStatus[deviceId] = { ...ensureWolStatus(deviceId), phase: 'idle' }
}

function doShutdown(w: WolDevice) {
  askConfirm({
    iconKind: 'power',
    title: locale.value === 'cn' ? `关闭 ${w.name}？` : `Shut down ${w.name}?`,
    subtitle: locale.value === 'cn' ? '设备上未保存的工作可能会丢失。' : 'Unsaved work on this device may be lost.',
    okText: locale.value === 'cn' ? '关机' : 'Shut down',
    danger: true,
    onOk: () => { void sendShutdown(w) },
  })
}

async function sendShutdown(w: WolDevice) {
  const deviceId = wolDeviceKey(w)
  wolStatus[deviceId] = { ...ensureWolStatus(deviceId), phase: 'shutting' }
  const proxy = getSocks5Proxy()
  try {
    const text = await httpPost(`http://${w.ip}:${w.agent_port || 32249}/api/v1/shutdown`, proxy)
    const data = JSON.parse(text)
    if (data?.success !== true)
      throw new Error(data?.message || 'Agent rejected the shutdown request')
    wolErrors[deviceId] = ''
  }
  catch (e: any) {
    wolErrors[deviceId] = cleanErr(e)
    wolStatus[deviceId] = { ...ensureWolStatus(deviceId), phase: 'idle' }
    showSnack(cleanErr(e), 'error')
    return
  }
  startPhasePoll(w, 'shutting', 0)
}

function startPhasePoll(w: WolDevice, phase: 'waking' | 'shutting', count: number) {
  const deviceId = wolDeviceKey(w)
  if (count >= 12) {
    wolStatus[deviceId] = { ...ensureWolStatus(deviceId), phase: 'idle' }
    return
  }
  const key = `${phase}_${deviceId}`
  phaseTimers[key] = setTimeout(async () => {
    if (wolRouters[w.router_ip]?.state !== 'reachable') {
      wolStatus[deviceId] = { ...ensureWolStatus(deviceId), phase: 'idle' }
      return
    }
    const token = beginDeviceProbe(deviceId)
    const proxy = getSocks5Proxy()
    try {
      const text = await httpGet(
        `http://${w.ip}:${w.agent_port || 32249}/api/v1/status`,
        proxy,
        WOL_STATUS_TIMEOUT_SECS,
      )
      const data = JSON.parse(text)
      const online = data.online === true
      if (!isLatestDeviceProbe(deviceId, token) || wolStatus[deviceId].phase !== phase)
        return
      const previous = ensureWolStatus(deviceId)
      wolStatus[deviceId] = applyWolSuccess(previous, data as WolStatusResponse, Date.now())
      wolErrors[deviceId] = ''
      if ((phase === 'waking' && online) || (phase === 'shutting' && !online)) {
        wolStatus[deviceId] = { ...wolStatus[deviceId], online, phase: 'idle' }
        if (!online)
          resetDeviceStats(deviceId)
        manageStatsPoll()
        showSnack(`${w.name} ${phase === 'waking' ? tt('online') : tt('offline')}`)
        return
      }
    }
    catch (e: any) {
      if (!isLatestDeviceProbe(deviceId, token) || wolStatus[deviceId].phase !== phase)
        return
      if (phase === 'shutting') {
        wolStatus[deviceId] = forceWolOffline(ensureWolStatus(deviceId), Date.now())
        wolErrors[deviceId] = ''
        resetDeviceStats(deviceId)
        showSnack(`${w.name} ${tt('offline')}`)
        return
      }
      wolErrors[deviceId] = cleanErr(e)
    }
    if (wolRouters[w.router_ip]?.state === 'reachable')
      startPhasePoll(w, phase, count + 1)
    else wolStatus[deviceId] = { ...ensureWolStatus(deviceId), phase: 'idle' }
  }, 5000)
}

function getWolPathType(): 'tunnel' | 'lan' {
  return netRunning.value ? 'tunnel' : 'lan'
}

const wolRouterIps = computed(() => [...new Set(
  wolDevices.value.map(w => w.router_ip?.trim()).filter(Boolean),
)])

const wolOnlineRouterCount = computed(() =>
  wolRouterIps.value.filter(routerIp => wolRouters[routerIp]?.state === 'reachable').length,
)

const sortedWol = computed<SortedWolDevice[]>(() => {
  const list = wolDevices.value.map((w) => {
    const deviceId = wolDeviceKey(w)
    return {
      ...w,
      deviceId,
      status: ensureWolStatus(deviceId),
      error: wolErrors[deviceId] || '',
    }
  })
  list.sort((a, b) => {
    const byIp = ipToNum(a.ip) - ipToNum(b.ip)
    return byIp || a.name.localeCompare(b.name)
  })
  return list
})

const wolOnlineCount = computed(() =>
  sortedWol.value.filter(w => getWolEffectiveState(w) === 'online').length,
)

// ==================== Computer commands ====================
const commandMenu = reactive({
  visible: false,
  deviceId: '',
  x: 0,
  y: 0,
})
const commandMenuRef = ref<HTMLElement | null>(null)
let commandMenuReturnFocus: HTMLElement | null = null
let longPressTimer: ReturnType<typeof setTimeout> | null = null
let longPressStart: { x: number, y: number } | null = null
let suppressNextDeviceClick = false

const execLoading = reactive({
  visible: false,
  deviceName: '',
  commandName: '',
})

const execSheet = reactive({
  visible: false,
  ok: false,
  deviceName: '',
  commandName: '',
  text: '',
})

const commandManager = reactive({
  visible: false,
  deviceId: '',
  name: '',
  command: '',
  mode: 'INSTANT' as CommandMode,
  editingIndex: -1,
  error: '',
})

const tasksSheet = reactive({
  visible: false,
  deviceId: '',
  deviceName: '',
  loading: false,
  error: '',
  tasks: [] as AgentTask[],
})
let tasksTimer: ReturnType<typeof setInterval> | null = null

const actionConfirm = reactive({
  visible: false,
  iconKind: 'bolt' as 'bolt' | 'dns' | 'power',
  title: '',
  subtitle: '',
  code: '',
  okText: '',
  danger: false,
  onOk: null as null | (() => void),
})

function askConfirm(opts: {
  iconKind: 'bolt' | 'dns' | 'power'
  title: string
  subtitle?: string
  code?: string
  okText: string
  danger?: boolean
  onOk: () => void
}) {
  actionConfirm.iconKind = opts.iconKind
  actionConfirm.title = opts.title
  actionConfirm.subtitle = opts.subtitle ?? ''
  actionConfirm.code = opts.code ?? ''
  actionConfirm.okText = opts.okText
  actionConfirm.danger = opts.danger ?? false
  actionConfirm.onOk = opts.onOk
  actionConfirm.visible = true
}

function closeActionConfirm() {
  actionConfirm.visible = false
  actionConfirm.onOk = null
}

function acceptActionConfirm() {
  const onOk = actionConfirm.onOk
  closeActionConfirm()
  onOk?.()
}

function commandAgentRequest(deviceId: string) {
  const device = sortedWol.value.find(item => item.deviceId === deviceId)
  if (!device?.ip)
    throw new Error(tt('noIpConfigured'))
  if (getWolEffectiveState(device) !== 'online')
    throw new Error(locale.value === 'cn' ? '设备当前不在线' : 'Device is not online')
  const token = device.command_token?.trim() || ''
  if (!token)
    throw new Error(tt('commandTokenRequired'))
  return {
    ip: device.ip,
    port: device.agent_port || 32249,
    proxy: getSocks5Proxy(),
    token,
  }
}

const commandMenuDevice = computed(() =>
  sortedWol.value.find(item => item.deviceId === commandMenu.deviceId) || null,
)

function canRunComputerCommands(w: SortedWolDevice) {
  return getWolEffectiveState(w) === 'online'
}

// One active polling execution at a time. Cancelling stops the loop and
// aborts the in-flight HTTP request through the Rust request registry.
interface CommandExecution {
  cancelled: boolean
  requestId: string | null
  timer: ReturnType<typeof setTimeout> | null
}
let activeExecution: CommandExecution | null = null

function stopActiveExecution() {
  if (!activeExecution)
    return
  activeExecution.cancelled = true
  if (activeExecution.timer) {
    clearTimeout(activeExecution.timer)
    activeExecution.timer = null
  }
  if (activeExecution.requestId) {
    void httpCancel(activeExecution.requestId)
    activeExecution.requestId = null
  }
}

function finishExecution() {
  execLoading.visible = false
  if (activeExecution)
    activeExecution.cancelled = true
  activeExecution = null
}

function cancelCommandExecution() {
  stopActiveExecution()
  finishExecution()
}

function openExecSheet(ok: boolean, deviceName: string, commandName: string, text: string) {
  execSheet.ok = ok
  execSheet.deviceName = deviceName
  execSheet.commandName = commandName
  execSheet.text = text
  execSheet.visible = true
}

async function copyExecSheet() {
  await writeText(execSheet.text)
  showSnack(tt('copied'))
}

async function runAgentCommand(w: SortedWolDevice, command: AppCommand) {
  if (activeExecution)
    return
  let request: ReturnType<typeof commandAgentRequest>
  try {
    request = commandAgentRequest(w.deviceId)
  }
  catch (e: any) {
    showSnack(cleanErr(e), 'error', 4500)
    return
  }
  closeCommandMenu(false)
  const execution: CommandExecution = { cancelled: false, requestId: null, timer: null }
  activeExecution = execution
  execLoading.deviceName = w.name
  execLoading.commandName = command.name
  execLoading.visible = true

  const longRunning = command.mode === 'LONG_RUNNING'
  const startedAt = Date.now()
  let taskId = ''
  let lastError = ''

  const schedule = () => {
    if (execution !== activeExecution || execution.cancelled)
      return
    execution.timer = setTimeout(() => {
      execution.timer = null
      void tick()
    }, 1000)
  }

  const tick = async () => {
    if (execution !== activeExecution || execution.cancelled)
      return
    const requestId = createUuid()
    execution.requestId = requestId
    try {
      const result = taskId
        ? await getAgentCommandStatus(request, taskId, requestId)
        : await executeAgentCommand(request, command, requestId)
      if (execution !== activeExecution || execution.cancelled)
        return
      execution.requestId = null
      if (!taskId && result.taskId)
        taskId = result.taskId
      if (result.status === 'SUCCESS') {
        finishExecution()
        if (longRunning) {
          showSnack(locale.value === 'cn' ? '命令已成功在后台启动' : 'Command started successfully in the background', 'success')
        }
        else {
          openExecSheet(true, w.name, command.name, result.output || (locale.value === 'cn' ? '（无输出）' : '(no output)'))
        }
        return
      }
      if (result.status === 'FAILED') {
        finishExecution()
        const reason = result.error || (locale.value === 'cn' ? '未知错误' : 'unknown error')
        if (longRunning)
          showSnack(reason, 'error', 5000)
        else openExecSheet(false, w.name, command.name, reason)
        return
      }
      lastError = ''
    }
    catch (e: any) {
      if (execution !== activeExecution || execution.cancelled)
        return
      execution.requestId = null
      if (e?.status === 401) {
        finishExecution()
        showSnack(tt('commandAuthFailed'), 'error', 5000)
        return
      }
      lastError = cleanErr(e)
    }
    const elapsed = Date.now() - startedAt
    if (longRunning) {
      // No FAILED within the 5s window counts as a successful background start.
      if (elapsed >= 5000) {
        finishExecution()
        if (!taskId && lastError)
          showSnack(lastError, 'error', 5000)
        else showSnack(locale.value === 'cn' ? '命令已成功在后台启动' : 'Command started successfully in the background', 'success')
        return
      }
      schedule()
      return
    }
    if (elapsed >= 30000) {
      finishExecution()
      showSnack(
        lastError || (locale.value === 'cn' ? '命令执行超时（30 秒），已停止轮询' : 'Command execution timed out (30s), polling stopped'),
        'error',
        5000,
      )
      return
    }
    schedule()
  }

  void tick()
}

function openCommandMenu(w: SortedWolDevice, event: MouseEvent | { clientX: number, clientY: number, currentTarget?: EventTarget | null }) {
  closeCommandMenu(false)
  commandMenu.deviceId = w.deviceId
  commandMenu.visible = true
  commandMenuReturnFocus = (event.currentTarget as HTMLElement | null) || null
  const target = event as MouseEvent
  commandMenu.x = target.clientX
  commandMenu.y = target.clientY
  void nextTick(() => {
    const menu = commandMenuRef.value
    if (!menu)
      return
    const rect = { width: menu.offsetWidth, height: menu.offsetHeight }
    const margin = 8
    commandMenu.x = Math.min(
      Math.max(margin, commandMenu.x),
      Math.max(margin, window.innerWidth - rect.width - margin),
    )
    // Expand downward by default; flip upward when close to the bottom edge.
    if (commandMenu.y + rect.height > window.innerHeight - margin) {
      const anchorTop = commandMenuReturnFocus?.getBoundingClientRect().top ?? commandMenu.y
      const above = anchorTop - rect.height - 8
      commandMenu.y = above >= margin
        ? above
        : Math.max(margin, window.innerHeight - rect.height - margin)
    }
    else {
      commandMenu.y = Math.max(margin, commandMenu.y)
    }
    const first = menu.querySelector<HTMLElement>('.pc-menu-item:not(:disabled)')
    first?.focus({ preventScroll: true })
  })
}

function openCommandMenuFromAnchor(w: SortedWolDevice, anchor: HTMLElement) {
  if (commandMenu.visible && commandMenu.deviceId === w.deviceId) {
    closeCommandMenu(true)
    return
  }
  const rect = anchor.getBoundingClientRect()
  openCommandMenu(w, {
    clientX: rect.right,
    clientY: rect.bottom + 4,
    currentTarget: anchor,
  })
}

function onCommandMenuKeydown(event: KeyboardEvent) {
  const menu = commandMenuRef.value
  if (!menu)
    return
  const items = Array.from(menu.querySelectorAll<HTMLButtonElement>('.pc-menu-item:not(:disabled)'))
  const currentIndex = items.indexOf(document.activeElement as HTMLButtonElement)
  const focusItem = (index: number) => {
    if (!items.length)
      return
    const normalized = ((index % items.length) + items.length) % items.length
    items[normalized]?.focus({ preventScroll: true })
  }

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault()
      focusItem(currentIndex + 1)
      break
    case 'ArrowUp':
      event.preventDefault()
      focusItem(currentIndex < 0 ? items.length - 1 : currentIndex - 1)
      break
    case 'Home':
      event.preventDefault()
      focusItem(0)
      break
    case 'End':
      event.preventDefault()
      focusItem(items.length - 1)
      break
    case 'Escape':
      event.preventDefault()
      closeCommandMenu()
      break
    case 'Tab':
      closeCommandMenu(false)
      break
  }
}

function closeCommandMenu(restoreFocus = true) {
  if (!commandMenu.visible)
    return
  commandMenu.visible = false
  commandMenu.deviceId = ''
  if (restoreFocus && commandMenuReturnFocus?.isConnected) {
    commandMenuReturnFocus.focus({ preventScroll: true })
  }
  commandMenuReturnFocus = null
}

function startDeviceLongPress(event: PointerEvent, w: SortedWolDevice) {
  if (event.pointerType !== 'touch' || event.button !== 0)
    return
  if ((event.target as Element | null)?.closest('button'))
    return
  cancelDeviceLongPress()
  longPressStart = { x: event.clientX, y: event.clientY }
  longPressTimer = setTimeout(() => {
    suppressNextDeviceClick = true
    openCommandMenu(w, event)
    longPressTimer = null
    longPressStart = null
  }, 500)
}

function moveDeviceLongPress(event: PointerEvent) {
  if (!longPressTimer || !longPressStart)
    return
  if (Math.hypot(event.clientX - longPressStart.x, event.clientY - longPressStart.y) > 10) {
    cancelDeviceLongPress()
  }
}

function cancelDeviceLongPress() {
  if (longPressTimer)
    clearTimeout(longPressTimer)
  longPressTimer = null
  longPressStart = null
}

function onDeviceHeadClick(w: SortedWolDevice) {
  if (suppressNextDeviceClick) {
    suppressNextDeviceClick = false
    return
  }
  toggleDeviceExpand(w)
}

function onDocumentCommandPointerDown(event: PointerEvent) {
  if (!commandMenu.visible)
    return
  const target = event.target as Node | null
  if (target && commandMenuRef.value?.contains(target))
    return
  if ((event.target as Element | null)?.closest?.('.pc-more'))
    return
  closeCommandMenu(false)
}

function confirmRunCommand(w: SortedWolDevice, command: AppCommand) {
  askConfirm({
    iconKind: command.mode === 'LONG_RUNNING' ? 'dns' : 'bolt',
    title: locale.value === 'cn' ? `在 ${w.name} 上执行「${command.name}」？` : `Run “${command.name}” on ${w.name}?`,
    subtitle: command.mode === 'LONG_RUNNING'
      ? (locale.value === 'cn' ? '命令将在电脑上后台长期运行，可在“后台任务”中查看或停止' : 'The command keeps running in the background; view or stop it under Background tasks')
      : (locale.value === 'cn' ? '命令将在电脑上立即执行，完成后显示输出' : 'The command runs immediately; the output is shown when it finishes'),
    code: command.command,
    okText: locale.value === 'cn' ? '执行' : 'Run',
    onOk: () => { void runAgentCommand(w, command) },
  })
}

const managerDevice = computed(() =>
  sortedWol.value.find(item => item.deviceId === commandManager.deviceId) || null,
)
const managerCommands = computed<AppCommand[]>(() => managerDevice.value?.commands ?? [])

function toRawCommandList(list: AppCommand[]): RawTomlCommand[] {
  return list.map(item => ({ name: item.name, command: item.command, mode: item.mode }))
}

function commitManagerCommands(list: RawTomlCommand[]) {
  const device = managerDevice.value
  if (!device?.mac) {
    commandManager.error = locale.value === 'cn' ? '设备缺少 mac 字段，无法保存命令' : 'Device has no mac field; cannot persist commands'
    return
  }
  const next = setDeviceCommandsInToml(wolToml.value, device.mac, list)
  wolToml.value = next
  localStorage.setItem('wolDevicesToml', next)
  loadWolConfig()
}

function managerFormValid() {
  return !!commandManager.name.trim() && !!commandManager.command.trim()
}

const managerDirty = computed(() =>
  commandManager.name !== '' || commandManager.command !== '' || commandManager.mode !== 'INSTANT',
)

function resetManagerForm() {
  commandManager.editingIndex = -1
  commandManager.name = ''
  commandManager.command = ''
  commandManager.mode = 'INSTANT'
  commandManager.error = ''
}

function openCommandManager(w: SortedWolDevice) {
  closeCommandMenu(false)
  commandManager.deviceId = w.deviceId
  resetManagerForm()
  commandManager.visible = true
}

function startEditCommand(command: AppCommand, index: number) {
  commandManager.editingIndex = index
  commandManager.name = command.name
  commandManager.command = command.command
  commandManager.mode = command.mode
  commandManager.error = ''
}

function submitManagerCommand() {
  if (!managerFormValid())
    return
  const entry: RawTomlCommand = {
    name: commandManager.name.trim(),
    command: commandManager.command.trim(),
    mode: commandManager.mode,
  }
  const current = managerCommands.value
  const list = commandManager.editingIndex >= 0
    ? current.map((item, idx) => (idx === commandManager.editingIndex
        ? entry
        : { name: item.name, command: item.command, mode: item.mode }))
    : [...toRawCommandList(current), entry]
  commitManagerCommands(list)
  if (!commandManager.error) {
    showSnack(tt('saved'))
    resetManagerForm()
  }
}

function deleteManagerCommand(index: number) {
  const list = managerCommands.value
    .filter((_, idx) => idx !== index)
    .map(item => ({ name: item.name, command: item.command, mode: item.mode }))
  if (commandManager.editingIndex === index)
    resetManagerForm()
  else if (commandManager.editingIndex > index)
    commandManager.editingIndex -= 1
  commitManagerCommands(list)
}

// ==================== Background tasks ledger ====================

function formatTaskElapsed(secs: number) {
  const total = Math.max(0, Math.floor(secs))
  const h = Math.floor(total / 3600)
  const m = Math.floor((total % 3600) / 60)
  const s = total % 60
  if (h)
    return locale.value === 'cn' ? `${h} 小时 ${m} 分` : `${h}h ${m}m`
  if (m)
    return locale.value === 'cn' ? `${m} 分 ${s} 秒` : `${m}m ${s}s`
  return locale.value === 'cn' ? `${total} 秒` : `${total}s`
}

async function refreshAgentTasks(silent = false) {
  if (!tasksSheet.deviceId)
    return
  let request: ReturnType<typeof commandAgentRequest>
  try {
    request = commandAgentRequest(tasksSheet.deviceId)
  }
  catch {
    tasksSheet.tasks = []
    return
  }
  try {
    if (!silent)
      tasksSheet.loading = true
    tasksSheet.tasks = await listAgentTasks(request)
    tasksSheet.error = ''
  }
  catch (e: any) {
    tasksSheet.error = e?.status === 401
      ? tt('commandAuthFailed')
      : (e?.status == null ? '' : cleanErr(e))
  }
  finally {
    tasksSheet.loading = false
  }
}

function stopTasksTimer() {
  if (tasksTimer) {
    clearInterval(tasksTimer)
    tasksTimer = null
  }
}

function openTasksSheet(w: SortedWolDevice) {
  closeCommandMenu(false)
  tasksSheet.deviceId = w.deviceId
  tasksSheet.deviceName = w.name
  tasksSheet.tasks = []
  tasksSheet.error = ''
  tasksSheet.visible = true
  void refreshAgentTasks()
  stopTasksTimer()
  tasksTimer = setInterval(() => {
    if (tasksSheet.visible)
      void refreshAgentTasks(true)
  }, 3000)
}

function closeTasksSheet() {
  tasksSheet.visible = false
  stopTasksTimer()
}

function confirmStopTask(task: AgentTask) {
  askConfirm({
    iconKind: task.mode === 'LONG_RUNNING' ? 'dns' : 'bolt',
    title: locale.value === 'cn' ? '停止该后台任务？' : 'Stop this background task?',
    subtitle: locale.value === 'cn'
      ? `${tasksSheet.deviceName} · 进程树将被强制结束（PID ${task.pid}）`
      : `${tasksSheet.deviceName} · the whole process tree will be killed (PID ${task.pid})`,
    code: task.command,
    okText: locale.value === 'cn' ? '停止' : 'Stop',
    danger: true,
    onOk: () => { void stopTaskNow(task) },
  })
}

async function stopTaskNow(task: AgentTask) {
  try {
    const res = await stopAgentTask(commandAgentRequest(tasksSheet.deviceId), task.taskId)
    if (res.status === 'FAILED')
      showSnack(res.error || (locale.value === 'cn' ? '停止失败' : 'Failed to stop task'), 'error', 4000)
    else
      showSnack(locale.value === 'cn' ? '已停止该任务' : 'Task stopped', 'success')
  }
  catch (e: any) {
    showSnack(e?.status === 401 ? tt('commandAuthFailed') : cleanErr(e), 'error', 4000)
  }
  void refreshAgentTasks(true)
}

// ==================== Network Tab ====================

// ==================== Network Tab ====================

interface StoredNet { config: NetworkTypes.NetworkConfig, source: string, rawToml: string }
const networks = ref<StoredNet[]>([])
const netIndex = ref(-1)
const netRunning = ref(false)
const netConnecting = ref(false)
const netConnectTime = ref(0)
const netSelfIp = ref('')
const netSelfNat = ref<any>(null)
const netIfaceIpv4 = ref<string[]>([])
const netIfaceIpv6 = ref<string[]>([])
const netPeers = ref<any[]>([])
const netServers = ref<any[]>([])
const netExpanded = reactive<Record<string, boolean>>({})
const tunnelCompact = reactive<Record<string, boolean>>({})
const netLoading = ref(false)
const showNetEditor = ref(false)
const editingNetToml = ref('')
const showSwitchMenu = ref(false)
const savingNet = ref(false)
let netPeriod: Utils.PeriodicTask | null = null
const showDeleteConfirm = ref(false)
const deleteTargetIdx = ref(-1)

function safeLastNetworkId(): string | null {
  try { return loadLastNetworkInstanceId() }
  catch { return localStorage.getItem('last_network_instance_id') }
}
function loadNetworks() {
  const raw = localStorage.getItem('networkList'); const list: StoredNet[] = JSON.parse(raw || '[]').filter((e: any) => e.config?.instance_id); networks.value = list; const lastId = safeLastNetworkId(); let idx = list.findIndex(e => e.config.instance_id === lastId); if (idx < 0 && list.length > 0)
    idx = 0; netIndex.value = idx
}
const currentNet = computed(() => netIndex.value >= 0 ? networks.value[netIndex.value] : null)
const netInstId = computed(() => currentNet.value?.config?.instance_id || '')
const netDiscovering = computed(() => netRunning.value && !netPeers.value.length && !netServers.value.length && netConnectTime.value > 0 && Date.now() - netConnectTime.value < 12000)
const connectBounce = ref(false)

async function fetchNetInfo() {
  if (!netInstId.value)
    return
  const previousRunning = netRunning.value
  const previousProxy = getSocks5Proxy()
  try {
    const resp = await collectNetworkInfo(netInstId.value)
    const infoMap = resp.info?.map || {}
    const info = infoMap[netInstId.value] || Object.values(infoMap)[0]
    if (info) {
      netRunning.value = info.running
      const my = info.my_node_info
      netSelfIp.value = formatIp(my?.virtual_ipv4)
      netSelfNat.value = my?.stun_info || null
      const ifv4s = my?.ips?.interface_ipv4s || []
      const ifv6s = my?.ips?.interface_ipv6s || []
      netIfaceIpv4.value = ifv4s.map((a: any) => formatIp(a)).filter(ip => ip && !ip.startsWith('127.') && !ip.startsWith('169.254.')).sort()
      netIfaceIpv6.value = ifv6s.map((a: any) => formatIpv6(a)).filter(ip => ip && ip !== '::1' && !ip.startsWith('fe80:') && !ip.startsWith('fd') && !ip.startsWith('fc')).sort()
      const all = (info.peer_route_pairs || []).filter((p: any) => p.route?.cost > 0)
      netPeers.value = all.filter((p: any) => !isServer(p))
      netServers.value = all.filter((p: any) => isServer(p))
    }
  }
  catch { /* */ }
  updateTrafficSpeed()
  scheduleProtocolFit()
  if (previousRunning !== netRunning.value || previousProxy !== getSocks5Proxy()) {
    invalidateWolRouterChecks()
    void checkAllWolStatus()
  }
}
async function refreshNet() { netLoading.value = true; await fetchNetInfo(); netLoading.value = false }

function doConnectWithBounce() {
  connectBounce.value = true
  setTimeout(() => {
    connectBounce.value = false
    doConnect()
  }, 350)
}

async function doConnect() {
  if (!currentNet.value || netConnecting.value)
    return; netConnecting.value = true; trafficHistory.splice(0, trafficHistory.length); lastTraffic = { up: 0, down: 0, ts: 0 }
  try {
    await runNetworkInstance(currentNet.value.config, true)
    saveLastNetworkInstanceId(currentNet.value.config.instance_id)
    await new Promise(r => setTimeout(r, 2000))
    await fetchNetInfo()
    if (netRunning.value) { netConnectTime.value = Date.now(); checkAllWolStatus() }
  }
  catch (e: any) { showSnack(String(e)) }
  finally { netConnecting.value = false }
}
async function doDisconnect() {
  try {
    await updateNetworkConfigState(netInstId.value, true)
    netRunning.value = false
    netConnectTime.value = 0
    invalidateWolRouterChecks()
    void checkAllWolStatus()
    showSnack(tt('disconnectedToast'))
  }
  catch (e: any) {
    showSnack(String(e))
  }
}

async function openNetEditor() {
  if (currentNet.value?.rawToml) { editingNetToml.value = currentNet.value.rawToml }
  else if (currentNet.value?.config) {
    try { editingNetToml.value = await parseNetworkConfig(currentNet.value.config) }
    catch { editingNetToml.value = '' }
  }
  else { editingNetToml.value = '' }
  showNetEditor.value = true
}
async function saveNetConfig() {
  if (!editingNetToml.value.trim())
    return; savingNet.value = true; try {
    const config = await generateNetworkConfig(editingNetToml.value); const raw = localStorage.getItem('networkList'); const list: StoredNet[] = JSON.parse(raw || '[]'); const idx = list.findIndex(e => e.config?.instance_id === config.instance_id); const isNew = idx < 0; if (idx >= 0)
      list[idx] = { config, source: 'user', rawToml: editingNetToml.value }; else list.push({ config, source: 'user', rawToml: editingNetToml.value }); localStorage.setItem('networkList', JSON.stringify(list)); loadNetworks(); if (isNew) {
      const newIdx = networks.value.findIndex(e => e.config.instance_id === config.instance_id); if (newIdx >= 0)
        switchToNet(newIdx)
    } showNetEditor.value = false; showSnack(tt('saved'))
  }
  catch (e: any) { showSnack(String(e)) }
  finally { savingNet.value = false }
}
function openAddNet() { editingNetToml.value = ''; showNetEditor.value = true; showSwitchMenu.value = false }
function switchToNet(idx: number) { netIndex.value = idx; netConnectTime.value = 0; saveLastNetworkInstanceId(networks.value[idx].config.instance_id); showSwitchMenu.value = false; refreshNet() }
function confirmDeleteNet(idx: number, e: MouseEvent) { e.stopPropagation(); deleteTargetIdx.value = idx; showDeleteConfirm.value = true; showSwitchMenu.value = false }
async function doDeleteNet() {
  const idx = deleteTargetIdx.value
  if (idx < 0 || idx >= networks.value.length)
    return
  const instId = networks.value[idx].config.instance_id
  try { await deleteNetworkInstance(instId) }
  catch { /* */ }
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
function formatIp(ip: any): string {
  if (!ip)
    return ''; if (typeof ip === 'string')
    return ip; if (ip.address?.addr !== undefined) { const a = ip.address.addr; return `${(a >>> 24) & 0xFF}.${(a >>> 16) & 0xFF}.${(a >>> 8) & 0xFF}.${a & 0xFF}` } return ''
}
function formatIpv6(ip: any): string {
  if (!ip)
    return ''
  if (typeof ip === 'string')
    return ip
  const p = [ip.part1, ip.part2, ip.part3, ip.part4]
  if (p.includes(undefined))
    return ''
  return p.map(x => `${(x >>> 16).toString(16)}:${(x & 0xFFFF).toString(16)}`).join(':')
}
function ipToNum(ip: string): number { const p = ip.split('.'); return p.length === 4 ? (+p[0] << 24) + (+p[1] << 16) + (+p[2] << 8) + (+p[3]) : 0 }
function costLabel(c: number | undefined): string { return c === undefined || c === 0 ? 'Local' : c === 1 ? 'P2P' : `Relay(${c})` }
function costCls(c: number | undefined): string {
  if (c === undefined || c === 0)
    return 'md-rt-l'; if (c === 1)
    return 'md-rt-p'; return 'md-rt-r'
}
function isServer(p: any): boolean { return p.route?.feature_flag?.is_public_server === true }
function peerKey(p: any): string { return `pn-${p.route?.peer_id}` }

function isRouterHost(hostname: string): boolean {
  if (!hostname)
    return false
  return /route|wrt|路由|home|家/i.test(hostname)
}

function displayHostname(hostname?: string): string {
  return (hostname || '').replace(/^PublicServer_/, '')
}

function tunnelSummary(types: unknown[], compact = false): string {
  const names = [...new Set(types.filter((t): t is string => typeof t === 'string' && !!t).map(t => t.toUpperCase()))]
  if (!names.length)
    return ''
  if (names.length === 1)
    return names[0]

  const prefixes = names.map(name => name.match(/^([A-Z0-9]+)-/)?.[1] || '')
  const prefix = prefixes[0] && prefixes.every(value => value === prefixes[0]) ? prefixes[0] : ''
  if (compact)
    return prefix ? `${prefix}:${names.length}` : `${names.length} Proto`

  const protocols = prefix ? names.map(name => name.slice(prefix.length + 1)) : names
  return prefix ? `${prefix}:${protocols.join('/')}` : protocols.join('/')
}

function formatBytes(b: number): string {
  const bytes = numericStatValue(b)
  if (!bytes || bytes < 0)
    return ''
  if (bytes < 1024)
    return `${bytes}B`
  if (bytes < 1048576)
    return `${(bytes / 1024).toFixed(0)}KB`
  if (bytes < 1073741824)
    return `${(bytes / 1048576).toFixed(1)}MB`
  if (bytes < 1099511627776)
    return `${(bytes / 1073741824).toFixed(1)}GB`
  return `${(bytes / 1099511627776).toFixed(1)}TB`
}

function qualityText(lat: string, loss: string): string {
  if (!lat)
    return ''
  const l = Number.parseFloat(loss) || 0
  if (l === 0)
    return lat
  return `${loss} · ${lat}`
}

function natTypeKey(value: unknown): string {
  if (typeof value === 'number')
    return String(value)
  if (typeof value !== 'string')
    return ''
  const trimmed = value.trim()
  return /^\d+$/.test(trimmed) ? trimmed : trimmed.toLowerCase().replace(/[^a-z0-9]/g, '')
}

function natLabel(s: any): { text: string, cls: string } {
  const t = natTypeKey(s?.udp_nat_type ?? s?.udpNatType)
  const m: Record<string, { text: string, cls: string }> = {
    1: { text: 'Open', cls: 'md-nat-g' },
    openinternet: { text: 'Open', cls: 'md-nat-g' },
    3: { text: 'NAT1', cls: 'md-nat-g' },
    fullcone: { text: 'NAT1', cls: 'md-nat-g' },
    4: { text: 'NAT2', cls: 'md-nat-g' },
    restricted: { text: 'NAT2', cls: 'md-nat-g' },
    5: { text: 'NAT3', cls: 'md-nat-b' },
    portrestricted: { text: 'NAT3', cls: 'md-nat-b' },
    6: { text: 'NAT4', cls: 'md-nat-o' },
    symmetric: { text: 'NAT4', cls: 'md-nat-o' },
  }
  return m[t] || { text: '', cls: '' }
}

function buildDetail(p: any): any {
  const r = p.route; const pr = p.peer; const d: any = { cost: r?.cost, host: r?.hostname, ver: r?.version }
  if (pr?.conns?.length) {
    let lat = 0; let loss = 0; let up = 0; let down = 0
    for (const c of pr.conns) {
      lat += numericStatValue(c.stats?.latency_us)
      loss += numericStatValue(c.loss_rate)
      up += numericStatValue(c.stats?.tx_bytes)
      down += numericStatValue(c.stats?.rx_bytes)
    }
    const n = pr.conns.length
    if (lat > 0) { d.lat = `${Math.round(lat / 1000 / n)}ms`; d.latNum = Math.round(lat / 1000 / n) }
    d.loss = n > 0 ? `${Math.round(loss / n * 100)}%` : ''
    if (up > 0)
      d.up = `↑${formatBytes(up)}`
    if (down > 0)
      d.down = `↓${formatBytes(down)}`
    const tunnelTypes = [...new Set(pr.conns.map((c: any) => c.tunnel?.tunnel_type || '?'))] as string[]
    d.tun = tunnelSummary(tunnelTypes)
    d.tunCompact = tunnelSummary(tunnelTypes, true)
    d.tunFull = tunnelTypes.map((t: string) => t.toUpperCase()).join(', ')
  }
  return d
}

function netPeerTitle(p: any, isServer: boolean): string {
  const hostname = displayHostname(p.route?.hostname)
  if (isServer)
    return hostname || tt('server')
  return hostname || formatIp(p.route?.ipv4_addr) || (locale.value === 'cn' ? '网络设备' : 'Network device')
}

function netPeerInlineIp(p: any): string {
  // 名称已回退为 IP 时不再重复展示行内 IP
  if (!displayHostname(p.route?.hostname))
    return ''
  return formatIp(p.route?.ipv4_addr)
}

function netPeerVersion(p: any): string {
  const version = String(buildDetail(p).ver || '').trim()
  if (!version)
    return '--'
  return /^v/i.test(version) ? version : `v${version}`
}

function tunnelDisplay(p: any): string {
  const d = buildDetail(p)
  return tunnelCompact[peerKey(p)] && d.tunCompact ? d.tunCompact : d.tun
}

let protocolFitFrame = 0
let protocolFitObserver: ResizeObserver | null = null
const observedProtocolRows = new WeakSet<HTMLElement>()

function scheduleProtocolFit() {
  if (protocolFitFrame)
    cancelAnimationFrame(protocolFitFrame)
  protocolFitFrame = requestAnimationFrame(() => {
    protocolFitFrame = 0
    updateProtocolFit()
  })
}

function measureChipWidth(element: HTMLElement, text: string): number {
  const probe = element.cloneNode(false) as HTMLElement
  probe.textContent = text
  probe.style.position = 'absolute'
  probe.style.visibility = 'hidden'
  probe.style.pointerEvents = 'none'
  probe.style.width = 'max-content'
  probe.style.minWidth = '0'
  probe.style.maxWidth = 'none'
  probe.style.flex = 'none'
  probe.style.overflow = 'visible'
  element.parentElement?.appendChild(probe)
  const width = probe.getBoundingClientRect().width
  probe.remove()
  return width
}

function updateProtocolFit() {
  const groups = Array.from(document.querySelectorAll<HTMLElement>('.md-card-tags[data-peer-key]'))
  for (const group of groups) {
    const key = group.dataset.peerKey
    const proto = group.querySelector('.md-chip-proto') as HTMLElement | null
    const row = group.closest('.md-row-bot, .net-peer-row-bottom') as HTMLElement | null
    if (!key || !proto || !row)
      continue

    if (protocolFitObserver && !observedProtocolRows.has(row)) {
      observedProtocolRows.add(row)
      protocolFitObserver.observe(row)
    }

    const d = buildDetail(netPeers.value.concat(netServers.value).find(p => peerKey(p) === key) || {})
    const full = d.tun
    const compact = d.tunCompact
    if (!full || !compact || full === compact) {
      delete tunnelCompact[key]
      continue
    }

    const children = Array.from(group.children) as HTMLElement[]
    const gap = Number.parseFloat(getComputedStyle(group).columnGap || getComputedStyle(group).gap || '0') || 0
    const fixedWidth = children.filter(child => child !== proto).reduce((sum, child) => sum + child.getBoundingClientRect().width, 0)
    const fullWidth = measureChipWidth(proto, full)
    const requiredWidth = fixedWidth + fullWidth + gap * Math.max(0, children.length - 1)
    const maxWidth = Number.parseFloat(getComputedStyle(group).maxWidth)
    const availableWidth = Number.isFinite(maxWidth) ? maxWidth : group.parentElement?.clientWidth || 0
    tunnelCompact[key] = requiredWidth > availableWidth + 0.5
  }
}

const sortedPeers = computed(() => {
  const list = [...netPeers.value]
  list.sort((a, b) => {
    const aRtr = isRouterHost(a.route?.hostname || '')
    const bRtr = isRouterHost(b.route?.hostname || '')
    if (aRtr !== bRtr)
      return aRtr ? -1 : 1
    return ipToNum(formatIp(a.route?.ipv4_addr)) - ipToNum(formatIp(b.route?.ipv4_addr))
  })
  return list
})

const sortedServers = computed(() => [...netServers.value].sort((a, b) => displayHostname(a.route?.hostname).localeCompare(displayHostname(b.route?.hostname))))

// Traffic speed tracking
const netTrafficSpeed = reactive({ up: '', down: '', upBytes: 0, downBytes: 0, upTotal: '', downTotal: '', totalBytes: 0 })
const trafficHistory = reactive<{ up: number, down: number }[]>([])
const TRAFFIC_MAX_POINTS = 40 // 2 min at 3s interval
let lastTraffic = { up: 0, down: 0, ts: 0 }
const TRAFFIC_UP_COLOR = 'var(--n3-good)'
const TRAFFIC_DOWN_COLOR = 'var(--n3-primary)'

function trafficChartMax(): number {
  return Math.max(1, ...trafficHistory.map(point => Math.max(point.up, point.down)))
}

function trafficChartSeries() {
  return [
    {
      values: trafficHistory.map(point => point.up),
      color: TRAFFIC_UP_COLOR,
      label: locale.value === 'cn' ? '上行' : 'Up',
    },
    {
      values: trafficHistory.map(point => point.down),
      color: TRAFFIC_DOWN_COLOR,
      label: locale.value === 'cn' ? '下行' : 'Down',
    },
  ]
}

function updateTrafficSpeed() {
  let up = 0; let down = 0
  for (const p of [...netPeers.value, ...netServers.value]) {
    const pr = p.peer
    if (pr?.conns?.length) {
      for (const c of pr.conns) {
        up += numericStatValue(c.stats?.tx_bytes)
        down += numericStatValue(c.stats?.rx_bytes)
      }
    }
  }
  netTrafficSpeed.upTotal = formatBytes(up)
  netTrafficSpeed.downTotal = formatBytes(down)
  netTrafficSpeed.totalBytes = up + down
  const now = Date.now()
  if (lastTraffic.ts > 0 && now > lastTraffic.ts) {
    const sec = (now - lastTraffic.ts) / 1000
    netTrafficSpeed.upBytes = Math.max(0, Math.round((up - lastTraffic.up) / sec))
    netTrafficSpeed.downBytes = Math.max(0, Math.round((down - lastTraffic.down) / sec))
    netTrafficSpeed.up = `${formatBytes(netTrafficSpeed.upBytes)}/s`
    netTrafficSpeed.down = `${formatBytes(netTrafficSpeed.downBytes)}/s`
    trafficHistory.push({ up: netTrafficSpeed.upBytes, down: netTrafficSpeed.downBytes })
    if (trafficHistory.length === 1) {
      trafficHistory.push({ up: netTrafficSpeed.upBytes, down: netTrafficSpeed.downBytes })
    }
    while (trafficHistory.length > TRAFFIC_MAX_POINTS) trafficHistory.shift()
  }
  lastTraffic = { up, down, ts: now }
}

function formatXLabel(total: number, idx: number): string {
  const sec = (total - 1 - idx) * 3
  if (sec === 0)
    return 'now'
  if (sec < 50)
    return `-${sec}s`
  const min = sec / 60
  // Round to nearest 0.5 to avoid floating point quirks
  const r = Math.round(min * 2) / 2
  const t = r.toFixed(1)
  return `-${t.endsWith('0') ? t.slice(0, -2) : t}m`
}

// ==================== Router Tab (LuCI proxy) ====================
interface LuciRouter { name: string, ip: string, username: string, password: string }
const luciToml = ref(localStorage.getItem('luciRoutersToml') || '')
const luciRouters = ref<LuciRouter[]>([])
const luciIdx = ref(-1)
const proxyUrl = ref('')
const luciIframeSrc = ref('')
const luciIframeKey = ref(0)
const luciIframeNonce = ref(0)
const luciLoading = ref(false)
const luciCurrentPath = ref('/cgi-bin/luci/admin/')
const showLuciEditor = ref(false)
const showLuciMenu = ref(false)
const lastLuciPathType = ref('')

function parseLuciToml(t: string): LuciRouter[] {
  if (!t?.trim())
    return []
  const devs: LuciRouter[] = []; const lines = t.split('\n'); let cur: Record<string, any> | null = null
  for (const line of lines) {
    const tl = line.trim(); if (!tl || tl.startsWith('#'))
      continue
    if (tl === '[[router]]') {
      if (cur?.ip)
        devs.push(cur as any); cur = {}; continue
    }
    if (cur) {
      const m = tl.match(/^(\w+)\s*=\s*(.+)$/); if (m) {
        let v: any = m[2].trim(); if ((v.startsWith('"') && v.endsWith('"')) || (v.startsWith('\'') && v.endsWith('\'')))
          v = v.slice(1, -1); cur[m[1]] = v
      }
    }
  }
  if (cur?.ip)
    devs.push(cur as any); return devs
}
function loadLuciConfig() {
  luciRouters.value = parseLuciToml(luciToml.value)
  if (luciIdx.value >= luciRouters.value.length)
    luciIdx.value = luciRouters.value.length - 1
}
const currentLuciRouter = computed(() => luciIdx.value >= 0 ? luciRouters.value[luciIdx.value] : null)

function luciRouterId(r: LuciRouter) {
  const input = `${r.ip}|${r.username}|${r.password}`
  let hash = 2166136261
  for (let i = 0; i < input.length; i++) {
    hash ^= input.charCodeAt(i)
    hash = Math.imul(hash, 16777619)
  }
  return `router-${(hash >>> 0).toString(16)}`
}

function buildLuciIframeUrl(path: string, routerId: string) {
  const separator = path.includes('?') ? '&' : '?'
  return `${proxyUrl.value}${path}${separator}__et_router=${encodeURIComponent(routerId)}&__et_nonce=${++luciIframeNonce.value}`
}

async function startLuciProxy(r: LuciRouter) {
  luciLoading.value = true
  luciIframeSrc.value = ''
  try {
    const socks5 = getSocks5Proxy()
    const routerId = luciRouterId(r)
    const url = await luciProxyStart(routerId, r.ip, r.username, r.password, socks5 || null)
    proxyUrl.value = url
    luciIframeSrc.value = buildLuciIframeUrl('/cgi-bin/luci/admin/', routerId)
    luciCurrentPath.value = '/cgi-bin/luci/admin/'
    lastLuciPathType.value = socks5 ? 'tunnel' : 'lan'
    luciIframeKey.value++
    localStorage.setItem('lastLuciRouterIp', r.ip)
  }
  catch (e: any) {
    showSnack(showDebug.value ? String(e) : ttx('routerConnectFailed', { ip: r.ip }), 'error', 3500)
    proxyUrl.value = ''
  }
  luciLoading.value = false
}

async function stopLuciProxyFn() {
  try { await luciProxyStop() }
  catch { /* */ }
  proxyUrl.value = ''
}

async function refreshLuciIframe() {
  const r = currentLuciRouter.value
  if (!r)
    return
  luciLoading.value = true
  luciIframeSrc.value = '' // hide iframe immediately, show spinner during restart
  try {
    const socks5 = getSocks5Proxy()
    const routerId = luciRouterId(r)
    const prevPath = await luciGetLastPath(routerId).catch(() => '/cgi-bin/luci/admin/')
    const url = await luciProxyStart(routerId, r.ip, r.username, r.password, socks5 || null)
    proxyUrl.value = url
    luciCurrentPath.value = prevPath
    luciIframeSrc.value = buildLuciIframeUrl(prevPath, routerId)
    lastLuciPathType.value = socks5 ? 'tunnel' : 'lan'
    luciIframeKey.value++
    localStorage.setItem('lastLuciRouterIp', r.ip)
  }
  catch (e: any) {
    showSnack(showDebug.value ? String(e) : ttx('routerConnectFailed', { ip: r.ip }), 'error', 3500)
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
    proxyUrl.value = ''
    luciIframeSrc.value = ''
    if (luciRouters.value.length > 0) {
      luciIdx.value = Math.min(idx, luciRouters.value.length - 1)
      startLuciProxy(luciRouters.value[luciIdx.value])
    }
    else {
      luciIdx.value = -1
    }
  }
  else if (idx < luciIdx.value) {
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
  if (r)
    await startLuciProxy(r)
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
  if (cur) {
    const idx = luciRouters.value.findIndex(r => r.ip === cur.ip)
    luciIdx.value = idx >= 0 ? idx : 0
    startLuciProxy(luciRouters.value[luciIdx.value])
  }
}

function onLuciDocClick(e: MouseEvent) {
  const t = e.target as HTMLElement
  if (!t.closest('.md-switch-menu') && !t.closest('.md-switch-btn'))
    showLuciMenu.value = false
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
  }
  catch {
    window.open(url, '_system')
  }
}

const HELP_CFG_WOL = `[[device]]
name = "My PC"
mac = "AA:BB:CC:DD:EE:FF"
ip = "192.168.1.100"
interface = "br-lan"
router_ip = "192.168.1.1"
agent_port = 32249
command_token = "shared-secret"
commands = [
  { name = "Ping", command = "ping -n 4 127.0.0.1", mode = "INSTANT" },
]`

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
  }
  catch {
    /* ignore */
  }
}

// ==================== Theme State Machine ====================
const themeMode = ref(localStorage.getItem('themeMode') || 'auto')

function applyTheme() {
  setTheme(resolveTheme(themeMode.value))
}

watch(themeMode, () => {
  localStorage.setItem('themeMode', themeMode.value)
  applyTheme()
})

// Follow system dark mode changes
if (typeof window !== 'undefined') {
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (themeMode.value === 'auto')
      applyTheme()
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
    const b0 = bytes[i]; const b1 = i + 1 < len ? bytes[i + 1] : 0; const b2 = i + 2 < len ? bytes[i + 2] : 0
    result += chars[(b0 >> 2) & 0x3F]
    result += chars[((b0 & 0x3) << 4) | ((b1 >> 4) & 0xF)]
    if (i + 1 < len)
      result += chars[((b1 & 0xF) << 2) | ((b2 >> 6) & 0x3)]
    if (i + 2 < len)
      result += chars[b2 & 0x3F]
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
    const c0 = lookup[str.charCodeAt(i)]; const c1 = i + 1 < len ? lookup[str.charCodeAt(i + 1)] : 0
    const c2 = i + 2 < len ? lookup[str.charCodeAt(i + 2)] : 0; const c3 = i + 3 < len ? lookup[str.charCodeAt(i + 3)] : 0
    bytes[j++] = (c0 << 2) | (c1 >> 4)
    if (i + 2 < len)
      bytes[j++] = ((c1 & 0xF) << 4) | (c2 >> 2)
    if (i + 3 < len)
      bytes[j++] = ((c2 & 0x3) << 6) | c3
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
    if (done)
      break
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
    if (done)
      break
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
    if (n.config) {
      try { netTomls.push(await parseNetworkConfig(n.config)) }
      catch { /* skip */ }
    }
  }
  const data = {
    v: 1,
    wol: wolToml.value,
    luci: luciToml.value,
    net: netTomls,
  }
  const json = JSON.stringify(data)
  const compressed = await compressDeflate(json)
  const encoded = `ETGC:2:${toBase64Url(compressed)}`
  try { await writeText(encoded) }
  catch { /* ignore */ }
  showSnack(tt('exportedToClipboard'))
}

function openImportDlg() {
  importText.value = ''
  showImportDlg.value = true
}

async function doImportFromClipboard() {
  const text = await readClipboard()
  if (!text) { showSnack(tt('clipboardEmpty')); return }
  importText.value = text
}

async function confirmImport() {
  let data: any
  try { data = await decodeExportData(importText.value) }
  catch { showSnack(tt('importFailed')); return }
  if (!data || typeof data.v !== 'number' || typeof data.wol !== 'string' || typeof data.luci !== 'string' || !Array.isArray(data.net) || !data.net.every((s: any) => typeof s === 'string')) {
    showSnack(tt('importFailed')); return
  }
  importPendingData.value = data
  showImportConfirm.value = true
}

async function executeImport() {
  const data = importPendingData.value
  if (!data)
    return
  showImportConfirm.value = false
  showImportDlg.value = false

  if (netRunning.value)
    await doDisconnect()

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
    try { await deleteNetworkInstance(n.config.instance_id) }
    catch { /* */ }
  }
  // Generate new network configs from TOML
  const newList: StoredNet[] = []
  for (const toml of data.net) {
    if (!toml.trim())
      continue
    try {
      const config = await generateNetworkConfig(toml)
      newList.push({ config, source: 'user', rawToml: toml })
    }
    catch { /* skip invalid */ }
  }
  localStorage.setItem('networkList', JSON.stringify(newList))
  loadNetworks()

  importPendingData.value = null
  showSnack(tt('importSuccess'))
}

function netName(): string { const n = currentNet.value; return n ? ((n.config as any).instance_name || n.config.network_name || tt('unnamed')) : tt('noNetwork') }

function onDocClick(e: MouseEvent) {
  const t = e.target as HTMLElement; if (!t.closest('.md-switch-menu') && !t.closest('.md-switch-btn'))
    showSwitchMenu.value = false
}

interface RouterTransferPayload {
  event: 'started' | 'progress' | 'finished' | 'failed'
  direction: 'upload' | 'download'
  path: string
  filename?: string | null
  error?: string | null
  transferred?: number | null
  content_length?: number | null
  duration_ms?: number | null
}

interface NativeTransferPayload {
  status: 'saved' | 'external-started' | 'failed'
  filename: string
  error?: string | null
}

const transferStates = new Map<string, { startedAt: number, lastShownAt: number, lastValue: number }>()

function formatTransferBytes(value?: number | null): string {
  if (!value || value <= 0)
    return ''
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = value
  let unit = 0
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024
    unit++
  }
  return `${size >= 10 || unit === 0 ? size.toFixed(0) : size.toFixed(1)} ${units[unit]}`
}

function formatTransferDuration(ms?: number | null): string {
  if (!ms || ms < 1000)
    return locale.value === 'cn' ? '不到 1 秒' : '< 1s'
  const seconds = Math.round(ms / 1000)
  if (seconds < 60)
    return locale.value === 'cn' ? `${seconds} 秒` : `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  const rest = seconds % 60
  return locale.value === 'cn' ? `${minutes} 分 ${rest} 秒` : `${minutes}m ${rest}s`
}

function transferName(payload: RouterTransferPayload | NativeTransferPayload): string {
  return payload.filename || ('path' in payload ? payload.path.split('/').pop() || payload.path : tt('transferUnknownFile'))
}

function transferAction(direction: 'upload' | 'download'): string {
  return tt(direction === 'upload' ? 'transferUpload' : 'transferDownload')
}

function transferError(error?: string | null): string {
  const raw = (error || '').trim()
  if (!raw)
    return tt('transferTimeout')
  if (/timed out|timeout/i.test(raw))
    return tt('transferTimeout')
  if (/no longer available/i.test(raw))
    return tt('transferUnavailable')
  const http = raw.match(/HTTP\s+(\d+)/i)
  if (http)
    return `${ttx('transferHttpError', { code: http[1] })} · ${raw.replace(/^.*?:\s*/, '')}`
  return raw
}

function handleRouterTransfer(payload: RouterTransferPayload) {
  const key = `${payload.direction}:${payload.path}`
  const name = transferName(payload)
  const action = transferAction(payload.direction)
  const now = Date.now()

  if (payload.event === 'started') {
    const state = transferStates.get(key)
    if (state && now - state.startedAt < 1000)
      return
    transferStates.set(key, { startedAt: now, lastShownAt: now, lastValue: 0 })
    const size = formatTransferBytes(payload.content_length)
    showSnack(ttx('transferStart', { action, name: size ? `${name} · ${size}` : name }), 'progress', 3500)
    return
  }

  if (payload.event === 'progress') {
    const state = transferStates.get(key) || { startedAt: now, lastShownAt: 0, lastValue: 0 }
    const value = payload.content_length
      ? Math.min(100, Math.round((payload.transferred || 0) * 100 / payload.content_length))
      : formatTransferBytes(payload.transferred)
    const numericValue = typeof value === 'number' ? value : state.lastValue
    if (now - state.lastShownAt < 2000 && numericValue - state.lastValue < 20)
      return
    transferStates.set(key, { ...state, lastShownAt: now, lastValue: numericValue })
    showSnack(ttx('transferProgress', { action, name, value: typeof value === 'number' ? `${value}%` : value }), 'progress', 3000)
    return
  }

  transferStates.delete(key)
  if (payload.event === 'finished') {
    const summary = [formatTransferBytes(payload.content_length || payload.transferred), formatTransferDuration(payload.duration_ms)].filter(Boolean).join(' · ')
    showSnack(ttx('transferSuccess', { action, name, summary: summary || 'OK' }), 'success', 4000)
  }
  else {
    showSnack(ttx('transferFailure', { action, name, error: transferError(payload.error) }), 'error', 5000)
  }
}

function handleNativeTransfer(payload: NativeTransferPayload) {
  const name = payload.filename || tt('transferUnknownFile')
  if (payload.status === 'saved')
    showSnack(ttx('transferSaved', { name }), 'success', 3500)
  else if (payload.status === 'external-started')
    showSnack(ttx('transferExternalStarted', { name }), 'progress', 3500)
  else showSnack(ttx('transferFailure', { action: tt('transferDownload'), name, error: transferError(payload.error) }), 'error', 5000)
}

function onNativeTransferEvent(event: Event) {
  const detail = (event as CustomEvent<NativeTransferPayload>).detail
  if (detail?.status && detail.filename)
    handleNativeTransfer(detail)
}

function closeCommandMenuOnViewportChange() {
  closeCommandMenu(false)
}

onMounted(async () => {
  applyTheme()
  window.addEventListener('resize', scheduleProtocolFit)
  protocolFitObserver = new ResizeObserver(scheduleProtocolFit)
  document.fonts?.ready.then(scheduleProtocolFit)

  if (type() === 'android') {
    try { await initMobileVpnService() }
    catch { /* */ }
    // Fork 适配层：迁移配置到 native 安全存储并注册状态，但保留 localStorage 键
    // （init_rpc_connection 等命令在 v2.7.1 的 Android 分支依赖该存储已初始化）
    try { await bootstrapAndroidStorageKeepLocal() }
    catch (e: any) { console.error('bootstrap android storage failed', e) }
  }
  cleanupFns.push(await listenGlobalEvents())
  cleanupFns.push(await listen<RouterTransferPayload>('router-transfer', ({ payload }) => handleRouterTransfer(payload)))
  window.addEventListener('easytier-native-transfer', onNativeTransferEvent)

  // 配置展示只依赖 localStorage，先于 RPC 初始化加载，避免后端异常导致列表空白
  loadWolConfig(); checkAllWolStatus(); wolPeriod = new Utils.PeriodicTask(checkAllWolStatus, WOL_STATUS_POLL_MS); wolPeriod.start()
  loadNetworks(); loadLuciConfig()
  try { await initRpcConnection(true) }
  catch (e: any) { console.error('init rpc connection failed', e) }
  if (netInstId.value)
    refreshNet(); netPeriod = new Utils.PeriodicTask(fetchNetInfo, 3000); netPeriod.start()
  if (activeTab.value === 'luci' && luciRouters.value.length > 0) {
    const lastIp = localStorage.getItem('lastLuciRouterIp')
    const idx = lastIp ? luciRouters.value.findIndex(r => r.ip === lastIp) : -1
    luciIdx.value = idx >= 0 ? idx : 0
    startLuciProxy(luciRouters.value[luciIdx.value])
  }
  document.addEventListener('click', onDocClick)
  document.addEventListener('click', onLuciDocClick)
  document.addEventListener('pointerdown', onDocumentCommandPointerDown, true)
  window.addEventListener('scroll', closeCommandMenuOnViewportChange, true)
  window.addEventListener('resize', closeCommandMenuOnViewportChange)
})
const cleanupFns: Array<() => void> = []

onUnmounted(() => {
  window.removeEventListener('easytier-native-transfer', onNativeTransferEvent)
  cleanupFns.forEach(fn => fn())
  wolPeriod?.stop()
  netPeriod?.stop()
  if (statsTimer)
    clearInterval(statsTimer)
  stopLuciProxyFn()
  for (const k of Object.keys(phaseTimers)) clearTimeout(phaseTimers[k])
  stopActiveExecution()
  closeTasksSheet()
  cancelDeviceLongPress()
  closeCommandMenu(false)
  if (snackTimer)
    clearTimeout(snackTimer)
  if (protocolFitFrame)
    cancelAnimationFrame(protocolFitFrame)
  protocolFitObserver?.disconnect()
  window.removeEventListener('resize', scheduleProtocolFit)
  window.removeEventListener('resize', closeCommandMenuOnViewportChange)
  window.removeEventListener('scroll', closeCommandMenuOnViewportChange, true)
  document.removeEventListener('click', onDocClick)
  document.removeEventListener('click', onLuciDocClick)
  document.removeEventListener('pointerdown', onDocumentCommandPointerDown, true)
})
</script>

<template>
  <div class="md-app">
    <!-- ==================== WOL Tab ==================== -->
    <div v-show="activeTab === 'wol'" class="md-page pc-page" :class="{ 'page-active': activeTab === 'wol' }">
      <header class="md-hdr">
        <span class="md-hdr-title">{{ tt('pcManagement') }}</span>
        <div class="flex-1" />
        <button class="md-settings-btn md-settings-btn-white" @click="openWolEditorFn">
          <svg width="18" height="18" viewBox="0 0 1024 1024" fill="currentColor"><path d="M714.965 128l-85.333 85.333H213.333v597.334h597.334V394.368L896 309.035v544.298A42.667 42.667 0 0 1 853.333 896H170.667A42.667 42.667 0 0 1 128 853.333V170.667A42.667 42.667 0 0 1 170.667 128h544.298z m159.062-38.4l60.373 60.416-392.192 392.192-60.245 0.128-0.086-60.459L874.027 89.6z" /></svg>
          {{ locale === 'cn' ? '编辑' : 'Edit' }}
        </button>
      </header>
      <div v-if="showDebug && wolDebug && sortedWol.length" class="px-4 py-1 text-xs opacity-60" style="color:var(--md-secondary);font-family:monospace;word-break:break-all;position:absolute;top:calc(var(--safe-top) + 56px);left:0;right:0;z-index:5">
        {{ wolDebug }}
      </div>
      <div class="md-scroll-view">
        <div class="md-scroll-content" :class="{ 'pc-content-center': !sortedWol.length }">
          <!-- WOL Hero Card -->
          <div v-if="sortedWol.length" class="md-card md-hero md-hero-purple">
            <div class="md-hero-head">
              <div class="flex-1 min-w-0">
                <div class="md-hero-title">
                  {{ tt('wolDevices') }}
                </div>
                <div class="md-hero-text">
                  {{ tt('wolDesc') }}
                </div>
              </div>
              <button class="md-settings-btn md-settings-btn-green" @click="checkAllWolStatus">
                <svg class="md-refresh-icon" :class="{ 'animate-spin': checkingWol }" width="18" height="18" viewBox="0 0 1024 1024" fill="currentColor"><path d="M128 170.666667l92.330667 92.330666A382.378667 382.378667 0 0 1 512 128c211.754667 0 384 172.288 384 384h-85.333333c0-164.693333-134.016-298.666667-298.666667-298.666667a297.557333 297.557333 0 0 0-231.125333 110.208L384 426.666667H128V170.666667z m768 682.666666v-256h-256l103.125333 103.125334A297.514667 297.514667 0 0 1 512 810.666667c-164.650667 0-298.666667-134.016-298.666667-298.666667H128c0 211.754667 172.245333 384 384 384a382.378667 382.378667 0 0 0 291.669333-134.997333L896 853.333333z" /></svg>
                {{ locale === 'cn' ? '刷新' : 'Refresh' }}
              </button>
            </div>
            <div class="md-hero-metrics">
              <div class="md-hero-metric">
                <div class="md-hero-mv">
                  {{ wolOnlineCount }} / {{ sortedWol.length }}
                </div>
                <div class="md-hero-ml">
                  {{ tt('pcs') }}
                </div>
              </div>
              <div class="md-hero-metric">
                <div class="md-hero-mv">
                  {{ wolRouterIps.length ? `${wolOnlineRouterCount} / ${wolRouterIps.length}` : '--' }}
                </div>
                <div class="md-hero-ml">
                  {{ locale === 'cn' ? '路由器' : 'Router' }}
                </div>
              </div>
              <div class="md-hero-metric">
                <div class="md-hero-mv">
                  {{ getWolPathType() === 'tunnel' ? (locale === 'cn' ? '组网' : 'Network') : (locale === 'cn' ? '局域网' : 'LAN') }}
                </div>
                <div class="md-hero-ml">
                  {{ tt('channel') }}
                </div>
              </div>
            </div>
          </div>

          <div v-if="!sortedWol.length" class="pc-empty-state">
            <div class="md-empty">
              <div class="md-empty-ico" aria-hidden="true">
                <svg width="44" height="44" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="5" width="16" height="14" rx="3" /><path d="M7 9h10" /><path d="M7 13h6" /></svg>
              </div>
              <div class="md-empty-text">
                {{ tt('emptyWol') }}
              </div>
            </div>
          </div>
          <div v-if="sortedWol.length" class="pc-list-head">
            <h2>{{ locale === 'cn' ? `设备 (${sortedWol.length})` : `Devices (${sortedWol.length})` }}</h2>
            <span class="pc-list-hint">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z" /></svg>
              {{ locale === 'cn' ? '右键设备卡片或点击' : 'Right-click a card or tap' }}
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z" /></svg>
              {{ locale === 'cn' ? '执行自定义命令' : 'to run custom commands' }}
            </span>
          </div>

          <div
            v-for="(w, i) in sortedWol"
            :key="w.deviceId"
            class="pc-card"
            :style="{ '--pc-i': i }"
            :class="{
              'pc-card-open': expandedDeviceIds.has(w.deviceId),
              'pc-card-dim': getWolEffectiveState(w) === 'routerOffline',
            }"
          >
            <article>
              <div
                class="pc-head sl"
                @click="onDeviceHeadClick(w)"
                @contextmenu.prevent="openCommandMenu(w, $event)"
                @pointerdown="startDeviceLongPress($event, w)"
                @pointermove="moveDeviceLongPress($event)"
                @pointerup="cancelDeviceLongPress"
                @pointercancel="cancelDeviceLongPress"
              >
                <div
                  class="pc-avatar"
                  :class="{ 'pc-avatar-online': getWolEffectiveState(w) === 'online' }"
                  aria-hidden="true"
                >
                  <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M21 2H3c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h7v2H8v2h8v-2h-2v-2h7c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H3V4h18v12z" /></svg>
                </div>

                <div class="pc-title">
                  <h3 class="pc-name" :title="w.name">
                    {{ w.name }}
                  </h3>
                  <div class="pc-meta">
                    <span class="pc-ip mono">{{ w.ip || tt('noIpConfigured') }}</span>
                    <span v-if="w.status.os" class="pc-os">{{ w.status.os }}</span>
                  </div>
                </div>

                <span
                  class="pc-status"
                  :class="`pc-status-${getWolEffectiveState(w)}`"
                  role="status"
                >
                  <span class="pc-status-dot" />
                  <span>{{ getWolPhaseLabel(w) }}</span>
                </span>

                <button
                  class="pc-icon-btn pc-power"
                  :class="{
                    'pc-power-on': canWakeDevice(w),
                    'pc-power-off': canShutdownDevice(w),
                  }"
                  type="button"
                  :disabled="!canWakeDevice(w) && !canShutdownDevice(w)"
                  :aria-pressed="canShutdownDevice(w)"
                  :aria-label="canShutdownDevice(w) ? `${tt('shutdown')}：${w.name}` : `${tt('wake')}：${w.name}`"
                  :title="canShutdownDevice(w) ? tt('shutdown') : tt('wake')"
                  @click.stop="canShutdownDevice(w) ? doShutdown(w) : doWake(w)"
                >
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M13 3h-2v10h2V3zm4.83 2.17-1.42 1.42C17.99 7.86 19 9.81 19 12c0 3.87-3.13 7-7 7s-7-3.13-7-7c0-2.19 1.01-4.14 2.58-5.42L6.17 5.17C4.23 6.82 3 9.26 3 12c0 4.97 4.03 9 9 9s9-4.03 9-9c0-2.74-1.23-5.18-3.17-6.83z" /></svg>
                </button>

                <button
                  class="pc-icon-btn pc-more"
                  type="button"
                  aria-haspopup="menu"
                  :aria-expanded="commandMenu.visible && commandMenu.deviceId === w.deviceId"
                  :aria-label="locale === 'cn' ? `${w.name} 的自定义命令` : `Custom commands for ${w.name}`"
                  :title="locale === 'cn' ? '自定义命令' : 'Custom commands'"
                  @click.stop="openCommandMenuFromAnchor(w, $event.currentTarget as HTMLElement)"
                >
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z" /></svg>
                </button>

                <button
                  class="pc-icon-btn pc-chevron"
                  type="button"
                  :aria-expanded="expandedDeviceIds.has(w.deviceId)"
                  :aria-label="expandedDeviceIds.has(w.deviceId) ? `${locale === 'cn' ? '收起' : 'Collapse'} ${w.name}` : `${locale === 'cn' ? '展开' : 'Expand'} ${w.name}`"
                  @click.stop="toggleDeviceExpand(w)"
                >
                  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="m6 9 6 6 6-6" /></svg>
                </button>
              </div>

              <div
                class="pc-body"
                :class="{ 'pc-body-open': expandedDeviceIds.has(w.deviceId) }"
                :inert="!expandedDeviceIds.has(w.deviceId)"
              >
                <div class="pc-body-inner">
                  <div class="pc-panel">
                    <div class="pc-info">
                      <span>{{ locale === 'cn' ? '系统' : 'System' }} <b>{{ w.status.os || '--' }}</b></span>
                      <span>{{ locale === 'cn' ? '已运行' : 'Uptime' }} <b>{{ formatSystemUptime(w.status.systemUptimeSecs) }}</b></span>
                      <span v-if="showDebug">{{ tt('macLabel') }} <b class="mono">{{ w.mac }}</b></span>
                    </div>

                    <div v-if="getWolEffectiveState(w) !== 'online'" class="pc-offline-state">
                      <svg v-if="isWolBusyState(w)" class="pc-busy-spinner" width="36" height="36" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 4V2A10 10 0 0 0 2 12h2a8 8 0 0 1 8-8z" /></svg>
                      <svg v-else width="40" height="40" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M21 2H3c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h7v2H8v2h8v-2h-2v-2h7c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H3V4h18v12zM8.7 13.3 11 11 8.7 8.7l1.4-1.4L12.4 9.6l2.3-2.3 1.4 1.4-2.3 2.3 2.3 2.3-1.4 1.4-2.3-2.3-2.3 2.3z" /></svg>
                      <p>{{ getWolPhaseLabel(w) }}</p>
                      <button v-if="canWakeDevice(w)" class="pc-tonal-btn pc-power-on" type="button" @click="doWake(w)">
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 22c1.1 0 2-.9 2-2h-4c0 1.1.89 2 2 2zm6-6v-5c0-3.07-1.64-5.64-4.5-6.32V4c0-.83-.67-1.5-1.5-1.5s-1.5.67-1.5 1.5v.68C7.63 5.36 6 7.92 6 11v5l-2 2v1h16v-1l-2-2z" /></svg>
                        {{ tt('wake') }}
                      </button>
                    </div>

                    <div v-else-if="statsLoading[w.deviceId] && !statsHistory[w.deviceId]?.length" class="pc-loading-state">
                      <i class="pi pi-spin pi-spinner" />
                      <span>{{ tt('loadingStats') }}</span>
                    </div>

                    <div v-else-if="statsError[w.deviceId] && !statsHistory[w.deviceId]?.length" class="pc-error-state">
                      <p>{{ statsError[w.deviceId] }}</p>
                      <button class="pc-tonal-btn" type="button" @click="fetchStats(w.deviceId, w.ip, w.agent_port || 32249)">
                        {{ tt('retry') }}
                      </button>
                    </div>

                    <div v-else class="pc-metrics">
                      <div v-if="statsError[w.deviceId]" class="pc-inline-error">
                        <span>{{ statsError[w.deviceId] }}</span>
                        <button class="pc-tonal-btn" type="button" @click="fetchStats(w.deviceId, w.ip, w.agent_port || 32249)">
                          {{ tt('retry') }}
                        </button>
                      </div>
                      <section class="pc-metric">
                        <div class="pc-metric-head">
                          <span class="pc-metric-icon" aria-hidden="true">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M15 9H9v6h6V9zm-2 4h-2v-2h2v2zm8-2V9h-2V7c0-1.1-.9-2-2-2h-2V3h-2v2H9V3H7v2H5c-1.1 0-2 .9-2 2v2H1v2h2v2H1v2h2v2c0 1.1.9 2 2 2h2v-2h2v2h2v-2h2v2h2c1.1 0 2-.9 2-2v-2h2v-2h-2v-2h2zm-4 6H7V7h10v10z" /></svg>
                          </span>
                          <div class="pc-metric-name">
                            <div class="pc-metric-label">
                              CPU
                            </div>
                            <div class="pc-metric-spec" :title="cpuModel(w.deviceId)">
                              {{ cpuModel(w.deviceId) }}<template v-if="cpuCores(w.deviceId)">
                                · {{ cpuCores(w.deviceId) }} {{ locale === 'cn' ? '核' : 'cores' }}
                              </template>
                            </div>
                          </div>
                          <div class="pc-metric-value">
                            {{ curCpuPct(w.deviceId).toFixed(0) }}<small>%</small>
                          </div>
                        </div>
                        <ComputerChart
                          :series="cpuChartSeries(w.deviceId)"
                          :max="100"
                          :name="locale === 'cn' ? 'CPU 使用率' : 'CPU usage'"
                          :format="formatChartPercent"
                          :area="true"
                          :x-start-label="historyStartLabel(w.deviceId)"
                          :sample-interval-secs="5"
                          :capacity="STATS_MAX_POINTS"
                          :locale="locale"
                        />
                      </section>

                      <section v-if="hasGPU(w.deviceId)" class="pc-metric">
                        <div class="pc-metric-head">
                          <span class="pc-metric-icon" aria-hidden="true">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M22 9V7h-2V5c0-1.1-.9-2-2-2H4c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-2h2v-2h-2v-2h2v-2h-2V9h2zm-4 10H4V5h14v14zM6 13h5v4H6v-4zm6-6h4v3h-4V7zM6 7h5v5H6V7zm6 4h4v6h-4v-6z" /></svg>
                          </span>
                          <div class="pc-metric-name">
                            <div class="pc-metric-label">
                              GPU
                            </div>
                            <div class="pc-metric-spec" :title="gpuModel(w.deviceId)">
                              {{ gpuModel(w.deviceId) }}
                            </div>
                          </div>
                          <div class="pc-metric-value">
                            {{ curGpuPct(w.deviceId).toFixed(0) }}<small>%</small>
                          </div>
                        </div>
                        <ComputerChart
                          :series="gpuChartSeries(w.deviceId)"
                          :max="100"
                          :name="locale === 'cn' ? 'GPU 使用率' : 'GPU usage'"
                          :format="formatChartPercent"
                          :area="true"
                          :x-start-label="historyStartLabel(w.deviceId)"
                          :sample-interval-secs="5"
                          :capacity="STATS_MAX_POINTS"
                          :locale="locale"
                        />
                      </section>

                      <section class="pc-metric">
                        <div class="pc-metric-head">
                          <span class="pc-metric-icon" aria-hidden="true">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M3 7h18c.55 0 1 .45 1 1v8c0 .55-.45 1-1 1h-1v2h-2v-2h-3v2h-2v-2H8v2H6v-2H3c-.55 0-1-.45-1-1V8c0-.55.45-1 1-1zm1 2v6h16V9H4zm2 1h2v4H6v-4zm4 0h2v4h-2v-4zm4 0h2v4h-2v-4z" /></svg>
                          </span>
                          <div class="pc-metric-name">
                            <div class="pc-metric-label">
                              {{ tt('memoryUsage') }}
                            </div>
                            <div class="pc-metric-spec" :title="memLabel(w.deviceId)">
                              {{ memLabel(w.deviceId) }}
                            </div>
                          </div>
                          <div class="pc-metric-value">
                            {{ curMemPct(w.deviceId).toFixed(0) }}<small>%</small>
                          </div>
                        </div>
                        <div class="pc-metric-sub">
                          <span>{{ locale === 'cn' ? '已用' : 'Used' }} <b>{{ memUsed(w.deviceId) }}G / {{ memTotal(w.deviceId) }}G</b></span>
                          <span>{{ locale === 'cn' ? '可用' : 'Available' }} <b>{{ Math.max(0, Number(memTotal(w.deviceId)) - Number(memUsed(w.deviceId))).toFixed(1) }}G</b></span>
                        </div>
                        <ComputerChart
                          :series="memGBChartSeries(w.deviceId)"
                          :max="Math.max(0.1, Number(memTotal(w.deviceId)) || 1)"
                          :name="locale === 'cn' ? '内存占用' : 'Memory usage'"
                          :format="formatChartMemory"
                          :area="true"
                          :x-start-label="historyStartLabel(w.deviceId)"
                          :sample-interval-secs="5"
                          :capacity="STATS_MAX_POINTS"
                          :locale="locale"
                        />
                      </section>

                      <section v-for="dk in diskList(w.deviceId)" :key="`disk-${dk.name}`" class="pc-metric">
                        <div class="pc-metric-head">
                          <span class="pc-metric-icon" aria-hidden="true">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M20 3H4c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H4V5h16v14zM6 7h5v5H6V7zm6 0h6v2h-6V7zm0 4h6v2h-6v-2z" /></svg>
                          </span>
                          <div class="pc-metric-name">
                            <div class="pc-metric-label">
                              {{ tt('disk') }}
                            </div>
                            <div class="pc-metric-spec" :title="dk.model || dk.name">
                              {{ dk.model || dk.name }}
                            </div>
                          </div>
                          <div class="pc-metric-value">
                            {{ dkTotalPct(w.deviceId, dk) }}<small>%</small>
                          </div>
                        </div>
                        <div class="pc-metric-sub pc-disk-sub">
                          <span>{{ dk.used }}GB / {{ dk.total }}GB</span>
                          <span class="pc-usage-bar"><i :style="{ width: `${dkTotalPct(w.deviceId, dk)}%` }" /></span>
                          <span class="pc-key" style="--pc-key: var(--pc-series-1)">{{ locale === 'cn' ? '读取' : 'Read' }} <b>{{ formatSpeedBps(curDiskSpeed(w.deviceId, dk.name).r) }}</b></span>
                          <span class="pc-key" style="--pc-key: var(--pc-series-2)">{{ locale === 'cn' ? '写入' : 'Write' }} <b>{{ formatSpeedBps(curDiskSpeed(w.deviceId, dk.name).w) }}</b></span>
                        </div>
                        <ComputerChart
                          :series="diskChartSeries(w.deviceId, dk.name)"
                          :max="diskChartMax(w.deviceId, dk.name)"
                          :name="locale === 'cn' ? '硬盘读写速度' : 'Disk throughput'"
                          :format="formatSpeedBps"
                          :x-start-label="historyStartLabel(w.deviceId)"
                          :sample-interval-secs="5"
                          :capacity="STATS_MAX_POINTS"
                          :locale="locale"
                        />
                        <div v-if="dk.parts?.length" class="pc-partitions">
                          <div v-for="part in dk.parts" :key="part.name" class="pc-partition">
                            <span>{{ part.name }}</span>
                            <span class="pc-usage-bar"><i :style="{ width: `${part.pct}%`, background: diskFillColor(part.pct) }" /></span>
                            <span>{{ part.used }}GB / {{ part.total }}GB</span>
                          </div>
                        </div>
                      </section>

                      <section v-for="nc in netList(w.deviceId)" :key="`net-${nc.name}`" class="pc-metric">
                        <div class="pc-metric-head">
                          <span class="pc-metric-icon" aria-hidden="true">
                            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z" /></svg>
                          </span>
                          <div class="pc-metric-name">
                            <div class="pc-metric-label">
                              {{ locale === 'cn' ? '网络' : 'Network' }}
                            </div>
                            <div class="pc-metric-spec" :title="nc.desc">
                              {{ nc.desc }}
                            </div>
                          </div>
                          <div class="pc-metric-value">
                            {{ formatSpeedBps(curNetSpeed(w.deviceId, nc.name).r) }}
                          </div>
                        </div>
                        <div class="pc-metric-sub">
                          <span class="pc-key" style="--pc-key: var(--pc-series-1)">{{ locale === 'cn' ? '下行' : 'Down' }} <b>{{ formatSpeedBps(curNetSpeed(w.deviceId, nc.name).r) }}</b></span>
                          <span class="pc-key" style="--pc-key: var(--pc-series-2)">{{ locale === 'cn' ? '上行' : 'Up' }} <b>{{ formatSpeedBps(curNetSpeed(w.deviceId, nc.name).s) }}</b></span>
                        </div>
                        <ComputerChart
                          :series="netChartSeries(w.deviceId, nc.name)"
                          :max="netChartMax(w.deviceId, nc.name)"
                          :name="locale === 'cn' ? '网络流量' : 'Network traffic'"
                          :format="formatSpeedBps"
                          :x-start-label="historyStartLabel(w.deviceId)"
                          :sample-interval-secs="5"
                          :capacity="STATS_MAX_POINTS"
                          :locale="locale"
                        />
                      </section>
                    </div>

                    <div v-if="showDebug" class="pc-debug-info">
                      {{ w.mac }} · {{ w.router_ip }} · {{ w.interface || 'br-lan' }} · {{ w.agent_port || 32249 }}
                    </div>

                    <button class="pc-collapse-btn" type="button" @click="toggleDeviceExpand(w)">
                      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                        <path d="m18 15-6-6-6 6" />
                      </svg>
                      <span>{{ locale === 'cn' ? '收起' : 'Collapse' }}</span>
                    </button>
                  </div>
                </div>
              </div>
            </article>
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="commandMenu.visible && commandMenuDevice"
      ref="commandMenuRef"
      class="pc-menu"
      role="menu"
      :style="{ left: `${commandMenu.x}px`, top: `${commandMenu.y}px` }"
      :aria-label="locale === 'cn' ? `${commandMenuDevice.name} 的自定义命令` : `Custom commands for ${commandMenuDevice.name}`"
      @keydown="onCommandMenuKeydown"
    >
      <div class="pc-menu-label">
        {{ locale === 'cn' ? `在 ${commandMenuDevice.name} 上执行` : `Run on ${commandMenuDevice.name}` }}
      </div>

      <div v-if="!canRunComputerCommands(commandMenuDevice)" class="pc-menu-note">
        {{ locale === 'cn' ? '设备未在线，暂不能执行命令' : 'Device is offline, commands unavailable' }}
      </div>

      <div v-if="!(commandMenuDevice.commands || []).length" class="pc-menu-note">
        {{ locale === 'cn' ? '还没有自定义命令' : 'No custom commands yet' }}
      </div>

      <button
        v-for="(command, cIdx) in commandMenuDevice.commands || []"
        :key="command.id || cIdx"
        class="pc-menu-item sl"
        type="button"
        role="menuitem"
        :disabled="!canRunComputerCommands(commandMenuDevice)"
        :title="command.command"
        @click="confirmRunCommand(commandMenuDevice, command)"
      >
        <span class="pc-menu-icon" aria-hidden="true">
          <svg v-if="command.mode === 'LONG_RUNNING'" width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M7.5 6q-.625 0-1.062.438T6 7.5t.438 1.063T7.5 9t1.063-.437T9 7.5t-.437-1.062T7.5 6m0 10q-.625 0-1.062.438T6 17.5t.438 1.063T7.5 19t1.063-.437T9 17.5t-.437-1.062T7.5 16M4 3h16q.425 0 .713.288T21 4v7q0 .425-.288.713T20 12H4q-.425 0-.712-.288T3 11V4q0-.425.288-.712T4 3m0 10h16q.425 0 .713.288T21 14v7q0 .425-.288.713T20 22H4q-.425 0-.712-.288T3 21v-7q0-.425.288-.712T4 13" /></svg>
          <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M9 15H5.9q-.6 0-.888-.537t.063-1.038l7.475-10.75q.25-.35.65-.487t.825.012t.625.525t.15.8L14 10h3.875q.65 0 .913.575t-.163 1.075L10.4 21.5q-.275.325-.675.425t-.775-.075t-.587-.537t-.163-.788z" /></svg>
        </span>
        <span class="pc-menu-text">
          <span>{{ command.name }}</span>
          <small class="mono">{{ command.command }}</small>
        </span>
      </button>

      <div class="pc-menu-divider" role="separator" />
      <button
        class="pc-menu-item sl"
        type="button"
        role="menuitem"
        :disabled="!canRunComputerCommands(commandMenuDevice)"
        @click="openTasksSheet(commandMenuDevice)"
      >
        <span class="pc-menu-icon" aria-hidden="true">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M5 21q-.825 0-1.412-.587T3 19V5q0-.825.588-1.412T5 3h4.2q.325-.9 1.088-1.45T12 1t1.713.55T14.8 3H19q.825 0 1.413.588T21 5v14q0 .825-.587 1.413T19 21zm3-4h5q.425 0 .713-.288T14 16t-.288-.712T13 15H8q-.425 0-.712.288T7 16t.288.713T8 17m0-4h8q.425 0 .713-.288T17 12t-.288-.712T16 11H8q-.425 0-.712.288T7 12t.288.713T8 13m0-4h8q.425 0 .713-.288T17 8t-.288-.712T16 7H8q-.425 0-.712.288T7 8t.288.713T8 9m4.538-4.962q.212-.213.212-.538t-.213-.537T12 2.75t-.537.213t-.213.537t.213.538t.537.212t.538-.213" /></svg>
        </span>
        <span>{{ locale === 'cn' ? '后台任务…' : 'Background tasks…' }}</span>
      </button>
      <button class="pc-menu-item sl" type="button" role="menuitem" @click="openCommandManager(commandMenuDevice)">
        <span class="pc-menu-icon" aria-hidden="true">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M4 21q-.425 0-.712-.288T3 20v-2.425q0-.4.15-.763t.425-.637L16.2 3.575q.3-.275.663-.425t.762-.15t.775.15t.65.45L20.425 5q.3.275.437.65T21 6.4q0 .4-.138.763t-.437.662l-12.6 12.6q-.275.275-.638.425t-.762.15zM17.6 7.8L19 6.4L17.6 5l-1.4 1.4z" /></svg>
        </span>
        <span>{{ locale === 'cn' ? '管理自定义命令…' : 'Manage custom commands…' }}</span>
      </button>
    </div>

    <!-- ==================== Network Tab ====================    <!-- ==================== Network Tab ==================== -->
    <div v-show="activeTab === 'net'" class="md-page md-page-bg" :class="{ 'page-active': activeTab === 'net' }">
      <header class="md-hdr">
        <template v-if="netRunning">
          <span class="md-hdr-title">{{ locale === 'cn' ? '组网' : 'Network' }}</span>
          <div class="flex-1" />
          <button class="md-settings-btn md-settings-btn-red" @click="doDisconnect">
            <svg width="18" height="18" viewBox="0 0 1024 1024" fill="currentColor"><path d="M797.248 111.936a210.944 210.944 0 0 1 114.816 114.816c10.624 25.728 16 53.312 15.936 81.216a210.624 210.624 0 0 1-61.888 150.016L744.32 580.48a11.264 11.264 0 0 1-8.256 3.328 11.2 11.2 0 0 1-8.256-3.328l-56.768-56.768a11.264 11.264 0 0 1-3.392-8.32 11.264 11.264 0 0 1 3.392-8.192l122.24-122.24a109.12 109.12 0 0 0-118.848-177.792 108.288 108.288 0 0 0-35.392 23.68L517.12 353.088a11.392 11.392 0 0 1-8.256 3.328 11.52 11.52 0 0 1-8.256-3.328l-56.768-56.832a11.456 11.456 0 0 1 0-16.128l122.176-122.24a210.88 210.88 0 0 1 150.016-61.888c27.84-0.128 55.488 5.312 81.28 15.936zM299.584 217.664a31.04 31.04 0 0 0-44.224 0l-36.224 36.096a31.168 31.168 0 0 0-6.784 34.24 31.36 31.36 0 0 0 6.784 10.24l488.384 488.32a30.976 30.976 0 0 0 34.24 6.848 30.912 30.912 0 0 0 10.176-6.848l36.16-36.16a31.36 31.36 0 0 0 0-44.416l-488.512-488.32z m224 451.136a11.2 11.2 0 0 0-8.32-3.392 11.328 11.328 0 0 0-8.192 3.392l-122.24 122.24a108.352 108.352 0 0 1-76.992 31.808 108.16 108.16 0 0 1-77.12-31.872 109.184 109.184 0 0 1 0-154.176l122.688-122.048a11.392 11.392 0 0 0 3.328-8.32 11.52 11.52 0 0 0-3.328-8.192L296.576 441.6a11.52 11.52 0 0 0-8.256-3.328 11.392 11.392 0 0 0-8.192 3.328l-122.24 122.24a210.752 210.752 0 0 0-61.888 150.016 210.496 210.496 0 0 0 61.888 150.016 210.368 210.368 0 0 0 150.016 61.888 210.752 210.752 0 0 0 150.08-61.888l122.176-122.24a11.52 11.52 0 0 0 0-16.192l-56.576-56.704z" /></svg>
            {{ tt('disconnect') }}
          </button>
        </template>
        <template v-else>
          <div class="flex-1 flex items-center gap-2 min-w-0">
            <span class="md-hdr-title truncate">{{ netName() }}</span>
            <span v-if="netConnecting" class="md-dot md-dot-on flex-shrink-0 ml-2 animate-pulse" />
            <span v-if="netConnecting" class="md-connected-text">{{ tt('connecting') }}</span>
          </div>
          <button class="md-hdr-btn md-hdr-btn-glass" :title="tt('editConfig')" @click="openNetEditor">
            <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M714.965 128l-85.333 85.333H213.333v597.334h597.334V394.368L896 309.035v544.298A42.667 42.667 0 0 1 853.333 896H170.667A42.667 42.667 0 0 1 128 853.333V170.667A42.667 42.667 0 0 1 170.667 128h544.298z m159.062-38.4l60.373 60.416-392.192 392.192-60.245 0.128-0.086-60.459L874.027 89.6z" /></svg>
          </button>
          <div style="position:relative">
            <button class="md-hdr-btn md-hdr-btn-glass md-switch-btn" :title="tt('switchNetwork')" @click.stop="showSwitchMenu = !showSwitchMenu">
              <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M900.4 424.87c19.47 0 37.03-11.73 44.49-29.73 7.46-17.98 3.33-38.7-10.43-52.48L713.97 122.19c-7.3-7.3-19.12-7.3-26.42 0l-41.69 41.69c-7.3 7.3-7.3 19.13 0 26.42l138.28 138.27H86.32c-10.19 0-18.46 8.26-18.46 18.46v59.39c0 10.19 8.26 18.46 18.46 18.46H900.4zM937.65 598.72H123.8c-19.47 0-37.03 11.73-44.49 29.73-7.46 17.98-3.33 38.7 10.43 52.48l220.49 220.48c7.3 7.3 19.12 7.3 26.42 0l41.69-41.69c7.3-7.3 7.3-19.13 0-26.42L240.06 695.02h697.59c10.32 0 18.68-8.37 18.68-18.68v-58.93c0-10.32-8.36-18.69-18.68-18.69z" /></svg>
            </button>
            <div v-if="showSwitchMenu" class="md-switch-menu md-card" @click.stop>
              <div class="md-sw-item" @click="openAddNet">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="var(--md-primary)"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" /></svg>
                <span style="color:var(--md-primary);font-weight:500">{{ tt('addNetwork') }}</span>
              </div>
              <div class="md-sw-divider" />
              <div v-for="(n, i) in networks" :key="n.config.instance_id" class="md-sw-item" @click="switchToNet(i)">
                <span v-if="i === netIndex" class="md-sw-check">&#10003;</span>
                <span v-else style="width:18px;display:inline-block" />
                <span class="flex-1 truncate">{{ (n.config as any).instance_name || n.config.network_name || tt('unnamed') }}</span>
                <button class="md-sw-del" @click="confirmDeleteNet(i, $event)">
                  &times;
                </button>
              </div>
            </div>
          </div>
        </template>
      </header>

      <!-- No networks configured -->
      <div v-if="!networks.length && !netRunning && !netConnecting" class="md-scroll-view" style="display:flex;align-items:center;justify-content:center">
        <div class="md-empty">
          <div class="md-empty-ico" aria-hidden="true">
            <svg width="44" height="44" viewBox="-4 -4 32 32" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="m12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z" /><path d="m22 17.65-9.17 4.16a2 2 0 0 1-1.66 0L2 17.65" /><path d="m22 12.65-9.17 4.16a2 2 0 0 1-1.66 0L2 12.65" /></svg>
          </div>
          <div class="md-empty-text">
            {{ tt('emptyNet') }}
          </div>
        </div>
      </div>

      <!-- Connect button (disconnected / connecting) -->
      <div v-if="(networks.length && !netRunning && !netConnecting) || (netConnecting && !netRunning)" class="md-scroll-view" style="display:flex;align-items:center;justify-content:center">
        <button class="md-connect-btn" :class="{ 'md-connect-btn-bounce': connectBounce, 'md-connect-btn-loading': netConnecting && !netRunning }" :disabled="netConnecting" @click="doConnectWithBounce">
          <div class="md-connect-ico">
            <svg class="md-connect-icon" :class="{ 'ico-hidden': netConnecting }" width="32" height="32" viewBox="0 0 1024 1024" fill="currentColor"><path d="M594.6 716.6a10.68 10.68 0 0 0-15 0l-154.9 154.9c-71.75 71.67-192.79 79.35-272 0-79.35-79.35-71.67-200.25 0-272l154.9-154.9a10.68 10.68 0 0 0 0-15.07l-53.02-53.1a10.68 10.68 0 0 0-15.07 0L84.62 531.41a288.23 288.23 0 0 0 0 407.96 288.3 288.3 0 0 0 407.96 0l154.9-154.9a10.68 10.68 0 0 0 0-15.07l-52.8-52.8z m344.84-631.97a288.16 288.16 0 0 0-407.96 0l-155.05 154.9a10.68 10.68 0 0 0 0 15.07l52.95 52.95a10.68 10.68 0 0 0 15 0l154.97-154.98c71.67-71.67 192.79-79.28 271.92 0 79.35 79.35 71.75 200.32 0 272.07L716.37 579.46a10.68 10.68 0 0 0 0 15.07l53.1 53.02a10.68 10.68 0 0 0 15.07 0l154.9-154.9a288.67 288.67 0 0 0 0-408.1zM642.8 325.82a10.68 10.68 0 0 0-15.07 0L325.75 627.66a10.68 10.68 0 0 0 0 15.07l52.8 52.8a10.68 10.68 0 0 0 15.07 0l301.84-301.9a10.68 10.68 0 0 0 0-15.07l-52.66-52.73z" /></svg>
            <svg class="animate-spin" :class="{ 'ico-hidden': !netConnecting }" width="32" height="32" viewBox="0 0 24 24" fill="currentColor"><path d="M12 4V2A10 10 0 0 0 2 12h2a8 8 0 0 1 8-8z" /></svg>
          </div>
          <span>{{ netConnecting ? tt('connecting') : tt('connect') }}</span>
        </button>
      </div>

      <!-- Connected -->
      <template v-if="netRunning">
        <div class="md-scroll-view">
          <div class="md-scroll-content">
            <!-- Network Hero Card (integrates connection info + green accent) -->
            <div class="md-card md-hero md-hero-connected">
              <div class="md-hero-head">
                <div class="flex-1 min-w-0">
                  <div class="md-hero-title">
                    {{ netName() }}
                  </div>
                  <div class="md-hero-conn-info">
                    <span class="md-hero-ip">{{ netSelfIp || '...' }}</span>
                  </div>
                </div>
                <div class="md-hero-head-right">
                  <span v-if="natLabel(netSelfNat).text" class="md-chip md-chip-nat" :class="natLabel(netSelfNat).cls">{{ natLabel(netSelfNat).text }}</span>
                  <span class="md-status-pill"><span class="md-status-dot" />{{ tt('connected') }}</span>
                </div>
              </div>
              <div v-if="showDebug && (netIfaceIpv4.length || netIfaceIpv6.length)" class="md-hero-iface">
                <div v-for="ip in netIfaceIpv4" :key="`v4-${ip}`" class="md-hero-iface-row">
                  {{ ip }}
                </div>
                <div v-for="ip in netIfaceIpv6" :key="`v6-${ip}`" class="md-hero-iface-row">
                  {{ ip }}
                </div>
              </div>
              <div class="md-hero-metrics">
                <div class="md-hero-metric">
                  <div class="md-hero-mv">
                    {{ netPeers.length }}
                  </div>
                  <div class="md-hero-ml">
                    {{ locale === 'cn' ? '节点' : 'Peers' }}
                  </div>
                </div>
                <div class="md-hero-metric">
                  <div class="md-hero-mv">
                    {{ netServers.length }}
                  </div>
                  <div class="md-hero-ml">
                    {{ locale === 'cn' ? '服务器' : 'Servers' }}
                  </div>
                </div>
                <div class="md-hero-metric">
                  <div class="md-hero-mv">
                    {{ netTrafficSpeed.totalBytes ? formatBytes(netTrafficSpeed.totalBytes) : '--' }}
                  </div>
                  <div class="md-hero-ml">
                    {{ locale === 'cn' ? '流量' : 'Traffic' }}
                  </div>
                </div>
              </div>
            </div>

            <!-- Traffic Chart -->
            <div v-if="trafficHistory.length" class="md-card md-traffic-card">
              <div class="md-traffic-hdr">
                <div class="md-traffic-total">
                  <span class="md-traffic-label" :style="{ color: TRAFFIC_UP_COLOR }">&#8593; {{ netTrafficSpeed.upTotal }}</span>
                  <span class="md-traffic-label" :style="{ color: TRAFFIC_DOWN_COLOR }">&#8595; {{ netTrafficSpeed.downTotal }}</span>
                </div>
                <div class="md-traffic-current">
                  <span class="md-traffic-label" :style="{ color: TRAFFIC_UP_COLOR }">&#8593; {{ netTrafficSpeed.up || '--' }}</span>
                  <span class="md-traffic-label" :style="{ color: TRAFFIC_DOWN_COLOR }">&#8595; {{ netTrafficSpeed.down || '--' }}</span>
                </div>
              </div>
              <ComputerChart
                :series="trafficChartSeries()"
                :max="trafficChartMax()"
                :name="locale === 'cn' ? '组网流量' : 'Network traffic'"
                :format="formatSpeedBps"
                :area="false"
                :x-start-label="formatXLabel(trafficHistory.length, 0)"
                :sample-interval-secs="3"
                :capacity="TRAFFIC_MAX_POINTS"
                :locale="locale"
              />
            </div>

            <!-- Network Devices -->
            <div v-if="netPeers.length" class="md-section">
              <div class="md-sec-hdr">
                <span class="md-sec-title">{{ tt('networkDevices') }} ({{ netPeers.length }})</span>
              </div>
              <article
                v-for="(p, pi) in sortedPeers"
                :key="peerKey(p)"
                class="net-peer-card"
                :class="{ 'net-peer-card-open': netExpanded[peerKey(p)] }"
                :style="{ '--card-i': pi + 2 }"
                role="button"
                tabindex="0"
                :aria-expanded="!!netExpanded[peerKey(p)]"
                @click="netExpanded[peerKey(p)] = !netExpanded[peerKey(p)]"
                @keydown.enter.prevent="netExpanded[peerKey(p)] = !netExpanded[peerKey(p)]"
                @keydown.space.prevent="netExpanded[peerKey(p)] = !netExpanded[peerKey(p)]"
              >
                <div class="net-peer-main">
                  <div class="net-peer-avatar" aria-hidden="true">
                    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M12 21l3.5-4.36c-.9-.56-1.94-.9-3.5-.9s-2.6.34-3.5.9L12 21zm0-8c1.8 0 3.4.72 4.6 1.88l1.4-1.76C16.9 11.9 14.6 11 12 11s-4.9.9-6 2.12l1.4 1.76C8.6 13.72 10.2 13 12 13zm0-5c3.03 0 5.78 1.21 7.78 3.17l1.42-1.76C18.22 7.22 15.32 6 12 6S5.78 7.22 2.8 9.41l1.42 1.76C6.22 9.21 8.97 8 12 8z" /></svg>
                  </div>
                  <div class="net-peer-body">
                    <div class="net-peer-row">
                      <div class="net-peer-title">
                        <span class="net-peer-name" :title="netPeerTitle(p, false)">{{ netPeerTitle(p, false) }}</span>
                        <template v-if="netPeerInlineIp(p)">
                          <span class="net-peer-dot" aria-hidden="true">·</span>
                          <span class="net-peer-ip mono">{{ netPeerInlineIp(p) }}</span>
                        </template>
                      </div>
                      <span v-if="buildDetail(p).lat" class="net-peer-ping">{{ qualityText(buildDetail(p).lat, buildDetail(p).loss) }}</span>
                      <span v-else class="net-peer-ping net-peer-ping-muted">--</span>
                    </div>
                    <div class="net-peer-row net-peer-row-bottom">
                      <div class="net-peer-left">
                        <span v-if="buildDetail(p).up || buildDetail(p).down" class="net-peer-flow">
                          <span v-if="buildDetail(p).up" class="net-peer-flow-up">{{ buildDetail(p).up }}</span>
                          <span v-if="buildDetail(p).down" class="net-peer-flow-down">{{ buildDetail(p).down }}</span>
                        </span>
                      </div>
                      <div class="md-card-tags net-peer-tags" :data-peer-key="peerKey(p)">
                        <span class="md-rt" :class="costCls(p.route?.cost)">{{ costLabel(p.route?.cost) }}</span>
                        <span v-if="buildDetail(p).tun" class="md-chip md-chip-proto">{{ tunnelDisplay(p) }}</span>
                        <span v-if="natLabel(p.route?.stun_info).text" class="md-chip md-chip-nat" :class="natLabel(p.route?.stun_info).cls">{{ natLabel(p.route?.stun_info).text }}</span>
                      </div>
                    </div>
                  </div>
                </div>
                <div class="net-peer-expand" :inert="!netExpanded[peerKey(p)]">
                  <div class="net-peer-expand-inner">
                    <div class="net-peer-info">
                      <span>{{ tt('version') }} <b class="mono">{{ netPeerVersion(p) }}</b></span>
                    </div>
                  </div>
                </div>
              </article>
            </div>

            <!-- Servers -->
            <div v-if="netServers.length" class="md-section">
              <div class="md-sec-hdr">
                <span class="md-sec-title">{{ tt('servers') }} ({{ netServers.length }})</span>
              </div>
              <article
                v-for="(p, si) in sortedServers"
                :key="peerKey(p)"
                class="net-peer-card net-peer-card-server"
                :class="{ 'net-peer-card-open': netExpanded[peerKey(p)] }"
                :style="{ '--card-i': si + 2 }"
                role="button"
                tabindex="0"
                :aria-expanded="!!netExpanded[peerKey(p)]"
                @click="netExpanded[peerKey(p)] = !netExpanded[peerKey(p)]"
                @keydown.enter.prevent="netExpanded[peerKey(p)] = !netExpanded[peerKey(p)]"
                @keydown.space.prevent="netExpanded[peerKey(p)] = !netExpanded[peerKey(p)]"
              >
                <div class="net-peer-main">
                  <div class="net-peer-avatar" aria-hidden="true">
                    <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M7.5 6q-.625 0-1.062.438T6 7.5t.438 1.063T7.5 9t1.063-.437T9 7.5t-.437-1.062T7.5 6m0 10q-.625 0-1.062.438T6 17.5t.438 1.063T7.5 19t1.063-.437T9 17.5t-.437-1.062T7.5 16M4 3h16q.425 0 .713.288T21 4v7q0 .425-.288.713T20 12H4q-.425 0-.712-.288T3 11V4q0-.425.288-.712T4 3m0 10h16q.425 0 .713.288T21 14v7q0 .425-.288.713T20 22H4q-.425 0-.712-.288T3 21v-7q0-.425.288-.712T4 13" /></svg>
                  </div>
                  <div class="net-peer-body">
                    <div class="net-peer-row">
                      <span class="net-peer-name" :title="netPeerTitle(p, true)">{{ netPeerTitle(p, true) }}</span>
                      <span v-if="buildDetail(p).lat" class="net-peer-ping">{{ qualityText(buildDetail(p).lat, buildDetail(p).loss) }}</span>
                      <span v-else class="net-peer-ping net-peer-ping-muted">--</span>
                    </div>
                    <div class="net-peer-row net-peer-row-bottom">
                      <div class="net-peer-left">
                        <span v-if="buildDetail(p).up || buildDetail(p).down" class="net-peer-flow">
                          <span v-if="buildDetail(p).up" class="net-peer-flow-up">{{ buildDetail(p).up }}</span>
                          <span v-if="buildDetail(p).down" class="net-peer-flow-down">{{ buildDetail(p).down }}</span>
                        </span>
                      </div>
                      <div class="md-card-tags net-peer-tags" :data-peer-key="peerKey(p)">
                        <span class="md-rt" :class="costCls(p.route?.cost)">{{ costLabel(p.route?.cost) }}</span>
                        <span v-if="buildDetail(p).tun" class="md-chip md-chip-proto">{{ tunnelDisplay(p) }}</span>
                        <span v-if="natLabel(p.route?.stun_info).text" class="md-chip md-chip-nat" :class="natLabel(p.route?.stun_info).cls">{{ natLabel(p.route?.stun_info).text }}</span>
                      </div>
                    </div>
                  </div>
                </div>
                <div class="net-peer-expand" :inert="!netExpanded[peerKey(p)]">
                  <div class="net-peer-expand-inner">
                    <div class="net-peer-info">
                      <span>{{ tt('version') }} <b class="mono">{{ netPeerVersion(p) }}</b></span>
                    </div>
                  </div>
                </div>
              </article>
            </div>

            <div v-if="!netPeers.length && !netServers.length" class="flex flex-col items-center justify-center py-12 gap-3 opacity-50">
              <template v-if="netDiscovering || netLoading">
                <i class="pi pi-spin pi-spinner text-xl md-text-muted" /><div class="text-base md-text-muted">
                  {{ tt('discoveringPeers') }}
                </div>
              </template>
              <template v-else>
                <div class="text-base md-text-muted">
                  {{ tt('noPeersConnected') }}
                </div>
              </template>
            </div>
          </div>
        </div>
      </template>
    </div>

    <!-- ==================== Router Tab ==================== -->
    <div v-show="activeTab === 'luci'" class="md-page md-page-bg" :class="{ 'page-active': activeTab === 'luci' }">
      <header class="md-hdr">
        <span class="md-hdr-title">{{ currentLuciRouter?.ip || tt('routers') }}</span>
        <span v-if="currentLuciRouter && getWolPathType() === 'tunnel'" class="md-chip">{{ tt('etLan') }}</span>
        <span v-else-if="currentLuciRouter && getWolPathType() === 'lan'" class="md-chip">{{ tt('lanDirect') }}</span>
        <div class="flex-1" />
        <button v-if="proxyUrl" class="md-hdr-btn md-hdr-btn-glass" :title="tt('refresh')" @click="refreshLuciIframe">
          <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M128 170.666667l92.330667 92.330666A382.378667 382.378667 0 0 1 512 128c211.754667 0 384 172.288 384 384h-85.333333c0-164.693333-134.016-298.666667-298.666667-298.666667a297.557333 297.557333 0 0 0-231.125333 110.208L384 426.666667H128V170.666667z m768 682.666666v-256h-256l103.125333 103.125334A297.514667 297.514667 0 0 1 512 810.666667c-164.650667 0-298.666667-134.016-298.666667-298.666667H128c0 211.754667 172.245333 384 384 384a382.378667 382.378667 0 0 0 291.669333-134.997333L896 853.333333z" /></svg>
        </button>
        <div style="position:relative">
          <button class="md-hdr-btn md-hdr-btn-glass md-switch-btn" :title="tt('switchNetwork')" @click.stop="showLuciMenu = !showLuciMenu">
            <svg width="20" height="20" viewBox="0 0 1024 1024" fill="currentColor"><path d="M900.4 424.87c19.47 0 37.03-11.73 44.49-29.73 7.46-17.98 3.33-38.7-10.43-52.48L713.97 122.19c-7.3-7.3-19.12-7.3-26.42 0l-41.69 41.69c-7.3 7.3-7.3 19.13 0 26.42l138.28 138.27H86.32c-10.19 0-18.46 8.26-18.46 18.46v59.39c0 10.19 8.26 18.46 18.46 18.46H900.4zM937.65 598.72H123.8c-19.47 0-37.03 11.73-44.49 29.73-7.46 17.98-3.33 38.7 10.43 52.48l220.49 220.48c7.3 7.3 19.12 7.3 26.42 0l41.69-41.69c7.3-7.3 7.3-19.13 0-26.42L240.06 695.02h697.59c10.32 0 18.68-8.37 18.68-18.68v-58.93c0-10.32-8.36-18.69-18.68-18.69z" /></svg>
          </button>
          <div v-if="showLuciMenu" class="md-switch-menu md-card" @click.stop>
            <div class="md-sw-item" @click="openLuciEditorFn">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="var(--md-primary)"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" /></svg>
              <span style="color:var(--md-primary);font-weight:500">{{ tt('editRouter') }}</span>
            </div>
            <div v-if="luciRouters.length" class="md-sw-divider" />
            <div v-for="(r, i) in luciRouters" :key="r.ip" class="md-sw-item" @click="switchLuciRouter(i)">
              <span v-if="i === luciIdx" class="md-sw-check">&#10003;</span>
              <span v-else style="width:18px;display:inline-block" />
              <span class="flex-1 truncate" :style="i === luciIdx ? 'color:var(--md-primary);font-weight:500' : ''">{{ r.ip }}</span>
            </div>
          </div>
        </div>
      </header>
      <div class="md-iframe-area">
        <div v-if="!luciRouters.length" class="md-empty">
          <div class="md-empty-ico" aria-hidden="true">
            <svg width="44" height="44" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="8" /><path d="m15.2 8.8-1.7 4.7-4.7 1.7 1.7-4.7 4.7-1.7Z" /></svg>
          </div>
          <div class="md-empty-text">
            {{ tt('emptyRouter') }}
          </div>
        </div>
        <iframe v-else-if="luciIframeSrc" :key="luciIframeKey" :src="luciIframeSrc" class="luci-iframe" />
        <div v-else class="flex flex-col items-center justify-center h-full gap-3">
          <svg class="animate-spin" width="32" height="32" viewBox="0 0 24 24" fill="var(--muted)"><path d="M12 4V2A10 10 0 0 0 2 12h2a8 8 0 0 1 8-8z" /></svg>
          <div class="text-base md-text-muted">
            {{ tt('connectingRouter') }}
          </div>
        </div>
      </div>
    </div>

    <!-- ==================== Settings Tab ==================== -->
    <div v-show="activeTab === 'settings'" class="md-page md-page-bg" :class="{ 'page-active': activeTab === 'settings' }">
      <header class="md-hdr">
        <span class="md-hdr-title">{{ tt('settings') }}</span>
        <div class="flex-1" />
        <div style="width:36px" />
      </header>
      <div class="md-scroll-view">
        <div class="md-scroll-content">
          <!-- Settings Hero Card -->
          <div class="md-card md-hero md-hero-blue">
            <div class="md-hero-head">
              <div class="flex-1 min-w-0">
                <div class="md-hero-title">
                  EasyTier
                </div>
                <div class="md-hero-text">
                  {{ locale === 'cn' ? '异地组网和远程设备管理' : 'Virtual networking & remote device management' }}
                </div>
              </div>
              <button class="md-settings-btn md-settings-btn-primary" style="white-space:nowrap" @click="showAboutDlg = true">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><line x1="12" y1="16" x2="12" y2="12" /><line x1="12" y1="8" x2="12.01" y2="8" /></svg>
                v{{ pkg.version }}
              </button>
            </div>
            <div class="md-hero-metrics">
              <div class="md-hero-metric">
                <div class="md-hero-mv">
                  {{ wolDevices.length }}
                </div>
                <div class="md-hero-ml">
                  {{ locale === 'cn' ? '电脑' : 'PCs' }}
                </div>
              </div>
              <div class="md-hero-metric">
                <div class="md-hero-mv">
                  {{ luciRouters.length }}
                </div>
                <div class="md-hero-ml">
                  {{ locale === 'cn' ? '路由器' : 'Routers' }}
                </div>
              </div>
              <div class="md-hero-metric">
                <div class="md-hero-mv">
                  {{ networks.length }}
                </div>
                <div class="md-hero-ml">
                  {{ locale === 'cn' ? '网络' : 'Networks' }}
                </div>
              </div>
            </div>
          </div>

          <div class="md-card md-settings-card">
            <div class="md-row">
              <div class="md-symbol" :style="settingsIconStyle('home')">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" /><polyline points="9 22 9 12 15 12 15 22" /></svg>
              </div>
              <div class="flex-1">
                <div class="md-name">
                  {{ tt('defaultHome') }}
                </div>
                <div class="md-sub">
                  {{ tt('defaultHomeDesc') }}
                </div>
              </div>
              <select class="md-select" :value="defaultTab" @change="setDefaultTab(($event.target as HTMLSelectElement).value as Tab)" @click.stop>
                <option value="wol">
                  {{ tt('wolTab') }}
                </option>
                <option value="luci">
                  {{ tt('luciTab') }}
                </option>
                <option value="net">
                  {{ tt('netTab') }}
                </option>
              </select>
            </div>
          </div>
          <div class="md-card md-settings-card">
            <div class="md-row">
              <div class="md-symbol" :style="settingsIconStyle('theme')">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5" /><line x1="12" y1="1" x2="12" y2="3" /><line x1="12" y1="21" x2="12" y2="23" /><line x1="4.22" y1="4.22" x2="5.64" y2="5.64" /><line x1="18.36" y1="18.36" x2="19.78" y2="19.78" /><line x1="1" y1="12" x2="3" y2="12" /><line x1="21" y1="12" x2="23" y2="12" /><line x1="4.22" y1="19.78" x2="5.64" y2="18.36" /><line x1="18.36" y1="5.64" x2="19.78" y2="4.22" /></svg>
              </div>
              <div class="flex-1">
                <div class="md-name">
                  {{ tt('theme') }}
                </div>
                <div class="md-sub">
                  {{ tt('themeDesc') }}
                </div>
              </div>
              <select class="md-select" :value="themeMode" @change="themeMode = ($event.target as HTMLSelectElement).value" @click.stop>
                <option value="auto">
                  {{ tt('themeAuto') }}
                </option>
                <option value="light">
                  {{ tt('themeLight') }}
                </option>
                <option value="dark">
                  {{ tt('themeDark') }}
                </option>
              </select>
            </div>
          </div>
          <div class="md-card md-settings-card" @click="toggleLang">
            <div class="md-row">
              <div class="md-symbol" :style="settingsIconStyle('lang')">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><path d="M2 12h20" /><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" /></svg>
              </div>
              <div class="flex-1">
                <div class="md-name">
                  {{ tt('language') }}
                </div>
                <div class="md-sub">
                  {{ tt('languageDesc') }}
                </div>
              </div>
              <span class="md-label-stack"><b>{{ locale === 'cn' ? '中文' : 'English' }}</b><span>{{ locale === 'cn' ? 'Chinese' : '英文' }}</span></span>
            </div>
          </div>
          <div class="md-card md-settings-card" @click="showDebug = !showDebug">
            <div class="md-row">
              <div class="md-symbol" :style="settingsIconStyle('debug')">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6" /><polyline points="8 6 2 12 8 18" /></svg>
              </div>
              <div class="flex-1">
                <div class="md-name">
                  {{ tt('debug') }}
                </div>
                <div class="md-sub">
                  {{ tt('debugDesc') }}
                </div>
              </div>
              <span class="md-toggle" :class="{ 'md-toggle-on': showDebug }" />
            </div>
          </div>
          <div v-if="showDebug" class="md-card md-settings-card" @click="router.push('/')">
            <div class="md-row">
              <div class="md-symbol" :style="settingsIconStyle('advanced')">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg>
              </div>
              <div class="flex-1">
                <div class="md-name">
                  {{ tt('advancedSettings') }}
                </div>
                <div class="md-sub">
                  {{ tt('advancedSettingsDesc') }}
                </div>
              </div>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="var(--muted)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" opacity="0.5"><path d="M9 18l6-6-6-6" /></svg>
            </div>
          </div>
          <div class="md-card md-settings-card">
            <div class="md-row">
              <div class="md-symbol" :style="settingsIconStyle('config')">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></svg>
              </div>
              <div class="flex-1">
                <div class="md-name">
                  {{ tt('oneClickConfig') }}
                </div>
                <div class="md-sub">
                  {{ tt('oneClickConfigDesc') }}
                </div>
              </div>
              <button class="md-settings-btn md-settings-btn-export" @click.stop="doExport">
                {{ tt('exportLabel') }}
              </button>
              <button class="md-settings-btn md-settings-btn-primary" @click.stop="openImportDlg">
                {{ tt('importLabel') }}
              </button>
            </div>
          </div>
          <div class="md-card md-settings-card" @click="showHelpDlg = true">
            <div class="md-row">
              <div class="md-symbol" :style="settingsIconStyle('help')">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" /><line x1="12" y1="17" x2="12.01" y2="17" /></svg>
              </div>
              <div class="flex-1">
                <div class="md-name">
                  {{ tt('help') }}
                </div>
                <div class="md-sub">
                  {{ tt('helpDesc') }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- ==================== Bottom Tab Bar ==================== -->
    <nav class="md-tabs" aria-label="Tab bar">
      <button class="md-tab" :class="{ 'md-tab-active': activeTab === 'wol' }" :aria-current="activeTab === 'wol' ? 'page' : 'false'" @click="switchTab('wol')">
        <span class="md-tab-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="5" width="16" height="14" rx="3" /><path d="M7 9h10" /><path d="M7 13h6" /></svg></span>
        <span class="md-tab-label">{{ tt('wol') }}</span>
      </button>
      <button class="md-tab" :class="{ 'md-tab-active': activeTab === 'luci' }" :aria-current="activeTab === 'luci' ? 'page' : 'false'" @click="switchTab('luci')">
        <span class="md-tab-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="8" /><path d="m15.2 8.8-1.7 4.7-4.7 1.7 1.7-4.7 4.7-1.7Z" /></svg></span>
        <span class="md-tab-label">{{ tt('routers') }}</span>
      </button>
      <button class="md-tab" :class="{ 'md-tab-active': activeTab === 'net' }" :aria-current="activeTab === 'net' ? 'page' : 'false'" @click="switchTab('net')">
        <span class="md-tab-icon"><svg viewBox="-4 -4 32 32" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="m12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z" /><path d="m22 17.65-9.17 4.16a2 2 0 0 1-1.66 0L2 17.65" /><path d="m22 12.65-9.17 4.16a2 2 0 0 1-1.66 0L2 12.65" /></svg></span>
        <span class="md-tab-label">{{ tt('network') }}</span>
      </button>
      <button class="md-tab" :class="{ 'md-tab-active': activeTab === 'settings' }" :aria-current="activeTab === 'settings' ? 'page' : 'false'" @click="switchTab('settings')">
        <span class="md-tab-icon"><svg viewBox="-4 -4 32 32" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" /></svg></span>
        <span class="md-tab-label">{{ tt('settings') }}</span>
      </button>
    </nav>

    <!-- ==================== Dialogs ==================== -->
    <Dialog v-model:visible="showWolEditor" modal :header="tt('editDevicesToml')" :style="{ width: '90vw', maxWidth: '520px' }" class="md-dialog" :pt="{ closeButton: { style: 'display:none' }, header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }" @show="wolTextareaRef?.$el?.querySelector('textarea')?.focus()">
      <Textarea
        ref="wolTextareaRef" v-model="wolToml" rows="14" class="w-full md-dlg-textarea"
        placeholder="[[device]]&#10;name = &quot;My PC&quot;&#10;mac = &quot;AA:BB:CC:DD:EE:FF&quot;&#10;ip = &quot;192.168.2.2&quot;&#10;interface = &quot;br-lan&quot;&#10;router_ip = &quot;192.168.2.1&quot;&#10;agent_port = 32249&#10;command_token = &quot;shared-secret&quot;&#10;commands = [&#10;  { name = &quot;Ping&quot;, command = &quot;ping -n 4 127.0.0.1&quot;, mode = &quot;INSTANT&quot; },&#10;]"
      />
      <template #footer>
        <div class="md-dlg-ft">
          <button class="md-dlg-btn" @click="importFromClipboard('wol')">
            {{ tt('importFromClipboard') }}
          </button>
          <div class="flex-1" />
          <button class="md-dlg-btn" @click="showWolEditor = false">
            {{ tt('cancel') }}
          </button>
          <button class="md-dlg-btn md-dlg-btn-save" @click="saveWolConfig">
            {{ tt('save') }}
          </button>
        </div>
      </template>
    </Dialog>

    <Dialog v-model:visible="showNetEditor" modal :header="tt('editConfigDialog')" :style="{ width: '90vw', maxWidth: '560px' }" class="md-dialog" :pt="{ closeButton: { style: 'display:none' }, header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }" @show="netTextareaRef?.$el?.querySelector('textarea')?.focus()">
      <Textarea ref="netTextareaRef" v-model="editingNetToml" rows="16" class="w-full md-dlg-textarea" placeholder="instance_name = &quot;my-network&quot;&#10;..." />
      <template #footer>
        <div class="md-dlg-ft">
          <button class="md-dlg-btn" @click="importFromClipboard('net')">
            {{ tt('importFromClipboard') }}
          </button>
          <div class="flex-1" />
          <button class="md-dlg-btn" @click="showNetEditor = false">
            {{ tt('cancel') }}
          </button>
          <button class="md-dlg-btn md-dlg-btn-save" :disabled="savingNet" @click="saveNetConfig">
            {{ savingNet ? '...' : tt('save') }}
          </button>
        </div>
      </template>
    </Dialog>

    <!-- LuCI Router Config Editor -->
    <Dialog v-model:visible="showLuciEditor" modal :header="tt('editRoutersToml')" :style="{ width: '90vw', maxWidth: '520px' }" class="md-dialog" :pt="{ closeButton: { style: 'display:none' }, header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }">
      <Textarea
        v-model="luciToml" rows="12" class="w-full md-dlg-textarea"
        placeholder="[[router]]&#10;name = &quot;Home Router&quot;&#10;ip = &quot;192.168.2.1&quot;&#10;username = &quot;root&quot;&#10;password = &quot;admin&quot;"
      />
      <template #footer>
        <div class="md-dlg-ft">
          <button class="md-dlg-btn" @click="importFromClipboard('luci')">
            {{ tt('importFromClipboard') }}
          </button>
          <div class="flex-1" />
          <button class="md-dlg-btn" @click="showLuciEditor = false">
            {{ tt('cancel') }}
          </button>
          <button class="md-dlg-btn md-dlg-btn-save" @click="saveLuciConfig">
            {{ tt('save') }}
          </button>
        </div>
      </template>
    </Dialog>

    <!-- Command execution loading overlay (always cancellable) -->
    <div v-if="execLoading.visible" class="exec-overlay sl" role="dialog" aria-modal="true" :aria-label="locale === 'cn' ? '命令执行中' : 'Command running'">
      <div class="exec-card">
        <svg class="pc-busy-spinner" width="36" height="36" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 4V2A10 10 0 0 0 2 12h2a8 8 0 0 1 8-8z" /></svg>
        <div class="exec-title">
          {{ locale === 'cn' ? '正在执行命令' : 'Running command' }}
        </div>
        <div class="exec-name">
          {{ execLoading.commandName }}
        </div>
        <div class="exec-device">
          {{ locale === 'cn' ? `在 ${execLoading.deviceName} 上` : `on ${execLoading.deviceName}` }}
        </div>
        <button class="md-dlg-btn" type="button" @click="cancelCommandExecution">
          {{ locale === 'cn' ? '取消' : 'Cancel' }}
        </button>
        <div class="exec-hint">
          {{ locale === 'cn' ? '取消仅停止轮询，不会中断电脑端已开始的执行' : 'Cancel only stops polling; the PC keeps running the command' }}
        </div>
      </div>
    </div>

    <!-- Command result bottom sheet -->
    <div v-if="execSheet.visible" class="md-sheet-scrim" @click.self="execSheet.visible = false">
      <div class="md-sheet" role="dialog" aria-modal="true">
        <div class="md-sheet-handle" aria-hidden="true" />
        <div class="md-sheet-head">
          <span class="md-sheet-dot" :class="execSheet.ok ? 'md-sheet-dot-ok' : 'md-sheet-dot-err'" aria-hidden="true" />
          <div class="md-sheet-titles">
            <div class="md-sheet-title">
              {{ execSheet.ok ? (locale === 'cn' ? '执行成功' : 'Completed successfully') : (locale === 'cn' ? '执行失败' : 'Execution failed') }}
            </div>
            <div class="md-sheet-sub">
              {{ execSheet.commandName }} · {{ execSheet.deviceName }}
            </div>
          </div>
          <button class="pc-icon-btn" type="button" :aria-label="tt('close')" @click="execSheet.visible = false">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M19 6.41 17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" /></svg>
          </button>
        </div>
        <div class="md-sheet-body">
          <pre class="pc-output mono" :class="{ 'pc-output-err': !execSheet.ok }">{{ execSheet.text }}</pre>
        </div>
        <div class="md-sheet-ft">
          <button class="pc-tonal-btn" type="button" @click="copyExecSheet">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z" /></svg>
            {{ locale === 'cn' ? '复制' : 'Copy' }}
          </button>
          <div class="flex-1" />
          <button class="pc-tonal-btn" type="button" @click="execSheet.visible = false">
            {{ tt('close') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Manage custom commands bottom sheet -->
    <div v-if="commandManager.visible" class="md-sheet-scrim" @click.self="commandManager.visible = false">
      <div class="md-sheet" role="dialog" aria-modal="true" :aria-label="locale === 'cn' ? '管理自定义命令' : 'Manage custom commands'">
        <div class="md-sheet-handle" aria-hidden="true" />
        <div class="md-sheet-head">
          <div class="md-sheet-titles">
            <div class="md-sheet-title">
              {{ locale === 'cn' ? '管理自定义命令' : 'Manage custom commands' }}
            </div>
            <div class="md-sheet-sub">
              {{ managerDevice?.name }} · {{ managerDevice?.status.os || '--' }}
            </div>
          </div>
          <button class="pc-icon-btn" type="button" :aria-label="tt('close')" @click="commandManager.visible = false">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M19 6.41 17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" /></svg>
          </button>
        </div>

        <div class="md-sheet-body">
          <div v-if="!managerCommands.length" class="pc-command-empty">
            <p>{{ locale === 'cn' ? '暂无命令' : 'No commands' }}</p>
          </div>
          <div v-else class="pc-manager-list">
            <div
              v-for="(command, cIdx) in managerCommands"
              :key="command.id || cIdx"
              class="pc-manager-item"
              :class="{ 'pc-manager-item-edit': commandManager.editingIndex === cIdx }"
              role="button"
              tabindex="0"
              @click="startEditCommand(command, cIdx)"
              @keydown.enter.prevent="startEditCommand(command, cIdx)"
            >
              <span class="pc-menu-icon" aria-hidden="true">
                <svg v-if="command.mode === 'LONG_RUNNING'" width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M7.5 6q-.625 0-1.062.438T6 7.5t.438 1.063T7.5 9t1.063-.437T9 7.5t-.437-1.062T7.5 6m0 10q-.625 0-1.062.438T6 17.5t.438 1.063T7.5 19t1.063-.437T9 17.5t-.437-1.062T7.5 16M4 3h16q.425 0 .713.288T21 4v7q0 .425-.288.713T20 12H4q-.425 0-.712-.288T3 11V4q0-.425.288-.712T4 3m0 10h16q.425 0 .713.288T21 14v7q0 .425-.288.713T20 22H4q-.425 0-.712-.288T3 21v-7q0-.425.288-.712T4 13" /></svg>
                <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M9 15H5.9q-.6 0-.888-.537t.063-1.038l7.475-10.75q.25-.35.65-.487t.825.012t.625.525t.15.8L14 10h3.875q.65 0 .913.575t-.163 1.075L10.4 21.5q-.275.325-.675.425t-.775-.075t-.587-.537t-.163-.788z" /></svg>
              </span>
              <div class="pc-command-text">
                <span>{{ command.name }}</span>
                <small class="mono" :title="command.command">{{ command.command }}</small>
              </div>
              <button class="pc-icon-btn pc-danger-icon" type="button" :aria-label="locale === 'cn' ? `删除 ${command.name}` : `Delete ${command.name}`" @click.stop="deleteManagerCommand(cIdx)">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z" /></svg>
              </button>
            </div>
          </div>

          <div class="pc-manager-title">
            {{ commandManager.editingIndex >= 0 ? (locale === 'cn' ? '编辑命令' : 'Edit command') : (locale === 'cn' ? '新建命令' : 'New command') }}
          </div>
          <div v-if="commandManager.error" class="pc-run-error">
            {{ commandManager.error }}
          </div>

          <label class="pc-field">
            <span>{{ locale === 'cn' ? '名称' : 'Name' }}</span>
            <input v-model="commandManager.name" maxlength="24" autocomplete="off" :placeholder="locale === 'cn' ? '例如：重启资源管理器' : 'Example: Flush DNS cache'">
          </label>

          <label class="pc-field">
            <span>{{ locale === 'cn' ? '命令' : 'Command' }}</span>
            <textarea v-model="commandManager.command" rows="2" maxlength="300" class="mono" spellcheck="false" :placeholder="locale === 'cn' ? '例如：taskkill /f /im explorer.exe & start explorer.exe' : 'Example: ping -n 4 127.0.0.1'" />
          </label>

          <div class="pc-field">
            <span>{{ locale === 'cn' ? '执行模式' : 'Execution mode' }}</span>
            <div class="pc-mode-seg" role="group" :aria-label="locale === 'cn' ? '执行模式' : 'Execution mode'">
              <button type="button" :aria-checked="commandManager.mode === 'INSTANT'" @click="commandManager.mode = 'INSTANT'">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M9 15H5.9q-.6 0-.888-.537t.063-1.038l7.475-10.75q.25-.35.65-.487t.825.012t.625.525t.15.8L14 10h3.875q.65 0 .913.575t-.163 1.075L10.4 21.5q-.275.325-.675.425t-.775-.075t-.587-.537t-.163-.788z" /></svg>
                {{ locale === 'cn' ? '即时' : 'Instant' }}
              </button>
              <button type="button" :aria-checked="commandManager.mode === 'LONG_RUNNING'" @click="commandManager.mode = 'LONG_RUNNING'">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M7.5 6q-.625 0-1.062.438T6 7.5t.438 1.063T7.5 9t1.063-.437T9 7.5t-.437-1.062T7.5 6m0 10q-.625 0-1.062.438T6 17.5t.438 1.063T7.5 19t1.063-.437T9 17.5t-.437-1.062T7.5 16M4 3h16q.425 0 .713.288T21 4v7q0 .425-.288.713T20 12H4q-.425 0-.712-.288T3 11V4q0-.425.288-.712T4 3m0 10h16q.425 0 .713.288T21 14v7q0 .425-.288.713T20 22H4q-.425 0-.712-.288T3 21v-7q0-.425.288-.712T4 13" /></svg>
                {{ locale === 'cn' ? '长期' : 'Long-running' }}
              </button>
            </div>
          </div>
        </div>

        <div class="md-sheet-ft">
          <button v-if="commandManager.editingIndex >= 0" class="md-dlg-btn" type="button" @click="resetManagerForm">
            {{ locale === 'cn' ? '取消修改' : 'Discard edits' }}
          </button>
          <button v-else-if="managerDirty" class="md-dlg-btn" type="button" @click="resetManagerForm">
            {{ locale === 'cn' ? '清空' : 'Clear' }}
          </button>
          <div class="flex-1" />
          <button v-if="commandManager.editingIndex >= 0" class="pc-btn-filled" type="button" :disabled="!managerFormValid()" @click="submitManagerCommand">
            {{ locale === 'cn' ? '保存修改' : 'Save changes' }}
          </button>
          <button v-else-if="managerDirty" class="pc-btn-filled" type="button" :disabled="!managerFormValid()" @click="submitManagerCommand">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6z" /></svg>
            {{ locale === 'cn' ? '新增' : 'Add' }}
          </button>
          <button v-else class="pc-btn-filled" type="button" @click="commandManager.visible = false">
            {{ locale === 'cn' ? '完成' : 'Done' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Background tasks bottom sheet -->
    <div v-if="tasksSheet.visible" class="md-sheet-scrim" @click.self="closeTasksSheet">
      <div class="md-sheet" role="dialog" aria-modal="true" :aria-label="locale === 'cn' ? '后台任务' : 'Background tasks'">
        <div class="md-sheet-handle" aria-hidden="true" />
        <div class="md-sheet-head">
          <div class="md-sheet-titles">
            <div class="md-sheet-title">
              {{ locale === 'cn' ? '后台任务' : 'Background tasks' }}
            </div>
            <div class="md-sheet-sub">
              {{ tasksSheet.deviceName }} · {{ locale === 'cn' ? '每 3 秒自动刷新' : 'auto-refreshes every 3s' }}
            </div>
          </div>
          <button class="pc-icon-btn" type="button" :aria-label="tt('close')" @click="closeTasksSheet">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M19 6.41 17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z" /></svg>
          </button>
        </div>

        <div class="md-sheet-body">
          <div v-if="tasksSheet.error" class="pc-run-error">
            {{ tasksSheet.error }}
          </div>
          <div v-if="!tasksSheet.tasks.length" class="pc-command-empty">
            <p>{{ locale === 'cn' ? '没有运行中的任务' : 'No running tasks' }}</p>
          </div>
          <div v-else class="pc-manager-list">
            <div v-for="task in tasksSheet.tasks" :key="task.taskId" class="pc-manager-item pc-task-item">
              <span class="pc-menu-icon" aria-hidden="true">
                <svg v-if="task.mode === 'LONG_RUNNING'" width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M7.5 6q-.625 0-1.062.438T6 7.5t.438 1.063T7.5 9t1.063-.437T9 7.5t-.437-1.062T7.5 6m0 10q-.625 0-1.062.438T6 17.5t.438 1.063T7.5 19t1.063-.437T9 17.5t-.437-1.062T7.5 16M4 3h16q.425 0 .713.288T21 4v7q0 .425-.288.713T20 12H4q-.425 0-.712-.288T3 11V4q0-.425.288-.712T4 3m0 10h16q.425 0 .713.288T21 14v7q0 .425-.288.713T20 22H4q-.425 0-.712-.288T3 21v-7q0-.425.288-.712T4 13" /></svg>
                <svg v-else width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M9 15H5.9q-.6 0-.888-.537t.063-1.038l7.475-10.75q.25-.35.65-.487t.825.012t.625.525t.15.8L14 10h3.875q.65 0 .913.575t-.163 1.075L10.4 21.5q-.275.325-.675.425t-.775-.075t-.587-.537t-.163-.788z" /></svg>
              </span>
              <div class="pc-task-main">
                <span class="pc-task-cmd mono" :title="task.command">{{ task.command }}</span>
                <span class="pc-task-sub">PID {{ task.pid }} · {{ locale === 'cn' ? '已运行' : 'up' }} {{ formatTaskElapsed(task.elapsedSecs) }}</span>
              </div>
              <button class="pc-icon-btn pc-danger-icon" type="button" :aria-label="locale === 'cn' ? '停止该任务' : 'Stop task'" @click.stop="confirmStopTask(task)">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M9 16h6q.425 0 .713-.288T16 15V9q0-.425-.288-.712T15 8H9q-.425 0-.712.288T8 9v6q0 .425.288.713T9 16m3 6q-2.075 0-3.9-.788t-3.175-2.137T2.788 15.9T2 12t.788-3.9t2.137-3.175T8.1 2.788T12 2t3.9.788t3.175 2.137T21.213 8.1T22 12t-.788 3.9t-2.137 3.175t-3.175 2.138T12 22" /></svg>
              </button>
            </div>
          </div>
        </div>

        <div class="md-sheet-ft">
          <button class="pc-tonal-btn" type="button" :disabled="tasksSheet.loading" @click="refreshAgentTasks()">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M17.65 6.35a7.95 7.95 0 0 0-6.48-2.31c-4.24.37-7.57 3.9-7.14 8.14.35 3.86 3.61 6.84 7.48 6.84 1.24 0 2.41-.32 3.44-.87l-1.43-1.43a5.1 5.1 0 0 1-7.27-4.94c.24-2.71 2.49-4.86 5.21-4.93.27-.01.52 0 .77.03 1.1.16 2.1.67 2.88 1.42L12 13h8V5z" /></svg>
            {{ locale === 'cn' ? '刷新' : 'Refresh' }}
          </button>
          <div class="flex-1" />
          <button class="pc-btn-filled" type="button" @click="closeTasksSheet">
            {{ tt('close') }}
          </button>
        </div>
      </div>
    </div>

    <!-- Shared action confirmation dialog (run / shutdown / stop) -->
    <div v-if="actionConfirm.visible" class="cmd-confirm-scrim" @click.self="closeActionConfirm">
      <div class="cmd-confirm-card" role="alertdialog" aria-modal="true" :aria-label="actionConfirm.title">
        <span class="cmd-confirm-icon" :class="{ danger: actionConfirm.danger }" aria-hidden="true">
          <svg v-if="actionConfirm.iconKind === 'power'" width="32" height="32" viewBox="0 0 24 24" fill="currentColor"><path d="M13 3h-2v10h2V3zm4.83 2.17-1.42 1.42C17.99 7.86 19 9.81 19 12c0 3.87-3.13 7-7 7s-7-3.13-7-7c0-2.19 1.01-4.14 2.58-5.42L6.17 5.17C4.23 6.82 3 9.26 3 12c0 4.97 4.03 9 9 9s9-4.03 9-9c0-2.74-1.23-5.18-3.17-6.83z" /></svg>
          <svg v-else-if="actionConfirm.iconKind === 'dns'" width="32" height="32" viewBox="0 0 24 24" fill="currentColor"><path d="M7.5 6q-.625 0-1.062.438T6 7.5t.438 1.063T7.5 9t1.063-.437T9 7.5t-.437-1.062T7.5 6m0 10q-.625 0-1.062.438T6 17.5t.438 1.063T7.5 19t1.063-.437T9 17.5t-.437-1.062T7.5 16M4 3h16q.425 0 .713.288T21 4v7q0 .425-.288.713T20 12H4q-.425 0-.712-.288T3 11V4q0-.425.288-.712T4 3m0 10h16q.425 0 .713.288T21 14v7q0 .425-.288.713T20 22H4q-.425 0-.712-.288T3 21v-7q0-.425.288-.712T4 13" /></svg>
          <svg v-else width="32" height="32" viewBox="0 0 24 24" fill="currentColor"><path d="M9 15H5.9q-.6 0-.888-.537t.063-1.038l7.475-10.75q.25-.35.65-.487t.825.012t.625.525t.15.8L14 10h3.875q.65 0 .913.575t-.163 1.075L10.4 21.5q-.275.325-.675.425t-.775-.075t-.587-.537t-.163-.788z" /></svg>
        </span>
        <h3 class="cmd-confirm-title">
          {{ actionConfirm.title }}
        </h3>
        <p v-if="actionConfirm.subtitle" class="cmd-confirm-text">
          {{ actionConfirm.subtitle }}
        </p>
        <pre v-if="actionConfirm.code" class="cmd-confirm-code mono">{{ actionConfirm.code }}</pre>
        <div class="cmd-confirm-actions">
          <button class="md-dlg-btn" type="button" @click="closeActionConfirm">
            {{ tt('cancel') }}
          </button>
          <button class="pc-btn-filled" :class="{ danger: actionConfirm.danger }" type="button" @click="acceptActionConfirm">
            {{ actionConfirm.okText }}
          </button>
        </div>
      </div>
    </div>

    <!-- Delete Confirm Dialog -->
    <Dialog v-model:visible="showDeleteConfirm" modal :header="tt('delete')" :style="{ width: '85vw', maxWidth: '380px' }" class="md-dialog" :pt="{ header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }">
      <p class="text-base py-2" style="color:var(--md-text)">
        {{ tt('deleteNetworkConfirm') }}
      </p>
      <template #footer>
        <div class="md-dlg-ft" style="justify-content:flex-end;gap:8px">
          <button class="md-dlg-btn" @click="showDeleteConfirm = false">
            {{ tt('cancel') }}
          </button>
          <button class="md-dlg-btn md-dlg-btn-del" @click="doDeleteNet">
            {{ tt('confirm') }}
          </button>
        </div>
      </template>
    </Dialog>

    <!-- Import Config Dialog -->
    <Dialog v-model:visible="showImportDlg" modal :header="tt('importConfig')" :style="{ width: '90vw', maxWidth: '520px' }" class="md-dialog" :pt="{ closeButton: { style: 'display:none' }, header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }" @show="importTextareaRef?.$el?.querySelector('textarea')?.focus()">
      <Textarea ref="importTextareaRef" v-model="importText" rows="16" class="w-full md-dlg-textarea" placeholder="{ &quot;v&quot;: 1, &quot;wol&quot;: &quot;...&quot;, &quot;luci&quot;: &quot;...&quot;, &quot;net&quot;: [&quot;...&quot;] }" />
      <template #footer>
        <div class="md-dlg-ft">
          <button class="md-dlg-btn" @click="doImportFromClipboard">
            {{ tt('importFromClipboard') }}
          </button>
          <div class="flex-1" />
          <button class="md-dlg-btn" @click="showImportDlg = false">
            {{ tt('cancel') }}
          </button>
          <button class="md-dlg-btn md-dlg-btn-save" @click="confirmImport">
            {{ tt('save') }}
          </button>
        </div>
      </template>
    </Dialog>

    <!-- Import Confirm Dialog -->
    <Dialog v-model:visible="showImportConfirm" modal :header="tt('importConfig')" :style="{ width: '85vw', maxWidth: '400px' }" class="md-dialog" :pt="{ header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }">
      <p class="text-base py-2" style="color:var(--md-text)">
        {{ tt('importConfirmMsg') }}
      </p>
      <template #footer>
        <div class="md-dlg-ft" style="justify-content:flex-end;gap:8px">
          <button class="md-dlg-btn" @click="showImportConfirm = false">
            {{ tt('cancel') }}
          </button>
          <button class="md-dlg-btn md-dlg-btn-save" @click="executeImport">
            {{ tt('confirm') }}
          </button>
        </div>
      </template>
    </Dialog>

    <!-- Help Dialog -->
    <Dialog v-model:visible="showHelpDlg" modal :header="tt('help')" :style="{ width: '90vw', maxWidth: '540px' }" class="md-dialog" :pt="{ header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }">
      <!-- WOL -->
      <div class="md-help-section">
        <div class="md-help-title">
          {{ tt('helpWolTitle') }}
        </div>
        <div class="md-help-desc">
          {{ tt('helpWolDesc') }}
        </div>
        <div class="md-help-dep">
          {{ tt('helpWolDep') }}
        </div>
        <a class="md-settings-btn md-settings-btn-primary md-help-link" @click.prevent="openHelpUrl(HELP_WOL_URL)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" y1="14" x2="21" y2="3" /></svg>
          {{ tt('helpWolLink') }}
        </a>
        <div class="md-help-cfg-label">
          {{ tt('helpCfgTitle') }}
        </div>
        <div class="md-help-cfg-wrap">
          <pre class="md-help-code">{{ HELP_CFG_WOL }}</pre>
          <button class="md-help-copy" @click="copyHelpCfg(HELP_CFG_WOL, 0)">
            {{ copiedIdx === 0 ? tt('copiedCode') : tt('copyCode') }}
          </button>
        </div>
        <div v-for="(p, i) in tt('helpWolParams').split('\n')" :key="i" class="md-help-param">
          <code>{{ p.split(' — ')[0] }}</code> — {{ p.split(' — ').slice(1).join(' — ') }}
        </div>
        <div class="md-help-note">
          {{ tt('helpWolSocks5') }}
        </div>
      </div>
      <!-- PC Monitoring -->
      <div class="md-help-section">
        <div class="md-help-title">
          {{ tt('helpPcTitle') }}
        </div>
        <div class="md-help-desc">
          {{ tt('helpPcDesc') }}
        </div>
        <div class="md-help-desc" style="white-space:pre-line;margin-bottom:8px">
          {{ tt('helpPcFeat') }}
        </div>
      </div>
      <!-- LuCI -->
      <div class="md-help-section">
        <div class="md-help-title">
          {{ tt('helpLuciTitle') }}
        </div>
        <div class="md-help-desc">
          {{ tt('helpLuciDesc') }}
        </div>
        <div class="md-help-desc" style="white-space:pre-line;margin-bottom:8px">
          {{ tt('helpLuciFeat') }}
        </div>
        <div class="md-help-cfg-label">
          {{ tt('helpCfgTitle') }}
        </div>
        <div class="md-help-cfg-wrap">
          <pre class="md-help-code">{{ HELP_CFG_LUCI }}</pre>
          <button class="md-help-copy" @click="copyHelpCfg(HELP_CFG_LUCI, 1)">
            {{ copiedIdx === 1 ? tt('copiedCode') : tt('copyCode') }}
          </button>
        </div>
        <div v-for="(p, i) in tt('helpLuciParams').split('\n')" :key="i" class="md-help-param">
          <code>{{ p.split(' — ')[0] }}</code> — {{ p.split(' — ').slice(1).join(' — ') }}
        </div>
        <div class="md-help-note">
          {{ tt('helpLuciSocks5') }}
        </div>
      </div>
      <!-- EasyTier -->
      <div class="md-help-section">
        <div class="md-help-title">
          {{ tt('helpNetTitle') }}
        </div>
        <div class="md-help-desc">
          {{ tt('helpNetDesc') }}
        </div>
        <div class="md-help-desc" style="white-space:pre-line;margin-bottom:8px">
          {{ tt('helpNetFeat') }}
        </div>
        <div class="md-help-cfg-label">
          {{ tt('helpCfgTitle') }}
        </div>
        <div class="md-help-cfg-wrap">
          <pre class="md-help-code">{{ HELP_CFG_NET }}</pre>
          <button class="md-help-copy" @click="copyHelpCfg(HELP_CFG_NET, 2)">
            {{ copiedIdx === 2 ? tt('copiedCode') : tt('copyCode') }}
          </button>
        </div>
        <div v-for="(p, i) in tt('helpNetParams').split('\n')" :key="i" class="md-help-param">
          <code>{{ p.split(' — ')[0] }}</code> — {{ p.split(' — ').slice(1).join(' — ') }}
        </div>
      </div>
      <!-- Import & Export -->
      <div class="md-help-section">
        <div class="md-help-title">
          {{ tt('helpBackupTitle') }}
        </div>
        <div class="md-help-desc">
          {{ tt('helpBackupDesc') }}
        </div>
      </div>
      <template #footer>
        <div class="md-dlg-ft" style="justify-content:flex-end">
          <button class="md-dlg-btn" @click="showHelpDlg = false">
            {{ tt('close') }}
          </button>
        </div>
      </template>
    </Dialog>

    <!-- About Dialog -->
    <Dialog v-model:visible="showAboutDlg" modal :header="tt('about')" :style="{ width: '90vw', maxWidth: '540px' }" class="md-dialog md-about-dlg" :pt="{ header: { class: 'md-dialog-hdr' }, content: { class: 'md-dialog-content' }, footer: { class: 'md-dialog-ft' } }">
      <div class="md-help-section">
        <div class="md-help-title">
          EasyTier + WOLPlus <span class="md-about-ver">v{{ pkg.version }}</span>
        </div>
        <div class="md-help-desc">
          {{ tt('aboutDesc') }}
        </div>
      </div>
      <div class="md-help-section">
        <div class="md-help-title">
          {{ tt('aboutLinks') }}
        </div>
        <div class="md-help-desc" style="margin-bottom:4px">
          {{ tt('aboutThisProject') }}
        </div>
        <a class="md-settings-btn md-settings-btn-primary md-help-link" @click.prevent="openHelpUrl(ABOUT_THIS_URL)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" y1="14" x2="21" y2="3" /></svg>
          github.com/lymwhen/EasyTier
        </a>
        <div class="md-help-desc" style="margin-top:10px;margin-bottom:4px">
          {{ tt('aboutWolplusDep') }}
        </div>
        <a class="md-settings-btn md-settings-btn-primary md-help-link" @click.prevent="openHelpUrl(ABOUT_WOLPLUS_DEP_URL)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" y1="14" x2="21" y2="3" /></svg>
          github.com/lymwhen/luci-app-wolplus
        </a>
      </div>
      <hr class="md-help-divider">
      <div class="md-help-section">
        <div class="md-help-title">
          {{ tt('aboutThanks') }}
        </div>
        <div class="md-help-desc" style="margin-bottom:4px">
          <b>EasyTier</b> — {{ tt('aboutEasyTierDesc') }}
        </div>
        <a class="md-settings-btn md-settings-btn-primary md-help-link" @click.prevent="openHelpUrl(ABOUT_EASYTIER_URL)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" y1="14" x2="21" y2="3" /></svg>
          github.com/EasyTier/EasyTier
        </a>
        <div class="md-help-desc" style="margin-top:10px;margin-bottom:4px">
          <b>luci-app-wolplus</b> — {{ tt('aboutWolplusDesc') }}
        </div>
        <a class="md-settings-btn md-settings-btn-primary md-help-link" @click.prevent="openHelpUrl(ABOUT_WOLPLUS_URL)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" y1="14" x2="21" y2="3" /></svg>
          github.com/animegasan/luci-app-wolplus
        </a>
      </div>
      <template #footer>
        <div class="md-dlg-ft" style="justify-content:flex-end">
          <button class="md-dlg-btn" @click="showAboutDlg = false">
            {{ tt('close') }}
          </button>
        </div>
      </template>
    </Dialog>

    <!-- Snackbar -->
    <div v-if="snackShow" class="md-snackbar" :class="`md-snack-${snackType}`">
      <span class="md-snack-indicator" aria-hidden="true" />
      <span class="md-snack-message">{{ snackMsg }}</span>
    </div>
  </div>
</template>

<style>
:root {
  font-size: 15px;
  font-family:
    -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'SF Pro Text', 'Segoe UI', Roboto, 'PingFang SC',
    'Microsoft YaHei', sans-serif;
  color-scheme: light;
  --bg: #f1f1f0;
  --ink: #0d1117;
  --muted: #6f716f;
  --line: rgba(36, 38, 36, 0.12);
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
  --pc-series-1: #2a78d6;
  --pc-series-2: #eb6834;
  --pc-grid: rgba(68, 71, 78, 0.14);
  --pc-good: #0ca30c;
  --pc-warn: #fab219;
  --pc-off: #8e9099;
  --pc-bg: #f9f9ff;
  --pc-surface-lowest: #fff;
  --pc-surface-low: #f3f3fa;
  --pc-surface: #ededf4;
  --pc-surface-high: #e7e8ee;
  --pc-surface-highest: #e2e2e9;
  --pc-card-bg: #fff;
  --pc-card-open-bg: #fff;
  --pc-child-bg: #f5f6f9;
  --pc-child-icon-bg: #fff;
  --pc-on-surface: #191c20;
  --pc-on-surface-var: #44474e;
  --pc-outline: #74777f;
  --pc-outline-var: #c4c6d0;
  --pc-primary: #415f91;
  --pc-primary-container: #d6e3ff;
  --pc-on-primary-container: #001b3e;
  --pc-secondary-container: #dae2f9;
  --pc-on-secondary-container: #131c2b;
  --pc-error: #ba1a1a;
  --pw-on-bg: var(--n3-good-container);
  --pw-on-fg: var(--n3-on-good-container);
  --pw-off-bg: var(--n3-error-container);
  --pw-off-fg: var(--n3-on-error-container);
  --md-primary: #0a84ff;
  --md-text: #0d1117;
  --md-secondary: #6f716f;
  --md-muted: #6f716f;
  --md-border: rgba(36, 38, 36, 0.12);
  --md-shadow: 0 12px 34px rgba(45, 57, 76, 0.08);
  --md-radius: 22px;
  --n3-surface: #fff;
  --n3-surface-container-low: #f3f4f9;
  --n3-surface-container-high: #e6e8ee;
  --n3-on-surface-var: #44474e;
  --n3-outline-variant: #d9dbe2;
  --n3-primary: #2f6fed;
  --n3-primary-container: #d9e7ff;
  --n3-on-primary-container: #001b3f;
  --n3-secondary-container: #e0e2ec;
  --n3-on-secondary-container: #1a1c22;
  --n3-tertiary-container: #e8def8;
  --n3-on-tertiary-container: #1d192b;
  --n3-good: #1e7d3f;
  --n3-good-container: #e2f4e7;
  --n3-on-good-container: #0f3d1e;
  --n3-warn: #8a5a00;
  --n3-warn-container: #fdeecd;
  --n3-on-warn-container: #2b1d00;
  --n3-error: #d93025;
  --n3-error-container: #fde7e5;
  --n3-on-error-container: #410002;
  --n3-card-border: transparent;
  --n3-card-shadow: 0 1px 2px rgba(16, 18, 24, 0.08), 0 1px 3px 1px rgba(16, 18, 24, 0.06);
  --n3-hero-pink-a: #fdeef1;
  --n3-hero-pink-b: #fdf4f6;
  --n3-hero-green-a: #e6f5ec;
  --n3-hero-green-b: #f1f9f4;
  --n3-hero-blue-a: #e8f1ff;
  --n3-hero-blue-b: #f3f7ff;
  --safe-top: env(safe-area-inset-top, 0px);
  --safe-bottom: env(safe-area-inset-bottom, 0px);
}
html[data-theme='dark'] {
  color-scheme: dark;
  --bg: #000;
  --ink: #f7f7f7;
  --muted: #a0a0a0;
  --line: rgba(255, 255, 255, 0.13);
  --panel: rgba(0, 0, 0, 0.2);
  --panel-strong: rgba(12, 12, 12, 0.9);
  --accent: #8ab4f8;
  --accent-soft: rgba(138, 180, 248, 0.18);
  --green: #44d46f;
  --pc-series-1: #3987e5;
  --pc-series-2: #d95926;
  --pc-grid: rgba(196, 198, 208, 0.12);
  --pc-good: #8fdc8a;
  --pc-warn: #fab219;
  --pc-off: #8e9099;
  --pc-bg: #000;
  --pc-surface-lowest: #000;
  --pc-surface-low: #0b0b0d;
  --pc-surface: #111214;
  --pc-surface-high: #18191c;
  --pc-surface-highest: #202125;
  --pc-card-bg: #1c1f24;
  --pc-card-open-bg: #1c1f24;
  --pc-child-bg: #000;
  --pc-child-icon-bg: #111214;
  --pc-on-surface: #e2e2e9;
  --pc-on-surface-var: #c4c6d0;
  --pc-outline: #8e9099;
  --pc-outline-var: #2e3137;
  --pc-primary: #aac7ff;
  --pc-primary-container: #1d3a66;
  --pc-on-primary-container: #d6e3ff;
  --pc-secondary-container: #262d3a;
  --pc-on-secondary-container: #dae2f9;
  --pc-error: #ffb4ab;
  --pw-on-bg: var(--n3-good-container);
  --pw-on-fg: var(--n3-on-good-container);
  --pw-off-bg: var(--n3-error-container);
  --pw-off-fg: var(--n3-on-error-container);
  --md-primary: #8ab4f8;
  --md-text: #f7f7f7;
  --md-secondary: #a0a0a0;
  --md-muted: #a0a0a0;
  --md-border: rgba(255, 255, 255, 0.13);
  --md-shadow: 0 12px 34px rgba(0, 0, 0, 0.5);
  --n3-surface: #1c1f24;
  --n3-surface-container-low: #181a1f;
  --n3-surface-container-high: #2a2d33;
  --n3-on-surface-var: #c4c6d0;
  --n3-outline-variant: #3a3d44;
  --n3-primary: #a9c7ff;
  --n3-primary-container: #1d3a66;
  --n3-on-primary-container: #d6e3ff;
  --n3-secondary-container: #2c3038;
  --n3-on-secondary-container: #e0e2ec;
  --n3-tertiary-container: #4a4458;
  --n3-on-tertiary-container: #e6dff8;
  --n3-good: #8fd6a4;
  --n3-good-container: #1e3d28;
  --n3-on-good-container: #b8ecc6;
  --n3-warn: #eec26a;
  --n3-warn-container: #3d2e10;
  --n3-on-warn-container: #fdeecd;
  --n3-error: #ffb4ab;
  --n3-error-container: #5c0a0a;
  --n3-on-error-container: #ffdad6;
  --n3-card-border: #26282e;
  --n3-card-shadow: none;
  --n3-hero-pink-a: #2c1d23;
  --n3-hero-pink-b: #20181d;
  --n3-hero-green-a: #18271d;
  --n3-hero-green-b: #161e18;
  --n3-hero-blue-a: #121c28;
  --n3-hero-blue-b: #0d1319;
  background: #000;
}

.md-app {
  position: relative;
  overflow: hidden;
  height: 100vh;
  height: 100dvh;
  color: var(--ink);
  -webkit-text-size-adjust: 100%;
  -webkit-tap-highlight-color: transparent;
  background: var(--bg);
}
.md-page {
  position: absolute;
  inset: 0;
}
/* Net, router & settings pages reuse the computer page background (light #f9f9ff / dark #000) */
.md-page-bg {
  background: var(--pc-bg);
}

.md-dialog {
  --p-dialog-border-radius: 20px;
  --p-dialog-background: var(--panel-strong);
  --p-dialog-color: var(--ink);
}
.md-dialog.p-dialog {
  border-radius: 20px;
  overflow: hidden;
  background: var(--panel-strong);
  backdrop-filter: saturate(260%) contrast(1.12) blur(22px);
  -webkit-backdrop-filter: saturate(260%) contrast(1.12) blur(22px);
  box-shadow: 0 24px 80px rgba(30, 42, 60, 0.22);
  border: 1px solid var(--line);
}
html[data-theme='dark'] .md-dialog.p-dialog {
  border: 1px solid rgba(255, 255, 255, 0.08);
}
html[data-theme='dark'] .p-dialog {
  background: rgba(12, 12, 12, 0.95) !important;
}
html[data-theme='dark'] .p-dialog-header,
html[data-theme='dark'] .p-dialog-content,
html[data-theme='dark'] .p-dialog-footer {
  background: transparent !important;
}
.overflow-y-auto {
  scrollbar-width: none;
}
.overflow-y-auto::-webkit-scrollbar {
  display: none;
}
.md-text-muted {
  color: var(--muted);
}
.md-dlg-textarea {
  font-family: 'Courier New', monospace;
  font-size: 0.9rem;
  line-height: 1.5;
}
.md-dialog .p-dialog-header {
  background: transparent !important;
}
.md-dialog .p-dialog-content {
  background: transparent !important;
}
.md-dialog .p-dialog-footer {
  background: transparent !important;
}
.md-dialog .p-dialog-header-icon:focus,
.md-dialog .p-dialog-close-button:focus,
.md-dialog .p-dialog-header-close:focus {
  outline: none !important;
  box-shadow: none !important;
}
.md-dialog-hdr {
  font-size: 0.95rem !important;
  font-weight: 700 !important;
  padding: 14px 18px 0 !important;
  color: var(--ink) !important;
  background: transparent !important;
  border: none !important;
}
.md-dialog-content {
  padding: 10px 18px 0 !important;
  background: transparent !important;
}
.md-dialog-ft {
  padding: 0 !important;
  background: transparent !important;
  border: none !important;
}
.md-dialog .p-textarea {
  margin-bottom: 0 !important;
}
.md-dialog textarea {
  border: 1px solid var(--line) !important;
  border-radius: 12px !important;
  padding: 10px 12px !important;
  background: rgba(255, 255, 255, 0.5) !important;
  color: var(--ink) !important;
  font-size: 0.88rem !important;
  outline: none !important;
  box-shadow: none !important;
  resize: vertical;
}
html[data-theme='dark'] .md-dialog textarea {
  background: rgba(255, 255, 255, 0.06) !important;
}
.md-dialog textarea:focus {
  border-color: var(--accent) !important;
}

.md-dlg-ft {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px 12px;
  width: 100%;
}
.md-dlg-btn {
  padding: 8px 14px;
  border: none;
  border-radius: 10px;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  background: transparent;
  color: var(--accent);
  min-width: 64px;
  letter-spacing: 0.3px;
  transition:
    transform 160ms ease,
    background-color 160ms ease;
}
.md-dlg-btn:active {
  background: rgba(10, 132, 255, 0.15);
  transform: scale(0.92);
}
.md-dlg-btn-save {
  color: var(--accent);
  font-weight: 700;
  background: transparent;
}
.md-dlg-btn-save:active {
  background: rgba(10, 132, 255, 0.15);
}
.md-dlg-btn-del {
  color: #ef5350;
  background: transparent;
}
.md-dlg-btn-del:active {
  background: rgba(239, 83, 80, 0.15);
}
</style>

<style scoped>
/* ===== Glass Header ===== */
.md-hdr {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 12px;
  height: calc(var(--safe-top) + 56px);
  padding-top: var(--safe-top);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.36), rgba(238, 246, 255, 0.14)), var(--panel);
  border-bottom: 1px solid rgba(38, 56, 82, 0.075);
  backdrop-filter: saturate(260%) contrast(1.12) blur(22px);
  -webkit-backdrop-filter: saturate(260%) contrast(1.12) blur(22px);
  box-shadow:
    0 8px 20px rgba(40, 60, 90, 0.055),
    inset 0 -1px 0 rgba(255, 255, 255, 0.2);
  z-index: 10;
}
.md-hdr::before {
  content: '';
  position: absolute;
  inset: 0;
  z-index: 0;
  background: linear-gradient(180deg, #fcfcfc 0%, transparent 34%);
  opacity: 1;
  pointer-events: none;
}
.md-hdr > * {
  position: relative;
  z-index: 1;
}
html[data-theme='dark'] .md-hdr {
  background: linear-gradient(180deg, rgba(18, 18, 18, 0.34), rgba(0, 0, 0, 0.1)), rgba(0, 0, 0, 0.08);
  border-bottom-color: rgba(255, 255, 255, 0.09);
  backdrop-filter: brightness(1.04) saturate(360%) contrast(1.02) blur(18px);
  -webkit-backdrop-filter: brightness(1.04) saturate(360%) contrast(1.02) blur(18px);
  box-shadow:
    0 8px 20px rgba(0, 0, 0, 0.38),
    inset 0 -1px 0 rgba(255, 255, 255, 0.075);
}
html[data-theme='dark'] .md-hdr::before {
  background: linear-gradient(180deg, var(--bg) 0%, transparent 34%);
}
.md-hdr-title {
  font-size: 1.28rem;
  font-weight: 820;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  letter-spacing: -0.01em;
  margin-left: 2px;
}
.md-connected-text {
  font-size: 0.78rem;
  color: var(--green);
  font-weight: 600;
  white-space: nowrap;
}
.md-hdr-btn {
  display: inline-grid;
  place-items: center;
  width: 40px;
  height: 40px;
  flex: none;
  padding: 0;
  border: none;
  border-radius: 20px;
  background: transparent;
  color: var(--n3-on-surface-var);
  cursor: pointer;
  transition:
    transform 120ms ease,
    background-color 180ms ease,
    color 180ms ease;
}
.md-hdr-btn:hover:not(:disabled) {
  transform: scale(1.04);
}
.md-hdr-btn:active:not(:disabled) {
  transform: scale(0.94);
  background: color-mix(in srgb, currentColor 10%, transparent);
}
.md-hdr-btn-glass {
  background: transparent;
  backdrop-filter: none;
  -webkit-backdrop-filter: none;
  box-shadow: none;
}
.md-hdr-btn-glass:active:not(:disabled) {
  background: color-mix(in srgb, currentColor 12%, transparent);
  transform: scale(0.94);
}

/* Scroll view */
.md-scroll-view {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  right: 0;
  overflow-y: auto;
  -webkit-overflow-scrolling: touch;
  overscroll-behavior: auto;
  scrollbar-width: none;
}
.md-scroll-view::-webkit-scrollbar {
  display: none;
}
.md-scroll-content {
  padding: calc(var(--safe-top) + 66px) 12px calc(var(--safe-bottom) + 88px);
}
/* For Router tab iframe area */
.md-iframe-area {
  position: absolute;
  top: calc(var(--safe-top) + 56px);
  bottom: calc(var(--safe-bottom) + 76px);
  left: 0;
  right: 0;
}
/* Connection info is now integrated into the hero card (no separate bar) */
.md-hero-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}
/* Connected hero card: green-tinted accent */
.md-hero-connected {
  background: linear-gradient(135deg, var(--n3-hero-green-a), var(--n3-hero-green-b)) !important;
}
html[data-theme='dark'] .md-hero-connected {
  background: linear-gradient(135deg, var(--n3-hero-green-a), var(--n3-hero-green-b)) !important;
}
.md-status-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 12px;
  border-radius: 8px;
  background: var(--n3-good-container);
  color: var(--n3-on-good-container);
  font-size: 13px;
  font-weight: 500;
  flex-shrink: 0;
}
.md-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: currentColor;
}
.md-hero-head-right {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}
.md-hero-conn-info {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px 10px;
  margin-top: 4px;
}
.md-hero-iface {
  margin-top: 10px;
}
.md-hero-iface-row {
  font-size: 0.72rem;
  color: var(--muted);
  line-height: 1.6;
  word-break: break-all;
}
.md-hero-ip {
  font-size: 14px;
  font-weight: 400;
  color: var(--n3-on-surface-var);
  font-variant-numeric: tabular-nums;
}

/* Chips */
.md-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 32px;
  padding: 0 12px;
  border-radius: 8px;
  background: var(--n3-primary-container);
  color: var(--n3-on-primary-container);
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
}

/* NAT colors (chip with semantic colors) */
.md-nat-g {
  background: var(--n3-good-container);
  color: var(--n3-on-good-container);
}
.md-nat-b {
  background: var(--n3-primary-container);
  color: var(--n3-on-primary-container);
}
.md-nat-o {
  background: var(--n3-tertiary-container);
  color: var(--n3-on-tertiary-container);
}

.md-connect-btn {
  position: relative;
  overflow: visible;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 30px 44px;
  border: none;
  border-radius: 26px;
  min-width: 140px;
  background: var(--accent);
  color: #fff;
  font-size: 1.05rem;
  font-weight: 700;
  cursor: pointer;
  box-shadow: var(--md-shadow);
  transition:
    background 0.4s ease,
    color 0.4s ease,
    box-shadow 0.4s ease,
    transform 0.15s ease;
}
.md-connect-ico {
  position: relative;
  width: 32px;
  height: 32px;
  flex-shrink: 0;
}
.md-connect-ico svg {
  position: absolute;
  inset: 0;
  transition: opacity 0.3s ease;
}
.ico-hidden {
  opacity: 0;
  pointer-events: none;
}
.md-connect-btn-bounce {
  animation: connect-bounce 0.4s cubic-bezier(0.34, 1.56, 0.64, 1) forwards !important;
}
@keyframes connect-bounce {
  0% {
    transform: scale(1) !important;
  }
  35% {
    transform: scale(0.88);
  }
  100% {
    transform: scale(1);
  }
}
.md-connect-btn-loading {
  background: color-mix(in srgb, var(--accent) 60%, rgba(0, 0, 0, 0.25));
  color: #fff;
  cursor: not-allowed;
  box-shadow: none;
}
.md-connect-btn-loading::before,
.md-connect-btn-loading::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  pointer-events: none;
  animation: connect-ripple 1.6s cubic-bezier(0, 0.6, 0.4, 1) infinite;
}
.md-connect-btn-loading::before {
  background: color-mix(in srgb, var(--accent) 45%, transparent);
  transform: scale(1);
  opacity: 0;
}
.md-connect-btn-loading::after {
  background: color-mix(in srgb, var(--accent) 45%, transparent);
  transform: scale(1);
  opacity: 0;
  animation-delay: 0.55s;
}
html[data-theme='dark'] .md-connect-btn {
  background: var(--n3-primary-container);
  color: var(--n3-on-primary-container);
  box-shadow: var(--md-shadow);
}
html[data-theme='dark'] .md-connect-btn-loading {
  background: color-mix(in srgb, var(--n3-primary-container) 65%, rgba(0, 0, 0, 0.35));
  color: var(--n3-on-primary-container);
  box-shadow: none;
}
html[data-theme='dark'] .md-connect-btn-loading::before,
html[data-theme='dark'] .md-connect-btn-loading::after {
  background: color-mix(in srgb, var(--n3-primary-container) 60%, transparent);
}
html[data-theme='dark'] .md-connect-btn-loading svg {
  color: color-mix(in srgb, var(--n3-on-primary-container) 85%, transparent);
}
.md-connect-btn-loading svg {
  color: rgba(255, 255, 255, 0.8);
}
.md-connect-btn-loading:active {
  transform: none;
}
@keyframes connect-ripple {
  0% {
    transform: scale(1);
    opacity: 0.45;
  }
  100% {
    transform: scale(1.25);
    opacity: 0;
  }
}

/* Section */
.md-section {
  margin-bottom: 4px;
}
.md-sec-hdr {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 2px 8px 12px;
}
.md-sec-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--pc-primary);
  letter-spacing: 0.1px;
  flex: 1;
}

/* Card */
.md-card {
  background: var(--panel-strong);
  border-radius: var(--md-radius);
  box-shadow: var(--md-shadow);
  margin-bottom: 10px;
  cursor: pointer;
  transition:
    opacity 0.3s,
    box-shadow 0.3s,
    transform 0.3s;
}
html[data-theme='dark'] .md-card {
  box-shadow: var(--md-shadow);
}
.md-card-dim {
  opacity: 0.55;
}
.md-card-item {
  margin-bottom: 10px;
}
.md-settings-card {
  margin-bottom: 10px;
}
.md-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
}
.md-row-top {
  padding-bottom: 1px;
  align-items: center;
}
.md-row-bot {
  padding-top: 1px;
  align-items: flex-end;
}
.md-card-tags {
  display: flex;
  align-items: flex-end;
  justify-content: flex-end;
  flex-wrap: nowrap;
  gap: 4px;
  min-width: 0;
  max-width: 72%;
}
.md-card-tags > .md-rt,
.md-card-tags > .md-chip-nat {
  flex-shrink: 0;
}
.md-chip-proto {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.md-name {
  font-size: 0.9rem;
  font-weight: 760;
}
.md-sub {
  font-size: 0.76rem;
  color: var(--muted);
  margin-top: 2px;
  line-height: 1.3;
}
.md-chevron {
  width: 18px;
  height: 18px;
  opacity: 0.45;
  transition: transform 0.2s;
  flex-shrink: 0;
  color: var(--muted);
}
.md-chevron-open {
  transform: rotate(180deg);
}
.md-extra {
  margin: 0;
  padding: 8px 16px 12px;
  font-size: 0.76rem;
  color: var(--muted);
  line-height: 1.6;
  border-top: 1px solid var(--line);
}

/* Dot */
.md-dot {
  width: 11px;
  height: 11px;
  border-radius: 50%;
  flex-shrink: 0;
}
.md-dot-on {
  background: var(--green);
}
.md-dot-off {
  background: rgba(127, 127, 127, 0.35);
}
.md-dot-ph {
  animation: md-pulse 1s infinite;
}
.md-dot-waking {
  background: #ff9f0a;
}
.md-dot-shutting {
  background: #ef5350;
}
@keyframes md-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.3;
  }
}
@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.4;
  }
}
.animate-pulse {
  animation: pulse 1.5s ease-in-out infinite;
}

/* Route tag (chip with semantic colors) */
.md-rt {
  display: inline-flex;
  align-items: center;
  height: 28px;
  padding: 0 10px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0;
  flex-shrink: 0;
}
.md-rt-l {
  background: var(--n3-primary-container);
  color: var(--n3-on-primary-container);
}
.md-rt-p {
  background: var(--n3-good-container);
  color: var(--n3-on-good-container);
}
.md-rt-r {
  background: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
}

/* Switch menu */
.md-switch-menu {
  position: absolute;
  right: 0;
  top: 44px;
  z-index: 200;
  min-width: 200px;
  max-width: 260px;
  padding: 6px 0;
  overflow: hidden;
  border-radius: 16px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.36), rgba(238, 246, 255, 0.14)), var(--panel-strong);
  backdrop-filter: saturate(260%) contrast(1.12) blur(22px);
  -webkit-backdrop-filter: saturate(260%) contrast(1.12) blur(22px);
  box-shadow:
    0 12px 34px rgba(45, 57, 76, 0.08),
    inset 0 0 0 1px rgba(255, 255, 255, 0.76);
  border: 1px solid var(--line);
}
html[data-theme='dark'] .md-switch-menu {
  background: linear-gradient(180deg, rgba(52, 52, 52, 0.36), rgba(22, 22, 22, 0.14)), var(--panel-strong);
  box-shadow:
    0 12px 34px rgba(0, 0, 0, 0.24),
    inset 0 0 0 1px rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.08);
}
.md-sw-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  font-size: 0.95rem;
  cursor: pointer;
  white-space: nowrap;
  border-radius: 10px;
  margin: 0 4px;
  transition: background-color 160ms ease;
}
.md-sw-item:active {
  background: rgba(0, 0, 0, 0.06);
}
html[data-theme='dark'] .md-sw-item:active {
  background: rgba(255, 255, 255, 0.06);
}
.md-sw-del:active {
  background: rgba(239, 83, 80, 0.12);
  color: #ef5350;
}
.md-sw-divider {
  height: 1px;
  background: var(--line);
  margin: 4px 0;
}
.md-sw-check {
  color: var(--accent);
  font-weight: 700;
  width: 18px;
  display: inline-block;
}
.md-sw-del {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: none;
  cursor: pointer;
  color: var(--muted);
  font-size: 1.1rem;
  border-radius: 50%;
  flex-shrink: 0;
  transition: background-color 160ms ease;
}

.md-about-ver {
  font-size: 0.75rem;
  font-weight: 400;
  color: var(--muted);
  margin-left: 8px;
}

/* Toggle switch */
.md-toggle {
  width: 48px;
  height: 28px;
  border-radius: 999px;
  background: var(--n3-surface-container-high);
  border: 2px solid var(--pc-outline);
  box-sizing: border-box;
  flex-shrink: 0;
  position: relative;
  cursor: pointer;
  transition:
    background 0.2s,
    border-color 0.2s;
}
.md-toggle::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 4px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--pc-outline);
  transform: translateY(-50%);
  transition:
    left 0.25s cubic-bezier(0.2, 0, 0, 1),
    width 0.25s cubic-bezier(0.2, 0, 0, 1),
    height 0.25s cubic-bezier(0.2, 0, 0, 1),
    background-color 0.2s;
}
.md-toggle-on {
  background: var(--accent);
  border-color: var(--accent);
}
.md-toggle-on::after {
  left: 20px;
  width: 22px;
  height: 22px;
  background: #fff;
}
html[data-theme='dark'] .md-toggle-on::after {
  background: #0d1117;
}

/* Select dropdown — matches button style */
.md-select {
  height: 34px;
  padding: 0 26px 0 12px;
  border: none;
  border-radius: 17px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  line-height: 34px;
  background-color: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
  flex-shrink: 0;
  appearance: none;
  -webkit-appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 24 24'%3E%3Cpath d='M7 10l5 5 5-5z' fill='%236f716f'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  background-size: 9px 9px;
  transition:
    transform 160ms ease,
    background-color 160ms ease;
}
.md-select::-webkit-calendar-picker-indicator {
  opacity: 0;
  width: 0;
  height: 0;
}
.md-select:active {
  transform: scale(0.95);
  background-color: var(--n3-surface-container-high);
}
.md-select:focus {
  outline: none;
}

/* ===== Bottom Navigation ===== */
.md-tabs {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: calc(var(--safe-bottom) + 76px);
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  padding: 6px 16px calc(var(--safe-bottom) + 8px);
  overflow: hidden;
  background:
    linear-gradient(180deg, rgba(252, 255, 255, 0.075), rgba(222, 244, 255, 0.018)), rgba(246, 252, 255, 0.04);
  backdrop-filter: brightness(1.045) saturate(520%) contrast(1.03) blur(14px);
  -webkit-backdrop-filter: brightness(1.045) saturate(520%) contrast(1.03) blur(14px);
  border-top: 1px solid rgba(38, 56, 82, 0.07);
  box-shadow:
    0 -5px 16px rgba(36, 52, 76, 0.038),
    inset 0 1px 0 rgba(255, 255, 255, 0.14);
  z-index: 10;
}
.md-tabs::before {
  content: '';
  position: absolute;
  inset: 0;
  z-index: 0;
  background:
    linear-gradient(180deg, transparent 66%, rgba(252, 252, 252, 0.96) 100%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.055), rgba(211, 239, 255, 0.015));
  pointer-events: none;
}
.md-tabs > * {
  position: relative;
  z-index: 1;
}
html[data-theme='dark'] .md-tabs {
  background: linear-gradient(180deg, rgba(10, 10, 10, 0.1), rgba(0, 0, 0, 0.02)), rgba(0, 0, 0, 0.06);
  border-top-color: rgba(255, 255, 255, 0.085);
  box-shadow:
    0 -5px 16px rgba(0, 0, 0, 0.34),
    inset 0 1px 0 rgba(255, 255, 255, 0.07);
}
html[data-theme='dark'] .md-tabs::before {
  background:
    linear-gradient(180deg, transparent 66%, #000 100%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.025), rgba(255, 255, 255, 0.008));
}
.md-tab {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  gap: 4px;
  min-width: 0;
  padding: 4px 0;
  border-radius: 0;
  background: transparent;
  border: none;
  color: var(--n3-on-surface-var);
  cursor: pointer;
  font-size: 12px;
  font-weight: 500;
  transition:
    color 180ms ease,
    transform 120ms ease;
}
.md-tab:active {
  transform: scale(0.96);
}
.md-tab-active {
  color: var(--n3-primary);
  background: transparent;
  box-shadow: none;
}
.md-tab-icon {
  display: grid;
  place-items: center;
  width: 56px;
  height: 32px;
  border-radius: 16px;
  transition:
    background-color 180ms ease,
    color 180ms ease;
}
.md-tab-active .md-tab-icon {
  background: var(--n3-primary-container);
  color: var(--n3-on-primary-container);
}
.md-tab-icon svg {
  width: 22px;
  height: 22px;
}
.md-tab-label {
  font-size: 12px;
  line-height: 16px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
.animate-spin {
  animation: spin 1s linear infinite;
}

/* Snackbar */
.md-snackbar {
  position: fixed;
  bottom: calc(var(--safe-bottom) + 80px);
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 10px 18px;
  border-radius: 999px;
  font-size: 0.84rem;
  font-weight: 600;
  background: var(--ink);
  color: var(--bg);
  box-shadow: 0 8px 28px rgba(0, 0, 0, 0.28);
  z-index: 999;
  animation: md-snack-in 0.25s ease-out;
  width: max-content;
  max-width: min(88vw, 560px);
}
.md-snack-message {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.md-snack-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex: 0 0 auto;
  background: currentColor;
  opacity: 0.82;
}
.md-snack-progress .md-snack-indicator {
  animation: md-snack-pulse 1.1s ease-in-out infinite;
}
.md-snack-success {
  background: #18794e;
  color: #fff;
}
.md-snack-error {
  background: #b3261e;
  color: #fff;
}
.md-snack-info,
.md-snack-progress {
  background: var(--ink);
  color: var(--bg);
}
@keyframes md-snack-pulse {
  0%,
  100% {
    transform: scale(0.8);
    opacity: 0.55;
  }
  50% {
    transform: scale(1.2);
    opacity: 1;
  }
}
@keyframes md-snack-in {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

/* Traffic speed chart */
.md-traffic-card {
  --panel: var(--panel-strong);
  --muted: var(--n3-on-surface-var);
  padding: 16px;
  margin-top: 6px;
  margin-bottom: 16px;
  cursor: default;
}
/* Dark mode: list-style cards share the network list item surface */
html[data-theme='dark'] .md-settings-card,
html[data-theme='dark'] .md-traffic-card {
  background: var(--n3-surface);
}
.md-traffic-hdr {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 12px;
  font-weight: 600;
  margin-bottom: 6px;
}
.md-traffic-label {
  display: inline-flex;
  align-items: center;
  margin: 0;
  white-space: nowrap;
}
.md-traffic-total,
.md-traffic-current {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 12px;
  font-weight: 600;
  flex-shrink: 0;
}
.md-traffic-total {
  text-align: left;
}
.md-traffic-current {
  justify-content: flex-end;
}
.md-traffic-up-sub {
  color: var(--green);
  font-size: 0.7rem;
}
.md-traffic-down-sub {
  color: var(--accent);
  font-size: 0.7rem;
}

/* Network peer / server cards: n3 list layout, no hover state. */
.net-peer-card {
  margin-bottom: 12px;
  overflow: hidden;
  border: 0;
  border-radius: 24px;
  background: var(--n3-surface);
  box-shadow: var(--md-shadow);
  cursor: pointer;
  transition:
    background-color 0.3s,
    border-color 0.3s,
    box-shadow 0.3s;
}
.net-peer-card:focus-visible {
  outline: 2px solid var(--n3-primary);
  outline-offset: 2px;
}
.net-peer-main {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
}
.net-peer-avatar {
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  flex: none;
  border-radius: 12px;
  background: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
}
.net-peer-body {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.net-peer-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-width: 0;
}
.net-peer-row-bottom {
  align-items: center;
}
.net-peer-title {
  display: flex;
  align-items: baseline;
  gap: 4px;
  flex: 1;
  min-width: 0;
}
.net-peer-dot {
  flex: none;
  color: var(--n3-on-surface-var);
  font-size: 12px;
  line-height: 16px;
  opacity: 0.6;
}
.net-peer-name {
  flex: 0 1 auto;
  min-width: 0;
  overflow: hidden;
  font-size: 15px;
  line-height: 22px;
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.net-peer-ping {
  flex: none;
  color: var(--n3-primary);
  font-size: 12px;
  line-height: 16px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.net-peer-ping-muted {
  color: var(--n3-on-surface-var);
}
.net-peer-left {
  display: flex;
  flex: 1;
  align-items: center;
  min-width: 0;
}
.net-peer-ip {
  flex: none;
  color: var(--n3-on-surface-var);
  font-size: 12px;
  line-height: 16px;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.net-peer-flow {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--n3-on-surface-var);
  font-size: 11px;
  line-height: 15px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.net-peer-flow-up {
  color: var(--n3-good);
}
.net-peer-flow-down {
  color: var(--n3-primary);
}
.net-peer-tags {
  display: flex;
  flex: 0 1 auto;
  flex-wrap: wrap;
  justify-content: flex-end;
  align-content: center;
  gap: 8px;
  max-width: 58%;
  min-width: 0;
}
.net-peer-tags .md-chip {
  height: 24px;
  padding: 0 8px;
  font-size: 11px;
}
.net-peer-tags .md-rt {
  height: 24px;
  padding: 0 8px;
  font-size: 11px;
}
.net-peer-expand {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows 0.4s cubic-bezier(0.2, 0, 0, 1);
}
.net-peer-card-open .net-peer-expand {
  grid-template-rows: 1fr;
}
.net-peer-expand-inner {
  min-height: 0;
  overflow: hidden;
}
.net-peer-info {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 0 14px 12px;
  padding-top: 10px;
  border-top: 1px solid var(--n3-outline-variant);
  opacity: 0;
  transform: translateY(-4px);
  transition:
    opacity 0.25s ease,
    transform 0.35s cubic-bezier(0.2, 0, 0, 1);
}
.net-peer-card-open .net-peer-info {
  opacity: 1;
  transform: none;
}
.net-peer-info span {
  display: inline-flex;
  align-items: center;
  min-width: 0;
  height: 28px;
  padding: 0 12px;
  border-radius: 8px;
  background: var(--n3-secondary-container);
  color: var(--n3-on-surface-var);
  font-size: 12px;
}
.net-peer-info b {
  max-width: 100%;
  margin-left: 6px;
  overflow: hidden;
  color: var(--ink);
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Parent-child hierarchy: parent remains primary, children form an indented tree workspace */
.md-device-group {
  position: relative;
  margin-bottom: 12px;
}
.md-device-parent {
  position: relative;
  z-index: 1;
  margin-bottom: 0;
}
.md-device-group-open > .md-device-parent {
  border: 1px solid color-mix(in srgb, var(--accent) 22%, transparent);
  box-shadow:
    var(--md-shadow),
    inset 0 0 0 1px color-mix(in srgb, var(--accent) 12%, transparent);
}
.md-device-children {
  position: relative;
  margin: 0 0 2px 18px;
  padding: 0 0 0 20px;
}
.md-device-children::before {
  content: '';
  position: absolute;
  left: 5px;
  top: -8px;
  height: 32px;
  width: 2px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 38%, transparent);
}
.md-stats-card {
  padding: 12px 16px;
  margin-bottom: 8px;
  cursor: default;
  position: relative;
  border: 1px solid var(--line);
  border-radius: 16px;
  background: color-mix(in srgb, var(--panel-strong) 76%, transparent);
  box-shadow: none;
}
.md-device-children > .md-stats-card::before {
  content: '';
  position: absolute;
  left: -15px;
  top: 24px;
  width: 15px;
  border-top: 2px solid color-mix(in srgb, var(--accent) 30%, transparent);
  border-radius: 999px;
}
.md-device-children > .md-stats-card:not(:last-child)::after {
  content: '';
  position: absolute;
  left: -15px;
  top: 24px;
  height: calc(100% + 8px);
  width: 2px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent) 26%, transparent);
}
.md-device-children > .md-stats-card:last-of-type {
  margin-bottom: 6px;
}
.md-stats-loading-card {
  padding: 8px 16px;
}
.md-stats-info-card {
  padding: 10px 16px;
}
.md-stats-info {
  font-size: 0.68rem;
  color: var(--muted);
  opacity: 0.85;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.md-stats-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 16px 0;
  color: var(--muted);
  font-size: 0.82rem;
}
.md-stats-collapse {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  margin-top: -2px;
  margin-bottom: 2px;
  padding: 2px 0;
  color: var(--muted);
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  transition: color 0.2s;
}
.md-stats-collapse:active {
  color: var(--accent);
}
.md-stats-hdr {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 4px 8px;
  margin-bottom: 2px;
}
.md-stats-legends {
  margin-left: auto;
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 2px 10px;
}
.md-stats-label {
  font-size: 0.76rem;
  font-weight: 700;
  color: var(--ink);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.md-stats-svg {
  width: 100%;
  height: 90px;
  display: block;
}
.md-stats-chart-wrap {
  position: relative;
}
.md-stats-legend {
  font-size: 0.68rem;
  font-weight: 600;
  flex-shrink: 0;
}
.md-disk-total,
.md-part-row {
  position: relative;
  height: 20px;
  border-radius: 6px;
  background: rgba(127, 127, 127, 0.06);
  overflow: hidden;
  margin: 2px 0;
  display: flex;
  align-items: center;
  padding: 0 6px;
}
.md-part-fill-bg {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  border-radius: 6px;
  background: var(--accent);
  transition: width 0.3s;
  opacity: 0.22;
}
.md-disk-total-usage,
.md-part-usage {
  position: relative;
  z-index: 1;
  font-size: 0.64rem;
  color: var(--ink);
  flex-shrink: 0;
  margin-left: auto;
  opacity: 0.7;
}
.md-part-label {
  position: relative;
  z-index: 1;
  font-size: 0.68rem;
  color: var(--muted);
  flex-shrink: 0;
}

/* LuCI iframe */
.luci-iframe {
  width: 100%;
  height: 100%;
  border: none;
}

/* Settings action buttons (tonal-btn style) */
.md-settings-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 40px;
  padding: 0 18px;
  border: none;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  flex-shrink: 0;
  transition:
    transform 120ms ease,
    background-color 200ms ease,
    color 200ms ease;
  background: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
}
.md-settings-btn svg {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}
.md-hdr .md-settings-btn {
  height: 34px;
  padding: 0 12px;
  gap: 6px;
  font-size: 0.78rem;
  font-weight: 500;
  border-radius: 17px;
}
.md-hdr .md-settings-btn svg {
  width: 16px;
  height: 16px;
}
.md-refresh-icon {
  display: inline-block;
}
.md-settings-btn:hover:not(:disabled) {
  transform: scale(1.02);
}
.md-settings-btn:active:not(:disabled) {
  transform: scale(0.95);
}
.md-settings-btn:disabled {
  opacity: 0.4;
  cursor: default;
}
.md-settings-btn-export,
.md-settings-btn-green {
  background: var(--n3-good-container);
  color: var(--n3-on-good-container);
}
.md-settings-btn-primary {
  background: var(--n3-primary-container);
  color: var(--n3-on-primary-container);
}
.md-settings-btn-white {
  background: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
}
.md-settings-btn-red {
  background: var(--n3-error-container);
  color: var(--n3-on-error-container);
}
/* Compact tonal buttons inside hero cards and settings rows */
.md-hero .md-settings-btn,
.md-settings-card .md-settings-btn {
  height: 34px;
  padding: 0 12px;
  gap: 6px;
  border-radius: 17px;
  font-size: 13px;
}
.md-hero .md-settings-btn svg,
.md-settings-card .md-settings-btn svg {
  width: 16px;
  height: 16px;
}

/* Help / About dialog content */
.md-help-section {
  padding: 12px 0;
}
.md-help-section + .md-help-section {
  border-top: 1px solid var(--line);
}
.md-about-dlg .md-help-section + .md-help-section {
  border-top: none;
}
.md-help-divider {
  border: none;
  border-top: 1px solid var(--line);
  margin: 0;
}
.md-help-title {
  font-size: 0.92rem;
  font-weight: 700;
  color: var(--ink);
  margin-bottom: 4px;
}
.md-help-desc {
  font-size: 0.82rem;
  color: var(--muted);
  line-height: 1.5;
  margin-bottom: 4px;
}
.md-help-dep {
  font-size: 0.78rem;
  color: var(--muted);
  line-height: 1.4;
  margin-bottom: 8px;
}
.md-help-link {
  text-decoration: none;
  margin-bottom: 10px;
}
.md-help-cfg-label {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--muted);
  margin-bottom: 4px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.md-help-cfg-wrap {
  position: relative;
}
.md-help-code {
  font-size: 0.76rem;
  line-height: 1.55;
  padding: 10px 12px;
  margin: 0 0 4px;
  border-radius: 10px;
  background: rgba(127, 127, 127, 0.06);
  color: var(--ink);
  overflow-x: auto;
  white-space: pre;
  font-family: 'Cascadia Code', 'Fira Code', 'JetBrains Mono', 'SF Mono', 'Consolas', monospace;
}
html[data-theme='dark'] .md-help-code {
  background: rgba(255, 255, 255, 0.04);
}
.md-help-copy {
  position: absolute;
  top: 6px;
  right: 8px;
  padding: 3px 10px;
  border: none;
  border-radius: 6px;
  font-size: 0.72rem;
  font-weight: 700;
  cursor: pointer;
  background: rgba(127, 127, 127, 0.15);
  color: var(--muted);
  transition: opacity 0.15s;
}
.md-help-copy:active {
  opacity: 0.8;
}
html[data-theme='dark'] .md-help-copy {
  background: rgba(255, 255, 255, 0.1);
}
.md-help-param {
  font-size: 0.78rem;
  color: var(--muted);
  line-height: 1.6;
  margin-bottom: 2px;
}
.md-help-param code {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--ink);
  margin-right: 8px;
}
.md-help-note {
  padding: 10px 12px;
  border-radius: 10px;
  background: rgba(127, 127, 127, 0.06);
  font-size: 0.78rem;
  color: var(--muted);
  line-height: 1.5;
  margin-top: 8px;
}
html[data-theme='dark'] .md-help-note {
  background: rgba(255, 255, 255, 0.04);
}
.md-help-note code {
  font-size: 0.75rem;
  color: var(--ink);
}

/* Symbol icon (settings items, list items) */
.md-symbol {
  display: grid;
  place-items: center;
  width: 46px;
  height: 46px;
  border-radius: 16px;
  background: var(--tint, var(--accent-soft));
  color: var(--tone, var(--accent));
  flex-shrink: 0;
}
.md-symbol svg {
  width: 22px;
  height: 22px;
  stroke-width: 2.15;
}
.md-settings-card .md-symbol {
  width: 40px;
  height: 40px;
  border-radius: 14px;
}
.md-settings-card .md-symbol svg {
  width: 18px;
  height: 18px;
}
.md-settings-card .md-name {
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;
}
/* Label stack (right-aligned meta info) — bordered label style */
.md-label-stack {
  display: grid;
  gap: 2px;
  justify-items: end;
  flex-shrink: 0;
  padding: 4px 10px;
  border-radius: 8px;
  background: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
}
.md-label-stack b {
  color: currentColor;
  font-size: 11px;
  line-height: 14px;
  font-weight: 500;
}
.md-label-stack span {
  color: currentColor;
  font-size: 10px;
  line-height: 12px;
  opacity: 0.72;
}

/* Hero card (top of each tab) */
.md-hero {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 20px;
  margin-bottom: 16px;
  border: 0;
  border-radius: 28px;
  box-shadow: var(--md-shadow);
  background: linear-gradient(135deg, var(--n3-hero-blue-a), var(--n3-hero-blue-b));
  cursor: default;
}
html[data-theme='dark'] .md-hero {
  background: linear-gradient(135deg, var(--n3-hero-blue-a), var(--n3-hero-blue-b));
  box-shadow: var(--md-shadow);
}
/* Per-tab hero tints: 电脑=粉, 组网=绿, 设置=蓝 */
.md-hero-purple {
  background: linear-gradient(135deg, var(--n3-hero-pink-a), var(--n3-hero-pink-b)) !important;
}
html[data-theme='dark'] .md-hero-purple {
  background: linear-gradient(135deg, var(--n3-hero-pink-a), var(--n3-hero-pink-b)) !important;
}
.md-hero-blue {
  background: linear-gradient(135deg, var(--n3-hero-blue-a), var(--n3-hero-blue-b)) !important;
}
html[data-theme='dark'] .md-hero-blue {
  background: linear-gradient(135deg, var(--n3-hero-blue-a), var(--n3-hero-blue-b)) !important;
}
.md-hero-title {
  margin: 0;
  font-size: 20px;
  font-weight: 500;
  line-height: 28px;
  flex: 1;
  min-width: 0;
  word-break: break-word;
}
.md-hero-text {
  margin-top: 4px;
  color: var(--n3-on-surface-var);
  font-size: 13px;
  line-height: 20px;
  font-weight: 400;
}
.md-hero-metrics {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 10px;
}
.md-hero-metric {
  padding: 10px 14px 8px;
  border-radius: 16px;
  border: 0;
  background: var(--n3-surface);
  box-shadow: none;
}
html[data-theme='dark'] .md-hero-metric {
  background: #111214;
  box-shadow: none;
}
.md-hero-mv {
  font-size: 15px;
  font-weight: 500;
  line-height: 22px;
  letter-spacing: 0.15px;
  font-variant-numeric: tabular-nums;
}
.md-hero-ml {
  margin-top: 2px;
  color: var(--n3-on-surface-var);
  font-size: 12px;
  line-height: 16px;
  font-weight: 400;
}

/* ===== n2 computer list ===== */
.pc-page {
  color: var(--pc-on-surface);
  background: var(--pc-bg);
}

.pc-page .md-scroll-content {
  width: 100%;
  max-width: 1040px;
  margin: 0 auto;
  padding-right: 16px;
  padding-left: 16px;
}

.pc-empty-state {
  display: flex;
  min-height: min(52vh, 420px);
  align-items: center;
  justify-content: center;
  color: var(--pc-on-surface-var);
}
/* Vertically center the empty state within the viewport when no PCs exist */
.pc-page .md-scroll-content.pc-content-center {
  display: flex;
  flex-direction: column;
  justify-content: center;
  box-sizing: border-box;
  min-height: 100dvh;
}

/* Unified Material You empty state for PC / Router / Network tabs */
.md-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 24px;
  text-align: center;
}
.md-iframe-area .md-empty {
  height: 100%;
}
.md-empty-ico {
  display: grid;
  place-items: center;
  width: 80px;
  height: 80px;
  flex: none;
  border-radius: 24px;
  background: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
}
.md-empty-text {
  font-size: 15px;
  line-height: 22px;
  font-weight: 500;
  color: var(--n3-on-surface-var);
}

.pc-head.sl {
  position: relative;
  overflow: hidden;
  isolation: isolate;
}

.pc-list-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 2px 8px 12px;
  color: var(--pc-on-surface-var);
}

.pc-list-head h2 {
  margin: 0;
  color: var(--pc-primary);
  font-size: 14px;
  font-weight: 500;
  letter-spacing: 0.1px;
}

.pc-list-hint {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  font-size: 12px;
  white-space: nowrap;
}

.pc-list-hint svg {
  width: 16px;
  height: 16px;
  flex: none;
}

.pc-card {
  --card-i: var(--pc-i, 0);
  position: relative;
  margin-bottom: 12px;
  overflow: hidden;
  border-radius: 24px;
  background: var(--pc-card-bg);
  box-shadow: var(--md-shadow);
  transition:
    background-color 0.3s,
    box-shadow 0.3s,
    opacity 0.3s;
}

@keyframes pc-card-in {
  from {
    opacity: 0;
    transform: translateY(14px);
  }
  to {
    transform: translateY(0);
  }
}

html[data-theme='dark'] .pc-card {
  background: var(--pc-card-bg);
  box-shadow: var(--md-shadow);
}

.pc-card-open {
  background: var(--pc-card-open-bg);
}

.pc-card-dim {
  opacity: 0.55;
}

.pc-head {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto auto auto auto;
  grid-template-areas: 'av ti st pw mo ch';
  align-items: center;
  gap: 4px 14px;
  min-width: 0;
  padding: 12px 12px 12px 16px;
  border-radius: 24px;
  cursor: pointer;
  user-select: none;
  -webkit-user-select: none;
}

.pc-avatar {
  grid-area: av;
  display: grid;
  place-items: center;
  width: 40px;
  height: 40px;
  margin-inline: 8px;
  flex: none;
  border-radius: 12px;
  background: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
  transition:
    background-color 0.3s,
    color 0.3s;
}

.pc-card-open .pc-avatar {
  background: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
}

.pc-avatar-online,
.pc-card-open .pc-avatar-online {
  background: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
}

.pc-title {
  grid-area: ti;
  min-width: 0;
}

.pc-name {
  margin: 0;
  overflow: hidden;
  color: var(--pc-on-surface);
  font-size: 16px;
  font-weight: 500;
  line-height: 24px;
  letter-spacing: 0.15px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pc-meta {
  display: flex;
  gap: 8px;
  min-width: 0;
  overflow: hidden;
  color: var(--pc-on-surface-var);
  font-size: 13px;
  white-space: nowrap;
}

.pc-ip {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pc-os {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pc-os::before {
  content: '·';
  margin-right: 8px;
}

.pc-status {
  grid-area: st;
  display: inline-flex;
  align-items: center;
  justify-self: end;
  gap: 4px;
  min-width: 0;
  max-width: 100%;
  height: 24px;
  min-height: 24px;
  max-height: 24px;
  box-sizing: border-box;
  overflow: hidden;
  margin-right: 8px;
  padding: 0 8px;
  border: 0;
  border-radius: 8px;
  background: var(--n3-secondary-container);
  color: var(--pc-on-surface-var);
  font-size: 11px;
  font-weight: 500;
  line-height: 16px;
  white-space: nowrap;
}

.pc-status > span:last-child {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pc-status-dot {
  width: 6px;
  height: 6px;
  flex: none;
  border-radius: 50%;
  background: var(--pc-off);
}

.pc-status-online {
  background: var(--n3-good-container);
  color: var(--n3-on-good-container);
}

.pc-status-online .pc-status-dot {
  background: var(--pc-good);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--pc-good) 22%, transparent);
}

.pc-status-checking .pc-status-dot,
.pc-status-routerOffline .pc-status-dot {
  background: var(--pc-off);
  animation: pc-pulse 1.2s ease-in-out infinite;
}

.pc-status-waking .pc-status-dot,
.pc-status-shutting .pc-status-dot {
  width: 10px;
  height: 10px;
  border: 2px solid var(--pc-warn);
  border-right-color: transparent;
  background: transparent;
  animation: pc-spin 0.9s linear infinite;
}

@keyframes pc-spin {
  to {
    transform: rotate(360deg);
  }
}

@keyframes pc-pulse {
  50% {
    opacity: 0.35;
    transform: scale(0.82);
  }
}

.pc-icon-btn {
  position: relative;
  display: inline-grid;
  place-items: center;
  width: 40px;
  height: 40px;
  flex: none;
  padding: 0;
  border: 0;
  border-radius: 20px;
  background: transparent;
  color: var(--pc-on-surface-var);
  cursor: pointer;
  transition:
    transform 0.12s ease,
    background-color 0.18s ease,
    color 0.18s ease;
}

.pc-icon-btn:not(:disabled):hover {
  transform: scale(1.04);
}
.pc-icon-btn:not(:disabled):active {
  transform: scale(0.94);
  background: color-mix(in srgb, currentColor 12%, transparent);
}
.pc-icon-btn:disabled {
  opacity: 0.38;
  cursor: default;
}

.pc-power {
  grid-area: pw;
  border: 0;
  transition:
    background-color 0.25s,
    color 0.25s;
}

.pc-power-on {
  background: var(--pw-on-bg);
  color: var(--pw-on-fg);
}

.pc-power-off {
  background: var(--pw-off-bg);
  color: var(--pw-off-fg);
}

html[data-theme='dark'] .pc-power-on {
  background: var(--pw-on-bg);
  color: var(--pw-on-fg);
}

html[data-theme='dark'] .pc-power-off {
  background: var(--pw-off-bg);
  color: var(--pw-off-fg);
}

.pc-power:disabled {
  background: transparent;
  color: var(--pc-on-surface-var);
  box-shadow: inset 0 0 0 1px var(--n3-outline-variant);
}

.pc-more {
  grid-area: mo;
}
.pc-chevron {
  grid-area: ch;
}

.pc-chevron svg {
  transition: transform 0.35s cubic-bezier(0.2, 0, 0, 1);
}

.pc-card-open .pc-chevron svg {
  transform: rotate(180deg);
}

.pc-body {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows 0.4s cubic-bezier(0.2, 0, 0, 1);
}

.pc-body-open {
  grid-template-rows: 1fr;
}

.pc-body-inner {
  min-height: 0;
  overflow: hidden;
}

.pc-panel {
  position: relative;
  padding: 4px 20px 16px;
  opacity: 0;
  transform: translateY(-8px);
  transition:
    opacity 0.25s,
    transform 0.4s cubic-bezier(0.2, 0, 0, 1);
}

.pc-body-open .pc-panel {
  opacity: 1;
  transform: none;
}

.pc-info {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin: 8px 0 12px;
}

.pc-info span {
  display: inline-flex;
  align-items: center;
  min-width: 0;
  height: 28px;
  padding: 0 12px;
  border-radius: 8px;
  background: var(--pc-surface-high);
  color: var(--pc-on-surface-var);
  font-size: 12px;
}

.pc-info b {
  max-width: 220px;
  margin-left: 6px;
  overflow: hidden;
  color: var(--pc-on-surface);
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pc-offline-state,
.pc-loading-state,
.pc-error-state,
.pc-command-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  min-height: 150px;
  padding: 28px 16px;
  border-radius: 16px;
  background: var(--pc-child-bg);
  color: var(--pc-on-surface-var);
  text-align: center;
}

.pc-offline-state p,
.pc-error-state p,
.pc-command-empty p {
  margin: 0;
}

.pc-loading-state {
  min-height: 96px;
  flex-direction: row;
}

.pc-busy-spinner {
  color: var(--pc-primary);
  animation: pc-spin 0.9s linear infinite;
}

.pc-error-state {
  color: var(--pc-error);
}

html[data-theme='dark'] .pc-error-state {
  color: var(--pc-error);
}

.pc-inline-error {
  display: flex;
  grid-column: 1 / -1;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid color-mix(in srgb, var(--pc-error) 28%, transparent);
  border-radius: 12px;
  background: color-mix(in srgb, var(--pc-error) 8%, var(--pc-surface-lowest));
  color: var(--pc-error);
  font-size: 12px;
  overflow-wrap: anywhere;
}

.pc-inline-error .pc-tonal-btn {
  min-height: 32px;
  flex: none;
  padding: 0 12px;
  font-size: 12px;
}

.pc-tonal-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 40px;
  padding: 0 16px;
  border: 0;
  border-radius: 20px;
  background: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.pc-tonal-btn:disabled {
  opacity: 0.38;
  cursor: default;
}

.pc-tonal-btn.pc-power-on {
  background: var(--pw-on-bg);
  color: var(--pw-on-fg);
}

html[data-theme='dark'] .pc-tonal-btn.pc-power-on {
  background: var(--pw-on-bg);
  color: var(--pw-on-fg);
}

.pc-metrics {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.pc-metric {
  --panel: var(--pc-child-bg);
  --muted: var(--pc-on-surface-var);
  --chart-grid: var(--pc-grid);
  min-width: 0;
  padding: 16px 16px 12px;
  border-radius: 16px;
  background: var(--pc-child-bg);
  transition: background-color 0.3s;
}

html[data-theme='dark'] .pc-metric {
  background: var(--pc-child-bg);
}

.pc-metric-head {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.pc-metric-head + .computer-chart {
  margin-top: 12px;
}

.pc-metric-icon {
  display: grid;
  place-items: center;
  width: 36px;
  height: 36px;
  flex: none;
  border-radius: 12px;
  background: var(--pc-child-icon-bg);
  color: var(--pc-on-surface-var);
}

.pc-metric-name {
  flex: 1;
  min-width: 0;
}

.pc-metric-label {
  color: var(--pc-on-surface);
  font-size: 14px;
  font-weight: 500;
}

.pc-metric-spec {
  overflow: hidden;
  color: var(--pc-on-surface-var);
  font-size: 12px;
  line-height: 16px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pc-metric-value {
  flex: none;
  color: var(--pc-on-surface);
  font-size: 28px;
  font-weight: 400;
  line-height: 36px;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.pc-metric-value small {
  margin-left: 2px;
  color: var(--pc-on-surface-var);
  font-size: 14px;
}

.pc-metric-sub {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px 16px;
  min-height: 20px;
  margin: 8px 0;
  color: var(--pc-on-surface-var);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.pc-metric-sub b {
  color: var(--pc-on-surface);
  font-weight: 500;
}

.pc-disk-sub {
  gap: 6px 10px;
}

.pc-usage-bar {
  display: block;
  flex: 1 1 100%;
  height: 8px;
  overflow: hidden;
  border-radius: 4px;
  background: var(--pc-surface-highest);
}

.pc-usage-bar i {
  display: block;
  height: 100%;
  border-radius: 4px;
  background: var(--pc-series-1);
}

.pc-key {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.pc-key::before {
  width: 12px;
  height: 2px;
  border-radius: 1px;
  background: var(--pc-key);
  content: '';
}

.pc-partitions {
  display: grid;
  gap: 4px;
  margin-top: 8px;
}

.pc-partition {
  display: grid;
  grid-template-columns: minmax(48px, auto) minmax(60px, 1fr) auto;
  align-items: center;
  gap: 8px;
  color: var(--pc-on-surface-var);
  font-size: 11px;
}

.pc-debug-info {
  margin: 12px 0 0;
  grid-column: 1 / -1;
  overflow: hidden;
  padding: 10px 12px;
  border-radius: 10px;
  background: var(--pc-surface-high);
  color: var(--pc-on-surface-var);
  font-family: monospace;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pc-collapse-btn {
  display: flex;
  width: max-content;
  max-width: 100%;
  min-height: 40px;
  align-items: center;
  justify-content: center;
  gap: 6px;
  margin: 12px auto 0;
  padding: 0 16px;
  border: 0;
  border-radius: 20px;
  background: var(--n3-secondary-container);
  color: var(--n3-on-secondary-container);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
}

.pc-collapse-btn:hover {
  background: var(--n3-surface-container-high);
}

.pc-menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  min-height: 48px;
  padding: 8px 12px;
  border: 0;
  border-radius: 12px;
  background: var(--pc-surface-high);
  color: var(--pc-on-surface);
  text-align: left;
  cursor: pointer;
}

.pc-menu-item:disabled {
  opacity: 0.38;
  cursor: default;
}

.pc-menu-item:hover:not(:disabled) {
  background: color-mix(in srgb, var(--pc-primary) 10%, var(--pc-surface-high));
}

.pc-menu-icon {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  flex: none;
  border-radius: 10px;
  background: var(--pc-surface-highest);
  color: var(--pc-on-surface-var);
}

.pc-command-text,
.pc-menu-text {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
}

.pc-command-text small,
.pc-menu-text small {
  overflow: hidden;
  color: var(--pc-on-surface-var);
  font-size: 11px;
  line-height: 16px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pc-output {
  max-height: 260px;
  margin: 0;
  padding: 10px;
  overflow: auto;
  border-radius: 10px;
  background: rgba(0, 0, 0, 0.72);
  color: #e6edf3;
  font-family: 'Roboto Mono', 'JetBrains Mono', Consolas, monospace;
  font-size: 11px;
  line-height: 1.5;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  overscroll-behavior: contain;
}

.pc-run-error {
  margin-top: 6px;
  color: var(--pc-on-surface-var);
  font-size: 11px;
  line-height: 1.45;
}

.pc-run-error {
  color: var(--pc-error);
}

html[data-theme='dark'] .pc-run-error {
  color: var(--pc-error);
}

.pc-menu {
  position: fixed;
  z-index: 80;
  min-width: 240px;
  max-width: min(340px, calc(100vw - 16px));
  max-height: 50vh;
  padding: 8px 0;
  overflow: auto;
  border: none;
  border-radius: 16px;
  background: var(--pc-surface-high);
  color: var(--pc-on-surface);
  box-shadow: 0 2px 6px 2px rgba(0, 0, 0, .12), 0 1px 2px rgba(0, 0, 0, .2);
  transform-origin: top right;
  animation: pc-menu-in 0.18s cubic-bezier(0.2, 0, 0, 1);
}

@keyframes pc-menu-in {
  from {
    opacity: 0;
    transform: scale(0.92);
  }
}

.pc-menu-label,
.pc-menu-note {
  padding: 8px 16px 4px;
  color: var(--pc-on-surface-var);
  font-size: 12px;
}

.pc-menu-label {
  overflow: hidden;
  font-weight: 500;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pc-menu-note {
  padding-bottom: 8px;
}

.pc-menu .pc-menu-item {
  position: relative;
  overflow: hidden;
  isolation: isolate;
  border-radius: 0;
  background: transparent;
}

.pc-menu .pc-menu-item::before {
  position: absolute;
  inset: 0;
  z-index: -1;
  background: currentColor;
  content: '';
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.15s;
}

.pc-menu .pc-menu-item:not(:disabled):hover::before {
  opacity: 0.08;
}

.pc-menu .pc-menu-item:not(:disabled):active::before {
  opacity: 0.12;
}

.pc-menu-divider {
  height: 1px;
  margin: 8px 0;
  background: var(--pc-outline-var);
}

.pc-manager-title {
  color: var(--pc-primary);
  font-size: 13px;
  font-weight: 700;
}

.pc-field {
  display: grid;
  gap: 5px;
  min-width: 0;
  color: var(--pc-on-surface-var);
  font-size: 12px;
  font-weight: 600;
}

.pc-field input,
.pc-field select {
  width: 100%;
  min-width: 0;
  height: 40px;
  padding: 0 12px;
  border: 1px solid var(--pc-outline);
  border-radius: 10px;
  outline: none;
  background: rgba(255, 255, 255, 0.55);
  color: var(--pc-on-surface);
  font-size: 14px;
}

html[data-theme='dark'] .pc-field input,
html[data-theme='dark'] .pc-field select {
  background: rgba(255, 255, 255, 0.06);
}

.pc-field input:focus,
.pc-field select:focus {
  border-color: var(--pc-primary);
}

.pc-manager-list {
  display: grid;
  gap: 4px;
  margin-left: 0;
}

.pc-manager-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 4px 6px 12px;
  border-radius: 12px;
  background: var(--pc-surface-high);
}

.pc-danger-icon {
  color: var(--pc-error);
}

/* Shared page-entry animation: replays whenever a tab becomes active. */
.md-page.page-active .md-hero,
.md-page.page-active .md-hero-metric,
.md-page.page-active .md-settings-card,
.md-page.page-active .md-traffic-card,
.md-page.page-active .net-peer-card,
.md-page.page-active .md-empty,
.md-page.page-active .pc-card {
  animation: pc-card-in 0.35s ease-out both;
  animation-delay: calc(var(--card-i, 0) * 60ms);
}

.md-page.page-active .md-hero {
  --card-i: 0;
}
.md-page.page-active .md-hero-metric:nth-child(2) {
  --card-i: 1;
}
.md-page.page-active .md-hero-metric:nth-child(3) {
  --card-i: 2;
}
.md-page.page-active .md-hero-metric:nth-child(4) {
  --card-i: 3;
}
.md-page.page-active .md-settings-card:nth-child(2) {
  --card-i: 1;
}
.md-page.page-active .md-settings-card:nth-child(3) {
  --card-i: 2;
}
.md-page.page-active .md-settings-card:nth-child(4) {
  --card-i: 3;
}
.md-page.page-active .md-settings-card:nth-child(5) {
  --card-i: 4;
}
.md-page.page-active .md-settings-card:nth-child(6) {
  --card-i: 5;
}
.md-page.page-active .md-settings-card:nth-child(7) {
  --card-i: 6;
}
.md-page.page-active .md-settings-card:nth-child(8) {
  --card-i: 7;
}
.md-page.page-active .md-settings-card:nth-child(9) {
  --card-i: 8;
}
.md-page.page-active .md-traffic-card {
  --card-i: 1;
}

@media (max-width: 720px) {
  .pc-metrics {
    grid-template-columns: minmax(0, 1fr);
  }
}

@media (max-width: 600px) {
  .pc-page .md-scroll-content {
    padding-right: 12px;
    padding-left: 12px;
  }

  .pc-list-head {
    padding: 0 4px 8px;
  }

  .pc-list-hint {
    display: none;
  }

  .pc-head {
    grid-template-columns: auto minmax(0, 1fr) auto auto auto;
    grid-template-areas:
      'av ti pw mo ch'
      'av st pw mo ch';
    gap: 2px 10px;
    padding: 12px 8px 12px 12px;
  }

  .pc-avatar {
    width: 40px;
    height: 40px;
    border-radius: 12px;
  }

  .pc-status {
    justify-self: start;
    margin: 0;
    padding: 0 8px;
    background: var(--n3-secondary-container);
    font-size: 11px;
  }

  .pc-status-online {
    background: var(--n3-good-container);
  }

  .pc-os {
    display: none;
  }

  .pc-panel {
    padding: 4px 12px 12px;
  }

  .pc-info b {
    max-width: 150px;
  }

  .pc-metric {
    padding: 14px 12px 10px;
  }

  .pc-metric-value {
    font-size: 24px;
    line-height: 32px;
  }

  .pc-metric-icon {
    width: 32px;
    height: 32px;
  }

  .pc-metric-head {
    gap: 8px;
  }

  .pc-avatar {
    margin-inline: 4px;
  }

  .pc-menu {
    max-width: calc(100vw - 16px);
  }

  .net-peer-main {
    gap: 10px;
    padding: 14px;
  }

  .net-peer-name {
    font-size: 15px;
  }

  .net-peer-tags {
    max-width: 62%;
    gap: 6px;
  }
}

@media (max-width: 380px) {
  .pc-head {
    gap: 2px 7px;
    padding-left: 10px;
  }

  .pc-icon-btn {
    width: 36px;
    height: 36px;
    border-radius: 18px;
  }

  .pc-name {
    font-size: 15px;
  }

  .pc-meta {
    font-size: 12px;
  }

  .pc-metric-value {
    font-size: 21px;
    line-height: 28px;
  }

  .pc-info span {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

@media (prefers-reduced-motion: reduce) {
  .pc-card,
  .md-page.page-active .md-hero,
  .md-page.page-active .md-hero-metric,
  .md-page.page-active .md-settings-card,
  .md-page.page-active .md-traffic-card,
  .md-page.page-active .net-peer-card,
  .net-peer-expand,
  .net-peer-info,
  .pc-body,
  .pc-panel,
  .pc-chevron svg,
  .pc-status-dot,
  .pc-menu {
    transition: none !important;
    animation: none !important;
  }
}

/* ==================== Remote command execution ==================== */
.exec-overlay {
  position: fixed;
  inset: 0;
  z-index: 210;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
  animation: exec-fade 0.18s ease-out;
}

@keyframes exec-fade {
  from {
    opacity: 0;
  }
}

.exec-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  min-width: 240px;
  max-width: min(360px, calc(100vw - 48px));
  padding: 28px 32px 20px;
  border-radius: 28px;
  background: var(--pc-surface-high);
  color: var(--pc-on-surface);
  box-shadow: 0 2px 6px 2px rgba(0, 0, 0, .12), 0 1px 2px rgba(0, 0, 0, .2);
  text-align: center;
}

.exec-title {
  margin-top: 4px;
  font-size: 13px;
  color: var(--pc-on-surface-var);
}

.exec-name {
  font-size: 16px;
  font-weight: 600;
  overflow-wrap: anywhere;
}

.exec-device {
  font-size: 12px;
  color: var(--pc-on-surface-var);
}

.exec-hint {
  margin-top: 6px;
  font-size: 11px;
  line-height: 1.5;
  color: var(--pc-on-surface-var);
  opacity: 0.85;
}

.md-sheet-scrim {
  position: fixed;
  inset: 0;
  z-index: 220;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
  animation: exec-fade 0.18s ease-out;
}

.md-sheet {
  display: flex;
  flex-direction: column;
  width: min(640px, 100%);
  max-height: 82vh;
  padding: 10px 20px calc(16px + env(safe-area-inset-bottom, 0px));
  border-radius: 28px 28px 0 0;
  background: var(--pc-surface-high);
  color: var(--pc-on-surface);
  box-shadow: 0 -2px 12px rgba(0, 0, 0, 0.22);
  animation: sheet-in 0.24s cubic-bezier(0.2, 0, 0, 1);
}

@keyframes sheet-in {
  from {
    transform: translateY(40px);
    opacity: 0.6;
  }
}

.md-sheet-handle {
  flex: none;
  width: 32px;
  height: 4px;
  margin: 0 auto 12px;
  border-radius: 2px;
  background: var(--pc-outline-var);
}

.md-sheet-head {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 10px;
}

.md-sheet-titles {
  flex: 1;
  min-width: 0;
}

.md-sheet-title {
  font-size: 16px;
  font-weight: 600;
}

.md-sheet-sub {
  overflow: hidden;
  font-size: 12px;
  color: var(--pc-on-surface-var);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.md-sheet-dot {
  flex: none;
  width: 10px;
  height: 10px;
  border-radius: 50%;
}

.md-sheet-dot-ok {
  background: var(--pc-good);
}

.md-sheet-dot-err {
  background: var(--pc-error);
}

.md-sheet-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-right: 4px;
}

.md-sheet-ft {
  display: flex;
  flex: none;
  align-items: center;
  gap: 8px;
  padding-top: 14px;
}

.pc-output-err {
  color: var(--pc-error);
}

.pc-manager-item-edit {
  border-color: var(--pc-primary);
  background: var(--pc-secondary-container);
}

.pc-mode-seg {
  display: inline-flex;
  width: fit-content;
  overflow: hidden;
  border: 1px solid var(--pc-outline-var);
  border-radius: 999px;
}

.pc-mode-seg button {
  padding: 8px 18px;
  border: none;
  border-radius: 0 !important;
  background: transparent;
  color: var(--pc-on-surface-var);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s;
}

.pc-mode-seg button + button {
  border-left: 1px solid var(--pc-outline-var) !important;
}

.pc-mode-seg button:hover {
  background: rgba(128, 128, 128, 0.12);
}

/* ============ Remote commands v2: reference-aligned skin ============ */
.pc-menu .pc-menu-icon {
  width: 20px;
  height: 20px;
  border-radius: 0;
  background: transparent;
  color: var(--pc-on-surface-var);
}

html[data-theme='dark'] .pc-menu,
html[data-theme='dark'] .cmd-confirm-card {
  box-shadow: 0 0 0 1px #2a2c31;
}

.pc-manager-item {
  background: var(--pc-surface-highest);
  cursor: pointer;
  user-select: none;
  transition: background-color 0.15s;
}

.pc-manager-item:hover {
  background: color-mix(in srgb, var(--pc-on-surface) 8%, var(--pc-surface-highest));
}

.pc-task-item {
  cursor: default;
}

.pc-task-item:hover {
  background: var(--pc-surface-highest);
}

.pc-task-main {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  gap: 2px;
}

.pc-task-cmd {
  overflow: hidden;
  font-size: 13px;
  line-height: 18px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pc-task-sub {
  font-size: 11px;
  line-height: 16px;
  color: var(--pc-on-surface-var);
}

.pc-field textarea {
  width: 100%;
  min-width: 0;
  min-height: 64px;
  padding: 10px 12px;
  border: 1px solid var(--pc-outline);
  border-radius: 10px;
  outline: none;
  background: rgba(255, 255, 255, 0.55);
  color: var(--pc-on-surface);
  font-size: 14px;
  line-height: 1.5;
  resize: vertical;
}

html[data-theme='dark'] .pc-field textarea {
  background: rgba(255, 255, 255, 0.06);
}

.pc-field textarea:focus {
  border-color: var(--pc-primary);
}

.pc-mode-seg {
  display: inline-flex;
  height: 40px;
  overflow: hidden;
  border: 1px solid var(--pc-outline);
  border-radius: 20px;
}

.pc-mode-seg button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 16px;
  border: none;
  border-radius: 0 !important;
  background: transparent;
  color: var(--pc-on-surface-var);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
}

.pc-mode-seg button + button {
  border-left: 1px solid var(--pc-outline) !important;
}

.pc-mode-seg button:hover {
  background: rgba(128, 128, 128, 0.12);
}

.pc-mode-seg [aria-checked='true'] {
  background: var(--pc-secondary-container) !important;
  color: var(--pc-on-secondary-container) !important;
}

.cmd-confirm-scrim {
  position: fixed;
  inset: 0;
  z-index: 240;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: rgba(0, 0, 0, 0.7);
  animation: exec-fade 0.15s ease-out;
}

.cmd-confirm-card {
  display: flex;
  flex-direction: column;
  gap: 16px;
  width: min(400px, 100%);
  max-height: calc(100vh - 48px);
  padding: 24px;
  overflow: auto;
  border-radius: 28px;
  background: var(--pc-surface-high);
  color: var(--pc-on-surface);
  box-shadow:
    0 2px 6px 2px rgba(0, 0, 0, 0.12),
    0 1px 2px rgba(0, 0, 0, 0.2);
  animation: cmd-confirm-in 0.25s cubic-bezier(0.2, 0, 0, 1);
}

@keyframes cmd-confirm-in {
  from {
    opacity: 0;
    transform: translateY(-12px) scale(0.96);
  }
}

.cmd-confirm-icon {
  align-self: center;
  color: var(--pc-primary);
}

.cmd-confirm-icon.danger {
  color: var(--pc-error);
}

.cmd-confirm-title {
  margin: 0;
  font-size: 24px;
  line-height: 32px;
  font-weight: 400;
  text-align: center;
  overflow-wrap: anywhere;
}

.cmd-confirm-text {
  margin: 0;
  color: var(--pc-on-surface-var);
  font-size: 14px;
  line-height: 20px;
  overflow-wrap: anywhere;
}

.cmd-confirm-code {
  max-height: 120px;
  margin: -8px 0 0;
  padding: 10px 12px;
  overflow: auto;
  border-radius: 12px;
  background: var(--pc-surface-highest);
  color: var(--pc-on-surface-var);
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.cmd-confirm-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
  margin-top: 8px;
}

.pc-btn-filled {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  height: 40px;
  padding: 0 24px;
  border: 0;
  border-radius: 20px;
  background: var(--pc-primary);
  color: #fff;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

html[data-theme='dark'] .pc-btn-filled {
  color: #002b59;
}

.pc-btn-filled.danger {
  background: var(--pc-error);
  color: #fff;
}

html[data-theme='dark'] .pc-btn-filled.danger {
  color: #410e0b;
}

.pc-btn-filled:disabled {
  opacity: 0.38;
  cursor: default;
}

</style>
