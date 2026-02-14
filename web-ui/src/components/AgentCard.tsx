/**
 * AgentCard component - displays agent persona with status and capabilities.
 * Redesigned in Phase 5-04 to foreground personality traits, avatar,
 * CAN/CANNOT boundaries, and reliability metrics.
 * Enhanced in Phase 5-05 with live reliability metrics via useAgentMetrics hook.
 */

import React, { useState } from 'react';
import type { Agent } from '../types/events';
import { StatusIndicator } from './StatusIndicator';
import { PersonalityTraits } from './PersonalityTraits';
import { CapabilityBoundaries } from './CapabilityBoundaries';
import { useAgentMetrics } from '../hooks/useAgentMetrics';

/**
 * Component props.
 */
export interface AgentCardProps {
  /** Agent configuration object (with persona fields) */
  agent: Agent;

  /** Last activity timestamp (ISO 8601 string) */
  lastActivity?: string;

  /** Click handler for opening agent detail modal */
  onClick?: (agentId: string) => void;

  /** Optional className for styling */
  className?: string;
}

/**
 * Format timestamp for tooltip display.
 */
function formatLastActivity(timestamp: string | undefined): string {
  if (!timestamp) return 'No recent activity';

  try {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;

    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;

    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays}d ago`;
  } catch {
    return 'Unknown';
  }
}

/**
 * Get default avatar emoji if none provided.
 */
function getDefaultAvatar(role: string): string {
  const roleMap: Record<string, string> = {
    orchestrator: '🎭',
    executor: '⚡',
    monitor: '👁️',
    analyst: '📊',
    debugger: '🐛',
    deployer: '🚀',
    default: '🤖',
  };

  const normalizedRole = role.toLowerCase();
  return roleMap[normalizedRole] || roleMap.default;
}

/**
 * Get color for reliability metric badge.
 */
function getMetricColor(value: number): string {
  if (value >= 95) return 'text-green-600 dark:text-green-400';
  if (value >= 80) return 'text-yellow-600 dark:text-yellow-400';
  if (value >= 60) return 'text-orange-600 dark:text-orange-400';
  return 'text-red-600 dark:text-red-400';
}

/**
 * Reliability metric badge component.
 * Shows loading spinner when metrics are being fetched,
 * "--" for null/insufficient data, and color-coded percentage otherwise.
 */
function MetricBadge({
  label,
  value,
  eventCount,
  loading,
}: {
  label: string;
  value: number | null | undefined;
  eventCount?: number;
  loading?: boolean;
}): React.ReactElement {
  if (loading) {
    return (
      <span className="text-xs text-gray-400 dark:text-gray-500 animate-pulse">
        {label} ...
      </span>
    );
  }

  if (value === undefined || value === null) {
    return (
      <span
        className="text-xs text-gray-400 dark:text-gray-500"
        title={`${label}: insufficient data${eventCount !== undefined ? ` (${eventCount} events)` : ''}`}
      >
        {label} --
      </span>
    );
  }

  const rounded = Math.round(value * 10) / 10;
  const tooltipBase = eventCount !== undefined
    ? `Based on ${eventCount} events`
    : 'Based on event history';

  return (
    <span
      className={`text-xs font-medium ${getMetricColor(value)}`}
      title={`${label}: ${rounded}% (${tooltipBase})`}
    >
      {label} {rounded}%
    </span>
  );
}

/**
 * AgentCard component.
 *
 * Layout:
 * 1. Top: Avatar (left) + Name/Role/Traits (right) + Metrics (far right)
 * 2. Middle: Status indicator + Skill tags
 * 3. Bottom: Expandable Capabilities (CAN/CANNOT)
 *
 * @example
 * ```tsx
 * <AgentCard
 *   agent={{
 *     id: 'k8s-monitor',
 *     name: 'Kubernetes Monitor',
 *     role: 'Infrastructure Specialist',
 *     avatar: '🤖',
 *     personality_traits: ['methodical', 'proactive', 'detail-oriented'],
 *     can: ['kubectl operations', 'pod debugging'],
 *     cannot: ['modify RBAC'],
 *     skills: ['kubectl', 'jq'],
 *     status: 'idle',
 *   }}
 * />
 * ```
 */
export const AgentCard = React.memo(function AgentCard({
  agent,
  lastActivity,
  onClick,
  className = '',
}: AgentCardProps): React.ReactElement {
  const [showTooltip, setShowTooltip] = useState(false);

  // Live reliability metrics from API (polls every 5s)
  const {
    uptime_percent: liveUptime,
    success_rate: liveSuccess,
    event_count: metricsEventCount,
    loading: metricsLoading,
  } = useAgentMetrics(agent.id, 5000);

  // Prefer live metrics over static agent props (fallback to agent props)
  const effectiveUptime = liveUptime ?? agent.uptime_percent ?? null;
  const effectiveSuccess = liveSuccess ?? agent.success_rate ?? null;

  const avatar = agent.avatar || getDefaultAvatar(agent.role);
  const formattedActivity = formatLastActivity(lastActivity);

  const handleClick = () => {
    if (onClick) {
      onClick(agent.id);
    }
  };

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleClick();
    }
  };

  return (
    <div
      className={`relative bg-white dark:bg-gray-800 rounded-lg shadow-md border border-gray-200 dark:border-gray-700 p-4 transition-all hover:shadow-lg hover:-translate-y-0.5 cursor-pointer flex flex-col ${className}`}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
      role="button"
      tabIndex={0}
      aria-label={`Agent ${agent.name}, role: ${agent.role}, status: ${agent.status}`}
      data-agent-id={agent.id}
    >
      {/* ===== TOP SECTION: Avatar + Name/Role + Metrics ===== */}
      <div className="flex items-start gap-3 mb-3">
        {/* Avatar (large emoji) */}
        <div className="w-14 h-14 flex items-center justify-center text-4xl bg-gray-100 dark:bg-gray-700 rounded-full flex-shrink-0">
          {avatar}
        </div>

        {/* Name, Role, Traits */}
        <div className="flex-1 min-w-0">
          <h3 className="text-base font-semibold text-gray-900 dark:text-gray-100 truncate">
            {agent.name}
          </h3>
          <p className="text-sm text-gray-600 dark:text-gray-400 truncate">
            {agent.role}
          </p>

          {/* Personality traits badges */}
          {agent.personality_traits && agent.personality_traits.length > 0 && (
            <PersonalityTraits
              traits={agent.personality_traits}
              className="mt-1.5"
            />
          )}
        </div>

        {/* Reliability metrics (right aligned, live from API) */}
        <div className="flex flex-col items-end gap-0.5 flex-shrink-0">
          <MetricBadge
            label="Uptime"
            value={effectiveUptime}
            eventCount={metricsEventCount}
            loading={metricsLoading}
          />
          <MetricBadge
            label="Success"
            value={effectiveSuccess}
            eventCount={metricsEventCount}
            loading={metricsLoading}
          />
        </div>
      </div>

      {/* ===== MIDDLE SECTION: Status + Skills ===== */}
      <div className="flex items-center gap-2 mb-3">
        <StatusIndicator status={agent.status} label={agent.status} />

        {/* Divider */}
        {agent.skills && agent.skills.length > 0 && (
          <span className="text-gray-300 dark:text-gray-600" aria-hidden="true">|</span>
        )}

        {/* Skill tags */}
        {agent.skills && agent.skills.length > 0 && (
          <div className="flex flex-wrap gap-1 flex-1">
            {agent.skills.slice(0, 3).map((skill) => (
              <span
                key={skill}
                className="px-1.5 py-0.5 text-xs bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded"
              >
                {skill}
              </span>
            ))}
            {agent.skills.length > 3 && (
              <span className="px-1.5 py-0.5 text-xs text-gray-500 dark:text-gray-400">
                +{agent.skills.length - 3}
              </span>
            )}
          </div>
        )}
      </div>

      {/* ===== BOTTOM SECTION: Capabilities (expandable) ===== */}
      {((agent.can && agent.can.length > 0) ||
        (agent.cannot && agent.cannot.length > 0)) && (
        <div
          className="mt-auto pt-2"
          onClick={(e) => e.stopPropagation()}
          onKeyDown={(e) => e.stopPropagation()}
        >
          <CapabilityBoundaries
            can={agent.can || []}
            cannot={agent.cannot || []}
          />
        </div>
      )}

      {/* ===== TOOLTIP: Full personality + last activity ===== */}
      {showTooltip && (
        <div className="absolute z-10 left-1/2 transform -translate-x-1/2 bottom-full mb-2 w-72 p-3 bg-gray-900 text-white text-sm rounded-lg shadow-xl pointer-events-none">
          {agent.personality && (
            <div className="mb-2">
              <strong>Personality:</strong>
              <p className="mt-1 text-gray-200">{agent.personality}</p>
            </div>
          )}
          {agent.communication_style && (
            <div className="mb-2">
              <strong>Style:</strong>
              <span className="ml-1 text-gray-300">{agent.communication_style}</span>
            </div>
          )}
          <div>
            <strong>Last Activity:</strong>
            <span className="ml-1 text-gray-300">{formattedActivity}</span>
          </div>
          {/* Tooltip arrow */}
          <div className="absolute left-1/2 transform -translate-x-1/2 top-full w-0 h-0 border-l-8 border-r-8 border-t-8 border-transparent border-t-gray-900" />
        </div>
      )}
    </div>
  );
});
