import { useAppContext } from '../context/AppContext'
export function Logs() {
  const { t, logs } = useAppContext()
  return (
    <div className='panel-stack'>
      <p className='hint'>{t('logs_note')}</p>
      <div className='log-list' aria-label={t('tab_logs')}>
        {logs.length ? (
          logs.map((log, index) => <p key={index}>{log}</p>)
        ) : (
          <p className='hint'>{t('no_logs')}</p>
        )}
      </div>
    </div>
  )
}
