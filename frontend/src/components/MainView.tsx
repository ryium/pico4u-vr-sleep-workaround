import { useState } from 'react'
import { Button } from '@charcoal-ui/react'
import { useAppContext } from '../context/AppContext'
import { AppHeader } from './ui/AppHeader'
import { Sheet } from './ui/Sheet'
import { ModeSelectScreen } from './screens/ModeSelectScreen'
import { PrepareScreen } from './screens/PrepareScreen'
import { OperateScreen } from './screens/OperateScreen'
import { ErrorCard } from './ErrorCard'
import { Settings } from './Settings'
import { Logs } from './Logs'
import type { ConnectionMode } from '../types'

type Screen = 'mode' | 'prepare' | 'operate'
type SheetType = 'settings' | 'logs' | null

export function MainView() {
  const {
    t,
    connectionMode,
    autoConnectStatus,
    setAutoConnectStatus,
    retryAutoConnect,
    autoConnectType,
    autoConnectAttempt,
    deviceIp,
    connectionError,
  } = useAppContext()

  const [screen, setScreen] = useState<Screen>(connectionMode ? 'operate' : 'mode')
  const [targetMode, setTargetMode] = useState<ConnectionMode>(connectionMode ?? 'wireless')
  const [activeSheet, setActiveSheet] = useState<SheetType>(null)

  // ─── Auto-connecting / Failed State ───
  if (
    autoConnectStatus === 'idle' ||
    autoConnectStatus === 'connecting' ||
    autoConnectStatus === 'failed'
  ) {
    return (
      <div className='flex flex-col h-screen bg-[var(--app-bg)] text-[var(--app-text)]'>
        <AppHeader onOpenSettings={() => setActiveSheet('settings')} />

        <div className='flex-1 flex flex-col items-center justify-center px-5'>
          <div className='bg-[var(--app-card)] rounded-2xl p-6 shadow-sm border border-[var(--app-border)] w-full max-w-sm text-center'>
            {(autoConnectStatus === 'idle' || autoConnectStatus === 'connecting') && (
              <div className='flex flex-col items-center gap-4 py-4'>
                <span className='w-8 h-8 border-3 border-gray-200 dark:border-gray-700 border-t-brand rounded-full animate-spin' />
                <div>
                  <p className='text-sm font-semibold text-[var(--app-text)]'>
                    {autoConnectType === 'wired'
                      ? t('auto_connect_wired', { attempt: autoConnectAttempt })
                      : t('auto_connect_wireless', {
                          ip: deviceIp,
                          attempt: autoConnectAttempt,
                        })}
                  </p>
                  {deviceIp && autoConnectType === 'wireless' && (
                    <p className='mt-1 text-xs font-mono text-[var(--app-text-muted)]'>
                      {deviceIp}
                    </p>
                  )}
                </div>
              </div>
            )}

            {autoConnectStatus === 'failed' && (
              <div className='flex flex-col items-center gap-4 animate-fadeIn'>
                <div className='w-full'>
                  <ErrorCard
                    error={connectionError}
                    fallbackMessage={t('auto_connect_failed_desc', { ip: deviceIp })}
                  />
                </div>

                <div className='flex flex-col gap-2 w-full mt-1'>
                  <Button onClick={retryAutoConnect} variant='Default' fullWidth>
                    {t('btn_try_again')}
                  </Button>
                  <Button
                    onClick={() => {
                      setAutoConnectStatus('skipped')
                      setTargetMode('wireless')
                      setScreen('prepare')
                    }}
                    variant='Navigation'
                    fullWidth
                  >
                    {t('btn_wireless_setup')}
                  </Button>
                  <button
                    type='button'
                    onClick={() => setAutoConnectStatus('skipped')}
                    className='text-xs text-[var(--app-text-muted)] hover:text-[var(--app-text)] transition-colors py-1 cursor-pointer'
                  >
                    {t('btn_manual_setup')}
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>

        <Sheet
          title={t('settings_title')}
          isOpen={activeSheet === 'settings'}
          onClose={() => setActiveSheet(null)}
        >
          <Settings onClose={() => setActiveSheet(null)} />
        </Sheet>
      </div>
    )
  }

  // ─── Main Wizard Flow ───
  return (
    <div className='flex flex-col h-screen bg-[var(--app-bg)] text-[var(--app-text)]'>
      <AppHeader onOpenSettings={() => setActiveSheet('settings')} />

      {screen === 'mode' && (
        <ModeSelectScreen
          onNext={(mode) => {
            setTargetMode(mode)
            setScreen('prepare')
          }}
        />
      )}

      {screen === 'prepare' && (
        <PrepareScreen
          mode={targetMode}
          onBack={() => setScreen('mode')}
          onSuccess={() => setScreen('operate')}
        />
      )}

      {screen === 'operate' && (
        <OperateScreen
          onChangeConnection={() => setScreen('mode')}
          onOpenLogs={() => setActiveSheet('logs')}
        />
      )}

      {/* Settings Sheet */}
      <Sheet
        title={t('settings_title')}
        isOpen={activeSheet === 'settings'}
        onClose={() => setActiveSheet(null)}
      >
        <Settings onClose={() => setActiveSheet(null)} />
      </Sheet>

      {/* Logs Sheet */}
      <Sheet
        title={t('tab_logs')}
        isOpen={activeSheet === 'logs'}
        onClose={() => setActiveSheet(null)}
      >
        <Logs />
      </Sheet>
    </div>
  )
}
