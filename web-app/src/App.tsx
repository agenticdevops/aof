import React, { useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import { Provider } from 'react-redux'
import { PersistGate } from 'redux-persist/integration/react'
import { store, persistor, useAppSelector } from '@/store'
import Layout from '@/components/layout/Layout'
import WelcomePage from '@/pages/WelcomePage'
import OnboardingWizard from '@/pages/OnboardingWizard'
import ConfigurationPage from '@/pages/ConfigurationPage'
import LoadingSpinner from '@/components/common/LoadingSpinner'

function AppContent() {
  const { navigation, theme } = useAppSelector((state) => state.app)

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

  return (
    <Layout>
      <Routes>
        <Route path="/" element={navigation === 'welcome' ? <WelcomePage /> : <ConfigurationPage />} />
        <Route path="/welcome" element={<WelcomePage />} />
        <Route path="/wizard" element={<OnboardingWizard />} />
        <Route path="/config" element={<ConfigurationPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </Layout>
  )
}

function App() {
  return (
    <Provider store={store}>
      <PersistGate loading={<LoadingSpinner fullPage />} persistor={persistor}>
        <Router>
          <AppContent />
        </Router>
      </PersistGate>
    </Provider>
  )
}

export default App
