/**
 * IntroductionCard component - renders agent introduction events
 * in the activity feed with special visual styling.
 *
 * Shows avatar emoji, agent name, role, intro message, and skills.
 * Visually distinct from regular activity items to highlight the
 * "meet the team" experience.
 */

import React from 'react';
import type { AgentIntroductionData } from '../types/events';

/**
 * Component props.
 */
export interface IntroductionCardProps {
  /** Introduction data from the coordination event */
  introduction: AgentIntroductionData;

  /** Event timestamp (ISO 8601) */
  timestamp: string;

  /** Optional className */
  className?: string;
}

/**
 * Format timestamp for display.
 */
function formatTime(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  } catch {
    return '';
  }
}

/**
 * IntroductionCard renders an agent's introduction event.
 *
 * @example
 * ```tsx
 * <IntroductionCard
 *   introduction={{
 *     agent_id: 'k8s-monitor',
 *     agent_name: 'Kubernetes Monitor',
 *     role: 'Infrastructure Specialist',
 *     avatar: '\u{1F916}',
 *     intro_message: 'I watch your clusters...',
 *     personality_summary: 'A methodical specialist...',
 *     skills: ['kubectl', 'pod-debugging'],
 *   }}
 *   timestamp="2026-02-14T10:30:00Z"
 * />
 * ```
 */
export const IntroductionCard = React.memo(function IntroductionCard({
  introduction,
  timestamp,
  className = '',
}: IntroductionCardProps): React.ReactElement {
  return (
    <div
      className={`bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3 ${className}`}
      data-testid={`introduction-${introduction.agent_id}`}
    >
      {/* Header: Avatar + Name + Role + Time */}
      <div className="flex items-center gap-2 mb-2">
        <span className="text-2xl" role="img" aria-label={`${introduction.agent_name} avatar`}>
          {introduction.avatar}
        </span>
        <div className="flex-1 min-w-0">
          <span className="font-semibold text-gray-900 dark:text-gray-100">
            {introduction.agent_name}
          </span>
          <span className="ml-1.5 text-xs text-blue-600 dark:text-blue-400">
            joined the squad
          </span>
        </div>
        <span className="text-xs text-gray-500 dark:text-gray-400 flex-shrink-0">
          {formatTime(timestamp)}
        </span>
      </div>

      {/* Role badge */}
      <div className="mb-2">
        <span className="inline-block px-2 py-0.5 text-xs font-medium bg-blue-100 dark:bg-blue-800 text-blue-700 dark:text-blue-300 rounded-full">
          {introduction.role}
        </span>
      </div>

      {/* Introduction message */}
      <p className="text-sm text-gray-700 dark:text-gray-300 italic mb-2">
        &ldquo;{introduction.intro_message}&rdquo;
      </p>

      {/* Skills list */}
      {introduction.skills.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {introduction.skills.map((skill) => (
            <span
              key={skill}
              className="px-1.5 py-0.5 text-xs bg-white dark:bg-gray-800 text-gray-600 dark:text-gray-400 rounded border border-gray-200 dark:border-gray-700"
            >
              {skill}
            </span>
          ))}
        </div>
      )}
    </div>
  );
});
