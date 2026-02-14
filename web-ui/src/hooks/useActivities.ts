/**
 * Hook for managing activities with WebSocket sync.
 */

import { useEffect } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { addActivityFromEvent } from '../store/activitiesSlice';
import type { CoordinationEvent } from '../types/events';
import type { AppDispatch, RootState } from '../store';

/**
 * Hook for activities management.
 * Automatically subscribes to CoordinationEvent stream from eventsSlice.
 */
export function useActivities() {
  const dispatch = useDispatch<AppDispatch>();
  const events = useSelector((state: RootState) => state.events.events);
  const activities = useSelector((state: RootState) => state.activities.activities);

  /**
   * Subscribe to new events and convert to activities.
   * Watches eventsSlice and adds new events to activitiesSlice.
   */
  useEffect(() => {
    // Only process the newest event if there are events
    if (events.length === 0) return;

    const latestEvent = events[events.length - 1];

    // Check if this event is already in activities (dedup by eventId)
    const exists = activities.some((a) => a.eventId === latestEvent.event_id);
    if (exists) return;

    // Add activity from event
    dispatch(addActivityFromEvent(latestEvent));
  }, [events, activities, dispatch]);
}
