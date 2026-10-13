import { useTranslation } from 'react-i18next'

export type WizardStep = 'mode' | 'prepare' | 'operate'

interface StepIndicatorProps {
  currentStep: WizardStep
}

export function StepIndicator({ currentStep }: StepIndicatorProps) {
  const { t } = useTranslation()

  const steps = [
    { key: 'mode', label: t('step_mode', '接続方法') },
    { key: 'prepare', label: t('step_prepare', '準備') },
    { key: 'operate', label: t('step_operate', '開始') },
  ]

  const getStepIndex = (s: WizardStep) => {
    if (s === 'mode') return 0
    if (s === 'prepare') return 1
    return 2
  }

  const currentIndex = getStepIndex(currentStep)

  return (
    <div className='flex items-center justify-between px-6 py-2 shrink-0'>
      {steps.map((step, idx) => {
        const isDone = idx < currentIndex
        const isActive = idx === currentIndex

        return (
          <div key={step.key} className='flex flex-col items-center gap-1.5 flex-1 relative'>
            {idx > 0 && (
              <div
                className={`absolute top-2 -left-1/2 w-full h-[2px] transition-colors ${
                  idx <= currentIndex ? 'bg-brand' : 'bg-gray-200 dark:bg-gray-700'
                }`}
              />
            )}

            <div
              className={`w-4 h-4 rounded-full flex items-center justify-center text-[10px] font-bold z-10 transition-all ${
                isActive
                  ? 'bg-brand text-white shadow-sm ring-4 ring-brand/20'
                  : isDone
                    ? 'bg-brand text-white'
                    : 'bg-gray-200 dark:bg-gray-700 text-transparent'
              }`}
            >
              {isDone ? '✓' : ''}
            </div>

            <span
              className={`text-[11px] font-medium transition-colors ${
                isActive
                  ? 'text-brand font-semibold'
                  : isDone
                    ? 'text-[var(--app-text)]'
                    : 'text-[var(--app-text-muted)]'
              }`}
            >
              {step.label}
            </span>
          </div>
        )
      })}
    </div>
  )
}
