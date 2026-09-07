import { useAppContext } from '../context/AppContext'

export function StatusDiagram() {
  const { t, connection, connectionMode, isConnected } = useAppContext()
  const status = connection.result.kind === 'success' ? connection.result.status : null
  const value = (connected: boolean | undefined) =>
    status ? t(connected ? 'available' : 'unavailable') : t('unknown')
  return (
    <figure className='connection-diagram' aria-label={t('diagram_label')}>
      <div className='device-row'>
        <div className='device-node'>
          <svg viewBox='0 0 36 36' aria-hidden='true'>
            <rect x='5' y='5' width='26' height='20' rx='3' />
            <path d='M2 29h32M14 25v4m8-4v4' />
          </svg>
          <span>{t('this_pc')}</span>
        </div>
        <div className={`connection-link ${isConnected ? 'connected' : ''}`}>
          <span>{connectionMode ? t(`mode_${connectionMode}`) : t('choose_mode')}</span>
          <div className='link-track'>
            <span />
          </div>
        </div>
        <div className={`device-node ${isConnected ? 'positive' : ''}`}>
          <svg viewBox='0 0 36 36' aria-hidden='true'>
            <path d='M5 18c0-4 2-7 5-8h16c3 1 5 4 5 8v2a4 4 0 0 1-4 4h-4l-2-3h-6l-2 3H9a4 4 0 0 1-4-4z' />
            <circle cx='13' cy='17' r='3' />
            <circle cx='23' cy='17' r='3' />
            <path d='M2 13l3 2m26 0 3-2' />
          </svg>
          <span>PICO 4 Ultra</span>
        </div>
      </div>
      <figcaption className='transport-details'>
        <span title={t('usb_status')} className={status?.is_usb_connected ? 'positive' : ''}>
          USB <b>{value(status?.is_usb_connected)}</b>
        </span>
        <span
          title={status?.wifi_ip ?? t('wifi_status')}
          className={status?.is_wifi_ip_available ? 'positive' : ''}
        >
          Wi-Fi <b>{value(status?.is_wifi_ip_available)}</b>
        </span>
        <span title={t('adb_status')} className={status?.is_adb_tcp_connected ? 'positive' : ''}>
          ADB TCP <b>{value(status?.is_adb_tcp_connected)}</b>
        </span>
      </figcaption>
    </figure>
  )
}
