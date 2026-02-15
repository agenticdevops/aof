import { createAsyncThunk, createSlice, PayloadAction } from '@reduxjs/toolkit'
import { configAPI } from '@/api/config'
import type { Agent, Tool, Platform } from '@/types'

export const fetchAgents = createAsyncThunk(
  'config/fetchAgents',
  async (_, { rejectWithValue }) => {
    try {
      const response = await configAPI.getAgents()
      return response.data
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

export const createAgent = createAsyncThunk(
  'config/createAgent',
  async (data: Partial<Agent>, { rejectWithValue }) => {
    try {
      const response = await configAPI.createAgent(data)
      return response.data
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

export const updateAgent = createAsyncThunk(
  'config/updateAgent',
  async ({ id, data }: { id: string; data: Partial<Agent> }, { rejectWithValue }) => {
    try {
      const response = await configAPI.updateAgent(id, data)
      return response.data
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

export const deleteAgent = createAsyncThunk(
  'config/deleteAgent',
  async (id: string, { rejectWithValue }) => {
    try {
      await configAPI.deleteAgent(id)
      return id
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

export const fetchTools = createAsyncThunk(
  'config/fetchTools',
  async (_, { rejectWithValue }) => {
    try {
      const response = await configAPI.getTools()
      return response.data
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

export const fetchPlatforms = createAsyncThunk(
  'config/fetchPlatforms',
  async (_, { rejectWithValue }) => {
    try {
      const response = await configAPI.getPlatforms()
      return response.data
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

export const testPlatform = createAsyncThunk(
  'config/testPlatform',
  async ({ platform, config }: { platform: string; config: any }, { rejectWithValue }) => {
    try {
      const response = await configAPI.testPlatform(platform, config)
      return response.data
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

export const fetchVersion = createAsyncThunk(
  'config/fetchVersion',
  async (_, { rejectWithValue }) => {
    try {
      const response = await configAPI.getVersion()
      return response.data
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

interface ConfigState {
  agents: Agent[]
  tools: Tool[]
  platforms: Platform[]
  version: string | null
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
  version: null,
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
  extraReducers: (builder) => {
    // Fetch Agents
    builder
      .addCase(fetchAgents.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(fetchAgents.fulfilled, (state, action) => {
        state.agents = action.payload
        state.isLoading = false
      })
      .addCase(fetchAgents.rejected, (state, action) => {
        state.error = action.payload as string
        state.isLoading = false
      })

    // Create Agent
    builder
      .addCase(createAgent.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(createAgent.fulfilled, (state, action) => {
        state.agents.push(action.payload)
        state.isLoading = false
      })
      .addCase(createAgent.rejected, (state, action) => {
        state.error = action.payload as string
        state.isLoading = false
      })

    // Update Agent
    builder
      .addCase(updateAgent.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(updateAgent.fulfilled, (state, action) => {
        const index = state.agents.findIndex((a) => a.id === action.payload.id)
        if (index !== -1) {
          state.agents[index] = action.payload
        }
        state.isLoading = false
      })
      .addCase(updateAgent.rejected, (state, action) => {
        state.error = action.payload as string
        state.isLoading = false
      })

    // Delete Agent
    builder
      .addCase(deleteAgent.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(deleteAgent.fulfilled, (state, action) => {
        state.agents = state.agents.filter((a) => a.id !== action.payload)
        state.isLoading = false
      })
      .addCase(deleteAgent.rejected, (state, action) => {
        state.error = action.payload as string
        state.isLoading = false
      })

    // Fetch Tools
    builder
      .addCase(fetchTools.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(fetchTools.fulfilled, (state, action) => {
        state.tools = action.payload
        state.isLoading = false
      })
      .addCase(fetchTools.rejected, (state, action) => {
        state.error = action.payload as string
        state.isLoading = false
      })

    // Fetch Platforms
    builder
      .addCase(fetchPlatforms.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(fetchPlatforms.fulfilled, (state, action) => {
        state.platforms = action.payload
        state.isLoading = false
      })
      .addCase(fetchPlatforms.rejected, (state, action) => {
        state.error = action.payload as string
        state.isLoading = false
      })

    // Test Platform
    builder
      .addCase(testPlatform.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(testPlatform.fulfilled, (state) => {
        state.isLoading = false
      })
      .addCase(testPlatform.rejected, (state, action) => {
        state.error = action.payload as string
        state.isLoading = false
      })

    // Fetch Version
    builder
      .addCase(fetchVersion.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(fetchVersion.fulfilled, (state, action) => {
        state.version = action.payload.version
        state.isLoading = false
      })
      .addCase(fetchVersion.rejected, (state, action) => {
        state.error = action.payload as string
        state.isLoading = false
      })
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
