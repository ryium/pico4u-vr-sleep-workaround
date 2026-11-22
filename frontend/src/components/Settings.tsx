import { Button, TextField, DropdownSelector, DropdownMenuItem, Checkbox } from '@charcoal-ui/react'
import { useState } from 'react'
import { useAppContext } from '../context/AppContext'

export function Settings({ onClose }: { onClose?: () => void }) {
  const {
    t,
    i18n,
    changeLanguage,
    dimAfterHours,
    updateDimDelay,
    keepAwakeInterval,
    updateKeepAwakeInterval,
    steamVrAutoLaunch,
    updateSteamVrAutoLaunch,
    isDebug,
    toggleDebugMode,
    theme,
    setTheme,
  } = useAppContext()

  const [pendingLanguage, setPendingLanguage] = useState(i18n.language)
  const [pendingTheme, setPendingTheme] = useState(theme)
  const [pendingDimAfterHours, setPendingDimAfterHours] = useState(dimAfterHours)
  const [pendingInterval, setPendingInterval] = useState(keepAwakeInterval)
  const [pendingSteamVrAutoLaunch, setPendingSteamVrAutoLaunch] = useState(steamVrAutoLaunch)
  const [pendingDebug, setPendingDebug] = useState(isDebug)

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
    if (pendingInterval !== keepAwakeInterval) {
      await updateKeepAwakeInterval(pendingInterval)
    }
    if (pendingSteamVrAutoLaunch !== steamVrAutoLaunch) {
      await updateSteamVrAutoLaunch(pendingSteamVrAutoLaunch)
    }
    if (pendingDebug !== isDebug) {
      await toggleDebugMode(pendingDebug)
    }
    onClose?.()
  }

  const handleCancel = () => {
    onClose?.()
  }

  return (
    <div className='flex flex-col w-full'>
      <div className='flex flex-col gap-6 pb-6'>
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

        <div>
          <TextField
            label={t('keep_awake_interval_label')}
            type='number'
            value={String(pendingInterval)}
            onChange={(val) => setPendingInterval(Number(val))}
            assistiveText={t('keep_awake_interval_note')}
            className='w-full text-left'
            showLabel
          />
        </div>

        <div className='flex flex-col gap-1'>
          <div className='text-sm font-bold text-[var(--app-text)]'>
            <Checkbox checked={pendingSteamVrAutoLaunch} onChange={setPendingSteamVrAutoLaunch}>
              {t('steamvr_auto_launch_label')}
            </Checkbox>
          </div>
          <p className='text-xs text-[var(--app-text-muted)] pl-6'>
            {t('steamvr_auto_launch_note')}
          </p>
        </div>

        <div className='mt-0.5 text-sm font-bold text-[var(--app-text)]'>
          <Checkbox checked={pendingDebug} onChange={setPendingDebug}>
            {t('debug_mode')}
          </Checkbox>
        </div>
      </div>

      <div className='shrink-0 pt-4 flex flex-col gap-3 border-t border-[var(--app-border)] bg-[var(--app-card)]'>
        <Button onClick={handleApply} variant='Primary' fullWidth>
          {t('btn_apply_changes')}
        </Button>
        <Button onClick={handleCancel} variant='Default' fullWidth>
          {t('cancel')}
        </Button>
      </div>
    </div>
  )
}
