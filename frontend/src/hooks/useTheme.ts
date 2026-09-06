import { useState, useEffect, useCallback } from 'react'

export type Theme = 'light' | 'dark' | 'system'

export function useTheme() {
  const [theme, setTheme] = useState<Theme>(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('theme')
      return saved === 'light' || saved === 'dark' ? saved : 'system'
    }
    return 'system'
  })

  const applyTheme = useCallback((theme: Theme) => {
    const root = window.document.documentElement
    const isDark =
      theme === 'dark' ||
      (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)

    root.dataset.theme = isDark ? 'dark' : 'light'
    root.style.colorScheme = isDark ? 'dark' : 'light'
    if (isDark) {
      root.classList.add('dark')
    } else {
      root.classList.remove('dark')
    }
  }, [])

  useEffect(() => {
    applyTheme(theme)
    localStorage.setItem('theme', theme)

    if (theme === 'system') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const handleChange = () => applyTheme('system')
      mediaQuery.addEventListener('change', handleChange)
      return () => mediaQuery.removeEventListener('change', handleChange)
    }
  }, [theme, applyTheme])

  return { theme, setTheme }
}
