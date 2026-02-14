# Claude Integration Handoff — What Claude Needs to Take Over

**Document for Claude (myself) — what to expect from builder.io and what to implement next**

---

## After Phase 1 Delivery from Builder.io

Builder.io will provide:

### 1. Git Repository
- **Branch:** `phase-1-builder-io`
- **Commit:** Organized commits with clear messages
- **PR:** Against `main` with description and screenshots
- **Files:** Complete React application at `/web-app`

### 2. Key Artifacts to Extract

#### Folder: `web-app/src/components/`
- 30+ component files (Button, Input, Modal, Card, etc.)
- All typed with TypeScript
- All have variant examples

**Check:**
```bash
ls -la web-app/src/components/common/ | wc -l  # Should be ~13 files
find web-app/src -name "*.tsx" | wc -l        # Should be ~70+ files
```

#### Folder: `web-app/src/store/`
- `store.ts` — Redux configuration
- `hooks.ts` — useAppDispatch, useAppSelector
- `slices/appSlice.ts` — Navigation, theme
- `slices/onboardingSlice.ts` — Wizard state
- `slices/configSlice.ts` — Config dashboard state

**Check:**
```bash
ls -la web-app/src/store/slices/  # Should have 3 slices
```

#### Files: `package.json`, `tsconfig.json`, `vite.config.ts`, `tailwind.config.ts`
- All configured and ready
- Proxies to `localhost:7777` for API + WebSocket

#### Documentation:
- `README.md` — How to run dev server
- Storybook stories for all components
- Component inventory (CSV or JSON)

---

## Claude's Integration Tasks

### Phase 1 Integration (Days 1-3 after handoff)

#### Task 1: Set Up API Client Layer

**Goal:** Create typed API client that wraps fetch/axios

**Create:** `web-app/src/api/client.ts`

```typescript
import axios from 'axios'

const API_BASE = process.env.REACT_APP_API_URL || 'http://localhost:7777'

const client = axios.create({
  baseURL: API_BASE,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Add request/response interceptors
client.interceptors.response.use(
  (response) => response,
  (error) => {
    // Normalize error responses
    const message = error.response?.data?.message || error.message
    return Promise.reject(new Error(message))
  }
)

export default client
```

**Create:** `web-app/src/api/config.ts`

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

**Create:** `web-app/src/api/conversation.ts`

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

#### Task 2: Wire Redux to API Calls

**Goal:** Update Redux slices to use API client

**Modify:** `web-app/src/store/slices/configSlice.ts`

Add async thunks:

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

// In slice.extraReducers:
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
```

**Implementation:**
- Add async thunks for all CRUD operations (fetchAgents, createAgent, updateAgent, deleteAgent, fetchTools, fetchPlatforms, testPlatform, fetchVersion)
- Wire them up in `extraReducers`
- Update component dispatch calls to use thunks

---

#### Task 3: Implement Form Submission Logic

**Goal:** Wire form submissions to Redux + API

**Modify:** `web-app/src/components/onboarding/StepAgentSetup.tsx`

```typescript
const StepAgentSetup = () => {
  const dispatch = useAppDispatch()
  const { agent, isLoading, error } = useAppSelector((state) => state.onboarding)
  const { register, handleSubmit, formState: { errors } } = useForm({
    defaultValues: agent,
  })

  const onSubmit = async (data: any) => {
    dispatch(updateAgent(data))
    // Form validation happens here
  }

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      {/* Form fields */}
      <button type="submit" disabled={isLoading}>
        {isLoading ? 'Creating...' : 'Next'}
      </button>
      {error && <Alert variant="error">{error}</Alert>}
    </form>
  )
}
```

**Implementation:**
- All 6 forms (onboarding 4 steps + 2 config modals) should wire to Redux
- Add error handling with Alert/Toast components
- Add loading states on buttons
- Validate using Zod schemas

---

#### Task 4: WebSocket Setup (Placeholder for Phase 2)

**Create:** `web-app/src/api/websocket.ts`

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

#### Task 5: Redux Persist Setup

**Modify:** `web-app/src/store/store.ts`

```typescript
import { persistStore, persistReducer } from 'redux-persist'
import storage from 'redux-persist/lib/storage'
import { appReducer } from './slices/appSlice'
import { configReducer } from './slices/configSlice'

