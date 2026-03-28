import { Button } from '@charcoal-ui/react'
import { useAppContext } from '../context/AppContext'
import { HowToSection } from './HowToSection'

export function ConnectionPanel() {
  const {
    t,
    connectionMode,
    deviceIp,
    checkDevices,
    setupWireless,
    wirelessSetupStatus,
    wiredSetupStatus,
    connectManual,
  } = useAppContext()

  const isSettingUp = wirelessSetupStatus === 'loading'
  const isChecking = wiredSetupStatus === 'loading'

  return (
    <div className='flex flex-col gap-4'>
      {/* Connection status */}
      <div className='bg-white dark:bg-gray-800 rounded-lg p-5 border border-gray-200 dark:border-gray-700'>
        <div className='flex justify-between items-center mb-4'>
          <div className='flex items-center gap-2'>
            <span className='w-2 h-2 rounded-full bg-brand shrink-0' />
            <span className='font-semibold text-sm text-gray-800 dark:text-gray-200'>
              {connectionMode === 'wired'
                ? t('mode_wired', { defaultValue: 'Wired' })
                : t('mode_wireless', { defaultValue: 'Wireless' })}
            </span>
          </div>
          {deviceIp && (
            <span className='font-mono text-[11px] text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-900 px-2 py-0.5 rounded border border-gray-200 dark:border-gray-700'>
              {deviceIp}
            </span>
          )}
        </div>
        <div className='flex flex-col gap-3'>
          {connectionMode === 'wired' ? (
            <>
              <Button onClick={checkDevices} fullWidth disabled={isChecking}>
                {isChecking ? t('wired_status_loading') : t('btn_check_devices')}
              </Button>

              {/* Wired setup status banner */}
              {wiredSetupStatus !== 'idle' && (
                <div
                  className={`flex items-center gap-2.5 rounded-lg px-4 py-3 text-sm font-medium animate-fadeIn ${
                    wiredSetupStatus === 'loading'
                      ? 'bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 border border-blue-200 dark:border-blue-800'
                      : wiredSetupStatus === 'success'
                        ? 'bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300 border border-green-200 dark:border-green-800'
                        : 'bg-red-50 dark:bg-red-950 text-red-700 dark:text-red-300 border border-red-200 dark:border-red-800'
                  }`}
                >
                  {/* Icon */}
                  {wiredSetupStatus === 'loading' && (
                    <span className='w-4 h-4 shrink-0 border-2 border-blue-300 dark:border-blue-600 border-t-blue-600 dark:border-t-blue-300 rounded-full animate-spin' />
                  )}
                  {wiredSetupStatus === 'success' && (
                    <svg className='w-4 h-4 shrink-0' viewBox='0 0 20 20' fill='currentColor'>
                      <path
                        fillRule='evenodd'
                        d='M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z'
                        clipRule='evenodd'
                      />
                    </svg>
                  )}
                  {wiredSetupStatus === 'error' && (
                    <svg className='w-4 h-4 shrink-0' viewBox='0 0 20 20' fill='currentColor'>
                      <path
                        fillRule='evenodd'
                        d='M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z'
                        clipRule='evenodd'
                      />
                    </svg>
                  )}
                  {/* Text */}
                  <span>
                    {wiredSetupStatus === 'loading' && t('wired_status_loading')}
                    {wiredSetupStatus === 'success' && t('wired_status_success')}
                    {wiredSetupStatus === 'error' && t('wired_status_error')}
                  </span>
                </div>
              )}
            </>
          ) : (
            <>
              <Button onClick={setupWireless} variant='Navigation' fullWidth disabled={isSettingUp}>
                {isSettingUp ? t('wireless_status_loading') : t('btn_setup_wireless')}
              </Button>

              {/* Wireless setup status banner */}
              {wirelessSetupStatus !== 'idle' && (
                <div
                  className={`flex items-center gap-2.5 rounded-lg px-4 py-3 text-sm font-medium animate-fadeIn ${
                    wirelessSetupStatus === 'loading'
                      ? 'bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300 border border-blue-200 dark:border-blue-800'
                      : wirelessSetupStatus === 'success'
                        ? 'bg-green-50 dark:bg-green-950 text-green-700 dark:text-green-300 border border-green-200 dark:border-green-800'
                        : 'bg-red-50 dark:bg-red-950 text-red-700 dark:text-red-300 border border-red-200 dark:border-red-800'
                  }`}
                >
                  {/* Icon */}
                  {wirelessSetupStatus === 'loading' && (
                    <span className='w-4 h-4 shrink-0 border-2 border-blue-300 dark:border-blue-600 border-t-blue-600 dark:border-t-blue-300 rounded-full animate-spin' />
                  )}
                  {wirelessSetupStatus === 'success' && (
                    <svg className='w-4 h-4 shrink-0' viewBox='0 0 20 20' fill='currentColor'>
                      <path
                        fillRule='evenodd'
                        d='M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z'
                        clipRule='evenodd'
                      />
                    </svg>
                  )}
                  {wirelessSetupStatus === 'error' && (
                    <svg className='w-4 h-4 shrink-0' viewBox='0 0 20 20' fill='currentColor'>
                      <path
                        fillRule='evenodd'
                        d='M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z'
                        clipRule='evenodd'
                      />
                    </svg>
                  )}
                  {/* Text */}
                  <span>
                    {wirelessSetupStatus === 'loading' && t('wireless_status_loading')}
                    {wirelessSetupStatus === 'success' && t('wireless_status_success')}
                    {wirelessSetupStatus === 'error' && t('wireless_status_error')}
                  </span>
                </div>
              )}

              <Button onClick={connectManual} fullWidth>
                {t('btn_connect_manual')}
              </Button>
            </>
          )}
        </div>
      </div>

      {/* How to */}
      <HowToSection />
    </div>
  )
}
