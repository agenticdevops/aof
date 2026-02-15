---
wave: 1
depends_on: []
files_modified:
  - web-app/src/api/client.ts
  - web-app/src/api/config.ts
  - web-app/src/api/conversation.ts
  - web-app/src/store/slices/configSlice.ts
  - web-app/src/store/slices/onboardingSlice.ts
  - web-app/src/store/store.ts
  - web-app/src/components/onboarding/StepAgentSetup.tsx
  - web-app/src/components/onboarding/StepPlatformConfig.tsx
  - web-app/src/test/setup.ts
  - web-app/src/test/mocks/handlers.ts
  - web-app/.env.local
  - web-app/.env.production
  - web-app/vitest.config.ts
autonomous: true
---

# Phase 1 Integration: API Client + Redux + Forms Wiring

**Handoff from builder.io:** Complete Phase 1 UI (40+ components, 3 pages, Redux store structure)

**Integration work:** Wire UI to backend API, Redux state management, form validation, testing infrastructure

**Success Criteria:**
- ✅ API client can fetch agents from localhost:7777
- ✅ Redux actions dispatch and update state
- ✅ Forms submit and create/update data via API
- ✅ Error messages display clearly
- ✅ Redux state persists to localStorage
- ✅ All tests pass (pnpm test)
- ✅ No TypeScript errors
- ✅ Manual E2E: Complete onboarding wizard → config persists

---

## Task 1: Create API Client Layer

**Description:** Set up typed axios client with request/response interceptors

**Implementation:**

Create `web-app/src/api/client.ts`:
```typescript
import axios from 'axios'

const API_BASE = process.env.VITE_API_URL || 'http://localhost:7777'

const client = axios.create({
  baseURL: API_BASE,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
})

client.interceptors.response.use(
  (response) => response,
  (error) => {
    const message = error.response?.data?.message || error.message
    return Promise.reject(new Error(message))
  }
)

export default client
```

Create `web-app/src/api/config.ts` with Agent, Tool, Platform endpoints:
```typescript
import client from './client'

export interface Agent {
  id: string
  name: string
  model: string
  type: string
}

export const configAPI = {
  // Agents
  getAgents: () => client.get<Agent[]>('/api/config/agents'),
  createAgent: (data: Partial<Agent>) => client.post<Agent>('/api/config/agents', data),
  updateAgent: (id: string, data: Partial<Agent>) =>
    client.put<Agent>(`/api/config/agents/${id}`, data),
  deleteAgent: (id: string) => client.delete(`/api/config/agents/${id}`),

  // Tools
  getTools: () => client.get('/api/config/tools'),

  // Platforms
  getPlatforms: () => client.get('/api/config/platforms'),
  testPlatform: (platform: string, config: any) =>
    client.post(`/api/config/platforms/${platform}/test`, config),

  // Version
  getVersion: () => client.get<{ version: string }>('/api/config/version'),
}
```

Create `web-app/src/api/conversation.ts`:
```typescript
import client from './client'

export const conversationAPI = {
  startSession: () => client.post('/api/conversation/session'),
  sendMessage: (sessionId: string, message: string) =>
    client.post(`/api/conversation/session/${sessionId}/message`, { message }),
  confirmAgent: (sessionId: string, agentData: any) =>
    client.post(`/api/conversation/session/${sessionId}/confirm`, agentData),
}
```

---

## Task 2: Wire Redux to API Calls

**Description:** Add async thunks to Redux slices for CRUD operations

**Implementation:**

Modify `web-app/src/store/slices/configSlice.ts`:
```typescript
import { createAsyncThunk, createSlice } from '@reduxjs/toolkit'
import { configAPI } from '../../api/config'

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
  async (data: any, { rejectWithValue }) => {
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
  async ({ id, data }: { id: string; data: any }, { rejectWithValue }) => {
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

const configSlice = createSlice({
  name: 'config',
  initialState: {
    agents: [],
    tools: [],
    platforms: [],
    version: null,
    isLoading: false,
    error: null as string | null,
  },
  reducers: {},
  extraReducers: (builder) => {
    // Agents
    builder
      .addCase(fetchAgents.pending, (state) => {
        state.isLoading = true
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
      .addCase(updateAgent.fulfilled, (state, action) => {
        const index = state.agents.findIndex((a) => a.id === action.payload.id)
        if (index !== -1) {
          state.agents[index] = action.payload
        }
        state.isLoading = false
      })

    // Delete Agent
    builder
      .addCase(deleteAgent.fulfilled, (state, action) => {
        state.agents = state.agents.filter((a) => a.id !== action.payload)
        state.isLoading = false
      })

    // Tools
    builder
      .addCase(fetchTools.fulfilled, (state, action) => {
        state.tools = action.payload
        state.isLoading = false
      })

    // Platforms
    builder
      .addCase(fetchPlatforms.fulfilled, (state, action) => {
        state.platforms = action.payload
        state.isLoading = false
      })

    // Version
    builder
      .addCase(fetchVersion.fulfilled, (state, action) => {
        state.version = action.payload
        state.isLoading = false
      })
  },
})

export default configSlice.reducer
```

---

## Task 3: Implement Form Submission Logic

**Description:** Wire form components to Redux and API

**Implementation:**

Modify `web-app/src/components/onboarding/StepAgentSetup.tsx`:
```typescript
import { useForm } from 'react-hook-form'
import { useAppDispatch, useAppSelector } from '@/store/hooks'
import { updateAgent } from '@/store/slices/onboardingSlice'
import { createAgent } from '@/store/slices/configSlice'
import { Alert } from '@/components/common/Alert'
import { Button } from '@/components/common/Button'

const StepAgentSetup = () => {
  const dispatch = useAppDispatch()
  const { agent, isLoading, error } = useAppSelector((state) => state.onboarding)
  const { register, handleSubmit, formState: { errors } } = useForm({
    defaultValues: agent,
  })

  const onSubmit = async (data: any) => {
    dispatch(updateAgent(data))
    // On success, trigger creation via Redux thunk
    // dispatch(createAgent(data))
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <input
        {...register('name', { required: 'Name is required' })}
        placeholder="Agent name"
      />
      {errors.name && <span className="text-error">{errors.name.message}</span>}

      <button type="submit" disabled={isLoading}>
        {isLoading ? 'Creating...' : 'Next'}
      </button>

      {error && <Alert variant="error">{error}</Alert>}
    </form>
  )
}

export default StepAgentSetup
```

Apply same pattern to:
- `StepPlatformConfig.tsx` - Platform config form
- `StepWelcome.tsx` - Project setup form
- `StepReview.tsx` - Final review form
- Config dashboard modals (agent creation, platform testing)

---

## Task 4: WebSocket Setup (Phase 2 Placeholder)

**Description:** Create WebSocket client structure for Phase 2 real-time events

**Implementation:**

File `web-app/src/api/websocket.ts` already exists. Add placeholder for Phase 2:
```typescript
// Phase 2: Wire up WebSocket events
// For now, just set up the client structure

export interface WebSocketMessage {
  type: string
  data: any
}

export class WebSocketClient {
  private ws: WebSocket | null = null
  private url: string
  private reconnectAttempts = 0
  private maxReconnectAttempts = 20

  constructor(url: string) {
    this.url = url
  }

  connect() {
    // Phase 2 implementation
  }

  disconnect() {
    // Phase 2 implementation
  }

  subscribe(callback: (message: WebSocketMessage) => void) {
    // Phase 2 implementation
  }
}

export const wsClient = new WebSocketClient('ws://localhost:7777/ws')
```

---

## Task 5: Redux Persist Setup

**Description:** Configure Redux Persist for state persistence across sessions

**Implementation:**

Modify `web-app/src/store/store.ts`:
```typescript
import { configureStore } from '@reduxjs/toolkit'
import { persistStore, persistReducer } from 'redux-persist'
import storage from 'redux-persist/lib/storage'
import { appReducer } from './slices/appSlice'
import { configReducer } from './slices/configSlice'
import { onboardingReducer } from './slices/onboardingSlice'

const persistConfig = {
  key: 'aof-root',
  storage,
  whitelist: ['app', 'config'],
  version: 1,
}

const persistedAppReducer = persistReducer(persistConfig, appReducer)

export const store = configureStore({
  reducer: {
    app: persistedAppReducer,
    config: configReducer,
    onboarding: onboardingReducer,
  },
})

export const persistor = persistStore(store)

export type RootState = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch
```

Update `web-app/src/main.tsx`:
```typescript
import { PersistGate } from 'redux-persist/integration/react'
import { Provider } from 'react-redux'
import App from './App'
import { store, persistor } from './store/store'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <Provider store={store}>
      <PersistGate loading={<LoadingSpinner />} persistor={persistor}>
        <App />
      </PersistGate>
    </Provider>
  </React.StrictMode>
)
```

---

## Task 6: Environment Configuration

**Description:** Set up environment variables for dev and production

**Implementation:**

Already created:
- `web-app/.env.local` - Dev: `http://localhost:7777`
- `web-app/.env.production` - Prod: `https://aof-api.example.com`

Verify they're set correctly and update as needed.

---

## Task 7: Testing Infrastructure Setup

**Description:** Configure Vitest, MSW mocks, and test helpers

**Implementation:**

Create `web-app/vitest.config.ts`:
```typescript
import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
})
```

