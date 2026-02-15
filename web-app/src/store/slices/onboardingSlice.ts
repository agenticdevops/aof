import { createSlice, createAsyncThunk, PayloadAction } from '@reduxjs/toolkit'
import {
  OnboardingState,
  Channel,
  AIModelProvider,
  Tool,
  XopsConfig,
  DEFAULT_XOPS_PERSONA,
} from '@/types/onboarding'
import { configAPI } from '@/api/config'

/**
 * Initial state for onboarding wizard
 */
const initialState: OnboardingState = {
  currentStep: 1,
  selectedChannels: [],
  selectedModel: 'anthropic',
  selectedTools: [],
  xopsConfig: {},
  loading: false,
  error: null,
}

/**
 * Async thunk to submit completed wizard and create Xops agent
 */
export const submitWizard = createAsyncThunk(
  'onboarding/submitWizard',
  async (_, { rejectWithValue, getState }) => {
    try {
      const state = getState() as any
      const { onboarding } = state

      // Build complete Xops configuration from wizard state
      const xopsConfig: XopsConfig = {
        name: 'Xops',
        persona: DEFAULT_XOPS_PERSONA,
        role: 'orchestrator',
        channels: onboarding.selectedChannels.map((channel: Channel) => {
          // Get channel configs from somewhere - for now, create minimal configs
          return {
            platform: channel,
            validated: false,
          }
        }),
        model: {
          provider: onboarding.selectedModel,
          model: getDefaultModelForProvider(onboarding.selectedModel),
          apiKey: '', // API key should come from form, not stored in Redux
        },
        tools: onboarding.selectedTools,
      }

      // Call API to create agent
      const response = await configAPI.createAgent({
        name: xopsConfig.name,
        type: 'orchestrator',
        instructions: generateInstructions(xopsConfig),
        capabilities: onboarding.selectedTools.map((t: Tool) => t.name),
      })

      return {
        ...xopsConfig,
        agentId: response.id,
        createdAt: new Date().toISOString(),
      }
    } catch (error: any) {
      return rejectWithValue(
        error.message || 'Failed to create Xops agent. Please check your configuration.'
      )
    }
  }
)

/**
 * Async thunk to validate channel connectivity
 */
export const validateChannels = createAsyncThunk(
  'onboarding/validateChannels',
  async (channels: Channel[], { rejectWithValue }) => {
    try {
      // Implement channel validation logic
      // For now, just return success
      return { success: true }
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

/**
 * Async thunk to validate AI model API key
 */
export const validateModel = createAsyncThunk(
  'onboarding/validateModel',
  async (
    { provider, apiKey }: { provider: AIModelProvider; apiKey: string },
    { rejectWithValue }
  ) => {
    try {
      // Test API key by making small LLM call
      // Implementation depends on LLM provider SDK
      return { success: true }
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

/**
 * Async thunk to discover available tools
 */
export const discoverTools = createAsyncThunk(
  'onboarding/discoverTools',
  async (_, { rejectWithValue }) => {
    try {
      const tools = await configAPI.getTools()
      return tools || []
    } catch (error: any) {
      return rejectWithValue(error.message || 'Failed to discover tools')
    }
  }
)

/**
 * Get default model for a given provider
 */
function getDefaultModelForProvider(provider: AIModelProvider): string {
  const defaults: Record<AIModelProvider, string> = {
    anthropic: 'claude-haiku-4.5',
    openai: 'gpt-3.5-turbo',
    google: 'gemini-pro',
    groq: 'llama2-70b-4096',
    ollama: 'llama2',
  }
  return defaults[provider]
}

/**
 * Generate system instructions for Xops based on configuration
 */
function generateInstructions(config: XopsConfig): string {
  const toolNames = config.tools.map((t) => t.name).join(', ')
  const channelNames = config.channels.map((c) => c.platform).join(', ')

  return `You are Xops, an orchestrator agent for DevOps operations.

Role: You orchestrate and coordinate specialized agents to accomplish DevOps tasks.

Capabilities:
- Available tools: ${toolNames}
- Communication channels: ${channelNames}

Communication style: ${config.persona.communication_style}

Your primary responsibility is to delegate tasks to specialized agents, synthesize their results, and report back to the user.`
}

const onboardingSlice = createSlice({
  name: 'onboarding',
  initialState,
  reducers: {
    /**
     * Set current step in the wizard
     */
    setCurrentStep: (state, action: PayloadAction<number>) => {
      state.currentStep = action.payload as any
    },

    /**
     * Update selected channels
     */
    updateChannels: (state, action: PayloadAction<Channel[]>) => {
      state.selectedChannels = action.payload
      state.error = null
    },

    /**
     * Update selected AI model
     */
    updateModel: (state, action: PayloadAction<AIModelProvider>) => {
      state.selectedModel = action.payload
      state.error = null
    },

    /**
     * Update selected tools
     */
    updateTools: (state, action: PayloadAction<Tool[]>) => {
      state.selectedTools = action.payload
      state.error = null
    },

    /**
     * Update Xops configuration
     */
    updateXopsConfig: (state, action: PayloadAction<Partial<XopsConfig>>) => {
      state.xopsConfig = { ...state.xopsConfig, ...action.payload }
    },

    /**
     * Clear error messages
     */
    clearErrors: (state) => {
      state.error = null
    },

    /**
     * Reset wizard to initial state
     */
    resetWizard: () => initialState,
  },

  extraReducers: (builder) => {
    // submitWizard thunk
    builder
      .addCase(submitWizard.pending, (state) => {
        state.loading = true
        state.error = null
      })
      .addCase(submitWizard.fulfilled, (state, action) => {
        state.loading = false
        state.xopsConfig = action.payload
        state.error = null
      })
      .addCase(submitWizard.rejected, (state, action) => {
        state.loading = false
        state.error = action.payload as string
      })

    // validateChannels thunk
    builder
      .addCase(validateChannels.pending, (state) => {
        state.loading = true
      })
      .addCase(validateChannels.fulfilled, (state) => {
        state.loading = false
        state.error = null
      })
      .addCase(validateChannels.rejected, (state, action) => {
        state.loading = false
        state.error = action.payload as string
      })

    // validateModel thunk
    builder
      .addCase(validateModel.pending, (state) => {
        state.loading = true
      })
      .addCase(validateModel.fulfilled, (state) => {
        state.loading = false
        state.error = null
      })
      .addCase(validateModel.rejected, (state, action) => {
        state.loading = false
        state.error = action.payload as string
      })

    // discoverTools thunk
    builder
      .addCase(discoverTools.pending, (state) => {
        state.loading = true
      })
      .addCase(discoverTools.fulfilled, (state) => {
        state.loading = false
        state.error = null
      })
      .addCase(discoverTools.rejected, (state, action) => {
        state.loading = false
        state.error = action.payload as string
      })
  },
})

export const { setCurrentStep, updateChannels, updateModel, updateTools, updateXopsConfig, clearErrors, resetWizard } =
  onboardingSlice.actions

export default onboardingSlice.reducer
