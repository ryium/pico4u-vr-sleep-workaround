import { Button, TextField, DropdownSelector, DropdownMenuItem } from '@charcoal-ui/react'
import { useState, useEffect } from 'react'
import { useAppContext } from '../context/AppContext'

export function Settings({ onClose }: { onClose?: () => void }) {
  const {
    t,
    i18n,
    changeLanguage,
    dimAfterHours,
    updateDimDelay,
    isDebug,
    toggleDebugMode,
    theme,
    setTheme,
  } = useAppContext()

  const [pendingLanguage, setPendingLanguage] = useState(i18n.language)
  const [pendingTheme, setPendingTheme] = useState(theme)
  const [pendingDimAfterHours, setPendingDimAfterHours] = useState(dimAfterHours)

  useEffect(() => {
    setPendingLanguage(i18n.language)
    setPendingTheme(theme)
    setPendingDimAfterHours(dimAfterHours)
  }, [i18n.language, theme, dimAfterHours])

  const handleApply = async () => {
    if (pendingLanguage !== i18n.language) {
      changeLanguage(pendingLanguage)
    }
    if (pendingTheme !== theme) {
      setTheme(pendingTheme as any)
    }
    if (pendingDimAfterHours !== dimAfterHours) {
      await updateDimDelay(pendingDimAfterHours)
    }
    onClose?.()
  }

  const handleCancel = () => {
    onClose?.()
  }

  return (
    <div className='flex flex-col gap-6'>
      <div>
        <DropdownSelector
          label={t('language_label')}
          value={pendingLanguage}
          onChange={(val) => setPendingLanguage(val)}
          className='w-full'
          showLabel
        >
          <DropdownMenuItem value='ja'>日本語</DropdownMenuItem>
          <DropdownMenuItem value='en'>English</DropdownMenuItem>
        </DropdownSelector>
      </div>

      <div>
        <DropdownSelector
          label={t('theme_label')}
          value={pendingTheme}
          onChange={(val) => setPendingTheme(val as any)}
          className='w-full'
          showLabel
        >
          <DropdownMenuItem value='light'>{t('theme_light')}</DropdownMenuItem>
          <DropdownMenuItem value='dark'>{t('theme_dark')}</DropdownMenuItem>
          <DropdownMenuItem value='system'>{t('theme_system')}</DropdownMenuItem>
        </DropdownSelector>
      </div>

      <div>
        <TextField
          label={t('dim_setting_label')}
          type='number'
          value={String(pendingDimAfterHours)}
          onChange={(val) => setPendingDimAfterHours(Number(val))}
          assistiveText={t('dim_setting_note')}
          className='w-full text-left'
          showLabel
        />
      </div>

      <div className='flex items-center gap-2 mt-2'>
        <input
          type='checkbox'
          id='debug-mode'
          checked={isDebug}
          onChange={toggleDebugMode}
          className='rounded text-brand focus:ring-brand bg-gray-100 dark:bg-gray-700 border-gray-300 dark:border-gray-600 w-4 h-4 cursor-pointer'
        />
        <label
          htmlFor='debug-mode'
          className='text-sm font-bold text-gray-900 dark:text-gray-100 cursor-pointer'
        >
          {t('debug_mode', { defaultValue: 'Debug Mode' })}
        </label>
      </div>

      <div className='mt-3 flex flex-col gap-3'>
        <Button onClick={handleApply} variant='Primary' fullWidth>
          {t('btn_apply_changes', { defaultValue: 'Apply Changes' })}
        </Button>
        <Button onClick={handleCancel} variant='Default' fullWidth>
          {t('cancel', { defaultValue: 'Cancel' })}
        </Button>
      </div>
    </div>
  )
}
