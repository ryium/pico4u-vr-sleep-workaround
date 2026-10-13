import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { AppError } from '../lib/errors'

interface ErrorCardProps {
  error: AppError | null
  fallbackMessage?: string
}

export function ErrorCard({ error, fallbackMessage }: ErrorCardProps) {
  const { t } = useTranslation()
  const [showDetail, setShowDetail] = useState(false)

  if (!error && !fallbackMessage) return null

  const code = error?.code ?? 'unknown'
  const titleKey = `errors.${code}.title`
  const title = error ? (t(titleKey) !== titleKey ? t(titleKey) : code) : fallbackMessage
  const hints = error ? (t(`errors.${code}.hints`, { returnObjects: true }) as string[]) : []

  return (
    <div className='flex flex-col gap-2 rounded-lg p-3 text-sm bg-red-50 dark:bg-red-950/50 text-red-700 dark:text-red-300 border border-red-200 dark:border-red-900 animate-fadeIn text-left'>
      <div className='flex items-center gap-2 font-semibold'>
        <svg className='w-4 h-4 shrink-0' viewBox='0 0 20 20' fill='currentColor'>
          <path
            fillRule='evenodd'
            d='M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z'
            clipRule='evenodd'
          />
        </svg>
        <span>{title}</span>
      </div>

      {Array.isArray(hints) && hints.length > 0 && (
        <ul className='list-disc list-inside text-xs space-y-1 text-red-600 dark:text-red-300/90 pl-1'>
          {hints.map((hint, idx) => (
            <li key={idx} className='leading-relaxed'>
              {hint}
            </li>
          ))}
        </ul>
      )}

      {error?.detail && (
        <div className='mt-1 pt-1 border-t border-red-200/60 dark:border-red-900/60'>
          <button
            type='button'
            onClick={() => setShowDetail((v) => !v)}
            className='text-[11px] underline opacity-80 hover:opacity-100 cursor-pointer'
          >
            {t('details_toggle')} {showDetail ? '▲' : '▼'}
          </button>
          {showDetail && (
            <pre className='mt-1 p-1.5 bg-black/10 dark:bg-black/30 rounded text-[10px] font-mono break-all whitespace-pre-wrap'>
              {error.detail}
            </pre>
          )}
        </div>
      )}
    </div>
  )
}
