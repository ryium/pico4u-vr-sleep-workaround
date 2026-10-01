import { useState, useEffect, useRef } from 'react'
import { Button, Icon } from '@charcoal-ui/react'
import { useAppContext } from '../context/AppContext'
import { Logs } from './Logs'
import packageJson from '../../package.json'
import { DeviceChecking } from './DeviceChecking'
import { ModeSelection } from './ModeSelection'
import { ConnectionPanel } from './ConnectionPanel'

type Tab = 'connection' | 'logs' | 'settings'
type HomeView = 'main' | 'settings'

export function MainView() {
  const {
    t,
    connectionMode,
    setConnectionMode,
    isRunning,
    toggleKeepAwake,
    isDebug,
    handleModeSelect,
    getDeviceModel,
    autoConnectStatus,
    setAutoConnectStatus,
    retryAutoConnect,
    autoConnectType,
    autoConnectAttempt,
    deviceIp,
  } = useAppContext()

  const [activeTab, setActiveTab] = useState<Tab>('connection')
  const [checkingMode, setCheckingMode] = useState<'wired' | 'wireless' | null>(null)
  const [status, setStatus] = useState<string>('')
  const [detectedModel, setDetectedModel] = useState<string | null>(null)
  const [homeView, setHomeView] = useState<HomeView>('main')

  // When auto-connect fails and user chooses wireless setup, go to device checking
  const handleFailedWirelessSetup = () => {
    setAutoConnectStatus('skipped')
    setCheckingMode('wireless')
  }

  const handleFailedManualSetup = () => {
    setAutoConnectStatus('skipped')
  }

  // Device checking polling
  const isCheckingRef = useRef(false)
  const autoProceededRef = useRef(false)

  useEffect(() => {
    let intervalId: number | undefined
    autoProceededRef.current = false
    let active = true

    const check = async () => {
      if (!checkingMode) return
      // Prevent overlapping checks
      if (isCheckingRef.current) return
      isCheckingRef.current = true

      try {
        setStatus(t('status_checking'))
        const model = await getDeviceModel()
        if (!active) return

        if (model) {
          setDetectedModel(model)
          if (model.includes('A9210')) {
            // Prevent multiple auto-proceed triggers
            if (autoProceededRef.current) return
            autoProceededRef.current = true

            setStatus(t('status_device_found'))
            // Clear the interval so no more checks run
            if (intervalId) {
              clearInterval(intervalId)
              intervalId = undefined
            }
            // Small delay so user sees the detection result and ADB stabilizes
            await new Promise((r) => setTimeout(r, 1500))
            if (!active) return
            onSelectMode(checkingMode)
          } else {
            setStatus(
              t('status_wrong_device', {
                model,
              }),
            )
          }
        } else {
          setDetectedModel(null)
          setStatus(t('status_waiting_usb'))
        }
      } finally {
        isCheckingRef.current = false
      }
    }

    if (checkingMode) {
      check()
      intervalId = window.setInterval(check, 3000)
    }

    return () => {
      active = false
      if (intervalId) clearInterval(intervalId)
    }
  }, [checkingMode, getDeviceModel, t])

  const onSelectMode = async (mode: 'wired' | 'wireless') => {
    setCheckingMode(null)
    setDetectedModel(null)
    setStatus('')
    await handleModeSelect(mode)
  }

  const handleCancel = () => {
    setCheckingMode(null)
    setDetectedModel(null)
    setStatus('')
  }

  const handleBackToModeSelect = () => {
    setConnectionMode(null)
    setActiveTab('connection')
  }

  // ─── Device checking state ───
  if (checkingMode) {
    return (
      <DeviceChecking
        checkingMode={checkingMode}
        status={status}
        detectedModel={detectedModel}
        onSelectMode={onSelectMode}
        handleCancel={handleCancel}
      />
    )
  }

  // ─── Auto-connecting / Auto-connect result state ───
  if (
    autoConnectStatus === 'idle' ||
    autoConnectStatus === 'connecting' ||
    autoConnectStatus === 'failed'
  ) {
    return (
      <div className='flex flex-col h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100'>
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
        <div className='flex-1 flex flex-col items-center justify-center px-5'>
          <div className='bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm border border-gray-100 dark:border-gray-700 w-full max-w-sm text-center'>
            {/* Connecting state — spinner */}
            {(autoConnectStatus === 'idle' || autoConnectStatus === 'connecting') && (
              <div className='flex flex-col items-center gap-4'>
                <span className='w-8 h-8 border-3 border-gray-200 dark:border-gray-600 border-t-brand rounded-full animate-spin' />
                <div>
                  <p className='text-sm font-semibold text-gray-700 dark:text-gray-300'>
                    {autoConnectType === 'wired'
                      ? t('auto_connect_wired', { attempt: autoConnectAttempt })
                      : autoConnectType === 'wireless'
                        ? t('auto_connect_wireless', {
                            ip: deviceIp,
                            attempt: autoConnectAttempt,
                          })
                        : t('auto_connect_wired', { attempt: autoConnectAttempt })}
                  </p>
                  {deviceIp && autoConnectType === 'wireless' && (
                    <p className='mt-1 text-xs font-mono text-gray-400 dark:text-gray-500'>
                      {deviceIp}
                    </p>
                  )}
                </div>
              </div>
            )}

            {/* Failed state — error with action buttons */}
            {autoConnectStatus === 'failed' && (
              <div className='flex flex-col items-center gap-4 animate-fadeIn'>
                {/* Error icon */}
                <div className='w-12 h-12 rounded-full bg-red-100 dark:bg-red-950 flex items-center justify-center'>
                  <svg
                    className='w-6 h-6 text-red-500 dark:text-red-400'
                    viewBox='0 0 20 20'
                    fill='currentColor'
                  >
                    <path
                      fillRule='evenodd'
                      d='M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z'
                      clipRule='evenodd'
                    />
                  </svg>
                </div>

                {/* Error message */}
                <div>
                  <p className='text-sm font-semibold text-red-600 dark:text-red-400'>
                    {t('auto_connect_failed_title')}
                  </p>
                  <p className='mt-1 text-xs text-gray-500 dark:text-gray-400'>
                    {t('auto_connect_failed_desc', { ip: deviceIp })}
                  </p>
                </div>

                {/* Action buttons */}
                <div className='flex flex-col gap-2 w-full mt-1'>
                  <Button onClick={retryAutoConnect} variant='Default' fullWidth>
                    {t('btn_try_again')}
                  </Button>
                  <Button onClick={handleFailedWirelessSetup} variant='Navigation' fullWidth>
                    {t('btn_wireless_setup')}
                  </Button>
                  <button
                    onClick={handleFailedManualSetup}
                    className='text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300 transition-colors py-1'
                  >
                    {t('btn_manual_setup')}
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    )
  }

  // ─── Mode selection (no mode chosen yet) ───
  if (!connectionMode) {
    return (
      <ModeSelection
        homeView={homeView}
        setHomeView={setHomeView}
        setCheckingMode={setCheckingMode}
      />
    )
  }

  // ─── Main operation view (mode selected) ───
  const tabs: { key: Tab; label: string }[] = isDebug
    ? [
        {
          key: 'connection',
          label: t('tab_connection'),
        },
        { key: 'logs', label: t('tab_logs') },
      ]
    : []

  const currentTab = !isDebug && activeTab === 'logs' ? 'connection' : activeTab

  return (
    <div className='flex flex-col h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100'>
      <header className='px-5 pt-4 pb-2 shrink-0 relative'>
        <div className='flex items-baseline gap-2'>
          <h1 className='text-lg font-bold leading-normal'>{t('title')}</h1>
          <span className='text-xs text-gray-400 dark:text-gray-500 font-mono font-medium'>
            v{packageJson.version}
            {import.meta.env.DEV ? '-dev' : ''}
          </span>
        </div>
        <p className='mt-0.5 text-gray-500 dark:text-gray-400 text-xs'>{t('subtitle')}</p>
        <button
          onClick={handleBackToModeSelect}
          className='absolute right-5 top-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors p-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800'
          title={t('back_to_mode_select')}
        >
          <Icon name='24/Close' />
        </button>
      </header>

      {/* Main action area */}
      <div className='flex flex-col items-center justify-center pt-3 pb-6 px-5 shrink-0'>
        {/* Status indicator */}
        <div className='mb-3 flex flex-col items-center gap-2'>
          <div className='relative flex h-6 w-6 items-center justify-center'>
            {isRunning && (
              <span className='animate-ping absolute inline-flex h-full w-full rounded-full bg-brand opacity-60' />
            )}
            <span
              className={`relative inline-flex rounded-full h-3.5 w-3.5 ${
                isRunning ? 'bg-brand' : 'bg-gray-300 dark:bg-gray-600'
              }`}
            />
          </div>
          <h2
            className={`text-xl font-black tracking-wider ${
              isRunning ? 'text-brand' : 'text-gray-400 dark:text-gray-500'
            }`}
          >
            {isRunning ? t('status_running') : t('status_stopped')}
          </h2>
        </div>

        {/* Start / Stop button */}
        <div className='w-full max-w-[240px]'>
          <Button onClick={toggleKeepAwake} variant={isRunning ? 'Danger' : 'Primary'} fullWidth>
            {isRunning ? t('btn_stop_keep_alive') : t('btn_start_keep_alive')}
          </Button>
        </div>
      </div>

      {/* Tab bar */}
      {tabs.length > 0 && (
        <div className='flex gap-1 bg-gray-100 dark:bg-gray-800 mx-5 rounded-lg p-1 shrink-0 mb-3'>
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key)}
              className={`flex-1 py-1.5 text-xs font-semibold text-center rounded-md transition-all ${
                currentTab === tab.key
                  ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm'
                  : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      )}

      {/* Tab content */}
      <div className='flex-1 overflow-y-auto px-5 pt-4 pb-5'>
        {isDebug ? (
          <>
            {currentTab === 'connection' && <ConnectionPanel />}
            {currentTab === 'logs' && <Logs />}
          </>
        ) : (
          <ConnectionPanel />
        )}
      </div>
    </div>
  )
}
