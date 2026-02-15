import { configureStore } from '@reduxjs/toolkit'
import { persistStore, persistReducer } from 'redux-persist'
import storage from 'redux-persist/lib/storage'
import appReducer from './slices/appSlice'
import onboardingReducer from './slices/onboardingSlice'
import configReducer from './slices/configSlice'
import auditReducer from './slices/auditSlice'
import dashboardReducer from './slices/dashboardSlice'
import chatReducer from './slices/chatSlice'
import personaReducer from './slices/personaSlice'
import { websocketMiddleware } from '@/middleware/websocketMiddleware'

const persistConfig = {
  key: 'aof-root',
  storage,
  whitelist: ['app', 'config'],
  version: 1,
}

const persistedAppReducer = persistReducer(persistConfig, appReducer)

export const store = configureStore({
  reducer: {
    app: persistedAppReducer,
    onboarding: onboardingReducer,
    config: configReducer,
    audit: auditReducer,
    dashboard: dashboardReducer,
    chat: chatReducer,
    persona: personaReducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: ['persist/PERSIST', 'persist/REHYDRATE', 'websocket/eventReceived'],
      },
    }).concat(websocketMiddleware),
})

export const persistor = persistStore(store)

export type RootState = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch
