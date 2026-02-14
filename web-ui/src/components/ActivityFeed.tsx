/**
 * ActivityFeed component - chronological timeline of agent activities.
 */

import React, { useEffect, useRef } from 'react';
import { useSelector } from 'react-redux';
import { ActivityItem } from './ActivityItem';
import { useActivities } from '../hooks/useActivities';
import { selectAllActivities, selectActivitiesLoading } from '../store/activitiesSlice';

/**
 * ActivityFeed component.
 */
export function ActivityFeed(): React.ReactElement {
  const activities = useSelector(selectAllActivities);
  const loading = useSelector(selectActivitiesLoading);
  const feedEndRef = useRef<HTMLDivElement>(null);

  // Initialize activity subscription
  useActivities();

  /**
   * Auto-scroll to newest activity on new arrival.
   */
  useEffect(() => {
    if (activities.length > 0) {
      feedEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [activities.length]);

  return (
    <div className="flex flex-col h-full bg-white dark:bg-gray-800 rounded-lg shadow">
      {/* Header */}
      <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Activity Feed</h3>
        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
          Live updates from agents (last 200 events)
        </p>
      </div>

      {/* Activity List */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {loading && activities.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-sm text-gray-500 dark:text-gray-400">Loading activity...</div>
          </div>
        ) : activities.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-sm text-gray-500 dark:text-gray-400">
              No activity yet. Waiting for agent actions...
            </div>
          </div>
        ) : (
          <>
            {activities.map((activity) => (
              <ActivityItem key={activity.eventId} activity={activity} />
            ))}
            <div ref={feedEndRef} />
          </>
        )}
      </div>
    </div>
  );
}
