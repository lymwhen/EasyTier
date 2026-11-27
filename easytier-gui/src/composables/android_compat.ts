import { bootstrapAndroidManagement } from './android_management'

const KEPT_KEYS = ['networkList', 'app_mode', 'last_network_instance_id'] as const

// Fork 适配层：上游 v2.7.1 的 Android 管理重构（#2593）要求先完成
// bootstrapAndroidManagement() 迁移，否则 lib.rs 的 init_rpc_connection、
// config.ts 的 load/saveLastNetworkInstanceId 等在 Android 上会直接抛错。
// 但上游实现会在迁移成功后删除 localStorage 明文键，而 fork 的 home.vue
// 路由链路（index.vue 不被挂载）以 localStorage 为配置真源。
// 这里复用上游 bootstrap 完成 native 安全存储迁移与状态注册，
// 随后原样恢复这些键，使两套存储保持一致且不破坏 fork 的显示逻辑。
export async function bootstrapAndroidStorageKeepLocal() {
  const kept = new Map<string, string | null>()
  for (const key of KEPT_KEYS)
    kept.set(key, localStorage.getItem(key))
  await bootstrapAndroidManagement()
  for (const [key, value] of kept) {
    if (value !== null)
      localStorage.setItem(key, value)
  }
}