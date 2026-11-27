import { describe, expect, it } from 'vitest'
import {
  applyWolFailure,
  applyWolRouterFailure,
  applyWolRouterSuccess,
  applyWolSuccess,
  createWolDeviceStatus,
  createWolRouterStatus,
  effectiveWolState,
} from './wol_status'

describe('wol status', () => {
  it('shows checking until the first router and device probes finish', () => {
    const router = createWolRouterStatus()
    const device = createWolDeviceStatus()

    expect(effectiveWolState(router, device, true, true)).toBe('checking')

    router.state = 'reachable'
    expect(effectiveWolState(router, device, true, true)).toBe('checking')
  })

  it('keeps one transient failure online and confirms the second failure', () => {
    const router = createWolRouterStatus()
    router.state = 'reachable'
    let device = applyWolSuccess(createWolDeviceStatus(), { online: true }, 1_000)

    device = applyWolFailure(device, 31_000)
    expect(effectiveWolState(router, device, true, true)).toBe('online')

    device = applyWolFailure(device, 61_000)
    expect(effectiveWolState(router, device, true, true)).toBe('offline')
  })

  it('lets router reachability take precedence over a stale device state', () => {
    const router = createWolRouterStatus()
    router.state = 'unreachable'
    const device = applyWolSuccess(createWolDeviceStatus(), { online: true }, 1_000)

    expect(effectiveWolState(router, device, true, true)).toBe('routerOffline')
  })

  it('absorbs one transient router failure before reporting offline', () => {
    let router = applyWolRouterSuccess(createWolRouterStatus(), 1_000)
    router = applyWolRouterFailure(router, 31_000)
    expect(router.state).toBe('reachable')

    router = applyWolRouterFailure(router, 61_000)
    expect(router.state).toBe('unreachable')
  })

  it('preserves a wake or shutdown phase while its probe is running', () => {
    const router = createWolRouterStatus()
    router.state = 'reachable'
    const device = applyWolSuccess(createWolDeviceStatus(), { online: false }, 1_000)
    device.phase = 'waking'

    expect(effectiveWolState(router, device, true, true)).toBe('waking')
  })
})
