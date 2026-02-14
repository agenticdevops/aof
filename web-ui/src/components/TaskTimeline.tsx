/**
 * TaskTimeline component - vertical timeline of task-related events.
 */

import React, { useState } from 'react';
import { useSelector } from 'react-redux';
import type { ActivityItem } from '../types/activities';
import type { RootState } from '../store';

/**
 * Props for TaskTimeline component.
 */
interface TaskTimelineProps {
  /** Task ID to show timeline for */
  taskId: string;
}

/**
 * Timeline event item props.
 */
interface TimelineEventProps {
  /** Activity item */
  activity: ActivityItem;

  /** Whether this is the last item */
  isLast: boolean;
}

/**
 * Get color class for activity color.
 */
function getColorClass(color: string): string {
  const colorMap: Record<string, string> = {
    red: 'bg-red-500',
    green: 'bg-green-500',
    blue: 'bg-blue-500',
    orange: 'bg-orange-500',
    yellow: 'bg-yellow-500',
    purple: 'bg-purple-500',
    gray: 'bg-gray-500',
  };
  return colorMap[color] || 'bg-gray-500';
}

/**
 * Single timeline event.
 */
function TimelineEvent({ activity, isLast }: TimelineEventProps): React.ReactElement {
  const [expanded, setExpanded] = useState(false);
  const colorClass = getColorClass(activity.color);

  return (
    <div className="flex gap-4 pb-8 relative">
      {/* Timeline line */}
      {!isLast && (
        <div className="absolute left-4 top-8 bottom-0 w-0.5 bg-gray-200 dark:bg-gray-700" />
      )}

      {/* Icon/Dot */}
      <div className="flex-shrink-0 relative z-10">
        <div className={`w-8 h-8 rounded-full ${colorClass} flex items-center justify-center text-white`}>
          <span className="text-sm">{activity.icon}</span>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <button
          onClick={() => setExpanded(!expanded)}
          className="w-full text-left focus:outline-none focus:ring-2 focus:ring-blue-500 rounded"
          aria-expanded={expanded}
        >
          <p className="text-sm font-medium text-gray-900 dark:text-white">
            {activity.description}
          </p>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            {new Date(activity.timestamp).toLocaleString()}
          </p>
        </button>

        {/* Expanded details */}
        {expanded && Object.keys(activity.details).length > 0 && (
          <div className="mt-2 p-3 bg-gray-50 dark:bg-gray-700 rounded text-xs">
            <pre className="overflow-x-auto">
              {JSON.stringify(activity.details, null, 2)}
            </pre>
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * TaskTimeline component.
 */
export function TaskTimeline({ taskId }: TaskTimelineProps): React.ReactElement {
  const activities = useSelector((state: RootState) => state.activities.activities);

  // Filter activities related to this task
  // For now, show all activities (future: filter by taskId in details)
  const taskActivities = activities.filter(
    (a) =>
      a.details.task_id === taskId ||
      a.details.taskId === taskId ||
      (a.activityType.includes('task') && a.details.id === taskId)
  );

  // Sort chronologically (oldest first for timeline)
  const sortedActivities = [...taskActivities].sort(
    (a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
  );

  if (sortedActivities.length === 0) {
    return (
      <div className="text-center py-8">
        <p className="text-sm text-gray-500 dark:text-gray-400">
          No history events for this task yet.
        </p>
      </div>
    );
  }

  return (
    <div className="py-4">
      {sortedActivities.map((activity, index) => (
        <TimelineEvent
          key={activity.eventId}
          activity={activity}
          isLast={index === sortedActivities.length - 1}
        />
      ))}
    </div>
  );
}
