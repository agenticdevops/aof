import { configureStore, PreloadedState } from '@reduxjs/toolkit'
import { persistStore, persistReducer } from 'redux-persist'
import storage from 'redux-persist/lib/storage'
import appReducer from './slices/appSlice'
import onboardingReducer from './slices/onboardingSlice'
import configReducer from './slices/configSlice'

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
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware({
      serializableCheck: {
        ignoredActions: ['persist/PERSIST', 'persist/REHYDRATE'],
        ignoredPaths: ['onboarding.completedSteps'],
      },
    }),
})

export const persistor = persistStore(store)

export type RootState = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch

export default store
