/**
 * AgentCard component - displays agent information with status indicator.
 * Shows avatar, name, role, personality, skills, and real-time status.
 */

import React, { useState } from 'react';
import type { Agent } from '../types/events';
import { StatusIndicator } from './StatusIndicator';

/**
 * Component props.
 */
export interface AgentCardProps {
  /** Agent configuration object */
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
 * AgentCard component.
 *
 * @example
 * ```tsx
 * <AgentCard
 *   agent={{
 *     id: 'agent-1',
 *     name: 'K8s Monitor',
 *     role: 'monitor',
 *     personality: 'Vigilant and detail-oriented',
 *     skills: ['kubernetes', 'observability'],
 *     status: 'working',
 *   }}
 *   lastActivity="2024-02-14T12:34:56Z"
 *   onClick={(id) => console.log('Clicked agent:', id)}
 * />
 * ```
 */
export function AgentCard({
  agent,
  lastActivity,
  onClick,
  className = '',
}: AgentCardProps): React.ReactElement {
  const [showTooltip, setShowTooltip] = useState(false);

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
      className={`relative bg-white dark:bg-gray-800 rounded-lg shadow-md border border-gray-200 dark:border-gray-700 p-4 transition-all hover:shadow-lg hover:scale-105 cursor-pointer ${className}`}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
      role="button"
      tabIndex={0}
      aria-label={`Agent ${agent.name}, role: ${agent.role}, status: ${agent.status}`}
    >
      {/* Avatar */}
      <div className="flex justify-center mb-3">
        <div className="w-16 h-16 flex items-center justify-center text-4xl bg-gray-100 dark:bg-gray-700 rounded-full">
          {avatar}
        </div>
      </div>

      {/* Agent name and role */}
      <div className="text-center mb-2">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">{agent.name}</h3>
        <p className="text-sm text-gray-600 dark:text-gray-400">{agent.role}</p>
      </div>

      {/* Personality description (truncated) */}
      {agent.personality && (
        <p className="text-xs text-gray-500 dark:text-gray-400 text-center mb-3 line-clamp-2">
          "{agent.personality}"
        </p>
      )}

      {/* Skills badges */}
      {agent.skills && agent.skills.length > 0 && (
        <div className="flex flex-wrap gap-1 justify-center mb-3">
          {agent.skills.slice(0, 3).map((skill) => (
            <span
              key={skill}
              className="px-2 py-1 text-xs font-medium bg-gray-800 dark:bg-gray-700 text-gray-100 rounded"
            >
              {skill}
            </span>
          ))}
          {agent.skills.length > 3 && (
            <span className="px-2 py-1 text-xs font-medium bg-gray-600 text-gray-100 rounded">
              +{agent.skills.length - 3}
            </span>
          )}
        </div>
      )}

      {/* Status indicator */}
      <div className="flex justify-center">
        <StatusIndicator status={agent.status} label={agent.status} />
      </div>

      {/* Tooltip (full personality + last activity) */}
      {showTooltip && (
        <div className="absolute z-10 left-1/2 transform -translate-x-1/2 bottom-full mb-2 w-64 p-3 bg-gray-900 text-white text-sm rounded-lg shadow-xl">
          {agent.personality && (
            <div className="mb-2">
              <strong>Personality:</strong>
              <p className="mt-1 text-gray-200">{agent.personality}</p>
            </div>
          )}
          <div>
            <strong>Last Activity:</strong>
            <p className="mt-1 text-gray-300">{formattedActivity}</p>
          </div>
          {/* Tooltip arrow */}
          <div className="absolute left-1/2 transform -translate-x-1/2 top-full w-0 h-0 border-l-8 border-r-8 border-t-8 border-transparent border-t-gray-900" />
        </div>
      )}
    </div>
  );
}
