/**
 * ActivityItem component - single collapsible activity in timeline.
 */

import React, { useState } from 'react';
import { formatRelativeTime } from '../utils/dateUtils';
import type { ActivityItem as ActivityItemType } from '../types/activities';

/**
 * Props for ActivityItem component.
 */
interface ActivityItemProps {
  /** Activity object */
  activity: ActivityItemType;
}

/**
 * Get border color class based on activity color.
 */
function getBorderColor(color: string): string {
  const colorMap: Record<string, string> = {
    red: 'border-red-500',
    green: 'border-green-500',
    blue: 'border-blue-500',
    orange: 'border-orange-500',
    yellow: 'border-yellow-500',
    purple: 'border-purple-500',
    gray: 'border-gray-500',
  };
  return colorMap[color] || 'border-gray-500';
}

/**
 * Get background color class based on activity color.
 */
function getBgColor(color: string): string {
  const colorMap: Record<string, string> = {
    red: 'bg-red-50 dark:bg-red-900/20',
    green: 'bg-green-50 dark:bg-green-900/20',
    blue: 'bg-blue-50 dark:bg-blue-900/20',
    orange: 'bg-orange-50 dark:bg-orange-900/20',
    yellow: 'bg-yellow-50 dark:bg-yellow-900/20',
    purple: 'bg-purple-50 dark:bg-purple-900/20',
    gray: 'bg-gray-50 dark:bg-gray-700',
  };
  return colorMap[color] || 'bg-gray-50 dark:bg-gray-700';
}

/**
 * ActivityItem component.
 */
export function ActivityItem({ activity }: ActivityItemProps): React.ReactElement {
  const [expanded, setExpanded] = useState(false);
  const relativeTime = formatRelativeTime(activity.timestamp);
  const borderColor = getBorderColor(activity.color);
  const bgColor = getBgColor(activity.color);

  return (
    <div
      className={`border-l-4 ${borderColor} ${bgColor} rounded-r-lg overflow-hidden transition-all`}
    >
      {/* Summary (always visible) */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full px-4 py-3 text-left hover:opacity-80 transition-opacity focus:outline-none focus:ring-2 focus:ring-blue-500"
        aria-expanded={expanded}
        aria-label={`Activity: ${activity.description}`}
      >
        <div className="flex items-start gap-3">
          {/* Icon */}
          <div className="flex-shrink-0 text-xl" aria-hidden="true">
            {activity.icon}
          </div>

          {/* Content */}
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-gray-900 dark:text-white">
              {activity.description}
            </p>
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
              Agent: {activity.agentName} • {relativeTime}
            </p>
          </div>

          {/* Expand indicator */}
          <div className="flex-shrink-0">
            <svg
              className={`w-5 h-5 text-gray-500 dark:text-gray-400 transition-transform ${
                expanded ? 'rotate-180' : ''
              }`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </div>
        </div>
      </button>

      {/* Details (collapsed by default) */}
      {expanded && (
        <div className="px-4 pb-3 border-t border-gray-200 dark:border-gray-600 mt-2">
          <dl className="space-y-2 mt-2">
            <div>
              <dt className="text-xs font-medium text-gray-500 dark:text-gray-400">Event ID</dt>
              <dd className="text-xs text-gray-900 dark:text-white font-mono">{activity.eventId}</dd>
            </div>
            <div>
              <dt className="text-xs font-medium text-gray-500 dark:text-gray-400">Activity Type</dt>
              <dd className="text-xs text-gray-900 dark:text-white">{activity.activityType}</dd>
            </div>
            <div>
              <dt className="text-xs font-medium text-gray-500 dark:text-gray-400">Timestamp</dt>
              <dd className="text-xs text-gray-900 dark:text-white">
                {new Date(activity.timestamp).toLocaleString()}
              </dd>
            </div>
            {Object.keys(activity.details).length > 0 && (
              <div>
                <dt className="text-xs font-medium text-gray-500 dark:text-gray-400">Details</dt>
                <dd className="text-xs text-gray-900 dark:text-white">
                  <pre className="mt-1 p-2 bg-gray-100 dark:bg-gray-800 rounded overflow-x-auto">
                    {JSON.stringify(activity.details, null, 2)}
                  </pre>
                </dd>
              </div>
            )}
          </dl>
        </div>
      )}
    </div>
  );
}
