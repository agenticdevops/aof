export { store, persistor, type RootState, type AppDispatch } from './store'
export { useAppDispatch, useAppSelector } from './hooks'

// App Slice exports
export {
  setNavigation,
  setTheme,
  toggleTheme,
  setFirstVisit,
  setDaemonUrl,
} from './slices/appSlice'

// Onboarding Slice exports
export {
  setStep,
  updateProject,
  updateAgent,
  addWizardPlatform,
  removeWizardPlatform,
  updatePlatforms,
  setLoading as setOnboardingLoading,
  setError as setOnboardingError,
  markStepCompleted,
  reset as resetOnboarding,
  type OnboardingProject,
  type OnboardingAgent,
  type OnboardingPlatform,
} from './slices/onboardingSlice'

// Config Slice exports
export {
  setAgents,
  addAgent,
  updateConfigAgent,
  removeAgent,
  setTools,
  addTool,
  updateTool,
  removeTool,
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
