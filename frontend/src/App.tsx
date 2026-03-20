import { AppContextProvider } from './context/AppContext'
import { MainView } from './components/MainView'

function App() {
  return (
    <AppContextProvider>
      <MainView />
    </AppContextProvider>
  )
}

export default App
