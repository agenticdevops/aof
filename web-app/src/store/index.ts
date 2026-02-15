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

// Onboarding Slice exports (New 3-step wizard)
export {
  setCurrentStep,
  updateChannels,
  updateModel,
  updateTools,
  updateXopsConfig,
  clearErrors,
  resetWizard,
  submitWizard,
  validateChannels,
  validateModel,
  discoverTools,
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
  // Async thunks
  fetchAgents,
  createAgent,
  updateAgent,
  deleteAgent,
  fetchTools,
  fetchPlatforms,
  testPlatform,
  fetchVersion,
} from './slices/configSlice'
