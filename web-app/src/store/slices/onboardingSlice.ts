import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import { Agent, Platform, Project } from '@/types'

interface OnboardingState {
  currentStep: 1 | 2 | 3 | 4
  project: Partial<Project>
  agent: Partial<Agent>
  platforms: Record<string, Partial<Platform>>
  isLoading: boolean
  error: string | null
  completedSteps: Set<number>
}

const initialState: OnboardingState = {
  currentStep: 1,
  project: {
    name: '',
    description: '',
  },
  agent: {
    name: '',
    model: 'claude',
    type: 'analyst',
    instructions: '',
    capabilities: [],
  },
  platforms: {},
  isLoading: false,
  error: null,
  completedSteps: new Set(),
}

const onboardingSlice = createSlice({
  name: 'onboarding',
  initialState,
  reducers: {
    setStep: (state, action: PayloadAction<1 | 2 | 3 | 4>) => {
      state.currentStep = action.payload
    },
    updateProject: (state, action: PayloadAction<Partial<Project>>) => {
      state.project = { ...state.project, ...action.payload }
    },
    updateAgent: (state, action: PayloadAction<Partial<Agent>>) => {
      state.agent = { ...state.agent, ...action.payload }
    },
    updatePlatforms: (state, action: PayloadAction<Record<string, Partial<Platform>>>) => {
      state.platforms = action.payload
    },
    addWizardPlatform: (state, action: PayloadAction<{ type: string; config: Partial<Platform> }>) => {
      state.platforms[action.payload.type] = action.payload.config
    },
    removeWizardPlatform: (state, action: PayloadAction<string>) => {
      delete state.platforms[action.payload]
    },
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
    },
    markStepCompleted: (state, action: PayloadAction<number>) => {
      state.completedSteps.add(action.payload)
    },
    reset: () => initialState,
  },
})

export const {
  setStep,
  updateProject,
  updateAgent,
  updatePlatforms,
  addWizardPlatform,
  removeWizardPlatform,
  setLoading,
  setError,
  markStepCompleted,
  reset,
} = onboardingSlice.actions
export default onboardingSlice.reducer
