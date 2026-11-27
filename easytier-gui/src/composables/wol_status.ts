export type WolPhase = 'idle' | 'waking' | 'shutting'
export type WolEffectiveState = 'checking' | 'routerOffline' | 'waking' | 'shutting' | 'online' | 'offline'
export type WolProbeState = 'checking' | 'reachable' | 'unreachable'

export interface WolStatusResponse {
  online?: unknown
  hostname?: unknown
  os?: unknown
  system_uptime_secs?: unknown
}

export interface WolDeviceStatus {
  online: boolean
  phase: WolPhase
  probeState: WolProbeState
  hostname?: string
  os?: string
  systemUptimeSecs?: number
  lastCheckedAt?: number
  lastSeenAt?: number
  consecutiveFailures: number
}

export interface WolRouterStatus {
  state: WolProbeState
  lastCheckedAt?: number
  consecutiveFailures: number
}

export const WOL_FAILURE_GRACE = 2

export function createWolDeviceStatus(): WolDeviceStatus {
  return {
    online: false,
    phase: 'idle',
    probeState: 'checking',
    consecutiveFailures: 0,
  }
}

export function createWolRouterStatus(): WolRouterStatus {
  return {
    state: 'checking',
    consecutiveFailures: 0,
  }
}

export function applyWolRouterSuccess(
  status: WolRouterStatus,
  now: number,
): WolRouterStatus {
  return {
    state: 'reachable',
    lastCheckedAt: now,
    consecutiveFailures: 0,
  }
}

export function applyWolRouterFailure(
  status: WolRouterStatus,
  now: number,
  grace = WOL_FAILURE_GRACE,
): WolRouterStatus {
  const consecutiveFailures = status.consecutiveFailures + 1
  const keepReachable = status.state === 'reachable'
    && consecutiveFailures < Math.max(1, grace)
  return {
    state: keepReachable ? 'reachable' : 'unreachable',
    lastCheckedAt: now,
    consecutiveFailures,
  }
}

export function effectiveWolState(
  router: WolRouterStatus | undefined,
  device: WolDeviceStatus,
  hasRouter: boolean,
  hasIp: boolean,
): WolEffectiveState {
  if (!hasRouter || router?.state === 'unreachable')
    return 'routerOffline'
  if (!router || router.state === 'checking')
    return 'checking'
  if (!hasIp)
    return 'offline'
  if (device.phase === 'waking')
    return 'waking'
  if (device.phase === 'shutting')
    return 'shutting'
  if (device.lastCheckedAt == null)
    return 'checking'
  return device.online ? 'online' : 'offline'
}

export function markWolChecking(status: WolDeviceStatus): WolDeviceStatus {
  return { ...status, probeState: 'checking' }
}

export function applyWolSuccess(
  status: WolDeviceStatus,
  response: WolStatusResponse,
  now: number,
): WolDeviceStatus {
  const online = response.online === true
  return {
    ...status,
    online,
    probeState: 'reachable',
    hostname: typeof response.hostname === 'string' && response.hostname
      ? response.hostname
      : status.hostname,
    os: typeof response.os === 'string' && response.os
      ? response.os
      : status.os,
    systemUptimeSecs: typeof response.system_uptime_secs === 'number'
      && Number.isFinite(response.system_uptime_secs)
      ? response.system_uptime_secs
      : status.systemUptimeSecs,
    lastCheckedAt: now,
    lastSeenAt: online ? now : status.lastSeenAt,
    consecutiveFailures: 0,
  }
}

export function applyWolFailure(
  status: WolDeviceStatus,
  now: number,
  grace = WOL_FAILURE_GRACE,
): WolDeviceStatus {
  const consecutiveFailures = status.consecutiveFailures + 1
  const keepLastOnline = status.online && consecutiveFailures < Math.max(1, grace)
  return {
    ...status,
    online: keepLastOnline,
    probeState: 'unreachable',
    lastCheckedAt: now,
    consecutiveFailures,
  }
}

export function forceWolOffline(
  status: WolDeviceStatus,
  now: number,
  keepPhase = false,
): WolDeviceStatus {
  return {
    ...status,
    online: false,
    probeState: 'reachable',
    phase: keepPhase ? status.phase : 'idle',
    lastCheckedAt: now,
    consecutiveFailures: 0,
  }
}
