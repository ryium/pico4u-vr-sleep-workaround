import type { ReactNode } from 'react'

interface ChoiceCardProps {
  selected: boolean
  onClick: () => void
  icon: ReactNode
  title: string
  description: string
  badge?: string
}

export function ChoiceCard({
  selected,
  onClick,
  icon,
  title,
  description,
  badge,
}: ChoiceCardProps) {
  return (
    <button
      type='button'
      onClick={onClick}
      className={`w-full text-left p-4 rounded-xl transition-all cursor-pointer relative border ${
        selected
          ? 'border-brand bg-brand/5 dark:bg-brand/10 shadow-sm ring-1 ring-brand/30'
          : 'border-[var(--app-border)] bg-[var(--app-card)] hover:border-gray-300 dark:hover:border-gray-600'
      }`}
    >
      {badge && (
        <span className='absolute top-3.5 right-3.5 px-2 py-0.5 rounded-full text-[10px] font-bold bg-brand text-white shadow-xs'>
          {badge}
        </span>
      )}

      <div className='flex items-start gap-3.5'>
        <div
          className={`p-2.5 rounded-lg shrink-0 ${
            selected
              ? 'bg-brand/10 text-brand'
              : 'bg-gray-100 dark:bg-gray-800 text-[var(--app-text-muted)]'
          }`}
        >
          {icon}
        </div>
        <div className='flex-1 pr-12'>
          <h3 className='text-sm font-bold text-[var(--app-text)] leading-snug'>{title}</h3>
          <p className='mt-1 text-xs text-[var(--app-text-muted)] leading-relaxed'>{description}</p>
        </div>
      </div>
    </button>
  )
}
