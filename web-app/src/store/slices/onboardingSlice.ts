import { createSlice, PayloadAction } from '@reduxjs/toolkit'

interface OnboardingProject {
  name: string
  description: string
}

interface OnboardingAgent {
  name: string
  model: string
  type: 'analyst' | 'coordinator' | 'specialist'
  instructions: string
  capabilities: string[]
}

interface OnboardingPlatform {
  connected: boolean
  config: Record<string, any>
  username?: string
  testResult?: {
    success: boolean
    message: string
  }
}

interface OnboardingState {
  currentStep: 1 | 2 | 3 | 4
  project: OnboardingProject
  agent: OnboardingAgent
  platforms: Record<string, OnboardingPlatform>
  completedSteps: number[]
  isLoading: boolean
  error: string | null
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
  completedSteps: [],
  isLoading: false,
  error: null,
}

const onboardingSlice = createSlice({
  name: 'onboarding',
  initialState,
  reducers: {
    setStep: (state, action: PayloadAction<1 | 2 | 3 | 4>) => {
      state.currentStep = action.payload
    },
    updateProject: (state, action: PayloadAction<Partial<OnboardingProject>>) => {
      state.project = { ...state.project, ...action.payload }
    },
    updateAgent: (state, action: PayloadAction<Partial<OnboardingAgent>>) => {
      state.agent = { ...state.agent, ...action.payload }
    },
    addWizardPlatform: (
      state,
      action: PayloadAction<{ key: string; platform: OnboardingPlatform }>
    ) => {
      state.platforms[action.payload.key] = action.payload.platform
    },
    removeWizardPlatform: (state, action: PayloadAction<string>) => {
      delete state.platforms[action.payload]
    },
    updatePlatforms: (state, action: PayloadAction<Record<string, OnboardingPlatform>>) => {
      state.platforms = action.payload
    },
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
    },
    markStepCompleted: (state, action: PayloadAction<1 | 2 | 3 | 4>) => {
      if (!state.completedSteps.includes(action.payload)) {
        state.completedSteps.push(action.payload)
      }
    },
    reset: () => initialState,
  },
})

export const {
  setStep,
  updateProject,
  updateAgent,
  addWizardPlatform,
  removeWizardPlatform,
  updatePlatforms,
  setLoading,
  setError,
  markStepCompleted,
  reset,
} = onboardingSlice.actions

export type { OnboardingProject, OnboardingAgent, OnboardingPlatform }
export default onboardingSlice.reducer