const persistConfig = {
  key: 'aof-root',
  storage,
  whitelist: ['app', 'config'], // Persist these slices
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
```

**In App.tsx:**
```typescript
import { PersistGate } from 'redux-persist/integration/react'

function App() {
  return (
    <Provider store={store}>
      <PersistGate loading={<LoadingSpinner />} persistor={persistor}>
        <Router />
      </PersistGate>
    </Provider>
  )
}
```

---

#### Task 6: Environment Configuration

**Create:** `.env.local`
```
VITE_API_URL=http://localhost:7777
VITE_WS_URL=ws://localhost:7777
```

**Create:** `.env.production`
```
VITE_API_URL=https://aof-api.example.com
VITE_WS_URL=wss://aof-api.example.com
```

---

#### Task 7: Testing Setup (Unit + Integration)

**Create:** `vitest.config.ts`
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

**Create:** `src/test/setup.ts`
```typescript
import { expect, afterEach, vi } from 'vitest'
import { cleanup } from '@testing-library/react'
import '@testing-library/jest-dom'

afterEach(() => {
  cleanup()
})

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})
```

**Create MSW Mock Server:** `src/test/mocks/handlers.ts`
```typescript
import { http, HttpResponse } from 'msw'

export const handlers = [
  // Agents
  http.get('http://localhost:7777/api/config/agents', () => {
    return HttpResponse.json([
      {
        id: '1',
        name: 'Test Agent',
        model: 'claude',
        type: 'analyst',
      },
    ])
  }),

  http.post('http://localhost:7777/api/config/agents', async ({ request }) => {
    const body = await request.json()
    return HttpResponse.json({
      id: '2',
      ...body,
    })
  }),

  // Add more handlers for each endpoint
]
```

**Create test examples:** `src/components/common/Button.test.tsx`
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

  it('disables when loading', () => {
    render(<Button loading>Submit</Button>)
    expect(screen.getByRole('button')).toBeDisabled()
  })
})
```

**package.json scripts:**
```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest --coverage",
    "type-check": "tsc --noEmit",
    "lint": "eslint src",
    "format": "prettier --write src"
  }
}
```

---

#### Task 8: Complete Phase 1 E2E Flow

**Create:** `src/test/e2e/onboarding.test.ts`

```typescript
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, beforeEach } from 'vitest'
import { Provider } from 'react-redux'
import { setupServer } from 'msw/node'
import App from '@/App'
import { store } from '@/store/store'
import { handlers } from './mocks/handlers'

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

### Integration Checklist (Before Phase 2)

After implementing Tasks 1-8, verify:

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
2. Run web app: `pnpm dev`
3. Browser: `localhost:5173`
4. Complete onboarding wizard
5. Verify config persists after refresh
6. Check browser DevTools → Application → localStorage (should have Redux state)
7. Navigate to Config dashboard
8. Create a new agent via UI
9. Verify it appears in agent list

---

## Phase 1 → Phase 2 Handoff

Once Phase 1 is complete and integrated:

1. **Create PR:** Integration work merged to `main`
2. **Builder.io builds Phase 2:** Mission Control + Squad Chat pages
3. **Claude integrates Phase 2:** WebSocket setup, real-time events
4. **Repeat for Phase 3 & 4**

---

## Troubleshooting Guide

### "Cannot find module '@/api/config'"

**Solution:** Verify `vite.config.ts` has alias:
```typescript
alias: {
  '@': path.resolve(__dirname, './src'),
}
```

### "API calls return 404"

**Solution:** Verify daemon is running:
```bash
# In another terminal:
aofctl serve
# Should see: "WebSocket server listening on 7777"
```

### "Redux state not persisting"

**Solution:** Verify Redux Persist setup in `store.ts`:
```typescript
const persistConfig = {
  key: 'aof-root',
  storage,
  whitelist: ['app', 'config'],
}
```

### "TypeScript errors in tests"

**Solution:** Install types:
```bash
pnpm add -D @testing-library/react @types/jest vitest
```

---

## Performance Optimization (After Phase 1)

Once integrated, optimize:

1. **Code splitting:** Lazy load phase 2 & 3 pages
2. **Component memoization:** Use `React.memo()` for expensive components
3. **Redux selectors:** Use `reselect` for derived state
4. **Bundle size:** Analyze with `vite-plugin-visualizer`

---

## Next: Begin Phase 2

Once Phase 1 ✅:

```bash
# Create phase 2 branch
git checkout -b phase-2-mission-control

# Start implementing:
# - Mission Control dashboard page
# - Squad Chat component
# - Real-time agent health
# - Standup results feed
```

See: `.planning/phases/02-mission-control-squad-chat/` for Phase 2 plans.

---

**Ready to integrate! 🚀**
