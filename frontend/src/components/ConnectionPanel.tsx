import { Button } from '@charcoal-ui/react'
import { useAppContext } from '../context/AppContext'
import { HowToSection } from './HowToSection'

export function ConnectionPanel() {
  const { t, connectionMode, deviceIp, checkDevices, setupWireless, connectManual } =
    useAppContext()

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
            <Button onClick={checkDevices} fullWidth>
              {t('btn_check_devices')}
            </Button>
          ) : (
            <>
              <Button onClick={setupWireless} variant='Navigation' fullWidth>
                {t('btn_setup_wireless')}
              </Button>
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
