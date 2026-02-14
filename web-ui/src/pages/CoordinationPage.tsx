/**
 * CoordinationPage - main coordination dashboard page.
 * Composes HeartbeatDashboard, StandupFeed, and CoordinationStatus components.
 */

import React from 'react';
import { useCoordination } from '../hooks/useCoordination';
import { HeartbeatDashboard } from '../components/HeartbeatDashboard';
import { StandupFeed } from '../components/StandupFeed';
import { CoordinationStatus } from '../components/CoordinationStatus';

/**
 * CoordinationPage component.
 *
 * Layout:
 * - Top: CoordinationStatus bar (mode + overhead)
 * - Main: HeartbeatDashboard (left 40%) + StandupFeed (right 60%)
 *
 * Uses useCoordination hook to fetch data and provide actions.
 *
 * @example
 * ```tsx
 * <CoordinationPage />
 * ```
 */
export function CoordinationPage(): React.ReactElement {
  const {
    health,
    latestStandup,
    metrics,
    isLoading,
    error,
    coordinationEnabled,
    triggerStandup,
    forceMode,
    refreshHealth,
  } = useCoordination();

  // Error state
  if (error && !coordinationEnabled) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-8">
        <div className="max-w-7xl mx-auto">
          <div className="bg-red-50 dark:bg-red-900 border border-red-200 dark:border-red-700 rounded-lg p-6 text-center">
            <svg
              className="w-12 h-12 mx-auto mb-4 text-red-500 dark:text-red-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <h2 className="text-lg font-semibold text-red-900 dark:text-red-100 mb-2">
              Coordination Error
            </h2>
            <p className="text-sm text-red-700 dark:text-red-300 mb-4">
              {error}
            </p>
            <button
              onClick={refreshHealth}
              className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg font-medium transition-colors shadow-sm"
            >
              Retry
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Coordination disabled state
  if (!coordinationEnabled && !isLoading) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-8">
        <div className="max-w-7xl mx-auto">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
            Coordination Overview
          </h1>

          <div className="bg-blue-50 dark:bg-blue-900 border border-blue-200 dark:border-blue-700 rounded-lg p-8 text-center">
            <svg
              className="w-16 h-16 mx-auto mb-4 text-blue-500 dark:text-blue-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <h2 className="text-xl font-semibold text-blue-900 dark:text-blue-100 mb-3">
              Coordination Protocols Not Enabled
            </h2>
            <p className="text-sm text-blue-700 dark:text-blue-300 mb-4 max-w-2xl mx-auto">
              Enable coordination protocols in your configuration to monitor agent health,
              view standup results, and track token overhead.
            </p>

            <div className="bg-white dark:bg-gray-800 rounded-lg p-4 text-left max-w-xl mx-auto mb-6">
              <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2">
                Example Configuration:
              </h3>
              <pre className="text-xs bg-gray-100 dark:bg-gray-900 p-3 rounded overflow-x-auto">
{`coordination:
  enabled: true
  heartbeat:
    frequency_secs: 30
    timeout_secs: 10
  standup:
    schedule_cron: "0 9 * * *"  # Daily at 9 AM
  auto_degrade:
    enabled: true
    max_overhead_percent: 30`}
              </pre>
            </div>

            <a
              href="https://docs.aof.sh/coordination/protocols"
              className="inline-block px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-colors shadow-sm"
              target="_blank"
              rel="noopener noreferrer"
            >
              View Documentation
            </a>
          </div>
        </div>
      </div>
    );
  }

  // Loading state
  if (isLoading && health.length === 0) {
    return (
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-8">
        <div className="max-w-7xl mx-auto">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
            Coordination Overview
          </h1>
          <div className="flex items-center justify-center py-20">
            <div className="text-center">
              <div className="w-12 h-12 border-4 border-blue-600 border-t-transparent rounded-full animate-spin mx-auto mb-4" />
              <p className="text-gray-600 dark:text-gray-400">
                Loading coordination data...
              </p>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Main coordination dashboard
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-8">
      <div className="max-w-7xl mx-auto">
        {/* Page header */}
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100 mb-6">
          Coordination Overview
        </h1>

        {/* Coordination status bar */}
        <CoordinationStatus
          metrics={metrics}
          onForceMode={forceMode}
          isLoading={isLoading}
          className="mb-6"
        />

        {/* Main content: HeartbeatDashboard (left) + StandupFeed (right) */}
        <div className="grid grid-cols-1 lg:grid-cols-5 gap-6">
          {/* Left panel: HeartbeatDashboard (40% width on large screens) */}
          <div className="lg:col-span-2">
            <HeartbeatDashboard
              health={health}
              coordinationEnabled={coordinationEnabled}
            />
          </div>

          {/* Right panel: StandupFeed (60% width on large screens) */}
          <div className="lg:col-span-3">
            <StandupFeed
              standupResult={latestStandup}
              onTriggerStandup={triggerStandup}
              isLoading={isLoading}
            />
          </div>
        </div>

        {/* Error message (non-blocking) */}
        {error && coordinationEnabled && (
          <div className="mt-6 bg-yellow-50 dark:bg-yellow-900 border border-yellow-200 dark:border-yellow-700 rounded-lg p-4">
            <div className="flex items-center gap-2">
              <svg
                className="w-5 h-5 text-yellow-600 dark:text-yellow-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                />
              </svg>
              <p className="text-sm text-yellow-800 dark:text-yellow-200">
                {error}
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
