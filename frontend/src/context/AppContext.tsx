import { createContext, useContext, ReactNode } from 'react'
import { useAppLogic } from '../hooks/useAppLogic'

type AppContextType = ReturnType<typeof useAppLogic>

const AppContext = createContext<AppContextType | null>(null)

export function AppContextProvider({ children }: { children: ReactNode }) {
  const logic = useAppLogic()

  return <AppContext.Provider value={logic}>{children}</AppContext.Provider>
}

export function useAppContext() {
  const context = useContext(AppContext)
  if (!context) {
    throw new Error('useAppContext must be used within an AppContextProvider')
  }
  return context
}
