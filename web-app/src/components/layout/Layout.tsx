import React from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import { useAppDispatch, useAppSelector, toggleTheme } from '@/store'
import { selectConnectionStatus } from '@/store/slices/appSlice'
import { Moon, Sun } from 'lucide-react'

interface LayoutProps {
  children: React.ReactNode
}

export const Layout: React.FC<LayoutProps> = ({ children }) => {
  const dispatch = useAppDispatch()
  const navigate = useNavigate()
  const location = useLocation()
  const { theme } = useAppSelector((state) => state.app)
  const connectionStatus = useAppSelector(selectConnectionStatus)

  const handleThemeToggle = () => {
    dispatch(toggleTheme())
  }

  const routes = [
    { path: '/welcome', label: 'Welcome' },
    { path: '/wizard', label: 'Setup Wizard' },
    { path: '/config', label: 'Configuration' },
    { path: '/mission-control', label: 'Mission Control' },
    { path: '/squad-chat', label: 'Squad Chat' },
  ]

  const isActivePath = (path: string) => {
    return location.pathname === path
  }

  // Connection status styling
  const getConnectionStyles = () => {
    switch (connectionStatus) {
      case 'connected':
        return {
          dotColor: 'bg-emerald-500',
          text: 'Connected',
          textColor: 'text-emerald-600 dark:text-emerald-400',
          pulse: '',
        }
      case 'connecting':
        return {
          dotColor: 'bg-yellow-500',
          text: 'Connecting...',
          textColor: 'text-yellow-600 dark:text-yellow-400',
          pulse: 'animate-pulse',
        }
      case 'disconnected':
        return {
          dotColor: 'bg-gray-400',
          text: 'Disconnected',
          textColor: 'text-gray-600 dark:text-gray-400',
          pulse: '',
        }
      case 'error':
        return {
          dotColor: 'bg-red-500',
          text: 'Error',
          textColor: 'text-red-600 dark:text-red-400',
          pulse: '',
        }
    }
  }

  const connectionStyles = getConnectionStyles()

  return (
    <div className="min-h-screen bg-white dark:bg-gray-900">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          {/* Top bar with logo and theme toggle */}
          <div className="flex items-center justify-between mb-4">
            <div 
              className="flex items-center gap-2 cursor-pointer hover:opacity-80 transition-opacity"
              onClick={() => navigate('/welcome')}
            >
              <div className="w-8 h-8 bg-sky-400 dark:bg-sky-500 rounded-lg flex items-center justify-center text-white font-bold text-sm">
                AOF
              </div>
              <div>
                <span className="text-sm font-bold text-sky-600 dark:text-sky-400">Agentic Ops</span>
                <span className="text-xs text-gray-500 dark:text-gray-400 ml-1">Framework</span>
              </div>
            </div>

            {/* Right side: Connection status + Theme Toggle */}
            <div className="flex items-center gap-4">
              {/* Connection Status Indicator */}
              <div
                className="flex items-center gap-2"
                title="WebSocket Connection Status"
              >
                <div
                  className={`w-2 h-2 rounded-full ${connectionStyles.dotColor} ${connectionStyles.pulse}`}
                />
                <span className={`text-xs font-medium ${connectionStyles.textColor} hidden sm:inline`}>
                  {connectionStyles.text}
                </span>
              </div>

              {/* Theme Toggle */}
              <button
                onClick={handleThemeToggle}
                className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                aria-label="Toggle theme"
                title={`Switch to ${theme === 'light' ? 'dark' : 'light'} mode`}
              >
                {theme === 'light' ? (
                  <Moon className="w-5 h-5 text-gray-600" />
                ) : (
                  <Sun className="w-5 h-5 text-yellow-400" />
                )}
              </button>
            </div>
          </div>

          {/* Navigation tabs */}
          <nav className="flex gap-1 border-t border-gray-200 dark:border-gray-700 pt-3">
            {routes.map((route) => (
              <button
                key={route.path}
                onClick={() => navigate(route.path)}
                className={`px-4 py-2 rounded-t-lg font-medium transition-colors ${
                  isActivePath(route.path)
                    ? 'bg-sky-400/60 text-white dark:bg-sky-500/60'
                    : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
              >
                {route.label}
              </button>
            ))}
          </nav>
        </div>
      </header>

      {/* Main Content */}
      <main>{children}</main>
    </div>
  )
}

export default Layout
