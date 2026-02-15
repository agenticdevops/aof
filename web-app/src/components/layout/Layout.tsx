import React from 'react'
import { useAppDispatch, useAppSelector, toggleTheme } from '@/store'

interface LayoutProps {
  children: React.ReactNode
}

export const Layout: React.FC<LayoutProps> = ({ children }) => {
  const dispatch = useAppDispatch()
  const { theme } = useAppSelector((state) => state.app)

  const handleThemeToggle = () => {
    dispatch(toggleTheme())
  }

  return (
    <div className="min-h-screen bg-white dark:bg-gray-900">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-2xl font-bold text-emerald-600 dark:text-emerald-400">AOF</span>
            <span className="text-gray-600 dark:text-gray-400">Agentic Ops Framework</span>
          </div>

          {/* Theme Toggle */}
          <button
            onClick={handleThemeToggle}
            className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
            aria-label="Toggle theme"
            title={`Switch to ${theme === 'light' ? 'dark' : 'light'} mode`}
          >
            {theme === 'light' ? (
              <svg className="w-5 h-5 text-gray-600" fill="currentColor" viewBox="0 0 20 20">
                <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
              </svg>
            ) : (
              <svg className="w-5 h-5 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
                <path
                  fillRule="evenodd"
                  d="M10 2a1 1 0 011 1v2a1 1 0 11-2 0V3a1 1 0 011-1zm4.586 1.414a1 1 0 01.707.293l1.414 1.414a1 1 0 01-1.414 1.414L13.879 3.12a1 1 0 010-1.414zm2.828 2.828a1 1 0 011.414 0l1.414 1.414a1 1 0 01-1.414 1.414l-1.414-1.414a1 1 0 010-1.414zm-2.828 2.828a1 1 0 011.414 0l1.414 1.414a1 1 0 11-1.414 1.414l-1.414-1.414a1 1 0 010-1.414zM10 7a1 1 0 011 1v2a1 1 0 11-2 0V8a1 1 0 011-1zm-4.586 1.414a1 1 0 010-1.414L6.12 6.121a1 1 0 011.414 1.414L5.414 8.414zm-2.828 2.828a1 1 0 010-1.414l1.414-1.414a1 1 0 011.414 1.414l-1.414 1.414a1 1 0 01-1.414 0zm2.828 2.828a1 1 0 010-1.414l1.414-1.414a1 1 0 111.414 1.414l-1.414 1.414a1 1 0 01-1.414 0zM10 13a1 1 0 011 1v2a1 1 0 11-2 0v-2a1 1 0 011-1zm4.586 1.414a1 1 0 010-1.414l1.414-1.414a1 1 0 111.414 1.414l-1.414 1.414a1 1 0 01-1.414 0zm2.828 2.828a1 1 0 010-1.414l1.414-1.414a1 1 0 011.414 1.414l-1.414 1.414a1 1 0 01-1.414 0zm-2.828 2.828a1 1 0 010-1.414l1.414-1.414a1 1 0 011.414 1.414l-1.414 1.414a1 1 0 01-1.414 0z"
                  clipRule="evenodd"
                />
              </svg>
            )}
          </button>
        </div>
      </header>

      {/* Main Content */}
      <main>{children}</main>
    </div>
  )
}

export default Layout
