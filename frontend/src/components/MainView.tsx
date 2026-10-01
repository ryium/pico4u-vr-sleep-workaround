import { useRef, useState } from 'react'
import { Button } from '@charcoal-ui/react'
import { useAppContext } from '../context/AppContext'
import { StatusDiagram } from './StatusDiagram'
import { ConnectionPanel } from './ConnectionPanel'
import { Settings } from './Settings'
import { Logs } from './Logs'
import packageJson from '../../package.json'

type Tab = 'connection' | 'settings' | 'logs'
export function MainView() {
  const { t, isRunning, isConnected, isDebug, connection, settings, runtime, toggleKeepAwake } =
    useAppContext()
  const [active, setActive] = useState<Tab>('connection')
  const tabs: Tab[] = isDebug ? ['connection', 'settings', 'logs'] : ['connection', 'settings']
  const selected = tabs.includes(active) ? active : 'settings'
  const refs = useRef<Partial<Record<Tab, HTMLButtonElement | null>>>({})
  const stateLabel =
    connection.result.kind === 'error'
      ? t('connection_error')
      : connection.result.kind === 'loading'
        ? t('checking')
        : isConnected
          ? t('connected')
          : t('disconnected')
  return (
    <main className='dashboard'>
      <header className='app-header'>
        <div className='app-mark' aria-hidden='true'>
          P<span>4</span>
        </div>
        <div className='header-copy'>
          <h1>VR Sleep Workaround</h1>
          <p>{t('subtitle')}</p>
        </div>
        <span className='version'>v{packageJson.version}</span>
      </header>
      <section className='summary-card' aria-label={t('dashboard_summary')}>
        <StatusDiagram />
        <div className='status-line' role='status'>
          <span>{t('tab_connection')}</span>
          <strong
            className={
              isConnected ? 'positive' : connection.result.kind === 'error' ? 'negative' : ''
            }
          >
            <span className='dot' />
            {stateLabel}
          </strong>
        </div>
        <div className='status-line'>
          <span>Keep Alive</span>
          <strong className={isRunning ? 'positive' : ''}>
            <span className={`dot ${isRunning ? 'pulse' : ''}`} />
            {isRunning ? t('running') : t('stopped')}
          </strong>
        </div>
        <Button
          fullWidth
          variant={isRunning ? 'Danger' : 'Primary'}
          disabled={
            runtime.busy ||
            connection.busy ||
            settings.isSaving ||
            !runtime.ready ||
            (!isRunning && !isConnected)
          }
          onClick={() => void toggleKeepAwake()}
        >
          {runtime.busy ? t('working') : isRunning ? t('stop') : t('start')}
        </Button>
        <p className='summary-note'>
          {isRunning && !isConnected ? t('running_disconnected') : t('keep_alive_note')}
        </p>
        {runtime.error && (
          <p role='alert' className='error-text'>
            {t('operation_failed')} {t('error_hint')}
          </p>
        )}
      </section>
      <div className='tab-list' role='tablist' aria-label={t('dashboard_sections')}>
        {tabs.map((tab, index) => (
          <button
            key={tab}
            id={`tab-${tab}`}
            role='tab'
            ref={(node) => {
              refs.current[tab] = node
            }}
            aria-selected={selected === tab}
            aria-controls={`panel-${tab}`}
            tabIndex={selected === tab ? 0 : -1}
            onClick={() => setActive(tab)}
            onKeyDown={(event) => {
              let next = index
              if (event.key === 'ArrowRight') next = (index + 1) % tabs.length
              else if (event.key === 'ArrowLeft') next = (index + tabs.length - 1) % tabs.length
              else if (event.key === 'Home') next = 0
              else if (event.key === 'End') next = tabs.length - 1
              else return
              event.preventDefault()
              setActive(tabs[next])
              refs.current[tabs[next]]?.focus()
            }}
          >
            {t(`tab_${tab}`)}
          </button>
        ))}
      </div>
      {tabs.map((tab) => (
        <section
          key={tab}
          id={`panel-${tab}`}
          role='tabpanel'
          aria-labelledby={`tab-${tab}`}
          className='tab-panel'
          hidden={selected !== tab}
          tabIndex={0}
        >
          {tab === 'connection' ? (
            <ConnectionPanel />
          ) : tab === 'settings' ? (
            <Settings />
          ) : (
            <Logs />
          )}
        </section>
      ))}
    </main>
  )
}
