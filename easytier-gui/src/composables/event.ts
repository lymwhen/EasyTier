import { listen } from "@tauri-apps/api/event";
import type { Event as TauriEvent } from "@tauri-apps/api/event";
import { type } from "@tauri-apps/plugin-os";
import { NetworkTypes } from "easytier-frontend-lib"
import { Utils } from "easytier-frontend-lib";

interface StoredGuiConfig {
    config: NetworkTypes.NetworkConfig
    source?: 'user' | 'webhook' | 'legacy'
}

const EVENTS = Object.freeze({
    SAVE_CONFIGS: 'save_configs',
    PRE_RUN_NETWORK_INSTANCE: 'pre_run_network_instance',
    POST_RUN_NETWORK_INSTANCE: 'post_run_network_instance',
    VPN_SERVICE_STOP: 'vpn_service_stop',
    DHCP_IP_CHANGED: 'dhcp_ip_changed',
    PROXY_CIDRS_UPDATED: 'proxy_cidrs_updated',
    EVENT_LAGGED: 'event_lagged',
});

function onSaveConfigs(event: TauriEvent<StoredGuiConfig[]>) {
    console.log(`Received event '${EVENTS.SAVE_CONFIGS}': ${event.payload}`);

    let localList: any[] = []
    try { localList = JSON.parse(localStorage.getItem('networkList') || '[]'); if (!Array.isArray(localList)) localList = [] } catch { localList = [] }

    const localIds = new Set(localList.map((e: any) => e?.config?.instance_id).filter(Boolean))

    const merged = [...localList]

    for (const entry of event.payload) {
        const id = entry.config?.instance_id
        if (!id || localIds.has(id)) continue
        merged.push({
            config: NetworkTypes.normalizeNetworkConfig(entry.config),
            source: entry.source ?? 'legacy',
        })
    }

    localStorage.setItem('networkList', JSON.stringify(merged))
}

function normalizeInstanceIdPayload(payload: unknown): string {
    if (typeof payload === 'string') {
        return payload
    }

    if (payload && typeof payload === 'object') {
        const uuid = payload as Partial<Utils.UUID>
        if (
            typeof uuid.part1 === 'number'
            && typeof uuid.part2 === 'number'
            && typeof uuid.part3 === 'number'
            && typeof uuid.part4 === 'number'
        ) {
            return Utils.UuidToStr(uuid as Utils.UUID)
        }
    }

    if (payload == null) {
        return ''
    }

    const fallback = String(payload)
    return fallback === '[object Object]' ? '' : fallback
}

async function onPreRunNetworkInstance(event: TauriEvent<unknown>) {
    const instanceId = normalizeInstanceIdPayload(event.payload)
    console.log(`Received event '${EVENTS.PRE_RUN_NETWORK_INSTANCE}', raw payload:`, event.payload, 'normalized:', instanceId)
    if (type() === 'android') {
        await prepareVpnService(instanceId);
    }
}

async function onPostRunNetworkInstance(event: TauriEvent<unknown>) {
    const instanceId = normalizeInstanceIdPayload(event.payload)
    console.log(`Received event '${EVENTS.POST_RUN_NETWORK_INSTANCE}', raw payload:`, event.payload, 'normalized:', instanceId)
    if (type() === 'android') {
        await onNetworkInstanceChange(instanceId);
    }
}

async function onVpnServiceStop(event: TauriEvent<unknown>) {
    console.log(`Received event '${EVENTS.VPN_SERVICE_STOP}', raw payload:`, event.payload)
    await syncMobileVpnService();
}

async function onDhcpIpChanged(event: TauriEvent<unknown>) {
    const instanceId = normalizeInstanceIdPayload(event.payload)
    console.log(`Received event '${EVENTS.DHCP_IP_CHANGED}' for instance: ${instanceId}`);
    if (type() === 'android') {
        await onNetworkInstanceChange(instanceId);
    }
}

async function onProxyCidrsUpdated(event: TauriEvent<unknown>) {
    const instanceId = normalizeInstanceIdPayload(event.payload)
    console.log(`Received event '${EVENTS.PROXY_CIDRS_UPDATED}' for instance: ${instanceId}`);
    if (type() === 'android') {
        await onNetworkInstanceChange(instanceId);
    }
}

async function onEventLagged(event: TauriEvent<unknown>) {
    if (type() === 'android') {
        await onNetworkInstanceChange(normalizeInstanceIdPayload(event.payload));
    }
}

export async function listenGlobalEvents() {
    const unlisteners = [
        await listen(EVENTS.SAVE_CONFIGS, onSaveConfigs),
        await listen(EVENTS.PRE_RUN_NETWORK_INSTANCE, onPreRunNetworkInstance),
        await listen(EVENTS.POST_RUN_NETWORK_INSTANCE, onPostRunNetworkInstance),
        await listen(EVENTS.VPN_SERVICE_STOP, onVpnServiceStop),
        await listen(EVENTS.DHCP_IP_CHANGED, onDhcpIpChanged),
        await listen(EVENTS.PROXY_CIDRS_UPDATED, onProxyCidrsUpdated),
        await listen(EVENTS.EVENT_LAGGED, onEventLagged),
    ];

    return () => {
        unlisteners.forEach(unlisten => unlisten());
    };
}
