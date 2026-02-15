import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import type { Agent, Tool, Platform } from '@/types'

interface ConfigState {
  agents: Agent[]
  tools: Tool[]
  platforms: Platform[]
  version: string
  isLoading: boolean
  error: string | null
  searchQuery: string
  selectedAgentId: string | null
  selectedPlatformId: string | null
}

const initialState: ConfigState = {
  agents: [],
  tools: [],
  platforms: [],
  version: '',
  isLoading: false,
  error: null,
  searchQuery: '',
  selectedAgentId: null,
  selectedPlatformId: null,
}

const configSlice = createSlice({
  name: 'config',
  initialState,
  reducers: {
    setAgents: (state, action: PayloadAction<Agent[]>) => {
      state.agents = action.payload
    },
    addAgent: (state, action: PayloadAction<Agent>) => {
      state.agents.push(action.payload)
    },
    updateConfigAgent: (state, action: PayloadAction<Agent>) => {
      const index = state.agents.findIndex((a) => a.id === action.payload.id)
      if (index >= 0) {
        state.agents[index] = action.payload
      }
    },
    removeAgent: (state, action: PayloadAction<string>) => {
      state.agents = state.agents.filter((a) => a.id !== action.payload)
      if (state.selectedAgentId === action.payload) {
        state.selectedAgentId = null
      }
    },
    setTools: (state, action: PayloadAction<Tool[]>) => {
      state.tools = action.payload
    },
    addTool: (state, action: PayloadAction<Tool>) => {
      state.tools.push(action.payload)
    },
    updateTool: (state, action: PayloadAction<Tool>) => {
      const index = state.tools.findIndex((t) => t.id === action.payload.id)
      if (index >= 0) {
        state.tools[index] = action.payload
      }
    },
    removeTool: (state, action: PayloadAction<string>) => {
      state.tools = state.tools.filter((t) => t.id !== action.payload)
    },
    setPlatforms: (state, action: PayloadAction<Platform[]>) => {
      state.platforms = action.payload
    },
    addPlatform: (state, action: PayloadAction<Platform>) => {
      state.platforms.push(action.payload)
    },
    updatePlatform: (state, action: PayloadAction<Platform>) => {
      const index = state.platforms.findIndex((p) => p.id === action.payload.id)
      if (index >= 0) {
        state.platforms[index] = action.payload
      }
    },
    removePlatform: (state, action: PayloadAction<string>) => {
      state.platforms = state.platforms.filter((p) => p.id !== action.payload)
      if (state.selectedPlatformId === action.payload) {
        state.selectedPlatformId = null
      }
    },
    setSearchQuery: (state, action: PayloadAction<string>) => {
      state.searchQuery = action.payload
    },
    setVersion: (state, action: PayloadAction<string>) => {
      state.version = action.payload
    },
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
    },
    setSelectedAgent: (state, action: PayloadAction<string | null>) => {
      state.selectedAgentId = action.payload
    },
    setSelectedPlatform: (state, action: PayloadAction<string | null>) => {
      state.selectedPlatformId = action.payload
    },
  },
})

export const {
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
  setLoading,
  setError,
  setSelectedAgent,
  setSelectedPlatform,
} = configSlice.actions

export default configSlice.reducer
