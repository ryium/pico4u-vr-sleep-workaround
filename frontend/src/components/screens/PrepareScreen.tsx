import { useEffect } from 'react'
import { Button } from '@charcoal-ui/react'
import { useAppContext } from '../../context/AppContext'
import { StepIndicator } from '../ui/StepIndicator'
import { SetupChecklist } from '../SetupChecklist'
import { ErrorCard } from '../ErrorCard'
import type { ConnectionMode } from '../../types'

interface PrepareScreenProps {
  mode: ConnectionMode
  onBack: () => void
  onSuccess: () => void
}

export function PrepareScreen({ mode, onBack, onSuccess }: PrepareScreenProps) {
  const {
    t,
    setupWireless,
    checkDevices,
    wirelessSetupStatus,
    wiredSetupStatus,
    setupSteps,
    connectionError,
    connectManual,
    isConnected,
  } = useAppContext()

  const isWireless = mode === 'wireless'
  const isBusy = isWireless ? wirelessSetupStatus === 'loading' : wiredSetupStatus === 'loading'
  const isError = isWireless ? wirelessSetupStatus === 'error' : wiredSetupStatus === 'error'

  // If successfully connected, navigate to OperateScreen
  useEffect(() => {
    if (isConnected) {
      onSuccess()
    }
  }, [isConnected, onSuccess])

  const handleStartSetup = async () => {
    if (isWireless) {
      const ok = await setupWireless()
      if (ok) onSuccess()
    } else {
      const ok = await checkDevices()
      if (ok) onSuccess()
    }
  }

  // Filter steps for wired mode (only first 3 steps)
  const displaySteps = isWireless
    ? setupSteps
    : setupSteps.filter((s) => s.id === 'usb' || s.id === 'authorized' || s.id === 'pico')

  return (
    <div className='flex-1 flex flex-col justify-between px-5 py-2 animate-fadeIn'>
      <div>
        <StepIndicator currentStep='prepare' />

        <div className='mt-4 mb-5 text-center'>
          <h2 className='text-lg font-bold text-[var(--app-text)]'>
            {isWireless ? '無線接続の準備' : '有線接続の確認'}
          </h2>
          <p className='mt-1 text-xs text-[var(--app-text-muted)]'>
            {isWireless
              ? 'PCとヘッドセットをUSBでつないでください'
              : 'ヘッドセットをUSBでPCに接続してください'}
          </p>
        </div>

        {/* Checklist */}
        <div className='mb-4'>
          <SetupChecklist steps={displaySteps} />
        </div>

        {/* Error Card */}
        {isError && (
          <div className='mb-4'>
            <ErrorCard error={connectionError} />
          </div>
        )}

        {/* Manual IP Link for wireless */}
        {isWireless && (
          <div className='text-center my-2'>
            <button
              type='button'
              onClick={() => void connectManual()}
              className='text-xs text-[var(--app-text-muted)] hover:text-[var(--app-text)] underline cursor-pointer'
            >
              {t('btn_connect_manual')}
            </button>
          </div>
        )}
      </div>

      <div className='flex flex-col gap-2 pb-6 pt-2'>
        <Button onClick={handleStartSetup} disabled={isBusy} fullWidth>
          {isBusy
            ? t('wireless_status_loading')
            : isError
              ? t('btn_try_again')
              : isWireless
                ? t('btn_setup_wireless')
                : t('btn_check_devices')}
        </Button>

        <button
          type='button'
          onClick={onBack}
          disabled={isBusy}
          className='text-xs text-[var(--app-text-muted)] hover:text-[var(--app-text)] py-2 text-center transition-colors cursor-pointer'
        >
          {t('btn_back')}
        </button>
      </div>
    </div>
  )
}
