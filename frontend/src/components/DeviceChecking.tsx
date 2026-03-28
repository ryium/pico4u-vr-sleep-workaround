import { Button } from '@charcoal-ui/react'
import connectHmdImg from '../assets/connect_hmd_to_pc.png'
import packageJson from '../../package.json'
import { useAppContext } from '../context/AppContext'

interface DeviceCheckingProps {
  checkingMode: 'wired' | 'wireless'
  status: string
  detectedModel: string | null
  onSelectMode: (mode: 'wired' | 'wireless') => void
  handleCancel: () => void
}

export function DeviceChecking({
  checkingMode,
  status,
  detectedModel,
  onSelectMode,
  handleCancel,
}: DeviceCheckingProps) {
  const { t } = useAppContext()

  return (
    <div className='flex flex-col h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100'>
      {/* Header */}
      <header className='px-5 pt-4 pb-2 shrink-0'>
        <div className='flex items-baseline gap-2'>
          <h1 className='text-lg font-bold leading-normal'>{t('title')}</h1>
          <span className='text-xs text-gray-400 dark:text-gray-500 font-mono font-medium'>
            v{packageJson.version}
            {import.meta.env.DEV ? '-dev' : ''}
          </span>
        </div>
        <p className='mt-0.5 text-gray-500 dark:text-gray-400 text-xs'>{t('subtitle')}</p>
      </header>

      <div className='flex-1 flex flex-col items-center justify-center px-5 pb-6'>
        <div className='bg-white dark:bg-gray-800 rounded-xl p-5 shadow-sm border border-gray-100 dark:border-gray-700 text-center w-full max-w-md'>
          <div className='flex flex-col gap-4'>
            <p className='text-base font-bold text-gray-900 dark:text-gray-100'>{status}</p>
            <img
              src={connectHmdImg}
              alt='Connect HMD to PC'
              className='max-w-full h-auto mx-auto rounded-lg dark:opacity-90'
            />
            {!detectedModel && (
              <div className='animate-pulse text-gray-400 text-sm'>Waiting...</div>
            )}
            <div className='flex flex-col gap-3 mt-2 items-center'>
              {detectedModel && (
                <p className='text-sm text-gray-500 dark:text-gray-400'>
                  Detected: {detectedModel}
                </p>
              )}
              <Button onClick={() => onSelectMode(checkingMode)} variant='Overlay'>
                {t('force_proceed', { defaultValue: 'Proceed Anyway' })}
              </Button>
              <Button onClick={handleCancel} variant='Default'>
                {t('cancel', { defaultValue: 'Cancel' })}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
