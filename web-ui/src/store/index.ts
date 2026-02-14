/**
 * Redux store configuration.
 * Exports configured store with events and config slices.
 */

import { configureStore } from '@reduxjs/toolkit';
import eventsReducer from './eventsSlice';
import configReducer from './configSlice';
import tasksReducer from './tasksSlice';
import chatReducer from './chatSlice';
import activitiesReducer from './activitiesSlice';
import conversationReducer from './conversationSlice';

/**
 * Configure Redux store with slices.
 */
export const store = configureStore({
  reducer: {
    events: eventsReducer,
    config: configReducer,
    tasks: tasksReducer,
    chat: chatReducer,
    activities: activitiesReducer,
    conversation: conversationReducer,
  },
  // Enable Redux DevTools in development
  devTools: import.meta.env.DEV,
});

/**
 * Root state type.
 */
export type RootState = ReturnType<typeof store.getState>;

/**
 * Dispatch type.
 */
export type AppDispatch = typeof store.dispatch;
