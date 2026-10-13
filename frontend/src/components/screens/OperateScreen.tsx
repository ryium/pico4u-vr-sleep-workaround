import { useState, useEffect } from 'react'
import { useAppContext } from '../../context/AppContext'
import { ConnectionChip } from '../ui/ConnectionChip'
import { PowerButton } from '../ui/PowerButton'

interface OperateScreenProps {
  onChangeConnection: () => void
  onOpenLogs: () => void
}

export function OperateScreen({ onChangeConnection, onOpenLogs }: OperateScreenProps) {
  const {
    t,
    connectionMode,
    deviceIp,
    isConnected,
    isRunning,
    toggleKeepAwake,
    keepAwakeInterval,
    dimAfterHours,
  } = useAppContext()

  // Elapsed time tracker while running
  const [elapsedSeconds, setElapsedSeconds] = useState(0)

  useEffect(() => {
    if (!isRunning) {
      setElapsedSeconds(0)
      return
    }

    const start = Date.now()
    const interval = setInterval(() => {
      setElapsedSeconds(Math.floor((Date.now() - start) / 1000))
    }, 1000)

    return () => clearInterval(interval)
  }, [isRunning])

  const formatElapsed = (sec: number) => {
    const h = Math.floor(sec / 3600)
    const m = Math.floor((sec % 3600) / 60)
    const s = sec % 60
    if (h > 0) {
      return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`
    }
    return `${m}:${String(s).padStart(2, '0')}`
  }

  return (
    <div className='flex-1 flex flex-col justify-between items-center px-5 py-4 animate-fadeIn select-none'>
      {/* Top Connection Chip */}
      <div className='pt-2'>
        <ConnectionChip mode={connectionMode} ip={deviceIp} connected={isConnected} />
      </div>

      {/* Center Action Area */}
      <div className='flex flex-col items-center justify-center my-auto'>
        <PowerButton
          isRunning={isRunning}
          onClick={toggleKeepAwake}
          disabled={!isConnected && !isRunning}
        />

        {/* Status text */}
        <div className='mt-3 text-center'>
          <p className='text-xs font-semibold text-[var(--app-text)]'>
            {isRunning
              ? `${keepAwakeInterval}秒ごとにスリープ回避信号を送信中`
              : t('status_stopped')}
          </p>

          {isRunning && (
            <p className='mt-1 text-[11px] text-[var(--app-text-muted)]'>
              経過 {formatElapsed(elapsedSeconds)}
              {dimAfterHours > 0 && ` · ${dimAfterHours}時間後に画面を暗くする`}
            </p>
          )}
        </div>
      </div>

      {/* Bottom links */}
      <div className='w-full flex justify-between items-center px-4 pb-4 pt-2 border-t border-[var(--app-border)] text-xs text-[var(--app-text-muted)]'>
        <button
          type='button'
          onClick={onChangeConnection}
          className='hover:text-[var(--app-text)] transition-colors cursor-pointer py-1'
        >
          接続を変更
        </button>

        <button
          type='button'
          onClick={onOpenLogs}
          className='hover:text-[var(--app-text)] transition-colors cursor-pointer py-1'
        >
          {t('tab_logs')}
        </button>
      </div>
    </div>
  )
}
