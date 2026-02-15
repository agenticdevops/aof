import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import { Agent, Tool, Platform } from '@/types'

interface ConfigState {
  agents: Agent[]
  tools: Tool[]
  platforms: Platform[]
  version: string
  isLoading: boolean
  error: string | null
  searchQuery: string
  selectedAgent: Agent | null
  selectedPlatform: Platform | null
}

const initialState: ConfigState = {
  agents: [],
  tools: [],
  platforms: [],
  version: '',
  isLoading: false,
  error: null,
  searchQuery: '',
  selectedAgent: null,
  selectedPlatform: null,
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
    },
    setTools: (state, action: PayloadAction<Tool[]>) => {
      state.tools = action.payload
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
    setSelectedAgent: (state, action: PayloadAction<Agent | null>) => {
      state.selectedAgent = action.payload
    },
    setSelectedPlatform: (state, action: PayloadAction<Platform | null>) => {
      state.selectedPlatform = action.payload
    },
  },
})

export const {
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
  setLoading,
  setError,
  setSelectedAgent,
  setSelectedPlatform,
} = configSlice.actions
export default configSlice.reducer
