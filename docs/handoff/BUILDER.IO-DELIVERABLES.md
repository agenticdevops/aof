# Builder.io Deliverables Specification

**What Claude expects back from Builder.io at the end of each phase**

---

## Phase 1 Deliverables (End of Week 1)

### Directory Structure

```
web-app/
├── src/
│   ├── main.tsx                    # Vite entry point
│   ├── App.tsx                     # Root component with routing
│   ├── types/
│   │   ├── index.ts                # All TypeScript interfaces
│   │   ├── agent.ts                # Agent-related types
│   │   ├── config.ts               # Configuration types
│   │   └── api.ts                  # API response types
│   ├── pages/
│   │   ├── WelcomePage.tsx         # Welcome/home page
│   │   └── OnboardingWizard.tsx    # 4-step wizard
│   │   └── ConfigurationPage.tsx   # Config dashboard (Agents/Tools/Platforms)
│   ├── components/
│   │   ├── common/
│   │   │   ├── Button.tsx
│   │   │   ├── Input.tsx
│   │   │   ├── TextArea.tsx
│   │   │   ├── Select.tsx
│   │   │   ├── Radio.tsx
│   │   │   ├── Card.tsx
│   │   │   ├── Modal.tsx
│   │   │   ├── Badge.tsx
│   │   │   ├── SearchBar.tsx
│   │   │   ├── EmptyState.tsx
│   │   │   ├── LoadingSpinner.tsx
│   │   │   ├── ConfirmDialog.tsx
│   │   │   └── FormField.tsx
│   │   ├── layout/
│   │   │   └── Layout.tsx          # App shell/navigation
│   │   ├── onboarding/
│   │   │   ├── WizardProgress.tsx
│   │   │   ├── StepWelcome.tsx
│   │   │   ├── StepAgentSetup.tsx
│   │   │   ├── StepPlatformConfig.tsx
│   │   │   └── StepReview.tsx
│   │   ├── config/
│   │   │   ├── TabNavigation.tsx
│   │   │   ├── AgentsTab.tsx
│   │   │   ├── AgentCard.tsx
│   │   │   ├── AgentDetailModal.tsx
│   │   │   ├── ToolsTab.tsx
│   │   │   ├── ToolCard.tsx
│   │   │   ├── PlatformsTab.tsx
│   │   │   ├── PlatformCard.tsx
│   │   │   └── PlatformDetailModal.tsx
│   ├── store/
│   │   ├── store.ts                # Redux store configuration
│   │   ├── hooks.ts                # useAppDispatch, useAppSelector
│   │   └── slices/
│   │       ├── appSlice.ts         # Navigation, theme, etc.
│   │       ├── onboardingSlice.ts  # Wizard state
│   │       └── configSlice.ts      # Config dashboard state
│   ├── styles/
│   │   └── globals.css             # Global styles (Tailwind imports)
│   ├── vite-env.d.ts               # Vite type definitions
│   └── index.html                  # HTML template
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.ts
├── postcss.config.js
├── tailwind.css
├── .eslintrc.json
├── .prettierrc.json
└── README.md
```

### Required Files

#### 1. **package.json**

Must include (exact versions or compatible):

```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.20.0",
    "redux": "^4.2.0",
    "@reduxjs/toolkit": "^1.9.0",
    "react-redux": "^8.1.0",
    "redux-persist": "^6.0.0",
    "react-hook-form": "^7.48.0",
    "@hookform/resolvers": "^3.3.0",
    "zod": "^3.22.0",
    "axios": "^1.6.0",
    "clsx": "^2.0.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "@types/node": "^20.0.0",
    "typescript": "^5.3.0",
    "vite": "^5.0.0",
    "@vitejs/plugin-react": "^4.2.0",
    "tailwindcss": "^3.3.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0",
    "eslint": "^8.50.0",
    "prettier": "^3.0.0"
  }
}
```

#### 2. **tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "strict": true,
    "esModuleInterop": true,
    "resolveJsonModule": true,
    "moduleResolution": "bundler",
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

#### 3. **vite.config.ts**

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:7777',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://localhost:7777',
        ws: true,
      },
    },
  },
})
```

#### 4. **tailwind.config.ts**

Must include design tokens:

```typescript
export default {
  content: ['./index.html', './src/**/*.{js,ts,jsx,tsx}'],
  theme: {
    colors: {
      success: '#10b981',
      error: '#ef4444',
      info: '#3b82f6',
      neutral: '#6b7280',
      'bg-light': '#ffffff',
      'bg-dark': '#1f2937',
    },
    fontSize: {
      display: '32px',
      heading: '20px',
      body: '14px',
      mono: '12px',
    },
    spacing: {
      xs: '4px',
      s: '8px',
      m: '16px',
      l: '24px',
      xl: '32px',
      xxl: '48px',
    },
    extend: {},
  },
  plugins: [],
}
```

#### 5. **Redux Store Files**

**store/store.ts:**
```typescript
import { configureStore } from '@reduxjs/toolkit'
import { persistStore, persistReducer } from 'redux-persist'
import storage from 'redux-persist/lib/storage'
import appReducer from './slices/appSlice'
import onboardingReducer from './slices/onboardingSlice'
import configReducer from './slices/configSlice'

