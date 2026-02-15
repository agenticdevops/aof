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
 * Uses PascalCase values matching Rust ActivityType enum serialization.
 */
export function getActivityMeta(type: ActivityType | string): { icon: string; color: string } {
  const metaMap: Record<string, { icon: string; color: string }> = {
    Started: { icon: '▶️', color: 'blue' },
    Completed: { icon: '✅', color: 'green' },
    Cancelled: { icon: '🚫', color: 'gray' },
    Thinking: { icon: '💭', color: 'purple' },
    Analyzing: { icon: '🔍', color: 'blue' },
    LlmCall: { icon: '📤', color: 'blue' },
    LlmWaiting: { icon: '⏳', color: 'blue' },
    LlmResponse: { icon: '📥', color: 'green' },
    ToolDiscovery: { icon: '🔧', color: 'blue' },
    ToolExecuting: { icon: '⚙️', color: 'orange' },
    ToolComplete: { icon: '✔️', color: 'green' },
    ToolFailed: { icon: '❌', color: 'red' },
    Memory: { icon: '💾', color: 'blue' },
    McpCall: { icon: '🔗', color: 'blue' },
    Validation: { icon: '✓', color: 'green' },
    Warning: { icon: '⚠️', color: 'yellow' },
    Error: { icon: '⚠️', color: 'red' },
    Info: { icon: 'ℹ️', color: 'blue' },
    Debug: { icon: '🐛', color: 'gray' },
  };

  return metaMap[type] || { icon: '•', color: 'gray' };
}

/**
 * Generate human-readable description from activity type and details.
 * Uses PascalCase values matching Rust ActivityType enum serialization.
 */
export function generateActivityDescription(
  type: ActivityType | string,
  agentName: string,
  details: Record<string, unknown>
): string {
  switch (type) {
    case 'Started':
      return `${agentName} started execution`;
    case 'Completed':
      return `${agentName} completed successfully`;
    case 'Cancelled':
      return `${agentName} execution cancelled`;
    case 'Thinking':
      return `${agentName} is thinking...`;
    case 'Analyzing':
      return `${agentName} is analyzing...`;
    case 'LlmCall':
      return `${agentName} calling LLM`;
    case 'LlmWaiting':
      return `${agentName} waiting for LLM response`;
    case 'LlmResponse':
      return `${agentName} received LLM response`;
    case 'ToolDiscovery':
      return `${agentName} discovering tools`;
    case 'ToolExecuting':
      return `${agentName} executing tool: ${details?.tool_name || 'unknown'}`;
    case 'ToolComplete':
      return `${agentName} completed tool: ${details?.tool_name || 'unknown'}`;
    case 'ToolFailed':
      return `${agentName} tool failed: ${details?.tool_name || 'unknown'}`;
    case 'Error':
      return `${agentName} encountered an error`;
    case 'Info':
      return `${agentName}: ${details?.message || (details?.metadata as Record<string, string>)?.type || 'info'}`;
    case 'Warning':
      return `${agentName}: ${details?.message || 'warning'}`;
    case 'Debug':
      return `${agentName}: ${details?.message || 'debug'}`;
    default:
      return `${agentName} performed action: ${type}`;
  }
}
