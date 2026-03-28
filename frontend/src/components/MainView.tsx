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
  } = useAppContext()

  const [activeTab, setActiveTab] = useState<Tab>('connection')
  const [checkingMode, setCheckingMode] = useState<'wired' | 'wireless' | null>(null)
  const [status, setStatus] = useState<string>('')
  const [detectedModel, setDetectedModel] = useState<string | null>(null)
  const [homeView, setHomeView] = useState<HomeView>('main')
  const isMounted = useRef(true)

  useEffect(() => {
    isMounted.current = true
    return () => {
      isMounted.current = false
    }
  }, [])

  // Ensure active tab doesn't stay on 'logs' when debug mode is disabled
  useEffect(() => {
    if (!isDebug && activeTab === 'logs') {
      setActiveTab('connection')
    }
  }, [isDebug, activeTab])

  // Device checking polling
  useEffect(() => {
    let intervalId: number | undefined

    const check = async () => {
      if (!checkingMode) return
      setStatus(t('status_checking'))
      const model = await getDeviceModel()
      if (!isMounted.current) return

      if (model) {
        setDetectedModel(model)
        if (model.includes('A9210')) {
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
    }

    if (checkingMode) {
      check()
      intervalId = window.setInterval(check, 2000)
    }

    return () => {
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
                activeTab === tab.key
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
            {activeTab === 'connection' && <ConnectionPanel />}
            {activeTab === 'logs' && <Logs />}
          </>
        ) : (
          <ConnectionPanel />
        )}
      </div>
    </div>
  )
}
