import React from 'react'
import ReactDOM from 'react-dom/client'
import { CharcoalProvider } from '@charcoal-ui/react'
import '@charcoal-ui/react/dist/index.css'
import '@charcoal-ui/theme/css/v2/light.css'
import '@charcoal-ui/theme/css/v2/dark.css'
import '@charcoal-ui/icons'
import App from './App'
import './style.css'
import './i18n'

document.documentElement.classList.add('ch-token-v2')

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <CharcoalProvider>
      <App />
    </CharcoalProvider>
  </React.StrictMode>,
)