const persistConfig = {
  key: 'root',
  storage,
  whitelist: ['app', 'config'],
}

const persistedAppReducer = persistReducer(persistConfig, appReducer)

export const store = configureStore({
  reducer: {
    app: persistedAppReducer,
    onboarding: onboardingReducer,
    config: configReducer,
  },
})

export const persistor = persistStore(store)

export type RootState = ReturnType<typeof store.getState>
export type AppDispatch = typeof store.dispatch
```

**store/slices/appSlice.ts:**
```typescript
import { createSlice, PayloadAction } from '@reduxjs/toolkit'

interface AppState {
  navigation: 'welcome' | 'wizard' | 'config' | 'missionControl'
  theme: 'light' | 'dark'
  firstVisit: boolean
  daemonUrl: string
}

const initialState: AppState = {
  navigation: 'welcome',
  theme: 'light',
  firstVisit: true,
  daemonUrl: 'http://localhost:7777',
}

const appSlice = createSlice({
  name: 'app',
  initialState,
  reducers: {
    setNavigation: (state, action: PayloadAction<AppState['navigation']>) => {
      state.navigation = action.payload
    },
    setTheme: (state, action: PayloadAction<'light' | 'dark'>) => {
      state.theme = action.payload
    },
    setFirstVisit: (state, action: PayloadAction<boolean>) => {
      state.firstVisit = action.payload
    },
  },
})

export const { setNavigation, setTheme, setFirstVisit } = appSlice.actions
export default appSlice.reducer
```

**store/slices/onboardingSlice.ts:**
```typescript
import { createSlice, PayloadAction } from '@reduxjs/toolkit'

interface OnboardingState {
  currentStep: 1 | 2 | 3 | 4
  project: { name: string; description: string }
  agent: {
    name: string
    model: string
    type: 'analyst' | 'coordinator' | 'specialist'
    instructions: string
    capabilities: string[]
  }
  platforms: Record<string, { connected: boolean; config: any }>
  isLoading: boolean
  error: string | null
}

const initialState: OnboardingState = {
  currentStep: 1,
  project: { name: '', description: '' },
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
}

const onboardingSlice = createSlice({
  name: 'onboarding',
  initialState,
  reducers: {
    setStep: (state, action: PayloadAction<1 | 2 | 3 | 4>) => {
      state.currentStep = action.payload
    },
    updateProject: (state, action: PayloadAction<Partial<OnboardingState['project']>>) => {
      state.project = { ...state.project, ...action.payload }
    },
    updateAgent: (state, action: PayloadAction<Partial<OnboardingState['agent']>>) => {
      state.agent = { ...state.agent, ...action.payload }
    },
    updatePlatforms: (state, action: PayloadAction<OnboardingState['platforms']>) => {
      state.platforms = action.payload
    },
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
    },
    reset: () => initialState,
  },
})

export const {
  setStep,
  updateProject,
  updateAgent,
  updatePlatforms,
  setLoading,
  setError,
  reset,
} = onboardingSlice.actions
export default onboardingSlice.reducer
```

**store/slices/configSlice.ts:**
```typescript
import { createSlice, PayloadAction } from '@reduxjs/toolkit'

interface Agent {
  id: string
  name: string
  model: string
  type: string
  instructions: string
  capabilities: string[]
}

interface Tool {
  id: string
  name: string
  description: string
  type: string
  status: 'ready' | 'needs-config'
}

interface Platform {
  id: string
  name: string
  icon: string
  connected: boolean
  config?: any
}

interface ConfigState {
  agents: Agent[]
  tools: Tool[]
  platforms: Platform[]
  version: string
  isLoading: boolean
  error: string | null
  searchQuery: string
}

const initialState: ConfigState = {
  agents: [],
  tools: [],
  platforms: [],
  version: '',
  isLoading: false,
  error: null,
  searchQuery: '',
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
    updateAgent: (state, action: PayloadAction<Agent>) => {
      const index = state.agents.findIndex((a) => a.id === action.payload.id)
      if (index >= 0) state.agents[index] = action.payload
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
    setSearchQuery: (state, action: PayloadAction<string>) => {
      state.searchQuery = action.payload
    },
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload
    },
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
    },
  },
})

export const {
  setAgents,
  addAgent,
  updateAgent,
  removeAgent,
  setTools,
  setPlatforms,
  setSearchQuery,
  setLoading,
  setError,
} = configSlice.actions
export default configSlice.reducer
```

### Component Specifications

Every component must have:
1. **Props interface** — Clearly typed
2. **Default export** — Named export optional
3. **PropTypes or TypeScript** — Full coverage
4. **Accessibility** — ARIA labels, semantic HTML
5. **Variants** — All variants in Storybook stories

### Example Component Structure

```typescript
// Button.tsx
import React from 'react'
import clsx from 'clsx'

interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
  loading?: boolean
  disabled?: boolean
  children: React.ReactNode
  onClick?: (e: React.MouseEvent<HTMLButtonElement>) => void
  className?: string
  type?: 'button' | 'submit' | 'reset'
}

export const Button: React.FC<ButtonProps> = ({
  variant = 'primary',
  size = 'md',
  loading = false,
  disabled = false,
  children,
  onClick,
  className,
  type = 'button',
}) => {
  return (
    <button
      type={type}
      disabled={disabled || loading}
      onClick={onClick}
      className={clsx(
        'font-medium rounded transition-colors',
        {
          'bg-success text-white hover:bg-success/90': variant === 'primary',
          'bg-neutral/10 text-neutral hover:bg-neutral/20': variant === 'secondary',
          'text-neutral hover:bg-neutral/5': variant === 'ghost',
          'px-4 py-2': size === 'md',
          'px-3 py-1 text-sm': size === 'sm',
          'px-6 py-3 text-lg': size === 'lg',
          'opacity-50 cursor-not-allowed': disabled || loading,
        },
        className
      )}
    >
      {loading ? '⏳ Loading...' : children}
    </button>
  )
}

export default Button
```

### Storybook Stories

For Phase 1, create `.stories.tsx` files for all components:

```typescript
// Button.stories.tsx
import type { Meta, StoryObj } from '@storybook/react'
import { Button } from './Button'

const meta: Meta<typeof Button> = {
  component: Button,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['primary', 'secondary', 'ghost'],
    },
    size: {
      control: 'select',
      options: ['sm', 'md', 'lg'],
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Primary: Story = {
  args: {
    children: 'Click me',
    variant: 'primary',
  },
}

export const Secondary: Story = {
  args: {
    children: 'Click me',
    variant: 'secondary',
  },
}

export const Ghost: Story = {
  args: {
    children: 'Click me',
    variant: 'ghost',
  },
}

export const Loading: Story = {
  args: {
    children: 'Submit',
    loading: true,
    variant: 'primary',
  },
}

export const Disabled: Story = {
  args: {
    children: 'Disabled',
    disabled: true,
  },
}
```

### Documentation

**README.md:**
```markdown
# AOF Web Application — Phase 1

Frontend for Agentic Ops Framework built with React 18 + TypeScript.

## Getting Started

```bash
pnpm install
pnpm dev
```

## Project Structure

- `src/pages/` — Page components (Welcome, Wizard, Config)
- `src/components/` — Reusable components
- `src/store/` — Redux slices and hooks
- `src/types/` — TypeScript interfaces

## Components

- Button, Input, TextArea, Select, Radio
- Card, Modal, Badge, SearchBar
- EmptyState, LoadingSpinner, FormField

See Storybook for interactive documentation.

## Running Storybook

```bash
pnpm storybook
```

## API Integration

API endpoints are proxied to http://localhost:7777.

See `/docs/api/COMPLETE-API-SPECIFICATION.md` for endpoint documentation.
```

### Git Repository

Create a branch and push:

```bash
git checkout -b phase-1-builder-io
git add .
git commit -m "feat: Phase 1 - Welcome, onboarding wizard, config dashboard"
git push -u origin phase-1-builder-io
```

Then create a PR with:
- Description of what was built
- List of components created
- Known limitations or deviations from spec
- Any questions for Claude

---

## Quality Checklist

Before delivering Phase 1, verify:

- [ ] All 30+ components created and typed
- [ ] All pages route correctly (Welcome → Wizard → Config)
- [ ] Design tokens from Tailwind config match spec exactly
- [ ] Forms have inline validation
- [ ] Loading states on all async operations
- [ ] Mobile responsive (320px, 768px, 1440px tested)
- [ ] Dark mode works (theme toggle in header)
- [ ] No TypeScript errors (`tsc --noEmit`)
- [ ] No console warnings
- [ ] Storybook runs and shows all component variants
- [ ] README documents how to run and structure
- [ ] Git history is clean (one commit or logical commits)
- [ ] PR has clear description and screenshots

---

## Next: Handoff to Claude

Once Phase 1 is delivered:

1. **Claude will:**
   - Wire up Redux to API endpoints
   - Implement WebSocket client
   - Add API error handling and retries
   - Implement form submission logic
   - Add persistent storage (Redux Persist)
   - Set up testing (Vitest + React Testing Library)
   - Optimize performance

2. **You'll then build:**
   - Phase 2: Mission Control + Squad Chat
   - Phase 3: Fleet Control Dashboard
   - Phase 4: Polish & Animations

3. **Loop back:** Feedback → Iterate → Deliver → Integrate

---

## Contact & Questions

Questions about spec? Found ambiguities? Create an issue or comment on the PR.

Claude will review and provide clarification.

Good luck! 🚀
