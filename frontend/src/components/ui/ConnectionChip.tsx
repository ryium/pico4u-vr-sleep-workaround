import type { ConnectionMode } from '../../types'

interface ConnectionChipProps {
  mode: ConnectionMode | null
  ip?: string
  connected: boolean
}

export function ConnectionChip({ mode, ip, connected }: ConnectionChipProps) {
  return (
    <div className='inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium bg-black/5 dark:bg-white/10 text-[var(--app-text)] border border-[var(--app-border)] shadow-2xs'>
      <span
        className={`w-2 h-2 rounded-full shrink-0 ${
          connected ? 'bg-brand' : 'bg-gray-400 dark:bg-gray-600'
        }`}
      />
      {mode === 'wireless' ? (
        <svg
          className='w-3.5 h-3.5 opacity-80'
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
      ) : (
        <svg
          className='w-3.5 h-3.5 opacity-80'
          viewBox='0 0 24 24'
          fill='none'
          stroke='currentColor'
          strokeWidth='2'
        >
          <circle cx='12' cy='12' r='2' />
          <path d='M12 2v8M12 14v8M8 6h8M8 18h8' />
        </svg>
      )}
      <span>
        {connected
          ? mode === 'wireless'
            ? `無線接続中${ip ? ` · ${ip}` : ''}`
            : 'USB接続中'
          : '未接続'}
      </span>
    </div>
  )
}
