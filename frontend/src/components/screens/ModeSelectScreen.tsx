import { useState } from 'react'
import { Button } from '@charcoal-ui/react'
import { useAppContext } from '../../context/AppContext'
import { StepIndicator } from '../ui/StepIndicator'
import { ChoiceCard } from '../ui/ChoiceCard'
import type { ConnectionMode } from '../../types'

interface ModeSelectScreenProps {
  onNext: (selectedMode: ConnectionMode) => void
}

export function ModeSelectScreen({ onNext }: ModeSelectScreenProps) {
  const { t, connectionMode } = useAppContext()
  const [selected, setSelected] = useState<ConnectionMode>(connectionMode ?? 'wireless')

  return (
    <div className='flex-1 flex flex-col justify-between px-5 py-2 animate-fadeIn'>
      <div>
        <StepIndicator currentStep='mode' />

        <div className='mt-5 mb-6 text-center'>
          <h2 className='text-lg font-bold text-[var(--app-text)]'>どうやって接続しますか？</h2>
          <p className='mt-1 text-xs text-[var(--app-text-muted)]'>あとから設定で変更できます</p>
        </div>

        <div className='flex flex-col gap-3.5'>
          {/* Card: Wired */}
          <ChoiceCard
            selected={selected === 'wired'}
            onClick={() => setSelected('wired')}
            icon={
              <svg
                className='w-6 h-6'
                viewBox='0 0 24 24'
                fill='none'
                stroke='currentColor'
                strokeWidth='2'
              >
                <circle cx='12' cy='12' r='2' />
                <path d='M12 2v8M12 14v8M8 6h8M8 18h8' />
              </svg>
            }
            title={t('mode_wired')}
            description='ケーブルでつなぐだけ。いちばん安定'
          />

          {/* Card: Wireless */}
          <ChoiceCard
            selected={selected === 'wireless'}
            onClick={() => setSelected('wireless')}
            icon={
              <svg
                className='w-6 h-6'
                viewBox='0 0 24 24'
                fill='none'
                stroke='currentColor'
                strokeWidth='2'
              >
                <path d='M5 12.55a11 11 0 0 1 14.08 0' />
                <path d='M1.42 9a16 16 0 0 1 21.16 0' />
                <path d='M8.53 16.11a6 6 0 0 1 6.95 0' />
                <line x1='12' y1='20' x2='12.01' y2='20' strokeWidth='3' />
              </svg>
            }
            title={t('mode_wireless')}
            description='最初だけUSBで準備が必要です'
            badge='おすすめ'
          />
        </div>
      </div>

      <div className='pb-6 pt-4'>
        <Button onClick={() => onNext(selected)} fullWidth>
          {t('btn_continue')}
        </Button>
      </div>
    </div>
  )
}
