/**
 * Main App component with WebSocket subscription, Redux integration, and routing.
 * Displays connection status and routes between Dashboard and CreateAgent pages.
 */

import React, { useState, useEffect } from 'react';
import { useSelector } from 'react-redux';
import { useWebSocket } from './hooks/useWebSocket';
import { StatusIndicator } from './components/StatusIndicator';
import { Dashboard } from './pages/Dashboard';
import { CreateAgent } from './pages/CreateAgent';
import type { RootState } from './store';

/**
 * Get WebSocket URL from environment or default to localhost.
 */
function getWebSocketUrl(): string {
  if (import.meta.env.DEV) {
    // Development: use Vite proxy
    return 'ws://localhost:8080/ws';
  }

  // Production: use same host as page
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${window.location.host}/ws`;
}

/**
 * Simple hash-based router.
 */
function useHashRoute(): string {
  const [route, setRoute] = useState(window.location.hash.slice(1) || '/');

  useEffect(() => {
    const handleHashChange = () => {
      setRoute(window.location.hash.slice(1) || '/');
    };

    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);

  return route;
}

/**
 * Main application component.
 */
export function App(): React.ReactElement {
  const wsUrl = getWebSocketUrl();
  const { connected, reconnectAttempts } = useWebSocket(wsUrl);
  const connectedState = useSelector((state: RootState) => state.events.connected);
  const route = useHashRoute();

  // Determine connection status label
  const connectionStatus = connected
    ? 'connected'
    : reconnectAttempts > 0
    ? 'reconnecting'
    : 'disconnected';

  const connectionLabel = connected
    ? 'Connected'
    : reconnectAttempts > 0
    ? `Reconnecting (attempt ${reconnectAttempts})`
    : 'Disconnected';

  // Route to page
  let page: React.ReactElement;
  if (route === '/create-agent') {
    page = <CreateAgent />;
  } else {
    page = <Dashboard />;
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 flex flex-col">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 shadow">
        <div className="max-w-full mx-auto px-4 py-6 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-6">
              <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
                AOF Mission Control
              </h1>
              <nav className="flex gap-4">
                <a
                  href="#/"
                  className={`px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                    route === '/' || route === ''
                      ? 'bg-blue-600 text-white'
                      : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                  }`}
                >
                  Dashboard
                </a>
                <a
                  href="#/create-agent"
                  className={`px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                    route === '/create-agent'
                      ? 'bg-blue-600 text-white'
                      : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                  }`}
                >
                  + Create Agent
                </a>
              </nav>
            </div>
            <StatusIndicator status={connectionStatus} label={connectionLabel} />
          </div>
        </div>
      </header>

      {/* Main Content */}
      {page}
    </div>
  );
}

export default App;