Create `web-app/src/test/setup.ts`:
```typescript
import { expect, afterEach, vi } from 'vitest'
import { cleanup } from '@testing-library/react'
import '@testing-library/jest-dom'
import { setupServer } from 'msw/node'
import { handlers } from './mocks/handlers'

afterEach(() => {
  cleanup()
})

export const server = setupServer(...handlers)

beforeAll(() => server.listen())
afterEach(() => server.resetHandlers())
afterAll(() => server.close())
```

Create `web-app/src/test/mocks/handlers.ts`:
```typescript
import { http, HttpResponse } from 'msw'

export const handlers = [
  http.get('http://localhost:7777/api/config/agents', () => {
    return HttpResponse.json([
      { id: '1', name: 'Test Agent', model: 'claude', type: 'analyst' },
    ])
  }),

  http.post('http://localhost:7777/api/config/agents', async ({ request }) => {
    const body = await request.json()
    return HttpResponse.json({ id: '2', ...body })
  }),

  http.get('http://localhost:7777/api/config/tools', () => {
    return HttpResponse.json([])
  }),

  http.get('http://localhost:7777/api/config/platforms', () => {
    return HttpResponse.json([])
  }),

  http.get('http://localhost:7777/api/config/version', () => {
    return HttpResponse.json({ version: '0.1.0' })
  }),
]
```

Create component tests: `web-app/src/components/common/Button.test.tsx`
```typescript
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, vi } from 'vitest'
import { Button } from './Button'

describe('Button', () => {
  it('renders with text', () => {
    render(<Button>Click me</Button>)
    expect(screen.getByText('Click me')).toBeInTheDocument()
  })

  it('calls onClick when clicked', async () => {
    const handleClick = vi.fn()
    const user = userEvent.setup()
    render(<Button onClick={handleClick}>Click</Button>)
    await user.click(screen.getByText('Click'))
    expect(handleClick).toHaveBeenCalled()
  })
})
```

---

## Task 8: Complete Phase 1 E2E Flow

**Description:** Validate end-to-end onboarding wizard integration

**Implementation:**

Create `web-app/src/test/e2e/onboarding.test.ts`:
```typescript
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { Provider } from 'react-redux'
import { setupServer } from 'msw/node'
import App from '@/App'
import { store } from '@/store/store'
import { handlers } from '../mocks/handlers'

const server = setupServer(...handlers)

beforeEach(() => {
  server.listen()
})

afterEach(() => {
  server.close()
})

describe('Complete Onboarding Flow', () => {
  it('walks through all 4 wizard steps', async () => {
    const user = userEvent.setup()

    render(
      <Provider store={store}>
        <App />
      </Provider>
    )

    // Step 1: Project setup
    expect(screen.getByText(/let's set up your project/i)).toBeInTheDocument()
    await user.type(screen.getByLabelText(/project name/i), 'My Project')
    await user.click(screen.getByRole('button', { name: /next/i }))

    // Step 2: Agent setup
    await waitFor(() => {
      expect(screen.getByText(/create your first agent/i)).toBeInTheDocument()
    })
    await user.type(screen.getByLabelText(/agent name/i), 'Analyzer')
    await user.click(screen.getByRole('button', { name: /next/i }))

    // Step 3: Platforms
    await waitFor(() => {
      expect(screen.getByText(/where should your agent listen/i)).toBeInTheDocument()
    })

    // Step 4: Review
    await waitFor(() => {
      expect(screen.getByText(/review and launch/i)).toBeInTheDocument()
    })
  })
})
```

---

## Verification Checklist

Before Phase 1 is complete:

- [ ] API client can fetch agents from `localhost:7777`
- [ ] Redux actions dispatch and update state correctly
- [ ] Forms submit and create/update data via API
- [ ] Error messages display clearly
- [ ] Redux state persists to localStorage
- [ ] Environment variables load from `.env.local`
- [ ] All tests pass: `pnpm test`
- [ ] No TypeScript errors: `pnpm type-check`
- [ ] Dev server runs: `pnpm dev`
- [ ] Build succeeds: `pnpm build`

**Manual Testing Steps:**
1. Start AOF daemon: `aofctl serve`
2. Run web app: `cd web-app && pnpm dev`
3. Browser: `localhost:5173`
4. Complete onboarding wizard
5. Verify config persists after refresh
6. Navigate to Config dashboard
7. Create a new agent via UI
8. Verify it appears in agent list

---

## Dependencies

- ✅ Builder.io Phase 1 UI delivery (PR #102)
- ✅ Backend API running on localhost:7777
- ✅ Redux store structure in place
- ✅ Component library (40+ components)

## Next Phase

Once Phase 1 integration is complete and tested:
- Phase 2: Mission Control + Squad Chat UI
- Phase 3: Real-time agent coordination (WebSocket)
- Phase 4: Deployment & hardening
