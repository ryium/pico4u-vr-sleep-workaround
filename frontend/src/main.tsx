import React from 'react'
import ReactDOM from 'react-dom/client'
import { CharcoalProvider } from '@charcoal-ui/react'
import '@charcoal-ui/react/dist/index.css'
import '@charcoal-ui/icons'
import App from './App'
import './style.css'
import './i18n'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <CharcoalProvider>
      <App />
    </CharcoalProvider>
  </React.StrictMode>,
)
