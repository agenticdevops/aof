/**
 * AgentGrid component - displays grid of agent cards with loading states.
 * Fetches agents from /api/config/agents and polls for version changes.
 */

import React, { useEffect, useState, useCallback } from 'react';
import { useSelector } from 'react-redux';
import type { RootState } from '../store';
import { AgentCard } from './AgentCard';
import { useAgentsConfig } from '../hooks/useAgentsConfig';
import { useConfigVersion } from '../hooks/useConfigVersion';
import { useAgentIntroduction } from '../hooks/useAgentIntroduction';
import type { Agent } from '../types/events';

/**
 * Component props.
 */
export interface AgentGridProps {
  /** Click handler for agent cards */
  onAgentClick?: (agentId: string) => void;

  /** Optional className for styling */
  className?: string;
}

/**
 * Skeleton loader for agent card (matches persona layout).
 */
function AgentCardSkeleton(): React.ReactElement {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-md border border-gray-200 dark:border-gray-700 p-4 animate-pulse">
      {/* Top section: Avatar + Name/Role + Metrics */}
      <div className="flex items-start gap-3 mb-3">
        <div className="w-14 h-14 bg-gray-300 dark:bg-gray-600 rounded-full flex-shrink-0" />
        <div className="flex-1 space-y-2">
          <div className="h-5 bg-gray-300 dark:bg-gray-600 rounded w-3/4" />
          <div className="h-4 bg-gray-300 dark:bg-gray-600 rounded w-1/2" />
          <div className="flex gap-1">
            <div className="h-5 w-16 bg-gray-300 dark:bg-gray-600 rounded-full" />
            <div className="h-5 w-14 bg-gray-300 dark:bg-gray-600 rounded-full" />
          </div>
        </div>
        <div className="space-y-1">
          <div className="h-3 w-14 bg-gray-300 dark:bg-gray-600 rounded" />
          <div className="h-3 w-16 bg-gray-300 dark:bg-gray-600 rounded" />
        </div>
      </div>

      {/* Middle: Status + Skills */}
      <div className="flex items-center gap-2 mb-3">
        <div className="w-3 h-3 bg-gray-300 dark:bg-gray-600 rounded-full" />
        <div className="h-4 w-12 bg-gray-300 dark:bg-gray-600 rounded" />
        <div className="flex gap-1">
          <div className="h-5 w-12 bg-gray-300 dark:bg-gray-600 rounded" />
          <div className="h-5 w-10 bg-gray-300 dark:bg-gray-600 rounded" />
        </div>
      </div>

      {/* Bottom: Capabilities placeholder */}
      <div className="h-8 bg-gray-200 dark:bg-gray-700 rounded" />
    </div>
  );
}

/**
 * Empty state component.
 */
function EmptyState(): React.ReactElement {
  return (
    <div className="col-span-full flex flex-col items-center justify-center py-12 text-center">
      <div className="text-6xl mb-4">🤖</div>
      <h3 className="text-xl font-semibold text-gray-700 dark:text-gray-300 mb-2">
        No Agents Configured
      </h3>
      <p className="text-gray-500 dark:text-gray-400 max-w-md">
        Add agents to your workspace configuration to see them here. Agents will appear automatically
        once configured.
      </p>
    </div>
  );
}

/**
 * Error state component.
 */
function ErrorState({ onRetry }: { onRetry: () => void }): React.ReactElement {
  return (
    <div className="col-span-full flex flex-col items-center justify-center py-12 text-center">
      <div className="text-6xl mb-4">⚠️</div>
      <h3 className="text-xl font-semibold text-gray-700 dark:text-gray-300 mb-2">
        Failed to Load Agents
      </h3>
      <p className="text-gray-500 dark:text-gray-400 mb-4 max-w-md">
        Unable to fetch agent configuration. Please check your connection and try again.
      </p>
      <button
        onClick={onRetry}
        className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
      >
        Retry
      </button>
    </div>
  );
}

/**
 * Toast notification component.
 */
function Toast({ message, onClose }: { message: string; onClose: () => void }): React.ReactElement {
  useEffect(() => {
    const timer = setTimeout(onClose, 3000);
    return () => clearTimeout(timer);
  }, [onClose]);

  return (
    <div className="fixed top-4 right-4 z-50 bg-blue-600 text-white px-4 py-3 rounded-lg shadow-lg flex items-center gap-2 animate-slide-in">
      <span>ℹ️</span>
      <span>{message}</span>
    </div>
  );
}

/**
 * Config version polling interval (10 seconds).
 */
const VERSION_POLL_INTERVAL = 10000;

