/**
 * Main App component with WebSocket subscription and Redux integration.
 * Displays connection status, activity log, agent grid, and Kanban board.
 */

import React, { Suspense, lazy } from 'react';
import { useSelector } from 'react-redux';
import { useWebSocket } from './hooks/useWebSocket';
import { StatusIndicator } from './components/StatusIndicator';
import { Skeleton } from './components/Skeleton';
import type { RootState } from './store';

// Lazy load heavy components
const AgentGrid = lazy(() =>
  import('./components/AgentGrid').then((m) => ({ default: m.AgentGrid }))
);
const KanbanBoard = lazy(() =>
  import('./components/KanbanBoard').then((m) => ({ default: m.KanbanBoard }))
);

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
 * Main application component.
 */
export function App(): React.ReactElement {
  const wsUrl = getWebSocketUrl();
  const { connected, reconnectAttempts } = useWebSocket(wsUrl);
  const events = useSelector((state: RootState) => state.events.events);
  const connectedState = useSelector((state: RootState) => state.events.connected);

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

  // Get last 20 events for display
  const recentEvents = events.slice(-20).reverse();

  // Calculate statistics
  const totalEvents = events.length;
  const lastEventTimestamp = events.length > 0 ? events[events.length - 1].timestamp : 'N/A';

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 shadow">
        <div className="max-w-7xl mx-auto px-4 py-6 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between">
            <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
              AOF Mission Control
            </h1>
            <StatusIndicator status={connectionStatus} label={connectionLabel} />
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 py-8 sm:px-6 lg:px-8">
        {/* Agent Grid */}
        <section className="mb-8">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-white mb-4">
            Agents
          </h2>
          <Suspense
            fallback={
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 xl:grid-cols-5 gap-4">
                {Array.from({ length: 5 }).map((_, i) => (
                  <Skeleton key={i} width="100%" height="200px" variant="rectangular" />
                ))}
              </div>
            }
          >
            <AgentGrid />
          </Suspense>
        </section>

        {/* Kanban Board */}
        <section className="mb-8">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-white mb-4">
            Tasks
          </h2>
          <Suspense
            fallback={
              <div className="flex gap-4">
                {Array.from({ length: 5 }).map((_, i) => (
                  <Skeleton key={i} width="280px" height="500px" variant="rectangular" />
                ))}
              </div>
            }
          >
            <KanbanBoard />
          </Suspense>
        </section>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Statistics Panel */}
          <div className="lg:col-span-1">
            <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
              <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
                Statistics
              </h2>
              <dl className="space-y-3">
                <div>
                  <dt className="text-sm text-gray-500 dark:text-gray-400">
                    Connection Status
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {connectedState ? 'Connected' : 'Disconnected'}
                  </dd>
                </div>
                <div>
                  <dt className="text-sm text-gray-500 dark:text-gray-400">
                    Total Events Received
                  </dt>
                  <dd className="text-lg font-medium text-gray-900 dark:text-white">
                    {totalEvents}
                  </dd>
                </div>
                <div>
                  <dt className="text-sm text-gray-500 dark:text-gray-400">
                    Last Event
                  </dt>
                  <dd className="text-sm font-medium text-gray-900 dark:text-white">
                    {lastEventTimestamp}
                  </dd>
                </div>
              </dl>
            </div>
          </div>

          {/* Activity Log */}
          <div className="lg:col-span-2">
            <div className="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
              <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
                Activity Log (Last 20 Events)
              </h2>
              {recentEvents.length === 0 ? (
                <p className="text-gray-500 dark:text-gray-400 text-center py-8">
                  No events received yet. Waiting for agent activity...
                </p>
              ) : (
                <ul className="space-y-3">
                  {recentEvents.map((event) => (
                    <li
                      key={event.event_id}
                      className="border-l-4 border-blue-500 pl-4 py-2 bg-gray-50 dark:bg-gray-700"
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <p className="text-sm font-medium text-gray-900 dark:text-white">
                            {event.activity.type}
                          </p>
                          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                            Agent: {event.agent_id}
                          </p>
                          {event.activity.details && (
                            <pre className="text-xs text-gray-600 dark:text-gray-300 mt-2 overflow-x-auto">
                              {JSON.stringify(event.activity.details, null, 2)}
                            </pre>
                          )}
                        </div>
                        <time className="text-xs text-gray-500 dark:text-gray-400 ml-4">
                          {new Date(event.timestamp).toLocaleTimeString()}
                        </time>
                      </div>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;
