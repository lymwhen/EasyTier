export type Theme = 'light' | 'dark' | 'amoled'

export function resolveTheme(themeMode: string, amoledMode: boolean): Theme {
  const sysDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  const resolvedDark = themeMode === 'dark' || (themeMode === 'auto' && sysDark)
  return (amoledMode && resolvedDark) ? 'amoled' : resolvedDark ? 'dark' : 'light'
}

export function applyTheme(theme: Theme) {
  document.documentElement.setAttribute('data-theme', theme)
  document.documentElement.classList.toggle('app-dark', theme !== 'light')

  const w = window as any
  if (w._easytier_theme) {
    if (theme === 'amoled' && w._easytier_theme.setAmoledMode) {
      w._easytier_theme.setAmoledMode(true)
    } else {
      w._easytier_theme.setStatusBarStyle(theme !== 'light')
    }
  }
}