/**
 * AgentGrid component.
 *
 * Features:
 * - Fetches agents from useAgentsConfig hook (from 04-01)
 * - Displays loading skeleton during initial load
 * - Maps agent status from Redux eventsSlice
 * - Polls config version every 10 seconds
 * - Refetches agents on version change
 * - Shows toast notification on config update
 * - Handles empty state (no agents)
 * - Handles error state with retry button
 *
 * @example
 * ```tsx
 * <AgentGrid onAgentClick={(id) => console.log('Clicked:', id)} />
 * ```
 */
export function AgentGrid({ onAgentClick, className = '' }: AgentGridProps): React.ReactElement {
  const { agents, loading, error, refetch } = useAgentsConfig();
  const [showToast, setShowToast] = useState(false);

  // Introduction toast handling (Phase 5-04)
  const { activeToasts, dismissToast, focusAgent } = useAgentIntroduction();

  // Get agent status from eventsSlice (maps agent_id to last activity)
  const events = useSelector((state: RootState) => state.events.events);

  // Build eventsByAgent map
  const eventsByAgent = React.useMemo(() => {
    const map: Record<string, typeof events> = {};
    events.forEach((event) => {
      if (!map[event.agent_id]) {
        map[event.agent_id] = [];
      }
      map[event.agent_id].push(event);
    });
    return map;
  }, [events]);

  /**
   * Poll config version and refetch if changed.
   */
  useConfigVersion(() => {
    console.log('Config version changed, reloading agents...');
    setShowToast(true);
    refetch();
  }, VERSION_POLL_INTERVAL);

  /**
   * Get last activity timestamp for an agent.
   */
  const getLastActivity = useCallback(
    (agentId: string): string | undefined => {
      const events = eventsByAgent[agentId];
      if (!events || events.length === 0) return undefined;

      // Get most recent event
      const latest = events[events.length - 1];
      return latest.timestamp;
    },
    [eventsByAgent],
  );

  /**
   * Get agent status from recent events.
   */
  const getAgentStatus = useCallback(
    (agentId: string): Agent['status'] => {
      const events = eventsByAgent[agentId];
      if (!events || events.length === 0) return 'idle';

      // Get most recent event
      const latest = events[events.length - 1];

      switch (latest.activity.type) {
        case 'agent_started':
        case 'thinking':
        case 'tool_executing':
          return 'working';
        case 'error':
        case 'tool_failed':
          return 'error';
        case 'agent_completed':
        case 'tool_completed':
          return 'idle';
        default:
          return 'idle';
      }
    },
    [eventsByAgent],
  );

  /**
   * Merge agent config with real-time status.
   */
  const agentsWithStatus: Agent[] = agents.map((agent) => ({
    ...agent,
    status: getAgentStatus(agent.id),
  }));

  // Loading state
  if (loading && agents.length === 0) {
    return (
      <div
        className={`grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5 ${className}`}
      >
        {Array.from({ length: 3 }).map((_, i) => (
          <AgentCardSkeleton key={i} />
        ))}
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className={`grid grid-cols-1 ${className}`}>
        <ErrorState onRetry={refetch} />
      </div>
    );
  }

  // Empty state
  if (agents.length === 0) {
    return (
      <div className={`grid grid-cols-1 ${className}`}>
        <EmptyState />
      </div>
    );
  }

  // Success state
  return (
    <>
      <div
        className={`grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5 ${className}`}
      >
        {agentsWithStatus.map((agent) => (
          <AgentCard
            key={agent.id}
            agent={agent}
            lastActivity={getLastActivity(agent.id)}
            onClick={onAgentClick}
          />
        ))}
      </div>

      {/* Config update toast */}
      {showToast && (
        <Toast message="Config updated, reloading agents..." onClose={() => setShowToast(false)} />
      )}

      {/* Introduction toasts (Phase 5-04) */}
      {activeToasts.length > 0 && (
        <div className="fixed top-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
          {activeToasts.map((toast) => (
            <div
              key={toast.agentName}
              className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl p-4 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-750 transition-colors"
              role="alert"
              aria-label={`Introduction from ${toast.agentName}`}
              onClick={() => {
                focusAgent(toast.agentName);
                dismissToast(toast.agentName);
              }}
            >
              <div className="flex items-start gap-3">
                <span className="text-2xl flex-shrink-0">{toast.avatar}</span>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-semibold text-gray-900 dark:text-gray-100">
                    {toast.agentName}
                  </p>
                  <p className="text-xs text-gray-600 dark:text-gray-400 mt-1 line-clamp-3">
                    &ldquo;{toast.message}&rdquo;
                  </p>
                </div>
                <button
                  type="button"
                  className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 flex-shrink-0"
                  onClick={(e) => {
                    e.stopPropagation();
                    dismissToast(toast.agentName);
                  }}
                  aria-label={`Dismiss introduction from ${toast.agentName}`}
                >
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </>
  );
}
