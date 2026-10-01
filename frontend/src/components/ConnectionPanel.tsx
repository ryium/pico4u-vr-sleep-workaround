import { useEffect, useState } from 'react'
import { Button } from '@charcoal-ui/react'
import { useAppContext } from '../context/AppContext'

export function ConnectionPanel() {
  const {
    t,
    connectionMode,
    handleModeSelect,
    deviceIp,
    connection,
    runtime,
    settings,
    setupWireless,
    connectManual,
  } = useAppContext()
  const [ip, setIp] = useState(deviceIp)
  const [setupComplete, setSetupComplete] = useState(false)
  useEffect(() => {
    setIp(deviceIp)
  }, [deviceIp])
  const locked =
    connection.busy ||
    runtime.busy ||
    runtime.running ||
    settings.isSaving ||
    !settings.config ||
    !runtime.ready
  return (
    <div className='panel-stack'>
      <fieldset className='mode-select' disabled={locked}>
        <legend>{t('connection_method')}</legend>
        {(['wired', 'wireless'] as const).map((mode) => (
          <label key={mode}>
            <input
              type='radio'
              name='mode'
              value={mode}
              checked={connectionMode === mode}
              onChange={() => {
                setSetupComplete(false)
                void handleModeSelect(mode)
              }}
            />
            <span>{t(`mode_${mode}`)}</span>
          </label>
        ))}
      </fieldset>
      {runtime.running && <p className='hint'>{t('mode_locked')}</p>}
      {connection.busy && (
        <p role='status' className='notice'>
          <span className='spinner' />
          {connection.autoStatus === 'connecting'
            ? t('connecting_attempt', { attempt: connection.attempt })
            : t('working')}
        </p>
      )}
      {connection.error && (
        <div className='notice error-text' role='alert'>
          <strong>{t('connection_failed')}</strong>
          <p>{t('error_hint')}</p>
          <details>
            <summary>{t('technical_details')}</summary>
            <code>{connection.error}</code>
          </details>
        </div>
      )}
      {!connectionMode && <p className='hint'>{t('choose_mode_hint')}</p>}
      {connectionMode === 'wired' && (
        <>
          <p className='hint'>{t('usb_hint')}</p>
          <Button fullWidth disabled={locked} onClick={() => void connection.connect('wired')}>
            {t('check_usb')}
          </Button>
        </>
      )}
      {connectionMode === 'wireless' && (
        <>
          <p className='hint'>{t('wireless_hint')}</p>
          <Button
            fullWidth
            disabled={locked}
            onClick={() => {
              void setupWireless().then(setSetupComplete)
            }}
          >
            {t('setup_wireless')}
          </Button>
          {setupComplete && connection.connected && (
            <p role='status' className='notice positive'>
              {t('unplug_usb')}
            </p>
          )}
          <form
            className='manual-form'
            onSubmit={(event) => {
              event.preventDefault()
              setSetupComplete(false)
              void connectManual(ip)
            }}
          >
            <label htmlFor='device-ip'>{t('ip_address')}</label>
            <div className='input-action'>
              <input
                id='device-ip'
                value={ip}
                onChange={(event) => setIp(event.target.value)}
                disabled={locked}
                required
                maxLength={21}
                placeholder='192.168.1.20'
                autoComplete='off'
                spellCheck={false}
              />
              <Button type='submit' disabled={locked || !ip.trim()}>
                {t('connect')}
              </Button>
            </div>
            <p className='hint'>{t('ip_note')}</p>
          </form>
        </>
      )}
      {settings.error && (
        <p role='alert' className='error-text'>
          {t('settings_failed')}
        </p>
      )}
      <details className='help'>
        <summary>{t('connection_help')}</summary>
        <p>{t('help_text')}</p>
      </details>
    </div>
  )
}
