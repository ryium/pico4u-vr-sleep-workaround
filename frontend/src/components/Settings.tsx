import { useState } from 'react'
import { Button } from '@charcoal-ui/react'
import { useAppContext } from '../context/AppContext'
import type { AppConfig } from '../types'
import type { Theme } from '../hooks/useTheme'

export function Settings() {
  const { t, i18n, changeLanguage, theme, setTheme, isDebug, toggleDebugMode, settings } =
    useAppContext()
  return (
    <div className='panel-stack settings-panel'>
      <label className='field'>
        {t('language_label')}
        <select
          aria-label={t('language_label')}
          value={i18n.resolvedLanguage ?? 'en'}
          onChange={(event) => changeLanguage(event.target.value)}
        >
          <option value='ja'>日本語</option>
          <option value='en'>English</option>
        </select>
      </label>
      <label className='field'>
        {t('theme_label')}
        <select
          aria-label={t('theme_label')}
          value={theme}
          onChange={(event) => setTheme(event.target.value as Theme)}
        >
          {(['system', 'light', 'dark'] as const).map((value) => (
            <option key={value} value={value}>
              {t(`theme_${value}`)}
            </option>
          ))}
        </select>
      </label>
      {settings.config ? (
        <RuntimeSettings config={settings.config} />
      ) : (
        <p className='hint' role='status'>
          {t(settings.error ? 'settings_failed' : 'loading_settings')}
        </p>
      )}
      <label className='checkbox-field'>
        <input
          type='checkbox'
          checked={isDebug}
          onChange={(event) => void toggleDebugMode(event.target.checked)}
        />
        {t('debug_mode')}
      </label>
    </div>
  )
}

function RuntimeSettings({ config }: { config: AppConfig }) {
  const { t, settings } = useAppContext()
  const [hours, setHours] = useState(String(config.dim_delay_hours))
  const [interval, setInterval] = useState(String(config.keep_awake_interval_secs))
  const [saved, setSaved] = useState(false)
  const [failed, setFailed] = useState(false)
  return (
    <form
      className='panel-stack'
      onSubmit={(event) => {
        event.preventDefault()
        setSaved(false)
        setFailed(false)
        void settings
          .update({ dim_delay_hours: Number(hours), keep_awake_interval_secs: Number(interval) })
          .then(() => setSaved(true))
          .catch(() => setFailed(true))
      }}
    >
      <label className='field'>
        {t('interval_label')}
        <input
          type='number'
          min='1'
          max='3600'
          step='1'
          required
          value={interval}
          disabled={settings.isSaving}
          onChange={(event) => {
            setInterval(event.target.value)
            setSaved(false)
          }}
        />
      </label>
      <label className='field'>
        {t('dim_label')}
        <input
          type='number'
          min='0'
          max='168'
          step='0.25'
          required
          value={hours}
          disabled={settings.isSaving}
          onChange={(event) => {
            setHours(event.target.value)
            setSaved(false)
          }}
        />
      </label>
      <p className='hint'>{t('next_start_note')}</p>
      <Button type='submit' fullWidth disabled={settings.isSaving}>
        {t(settings.isSaving ? 'saving' : 'save_settings')}
      </Button>
      {saved && (
        <p className='positive' role='status'>
          {t('saved')}
        </p>
      )}
      {failed && (
        <p className='error-text' role='alert'>
          {t('settings_failed')}
        </p>
      )}
    </form>
  )
}
