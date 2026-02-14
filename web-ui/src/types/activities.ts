/**
 * TypeScript types for activity timeline.
 */

import type { ActivityType } from './events';

/**
 * Activity item for timeline display.
 */
export interface ActivityItem {
  /** Unique event ID from CoordinationEvent */
  eventId: string;

  /** Agent ID that triggered this activity */
  agentId: string;

  /** Agent display name */
  agentName: string;

  /** Activity type (from CoordinationEvent) */
  activityType: ActivityType;

  /** Human-readable description */
  description: string;

  /** Additional details (from CoordinationEvent) */
  details: Record<string, unknown>;

  /** ISO 8601 timestamp */
  timestamp: string;

  /** Icon for display (emoji or icon name) */
  icon: string;

  /** Color for visual coding */
  color: string;
}

/**
 * Map activity type to icon and color.
 */
export function getActivityMeta(type: ActivityType): { icon: string; color: string } {
  const metaMap: Record<ActivityType, { icon: string; color: string }> = {
    agent_started: { icon: '▶️', color: 'blue' },
    agent_completed: { icon: '✅', color: 'green' },
    tool_called: { icon: '🔧', color: 'blue' },
    tool_executing: { icon: '⚙️', color: 'orange' },
    tool_completed: { icon: '✔️', color: 'green' },
    tool_failed: { icon: '❌', color: 'red' },
    thinking: { icon: '💭', color: 'purple' },
    error: { icon: '⚠️', color: 'red' },
    info: { icon: 'ℹ️', color: 'blue' },
    warning: { icon: '⚠️', color: 'yellow' },
    debug: { icon: '🐛', color: 'gray' },
  };

  return metaMap[type] || { icon: '•', color: 'gray' };
}

/**
 * Generate human-readable description from activity type and details.
 */
export function generateActivityDescription(
  type: ActivityType,
  agentName: string,
  details: Record<string, unknown>
): string {
  switch (type) {
    case 'agent_started':
      return `${agentName} started execution`;
    case 'agent_completed':
      return `${agentName} completed successfully`;
    case 'tool_called':
      return `${agentName} called tool: ${details.tool_name || 'unknown'}`;
    case 'tool_executing':
      return `${agentName} executing tool: ${details.tool_name || 'unknown'}`;
    case 'tool_completed':
      return `${agentName} completed tool: ${details.tool_name || 'unknown'}`;
    case 'tool_failed':
      return `${agentName} tool failed: ${details.tool_name || 'unknown'}`;
    case 'thinking':
      return `${agentName} is thinking...`;
    case 'error':
      return `${agentName} encountered an error`;
    case 'info':
      return `${agentName}: ${details.message || 'info'}`;
    case 'warning':
      return `${agentName}: ${details.message || 'warning'}`;
    case 'debug':
      return `${agentName}: ${details.message || 'debug'}`;
    default:
      return `${agentName} performed action: ${type}`;
  }
}
