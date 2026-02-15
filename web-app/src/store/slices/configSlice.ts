import { createAsyncThunk, createSlice, PayloadAction } from '@reduxjs/toolkit'
import { configAPI } from '@/api/config'
import type { Agent, Tool, Platform } from '@/types'
import type { BotTemplate, SquadConfig } from '@/types/agents'

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

// Squad-related thunks
export const listSquads = createAsyncThunk(
  'config/listSquads',
  async (_, { rejectWithValue }) => {
    try {
      return await configAPI.listSquads()
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

export const createSquadFromTemplate = createAsyncThunk(
  'config/createSquadFromTemplate',
  async (template: BotTemplate, { rejectWithValue }) => {
    try {
      const agents = await configAPI.createAgentFromTemplate(template)
      const squad: SquadConfig = {
        id: `squad-${Date.now()}`,
        name: template.name,
        templateId: template.id,
        agents: agents,
        createdAt: new Date(),
        lastModified: new Date()
      }
      return squad
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

export const deleteAgentFromSquad = createAsyncThunk(
  'config/deleteAgentFromSquad',
  async ({ squadId, agentId }: { squadId: string; agentId: string }, { rejectWithValue }) => {
    try {
      await configAPI.deleteAgentFromSquad(squadId, agentId)
      return { squadId, agentId }
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

export const deleteSquad = createAsyncThunk(
  'config/deleteSquad',
  async (squadId: string, { rejectWithValue }) => {
    try {
      await configAPI.deleteSquad(squadId)
      return squadId
    } catch (error: any) {
      return rejectWithValue(error.message)
    }
  }
)

interface ConfigState {
  agents: Agent[]
  tools: Tool[]
  platforms: Platform[]
  squads: SquadConfig[]
  version: string | null
  isLoading: boolean
  error: string | null
  searchQuery: string
  selectedAgentId: string | null
  selectedPlatformId: string | null
  selectedSquadId: string | null
}

const initialState: ConfigState = {
  agents: [],
  tools: [],
  platforms: [],
  squads: [],
  version: null,
  isLoading: false,
  error: null,
  searchQuery: '',
  selectedAgentId: null,
  selectedPlatformId: null,
  selectedSquadId: null,
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
    setSquads: (state, action: PayloadAction<SquadConfig[]>) => {
      state.squads = action.payload
    },
    addSquad: (state, action: PayloadAction<SquadConfig>) => {
      state.squads.push(action.payload)
    },
    updateSquad: (state, action: PayloadAction<SquadConfig>) => {
      const index = state.squads.findIndex((s) => s.id === action.payload.id)
      if (index >= 0) {
        state.squads[index] = action.payload
      }
    },
    removeSquad: (state, action: PayloadAction<string>) => {
      state.squads = state.squads.filter((s) => s.id !== action.payload)
      if (state.selectedSquadId === action.payload) {
        state.selectedSquadId = null
      }
    },
    setSelectedSquad: (state, action: PayloadAction<string | null>) => {
      state.selectedSquadId = action.payload
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

    // List Squads
    builder
      .addCase(listSquads.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(listSquads.fulfilled, (state, action) => {
        state.squads = action.payload
        state.isLoading = false
      })
      .addCase(listSquads.rejected, (state, action) => {
        state.error = action.payload as string
        state.isLoading = false
      })

    // Create Squad from Template
    builder
      .addCase(createSquadFromTemplate.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(createSquadFromTemplate.fulfilled, (state, action) => {
        state.squads.push(action.payload)
        state.isLoading = false
      })
      .addCase(createSquadFromTemplate.rejected, (state, action) => {
        state.error = action.payload as string
        state.isLoading = false
      })

    // Delete Agent from Squad
    builder
      .addCase(deleteAgentFromSquad.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(deleteAgentFromSquad.fulfilled, (state, action) => {
        const squad = state.squads.find((s) => s.id === action.payload.squadId)
        if (squad) {
          squad.agents = squad.agents.filter((a) => a.id !== action.payload.agentId)
        }
        state.isLoading = false
      })
      .addCase(deleteAgentFromSquad.rejected, (state, action) => {
        state.error = action.payload as string
        state.isLoading = false
      })

    // Delete Squad
    builder
      .addCase(deleteSquad.pending, (state) => {
        state.isLoading = true
        state.error = null
      })
      .addCase(deleteSquad.fulfilled, (state, action) => {
        state.squads = state.squads.filter((s) => s.id !== action.payload)
        state.isLoading = false
      })
      .addCase(deleteSquad.rejected, (state, action) => {
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
  setSquads,
  addSquad,
  updateSquad,
  removeSquad,
  setSelectedSquad,
} = configSlice.actions

export default configSlice.reducer
