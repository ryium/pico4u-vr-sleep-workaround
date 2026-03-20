import { Button, Icon } from '@charcoal-ui/react'
import packageJson from '../../package.json'
import { useAppContext } from '../context/AppContext'
import { Settings } from './Settings'

interface ModeSelectionProps {
  homeView: 'main' | 'settings'
  setHomeView: (view: 'main' | 'settings') => void
  setCheckingMode: (mode: 'wired' | 'wireless') => void
}

export function ModeSelection({ homeView, setHomeView, setCheckingMode }: ModeSelectionProps) {
  const { t } = useAppContext()

  return (
    <div className='flex flex-col h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100'>
      {/* Header */}
      <header className='px-5 pt-4 pb-2 shrink-0'>
        <div className='flex items-baseline gap-2'>
          <h1 className='text-lg font-bold leading-normal'>{t('title')}</h1>
          <span className='text-xs text-gray-400 dark:text-gray-500 font-mono font-medium'>
            v{packageJson.version}
          </span>
        </div>
        <p className='mt-0.5 text-gray-500 dark:text-gray-400 text-xs'>{t('subtitle')}</p>
      </header>

      {homeView === 'main' ? (
        <>
          <div className='flex-1 flex flex-col items-center justify-center px-5'>
            <div className='bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700 w-full max-w-sm text-center'>
              <h2 className='mt-0 mb-5 text-base font-bold text-gray-900 dark:text-gray-100'>
                {t('mode_select_title')}
              </h2>
              <div className='flex gap-3 flex-wrap items-center justify-center'>
                <Button onClick={() => setCheckingMode('wired')} variant='Default'>
                  {t('mode_wired')}
                </Button>
                <Button onClick={() => setCheckingMode('wireless')} variant='Default'>
                  {t('mode_wireless')}
                </Button>
              </div>
            </div>
          </div>

          {/* Settings link */}
          <div className='px-5 pb-4 shrink-0'>
            <button
              onClick={() => setHomeView('settings')}
              className='w-full text-center text-sm text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 hover:text-gray-900 dark:hover:text-gray-100 rounded-lg transition-all py-2.5'
            >
              <span className='inline-flex items-center justify-center gap-1.5'>
                <Icon name='24/Settings' />
                {t('settings_title', { defaultValue: '設定' })}
              </span>
            </button>
          </div>
        </>
      ) : (
        <>
          {/* Inline settings on home */}
          <div className='flex items-center justify-between px-5 py-2 shrink-0'>
            <h2 className='text-base font-bold text-gray-900 dark:text-gray-100'>
              {t('settings_title', { defaultValue: '設定' })}
            </h2>
          </div>
          <div className='flex-1 overflow-y-auto px-5 pb-4'>
            <Settings onClose={() => setHomeView('main')} />
          </div>
        </>
      )}
    </div>
  )
}
