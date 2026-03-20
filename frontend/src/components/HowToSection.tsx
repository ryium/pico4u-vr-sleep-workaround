import { useState } from 'react'
import { Icon } from '@charcoal-ui/react'
import { useAppContext } from '../context/AppContext'

export function HowToSection() {
  const { t, connectionMode } = useAppContext()
  const [showHowTo, setShowHowTo] = useState(false)

  return (
    <div className='bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 overflow-hidden shadow-sm'>
      <button
        onClick={() => setShowHowTo(!showHowTo)}
        className='w-full px-4 py-3 flex justify-between items-center text-sm font-bold text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors'
      >
        <div className='flex items-center gap-2'>
          <Icon name='16/Info' />
          {t('instructions_header')}
        </div>
        <Icon name={showHowTo ? '16/Up' : '16/Down'} />
      </button>

      {showHowTo && (
        <div className='px-4 pb-4 pt-3 border-t border-gray-100 dark:border-gray-700 text-sm text-gray-700 dark:text-gray-300 flex flex-col gap-4 bg-gray-50 dark:bg-gray-800/50'>
          <div className='flex items-start gap-3'>
            <span className='bg-white dark:bg-gray-700 text-gray-800 dark:text-gray-200 border border-gray-200 dark:border-gray-600 w-5 h-5 flex items-center justify-center rounded-full text-[10px] font-bold shrink-0 mt-0.5 shadow-sm'>
              1
            </span>
            <div>
              <b className='block text-gray-900 dark:text-gray-100 mb-1'>
                {connectionMode === 'wired'
                  ? t('instruction_wired_label')
                  : t('instruction_wireless_label')}
              </b>
              <p className='text-xs leading-relaxed opacity-80'>
                {connectionMode === 'wired'
                  ? t('instruction_wired_text')
                  : t('instruction_wireless_text')}
              </p>
            </div>
          </div>
          <div className='flex items-start gap-3'>
            <span className='bg-white dark:bg-gray-700 text-gray-800 dark:text-gray-200 border border-gray-200 dark:border-gray-600 w-5 h-5 flex items-center justify-center rounded-full text-[10px] font-bold shrink-0 mt-0.5 shadow-sm'>
              2
            </span>
            <div>
              <b className='block text-gray-900 dark:text-gray-100 mb-1'>
                {t('instruction_action_label')}
              </b>
              <p className='text-xs leading-relaxed opacity-80'>{t('instruction_action_text')}</p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
