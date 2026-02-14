/**
 * Redux slice for activity timeline.
 * Manages event stream converted to human-readable activity items.
 */

import { createSlice, createSelector, type PayloadAction } from '@reduxjs/toolkit';
import type { ActivityItem } from '../types/activities';
import { getActivityMeta, generateActivityDescription } from '../types/activities';
import type { CoordinationEvent } from '../types/events';
import type { RootState } from './index';

/**
 * Activities slice state structure.
 */
interface ActivitiesState {
  /** Activity items (limited to last 200) */
  activities: ActivityItem[];

  /** Loading state */
  loading: boolean;

  /** Error message (if any) */
  error: string | null;
}

/**
 * Initial activities slice state.
 */
const initialState: ActivitiesState = {
  activities: [],
  loading: false,
  error: null,
};

/**
 * Convert CoordinationEvent to ActivityItem.
 */
function coordinationEventToActivity(event: CoordinationEvent): ActivityItem {
  const meta = getActivityMeta(event.activity.type);
  const description = generateActivityDescription(
    event.activity.type,
    event.agent_id, // Use agent_id as name for now (future: lookup from config)
    event.activity.details
  );

  return {
    eventId: event.event_id,
    agentId: event.agent_id,
    agentName: event.agent_id, // TODO: Lookup agent name from config
    activityType: event.activity.type,
    description,
    details: event.activity.details,
    timestamp: event.timestamp,
    icon: meta.icon,
    color: meta.color,
  };
}

/**
 * Activities slice - manages activity timeline state.
 */
const activitiesSlice = createSlice({
  name: 'activities',
  initialState,
  reducers: {
    /**
     * Add activity item (keeps last 200).
     */
    addActivity: (state, action: PayloadAction<ActivityItem>) => {
      state.activities.push(action.payload);

      // Keep only last 200 activities
      if (state.activities.length > 200) {
        state.activities = state.activities.slice(-200);
      }
    },

    /**
     * Add activity from CoordinationEvent (convenience method).
     */
    addActivityFromEvent: (state, action: PayloadAction<CoordinationEvent>) => {
      const activity = coordinationEventToActivity(action.payload);
      state.activities.push(activity);

      // Keep only last 200 activities
      if (state.activities.length > 200) {
        state.activities = state.activities.slice(-200);
      }
    },

    /**
     * Set all activities (batch load).
     */
    setActivities: (state, action: PayloadAction<ActivityItem[]>) => {
      state.activities = action.payload;
      state.loading = false;
      state.error = null;
    },

    /**
     * Clear all activities.
     */
    clearActivities: (state) => {
      state.activities = [];
    },

    /**
     * Set loading state.
     */
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.loading = action.payload;
    },

    /**
     * Set error state.
     */
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload;
    },
  },
});

/**
 * Actions.
 */
export const {
  addActivity,
  addActivityFromEvent,
  setActivities,
  clearActivities,
  setLoading,
  setError,
} = activitiesSlice.actions;

/**
 * Selectors.
 */

/**
 * Base selector for activities array.
 */
const selectActivities = (state: RootState): ActivityItem[] => state.activities.activities;

/**
 * Select all activities (newest first for display).
 * Memoized to prevent unnecessary re-renders.
 */
export const selectAllActivities = createSelector(
  [selectActivities],
  (activities) => [...activities].reverse()
);

/**
 * Select activities since timestamp.
 */
export const selectActivitiesSince = (timestamp: string) => (state: RootState): ActivityItem[] =>
  state.activities.activities.filter((a) => a.timestamp > timestamp);

/**
 * Select activities by agent.
 */
export const selectActivitiesByAgent = (agentId: string) => (state: RootState): ActivityItem[] =>
  state.activities.activities.filter((a) => a.agentId === agentId);

/**
 * Select activities by type.
 */
export const selectActivitiesByType = (type: string) => (state: RootState): ActivityItem[] =>
  state.activities.activities.filter((a) => a.activityType === type);

/**
 * Select loading state.
 */
export const selectActivitiesLoading = (state: RootState): boolean => state.activities.loading;

/**
 * Select error state.
 */
export const selectActivitiesError = (state: RootState): string | null => state.activities.error;

/**
 * Select activity count.
 */
export const selectActivityCount = (state: RootState): number => state.activities.activities.length;

/**
 * Default export.
 */
export default activitiesSlice.reducer;
