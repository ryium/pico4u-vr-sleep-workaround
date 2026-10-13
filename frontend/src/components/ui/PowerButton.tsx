interface PowerButtonProps {
  isRunning: boolean
  onClick: () => void
  disabled?: boolean
}

export function PowerButton({ isRunning, onClick, disabled }: PowerButtonProps) {
  return (
    <div className='flex flex-col items-center justify-center my-3'>
      <button
        type='button'
        onClick={onClick}
        disabled={disabled}
        aria-label={isRunning ? 'スリープ回避 停止' : 'スリープ回避 開始'}
        className={`w-44 h-44 rounded-full flex flex-col items-center justify-center transition-all cursor-pointer select-none active:scale-95 relative ${
          disabled
            ? 'opacity-40 cursor-not-allowed bg-gray-200 dark:bg-gray-800 text-gray-400'
            : isRunning
              ? 'bg-[var(--app-card)] border-4 border-brand text-brand animate-glow'
              : 'bg-brand hover:bg-[#2f8c57] text-white shadow-md hover:shadow-lg'
        }`}
      >
        <div className='mb-2'>
          {isRunning ? (
            <svg className='w-10 h-10' viewBox='0 0 24 24' fill='currentColor'>
              <path d='M12 3a9 9 0 1 0 9 9c0-.46-.04-.92-.1-1.36a5.389 5.389 0 0 1-4.4 2.26 5.403 5.403 0 0 1-3.14-9.8c-.44-.06-.9-.1-1.36-.1z' />
            </svg>
          ) : (
            <svg className='w-10 h-10 ml-1' viewBox='0 0 24 24' fill='currentColor'>
              <path d='M8 5v14l11-7z' />
            </svg>
          )}
        </div>

        <span className='text-xl font-bold tracking-wider leading-none'>
          {isRunning ? '動作中' : '開始'}
        </span>
        <span className='text-[11px] mt-1.5 opacity-80 font-medium'>
          {isRunning ? 'タップで停止' : 'スリープ回避を開始'}
        </span>
      </button>
    </div>
  )
}
