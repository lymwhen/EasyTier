<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { resolveTheme, applyTheme } from '~/composables/theme'
import pkg from '~/../package.json'

applyTheme(resolveTheme(
  localStorage.getItem('themeMode') || 'auto',
  localStorage.getItem('amoledMode') === '1'
))

onBeforeMount(async () => {
  await getCurrentWindow().setTitle(`Easytier GUI: v${pkg.version}`)
})
</script>

<template>
  <Toast position="bottom-right" />
  <RouterView />
</template>
