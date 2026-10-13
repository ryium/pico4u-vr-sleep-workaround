import type { ReactNode } from 'react'

interface SheetProps {
  title: string
  isOpen: boolean
  onClose: () => void
  children: ReactNode
}

export function Sheet({ title, isOpen, onClose, children }: SheetProps) {
  if (!isOpen) return null

  return (
    <div className='fixed inset-0 z-50 flex flex-col justify-end bg-black/50 backdrop-blur-xs animate-fadeIn'>
      <div className='bg-[var(--app-card)] text-[var(--app-text)] rounded-t-2xl max-h-[85vh] flex flex-col shadow-2xl border-t border-[var(--app-border)] overflow-hidden'>
        {/* Header */}
        <div className='flex items-center justify-between px-5 py-3.5 border-b border-[var(--app-border)]'>
          <h2 className='text-sm font-bold'>{title}</h2>
          <button
            type='button'
            onClick={onClose}
            className='p-1 rounded-md text-[var(--app-text-muted)] hover:text-[var(--app-text)] hover:bg-black/5 dark:hover:bg-white/5 cursor-pointer text-sm font-bold'
          >
            ✕
          </button>
        </div>

        {/* Content */}
        <div className='flex-1 overflow-y-auto p-5 custom-scrollbar'>{children}</div>
      </div>
    </div>
  )
}
