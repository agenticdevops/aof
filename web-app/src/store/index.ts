export { default as store, persistor, type RootState, type AppDispatch } from './store'
export { useAppDispatch, useAppSelector } from './hooks'

// App slice exports
export {
  setNavigation,
  setTheme,
  setFirstVisit,
  toggleTheme,
} from './slices/appSlice'
export { default as appReducer } from './slices/appSlice'

// Onboarding slice exports
export {
  setStep,
  updateProject,
  updateAgent,
  updatePlatforms,
  addWizardPlatform,
  removeWizardPlatform,
  setLoading as setOnboardingLoading,
  setError as setOnboardingError,
  markStepCompleted,
  reset as resetOnboarding,
} from './slices/onboardingSlice'
export { default as onboardingReducer } from './slices/onboardingSlice'

// Config slice exports
export {
  setAgents,
  addAgent,
  updateConfigAgent,
  removeAgent,
  setTools,
  setPlatforms,
  addPlatform,
  updatePlatform,
  removePlatform,
  setSearchQuery,
  setVersion,
  setLoading as setConfigLoading,
  setError as setConfigError,
  setSelectedAgent,
  setSelectedPlatform,
} from './slices/configSlice'
export { default as configReducer } from './slices/configSlice'
