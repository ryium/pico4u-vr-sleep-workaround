import { useState } from 'react'
import { Button } from '@charcoal-ui/react'
import { useAppContext } from '../context/AppContext'
import { HowToSection } from './HowToSection'

export function ConnectionPanel() {
  const {
    t,
    connectionMode,
    isConnected,
    deviceIp,
    checkDevices,
    setupWireless,
    wirelessSetupStatus,
    wiredSetupStatus,
    connectManual,
    autoConnectStatus,
  } = useAppContext()

  const isSettingUp = wirelessSetupStatus === 'loading'
  const isChecking = wiredSetupStatus === 'loading'

  // Whether the current connection came from auto-connect (not a first-time USB setup)
  const isAutoConnected = autoConnectStatus === 'success'

  // Allow user to expand setup buttons even while connected
  const [showSetup, setShowSetup] = useState(false)

  // Show setup UI when: not connected, or user explicitly requested it, or an operation is in progress
  const shouldShowSetup = !isConnected || showSetup || isSettingUp || isChecking

  const handleCheckDevices = async () => {
    const success = await checkDevices()
    if (success) setShowSetup(false)
  }

  const handleSetupWireless = async () => {
    const success = await setupWireless()
    if (success) setShowSetup(false)
  }

  const handleConnectManual = async () => {
    const success = await connectManual()
    if (success) setShowSetup(false)
  }

  // Determine which connected message to show
  const connectedMessage = (() => {
    if (connectionMode === 'wired') {
      return t('wired_connected')
    }
    // First-time wireless setup (via USB) — show "unplug USB" message
    if (!isAutoConnected && wirelessSetupStatus === 'success') {
      return t('wireless_status_success')
    }
    // Auto-connect or reconnect — just show "connected to IP"
    return t('wireless_connected', { ip: deviceIp })
  })()

  return (
    <div className='flex flex-col gap-4'>
      {/* Connection status */}
      <div className='bg-white dark:bg-gray-800 rounded-lg p-5 border border-gray-200 dark:border-gray-700'>
        {/* Header: mode + IP */}
        <div className='flex justify-between items-center mb-4'>
          <div className='flex items-center gap-2'>
            <span
              className={`w-2 h-2 rounded-full shrink-0 ${isConnected ? 'bg-green-500' : 'bg-gray-300 dark:bg-gray-600'}`}
            />
            <span className='font-semibold text-sm text-gray-800 dark:text-gray-200'>
              {connectionMode === 'wired' ? t('mode_wired') : t('mode_wireless')}
            </span>
          </div>
          {deviceIp && (
            <span className='font-mono text-[11px] text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-900 px-2 py-0.5 rounded border border-gray-200 dark:border-gray-700'>
              {deviceIp}
            </span>
          )}
        </div>

        {/* Connected status banner */}
        {isConnected && !shouldShowSetup && (
          <div className='flex flex-col gap-3'>
            <div className='flex items-center gap-2.5 rounded-lg px-4 py-3 text-sm font-medium bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300 border border-green-200 dark:border-green-800 animate-fadeIn'>
              <svg className='w-4 h-4 shrink-0' viewBox='0 0 20 20' fill='currentColor'>
                <path
                  fillRule='evenodd'
                  d='M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z'
                  clipRule='evenodd'
                />
              </svg>
              <span>{connectedMessage}</span>
            </div>

            {/* Subtle re-setup link */}
            <button
              onClick={() => setShowSetup(true)}
              className='text-xs text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 transition-colors py-1'
            >
              {t('btn_reconnect')}
            </button>
          </div>
        )}

        {/* Setup buttons — shown when not connected or user clicked reconnect */}
        {shouldShowSetup && (
          <div className='flex flex-col gap-3'>
            {connectionMode === 'wired' ? (
              <>
                <Button onClick={handleCheckDevices} fullWidth disabled={isChecking}>
                  {isChecking ? t('wired_status_loading') : t('btn_check_devices')}
                </Button>

                {/* Wired setup status banner */}
                {wiredSetupStatus !== 'idle' && (
                  <StatusBanner status={wiredSetupStatus} t={t} type='wired' />
                )}
              </>
            ) : (
              <>
                <Button
                  onClick={handleSetupWireless}
                  variant='Navigation'
                  fullWidth
                  disabled={isSettingUp}
                >
                  {isSettingUp ? t('wireless_status_loading') : t('btn_setup_wireless')}
                </Button>

                {/* Wireless setup status banner */}
                {wirelessSetupStatus !== 'idle' && (
                  <StatusBanner status={wirelessSetupStatus} t={t} type='wireless' />
                )}

                <Button onClick={handleConnectManual} fullWidth>
                  {t('btn_connect_manual')}
                </Button>
              </>
            )}
          </div>
        )}
      </div>

      {/* How to */}
      <HowToSection />
    </div>
  )
}

// Extracted status banner to reduce duplication
function StatusBanner({
  status,
  t,
  type,
}: {
  status: 'idle' | 'loading' | 'success' | 'error'
  t: (key: string) => string
  type: 'wired' | 'wireless'
}) {
  if (status === 'idle') return null

  const colorClass =
    status === 'loading'
      ? 'bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 border border-blue-200 dark:border-blue-800'
      : status === 'success'
        ? 'bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300 border border-green-200 dark:border-green-800'
        : 'bg-red-50 dark:bg-red-950 text-red-700 dark:text-red-300 border border-red-200 dark:border-red-800'

  return (
    <div
      className={`flex items-center gap-2.5 rounded-lg px-4 py-3 text-sm font-medium animate-fadeIn ${colorClass}`}
    >
      {status === 'loading' && (
        <span className='w-4 h-4 shrink-0 border-2 border-blue-300 dark:border-blue-600 border-t-blue-600 dark:border-t-blue-300 rounded-full animate-spin' />
      )}
      {status === 'success' && (
        <svg className='w-4 h-4 shrink-0' viewBox='0 0 20 20' fill='currentColor'>
          <path
            fillRule='evenodd'
            d='M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z'
            clipRule='evenodd'
          />
        </svg>
      )}
      {status === 'error' && (
        <svg className='w-4 h-4 shrink-0' viewBox='0 0 20 20' fill='currentColor'>
          <path
            fillRule='evenodd'
            d='M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z'
            clipRule='evenodd'
          />
        </svg>
      )}
      <span>
        {status === 'loading' && t(`${type}_status_loading`)}
        {status === 'success' && t(`${type}_status_success`)}
        {status === 'error' && t(`${type}_status_error`)}
      </span>
    </div>
  )
}
