<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { isClientRunning } from '~/composables/backend'
import pkg from '~/../package.json'

const router = useRouter()
const ready = ref(false)

onBeforeMount(async () => {
  await getCurrentWindow().setTitle(`Easytier GUI: v${pkg.version}`)
})

onMounted(async () => {
  // Wait for the backend to initialize (index.vue handles init in its onMounted)
  const maxWait = 20000 // 20 second timeout
  const start = Date.now()
  while (!ready.value) {
    await new Promise(r => setTimeout(r, 500))
    try { ready.value = await isClientRunning() } catch { /* not yet */ }
    if (Date.now() - start > maxWait) break
  }

  // Navigate regardless — even if not ready, show the networks page
  try {
    const raw = localStorage.getItem('networkList')
    const list = JSON.parse(raw || '[]')
    if (list.length === 1) {
      router.replace('/home')
    } else {
      router.replace('/home')
    }
  } catch {
    router.replace('/networks')
  }
})
</script>

<template>
  <!-- Splash overlay (covers RouterView during init) -->
  <div v-if="!ready" class="splash">
    <div class="splash-inner">
      <svg class="splash-icon" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="#1976d2" stroke-width="1.5">
        <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
      </svg>
      <div class="splash-title">EasyTier</div>
      <div class="splash-spinner" />
    </div>
  </div>

  <!-- App (always rendered so index.vue can initialize) -->
  <Toast position="bottom-right" />
  <RouterView />
</template>

<style>
.splash {
  position: fixed;
  inset: 0;
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f5f5f5;
}
@media (prefers-color-scheme: dark) {
  .splash { background: #121212; }
}
.splash-inner {
  text-align: center;
}
.splash-icon {
  margin-bottom: 16px;
}
.splash-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: #212121;
  margin-bottom: 24px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
}
@media (prefers-color-scheme: dark) {
  .splash-title { color: #e0e0e0; }
}
.splash-spinner {
  width: 28px;
  height: 28px;
  border: 3px solid #e0e0e0;
  border-top-color: #1976d2;
  border-radius: 50%;
  animation: splash-spin 0.8s linear infinite;
  margin: 0 auto;
}
@keyframes splash-spin {
  to { transform: rotate(360deg); }
}
</style>
