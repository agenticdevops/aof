/**
 * Redux store configuration.
 * Exports configured store with events and config slices.
 */

import { configureStore } from '@reduxjs/toolkit';
import eventsReducer from './eventsSlice';
import configReducer from './configSlice';

/**
 * Configure Redux store with slices.
 */
export const store = configureStore({
  reducer: {
    events: eventsReducer,
    config: configReducer,
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
