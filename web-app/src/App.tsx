import React, { useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import { Provider } from 'react-redux'
import { PersistGate } from 'redux-persist/integration/react'
import { store, persistor, useAppSelector, useAppDispatch } from '@/store'
import Layout from '@/components/layout/Layout'
import WelcomePage from '@/pages/WelcomePage'
import OnboardingWizard from '@/pages/OnboardingWizard'
import ConfigurationPage from '@/pages/ConfigurationPage'
import MissionControl from '@/pages/MissionControl'
import SquadChat from '@/pages/SquadChat'
import LoadingSpinner from '@/components/common/LoadingSpinner'
import { ToastProvider, registerToastCallback, useToast } from '@/components/common/Toast'
import { useWebSocket } from '@/hooks/useWebSocket'
import { setConnectionStatus, selectWsUrl } from '@/store/slices/appSlice'
import { wsEventReceived, setToastCallback } from '@/middleware/websocketMiddleware'

function AppContent() {
  const { navigation, theme } = useAppSelector((state) => state.app)
  const wsUrl = useAppSelector(selectWsUrl)
  const dispatch = useAppDispatch()
  const { showToast } = useToast()

  // Apply theme class to document root
  useEffect(() => {
    if (theme === 'dark') {
      document.documentElement.classList.add('dark')
      document.documentElement.style.colorScheme = 'dark'
    } else {
      document.documentElement.classList.remove('dark')
      document.documentElement.style.colorScheme = 'light'
    }
  }, [theme])

  // Initialize WebSocket connection
  const { status } = useWebSocket({
    url: wsUrl,
    onMessage: (event) => {
      dispatch(wsEventReceived(event))
    },
    onConnect: () => {
      dispatch(setConnectionStatus('connected'))
      showToast('Connected to server', 'success')
    },
    onDisconnect: () => {
      dispatch(setConnectionStatus('disconnected'))
      showToast('Disconnected from server', 'warning')
    },
    onError: (error) => {
      dispatch(setConnectionStatus('error'))
      console.error('WebSocket error:', error)
      showToast('Connection error', 'error')
    },
  })

  // Update connection status in Redux
  useEffect(() => {
    dispatch(setConnectionStatus(status))
  }, [status, dispatch])

  // Register toast callback for middleware
  useEffect(() => {
    setToastCallback(showToast)
  }, [showToast])

  return (
    <Layout>
      <Routes>
        <Route path="/" element={navigation === 'welcome' ? <WelcomePage /> : <ConfigurationPage />} />
        <Route path="/welcome" element={<WelcomePage />} />
        <Route path="/wizard" element={<OnboardingWizard />} />
        <Route path="/config" element={<ConfigurationPage />} />
        <Route path="/mission-control" element={<MissionControl />} />
        <Route path="/squad-chat" element={<SquadChat />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </Layout>
  )
}

function App() {
  return (
    <Provider store={store}>
      <PersistGate loading={<LoadingSpinner fullPage />} persistor={persistor}>
        <ToastProvider>
          <Router>
            <AppContent />
          </Router>
        </ToastProvider>
      </PersistGate>
    </Provider>
  )
}

export default App
