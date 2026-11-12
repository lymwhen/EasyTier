export type Theme = 'light' | 'dark'

export function resolveTheme(themeMode: string): Theme {
  const sysDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  const resolvedDark = themeMode === 'dark' || (themeMode === 'auto' && sysDark)
  return resolvedDark ? 'dark' : 'light'
}

export function applyTheme(theme: Theme) {
  document.documentElement.setAttribute('data-theme', theme)
  document.documentElement.classList.toggle('app-dark', theme !== 'light')

  const w = window as any
  if (w._easytier_theme) {
    w._easytier_theme.setStatusBarStyle(theme !== 'light')
  }
}
