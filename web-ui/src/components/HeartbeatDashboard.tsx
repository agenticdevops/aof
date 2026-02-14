/**
 * HeartbeatDashboard component - displays agent health status grid.
 * Shows live agent health with color-coded status indicators (green/yellow/red).
 */

import React from 'react';
import type { AgentHealthRecord } from '../types/coordination';
import { StatusIndicator } from './StatusIndicator';

/**
 * Component props.
 */
export interface HeartbeatDashboardProps {
  /** Array of agent health records */
  health: AgentHealthRecord[];

  /** Whether coordination is enabled */
  coordinationEnabled: boolean;

  /** Optional className for styling */
  className?: string;
}

/**
 * Format timestamp as relative time (e.g., "5s ago", "2m ago").
 */
function formatRelativeTime(timestamp: string | null): string {
  if (!timestamp) return 'never';

  try {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSecs = Math.floor(diffMs / 1000);

    if (diffSecs < 60) return `${diffSecs}s ago`;

    const diffMins = Math.floor(diffSecs / 60);
    if (diffMins < 60) return `${diffMins}m ago`;

    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;

    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
  } catch {
    return 'unknown';
  }
}

/**
 * Get status color class for agent health status.
 */
function getStatusColor(status: string): string {
  switch (status) {
    case 'Healthy':
      return 'text-green-600 dark:text-green-400';
    case 'Degraded':
      return 'text-yellow-600 dark:text-yellow-400';
    case 'Unresponsive':
      return 'text-red-600 dark:text-red-400';
    default:
      return 'text-gray-600 dark:text-gray-400';
  }
}

/**
 * Get status dot color for visual indicator.
 */
function getStatusDotColor(status: string): string {
  switch (status) {
    case 'Healthy':
      return 'bg-green-500';
    case 'Degraded':
      return 'bg-yellow-500';
    case 'Unresponsive':
      return 'bg-red-500 animate-pulse';
    default:
      return 'bg-gray-500';
  }
}

/**
 * Agent health card component.
 */
function AgentHealthCard({ agent }: { agent: AgentHealthRecord }): React.ReactElement {
  const relativeTime = formatRelativeTime(agent.last_heartbeat);
  const statusColor = getStatusColor(agent.status);
  const dotColor = getStatusDotColor(agent.status);

  return (
    <div
      className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 shadow-sm hover:shadow-md transition-shadow"
      data-agent-id={agent.agent_id}
      data-status={agent.status}
    >
      {/* Agent name + status dot */}
      <div className="flex items-center gap-2 mb-2">
        <div className={`w-3 h-3 rounded-full ${dotColor}`} />
        <h3 className="font-medium text-gray-900 dark:text-gray-100 truncate flex-1">
          {agent.agent_id}
        </h3>
      </div>

      {/* Status text */}
      <div className={`text-sm font-semibold mb-1 ${statusColor}`}>
        {agent.status}
      </div>

      {/* Last heartbeat time */}
      <div className="text-xs text-gray-600 dark:text-gray-400 mb-1">
        Last heartbeat: {relativeTime}
      </div>

      {/* Response latency (if available) */}
      {agent.last_response_ms !== null && (
        <div className="text-xs text-gray-500 dark:text-gray-500 mb-1">
          Latency: {agent.last_response_ms}ms
        </div>
      )}

      {/* Consecutive misses (if > 0, shown in red) */}
      {agent.consecutive_misses > 0 && (
        <div className="text-xs text-red-600 dark:text-red-400 font-medium">
          Missed: {agent.consecutive_misses}x
        </div>
      )}

      {/* Degraded reason (if present) */}
      {agent.degraded_reason && (
        <div className="text-xs text-yellow-700 dark:text-yellow-300 mt-1 italic">
          {agent.degraded_reason}
        </div>
      )}
    </div>
  );
}

/**
 * Calculate summary statistics from health records.
 */
function calculateSummary(health: AgentHealthRecord[]): {
  healthy: number;
  degraded: number;
  unresponsive: number;
  total: number;
} {
  const summary = {
    healthy: 0,
    degraded: 0,
    unresponsive: 0,
    total: health.length,
  };

  health.forEach((agent) => {
    switch (agent.status) {
      case 'Healthy':
        summary.healthy += 1;
        break;
      case 'Degraded':
        summary.degraded += 1;
        break;
      case 'Unresponsive':
        summary.unresponsive += 1;
        break;
    }
  });

  return summary;
}

/**
 * HeartbeatDashboard component.
 *
 * Displays a grid of agent health cards with:
 * - Status indicator (green/yellow/red dot)
 * - Agent name
 * - Last heartbeat time (relative)
 * - Response latency
 * - Consecutive miss count
 * - Summary bar at top
 *
 * @example
 * ```tsx
 * <HeartbeatDashboard
 *   health={agentHealthRecords}
 *   coordinationEnabled={true}
 * />
 * ```
 */
export function HeartbeatDashboard({
  health,
  coordinationEnabled,
  className = '',
}: HeartbeatDashboardProps): React.ReactElement {
  // Empty state: no agents or coordination disabled
  if (!coordinationEnabled || health.length === 0) {
    return (
      <div className={`bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-8 text-center ${className}`}>
        <div className="text-gray-400 dark:text-gray-500 mb-2">
          <svg
            className="w-16 h-16 mx-auto mb-4 opacity-50"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
          </svg>
          <p className="text-lg font-medium">Coordination not enabled</p>
          <p className="text-sm mt-2">
            Enable coordination protocols in your configuration to see agent health status.
          </p>
          <a
            href="https://docs.aof.sh/coordination/protocols"
            className="text-blue-500 hover:text-blue-600 text-sm mt-4 inline-block"
            target="_blank"
            rel="noopener noreferrer"
          >
            Learn more about coordination protocols →
          </a>
        </div>
      </div>
    );
  }

  const summary = calculateSummary(health);
  const lastCheckTime = health.length > 0
    ? formatRelativeTime(
        health.reduce((latest, agent) =>
          agent.last_heartbeat && (!latest || agent.last_heartbeat > latest)
            ? agent.last_heartbeat
            : latest
        , null as string | null)
      )
    : 'never';

  return (
    <div className={className}>
      {/* Summary bar */}
      <div className="bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-gray-800 dark:to-gray-750 rounded-lg p-4 mb-4 shadow-sm">
        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
          Agent Health
        </h2>
        <div className="flex items-center gap-4 text-sm">
          <span className="text-green-600 dark:text-green-400 font-medium">
            {summary.healthy}/{summary.total} healthy
          </span>

          {summary.degraded > 0 && (
            <>
              <span className="text-gray-300 dark:text-gray-600">|</span>
              <span className="text-yellow-600 dark:text-yellow-400 font-medium">
                {summary.degraded} degraded
              </span>
            </>
          )}

          {summary.unresponsive > 0 && (
            <>
              <span className="text-gray-300 dark:text-gray-600">|</span>
              <span className="text-red-600 dark:text-red-400 font-medium">
                {summary.unresponsive} unresponsive
              </span>
            </>
          )}

          <span className="text-gray-300 dark:text-gray-600">|</span>
          <span className="text-gray-600 dark:text-gray-400">
            Last check: {lastCheckTime}
          </span>
        </div>
      </div>

      {/* Agent health cards grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {health.map((agent) => (
          <AgentHealthCard key={agent.agent_id} agent={agent} />
        ))}
      </div>
    </div>
  );
}
