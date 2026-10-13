import { Icon } from '@charcoal-ui/react'
import packageJson from '../../../package.json'
import { useAppContext } from '../../context/AppContext'

interface AppHeaderProps {
  onOpenSettings: () => void
}

export function AppHeader({ onOpenSettings }: AppHeaderProps) {
  const { t } = useAppContext()

  return (
    <header className='px-5 pt-4 pb-2 shrink-0 flex items-center justify-between'>
      <div className='flex items-center gap-2.5'>
        {/* App Icon */}
        <div className='w-8 h-8 rounded-lg bg-brand flex items-center justify-center text-white shadow-sm shrink-0'>
          <svg className='w-5 h-5' viewBox='0 0 24 24' fill='currentColor'>
            <path d='M12 3a9 9 0 1 0 9 9c0-.46-.04-.92-.1-1.36a5.389 5.389 0 0 1-4.4 2.26 5.403 5.403 0 0 1-3.14-9.8c-.44-.06-.9-.1-1.36-.1z' />
          </svg>
        </div>
        <div>
          <div className='flex items-baseline gap-1.5'>
            <h1 className='text-base font-bold leading-none tracking-tight'>{t('title')}</h1>
            <span className='text-[10px] text-[var(--app-text-muted)] font-mono'>
              v{packageJson.version}
              {import.meta.env.DEV ? '-dev' : ''}
            </span>
          </div>
        </div>
      </div>

      <button
        type='button'
        onClick={onOpenSettings}
        className='p-1.5 rounded-lg text-[var(--app-text-muted)] hover:text-[var(--app-text)] hover:bg-black/5 dark:hover:bg-white/5 transition-colors cursor-pointer'
        title={t('settings_title')}
      >
        <Icon name='24/Settings' />
      </button>
    </header>
  )
}
