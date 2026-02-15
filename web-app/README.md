# AOF Web Application — Phase 1 Complete

Frontend for **Agentic Ops Framework** built with React 18 + TypeScript + Tailwind CSS.

## Overview

Beautiful, responsive web dashboard for configuring and managing AI agents. Features dark mode support, real-time state management with Redux, and a complete onboarding wizard.

## 🚀 Quick Start

### Prerequisites
- Node.js >= 18.0.0
- pnpm >= 8.0.0 (or npm/yarn)

### Installation

```bash
cd web-app
pnpm install
```

### Development Server

```bash
pnpm dev
```

Server runs on `http://localhost:5173` with API proxy to `http://localhost:7777`

### Build for Production

```bash
pnpm build
```

### Run Storybook

```bash
pnpm storybook
```

View component library at `http://localhost:6006`

## 📁 Project Structure

```
web-app/
├── src/
│   ├── main.tsx                    # React entry point
│   ├── App.tsx                     # Root component with routing
│   ├── index.css                   # Global styles (Tailwind imports)
│   ├── vite-env.d.ts              # Vite type definitions
│   │
│   ├── types/
│   │   └── index.ts                # All TypeScript interfaces
│   │
│   ├── pages/
│   │   ├── WelcomePage.tsx         # Welcome/home page
│   │   ├── OnboardingWizard.tsx    # 4-step onboarding wizard
│   │   └── ConfigurationPage.tsx   # Config dashboard (Agents/Tools/Platforms)
│   │
│   ├── components/
│   │   ├── common/
│   │   │   ├── Button.tsx
│   │   │   ├── Input.tsx
│   │   │   ├── TextArea.tsx
│   │   │   ├── Select.tsx
│   │   │   ├── Card.tsx
│   │   │   ├── Modal.tsx
│   │   │   ├── Badge.tsx
│   │   │   ├── SearchBar.tsx
│   │   │   ├── EmptyState.tsx
│   │   │   ├── LoadingSpinner.tsx
│   │   │   ├── ConfirmDialog.tsx
│   │   │   ├── Checkbox.tsx
│   │   │   ├── FormField.tsx
│   │   │   ├── Button.stories.tsx
│   │   │   └── Badge.stories.tsx
│   │   │
│   │   ├── onboarding/
│   │   │   ├── WizardProgress.tsx
│   │   │   ├── StepWelcome.tsx
│   │   │   ├── StepAgentSetup.tsx
│   │   │   ├── StepPlatformConfig.tsx
│   │   │   └── StepReview.tsx
│   │   │
│   │   ├── config/
│   │   │   ├── TabNavigation.tsx
│   │   │   ├── AgentCard.tsx
│   │   │   └── PlatformCard.tsx
│   │   │
│   │   └── layout/
│   │       └── Layout.tsx           # App shell with header/theme toggle
│   │
│   └── store/
│       ├── store.ts                 # Redux store configuration
│       ├── hooks.ts                 # useAppDispatch, useAppSelector
│       ├── index.ts                 # Exports
│       └── slices/
│           ├── appSlice.ts         # Navigation, theme state
│           ├── onboardingSlice.ts  # Wizard state
│           └── configSlice.ts      # Config dashboard state
│
├── public/                          # Static assets
├── index.html                       # HTML template
├── package.json                     # Dependencies + scripts
├── tsconfig.json                    # TypeScript configuration
├── vite.config.ts                   # Vite build configuration
├── tailwind.config.ts               # Tailwind CSS config with design tokens
├── postcss.config.js                # PostCSS configuration
├── .eslintrc.json                   # ESLint configuration
├── .prettierrc.json                 # Prettier configuration
├── .env                             # Environment variables
├── .env.production                  # Production env variables
└── README.md                        # This file
```

## 🎨 Design System

### Colors
- **Primary (Success):** Emerald 500 (`#22c55e`)
- **Error:** Red 500 (`#ef4444`)
- **Info:** Blue 500 (`#3b82f6`)
- **Warning:** Yellow 500 (`#eab308`)
- **Neutral:** Gray 600 (`#4b5563`)

### Typography
- **Display:** 32px, weight 700, line-height 1.2
- **Heading:** 20px, weight 600, line-height 1.3
- **Body:** 14px, weight 400, line-height 1.5
- **Mono:** 12px (ui-monospace), weight 500

### Spacing Scale
- **XS:** 4px
- **S:** 8px
- **M:** 16px
- **L:** 24px
- **XL:** 32px
- **XXL:** 48px

## 🌙 Dark Mode

All components support both light and dark modes. Theme is persisted to localStorage via Redux Persist.

**Toggle theme:** Click moon/sun icon in header

## 📦 Component Library

### Common Components (13 total)

| Component | Props | Usage |
|-----------|-------|-------|
| Button | variant, size, loading, disabled, fullWidth, onClick | Primary actions, forms |
| Input | label, error, disabled, placeholder, value, onChange | Text input fields |
| TextArea | label, error, rows, placeholder, value, onChange | Multi-line text input |
| Select | label, options, value, onChange, placeholder | Dropdown selections |
| Card | elevation, clickable, hoverable, onClick | Content containers |
| Modal | isOpen, onClose, title, size, footer | Dialogs and modals |
| Badge | variant, size, icon | Status indicators |
| SearchBar | value, onChange, placeholder, fullWidth | Search and filtering |
| EmptyState | icon, title, description, action | Empty state messages |
| LoadingSpinner | size, fullPage, text | Loading indicators |
| ConfirmDialog | isOpen, title, message, onConfirm, onCancel | Confirmation dialogs |
| Checkbox | label, checked, onChange, disabled | Checkbox input |
| FormField | label, error, required, helperText | Form field wrapper |

### Page Components (3 total)

1. **WelcomePage**
   - Hero section with feature cards
   - "Begin Setup" call-to-action
   - Responsive mobile-first design

2. **OnboardingWizard**
   - 4-step setup flow
   - Progress indicator with checkmarks
   - Smart validation and error handling

3. **ConfigurationPage**
   - Tabbed interface (Agents, Tools, Platforms)
   - Agent/Platform card grids
   - Search, create, edit, delete operations

### Onboarding Components (5 total)

- **WizardProgress:** Step indicator with connections
- **StepWelcome:** Project name and description
- **StepAgentSetup:** Agent configuration with capabilities
- **StepPlatformConfig:** Platform connection modal
- **StepReview:** Configuration summary with expand/collapse

## 🏪 Redux Store

### Store Shape

```typescript
{
  app: {
    navigation: 'welcome' | 'wizard' | 'config' | 'missionControl',
    theme: 'light' | 'dark',
    firstVisit: boolean,
    daemonUrl: string,
  },
  onboarding: {
    currentStep: 1 | 2 | 3 | 4,
    project: { name: string, description: string },
    agent: {
      name: string,
      model: string,
      type: 'analyst' | 'coordinator' | 'specialist',
      instructions: string,
      capabilities: string[],
    },
    platforms: Record<string, Platform>,
    isLoading: boolean,
    error: string | null,
    completedSteps: Set<number>,
  },
  config: {
    agents: Agent[],
    tools: Tool[],
    platforms: Platform[],
    searchQuery: string,
    isLoading: boolean,
    error: string | null,
    selectedAgent: Agent | null,
    selectedPlatform: Platform | null,
  },
}
```

### Persistence

- **Persisted slices:** `app`, `config`
- **Storage:** localStorage
- **Key:** `aof-root`
- **Versioning:** 1

State persists across browser sessions automatically via Redux Persist.

## 🧪 Testing

### Run Tests

```bash
# Run all tests
pnpm test

# Watch mode
pnpm test --watch

# UI mode
pnpm test:ui

# Coverage
pnpm test:coverage
```

### Test Setup
- **Framework:** Vitest
- **Library:** React Testing Library
- **Mocking:** MSW (Mock Service Worker)

## 🔍 Linting & Formatting

```bash
# Lint code
pnpm lint

# Fix linting issues
pnpm lint:fix

# Format code
pnpm format

# Type check
pnpm type-check
```

## 🌐 Environment Variables

### Development (`.env`)
```
VITE_API_URL=http://localhost:7777
VITE_WS_URL=ws://localhost:7777
```

### Production (`.env.production`)
```
VITE_API_URL=https://api.aof.sh
VITE_WS_URL=wss://api.aof.sh
```

## 📱 Responsive Breakpoints

- **Mobile:** < 640px (full-width)
- **Tablet:** 640px - 1024px (2-column grids)
- **Desktop:** > 1024px (3+ column grids)

All components tested and optimized for mobile-first design.

## 🚀 Building & Deployment

### Development Build

```bash
pnpm dev
```

Runs on `http://localhost:5173` with hot reload and API proxy.

### Production Build

```bash
pnpm build
```

Creates optimized production build in `dist/` directory.

### Preview Production Build

```bash
pnpm preview
```

Locally preview production build on `http://localhost:4173`.

## 📚 Component Variants

Each component has multiple variants demonstrated in Storybook:

- Button: primary, secondary, ghost, danger (all sizes and states)
- Badge: success, error, warning, info, neutral (with icons)
- Input: normal, error, disabled, with helper text
- Select: normal, disabled, with options
- Card: flat, lifted, focused (with hover effects)
- Modal: small, medium, large (with/without close button)

Run `pnpm storybook` to view all component variants interactively.

## 📖 Documentation

- **API Specification:** `/docs/api/COMPLETE-API-SPECIFICATION.md`
- **Frontend Spec:** `/docs/frontend/WEB-APP-SPECIFICATION.md`
- **Handoff Guide:** `/docs/handoff/BUILDER.IO-DELIVERABLES.md`

## 🔄 State Flow

1. **User lands on welcome page**
2. **Clicks "Begin Setup" → navigates to wizard**
3. **Completes 4 steps → state saved in Redux**
4. **Submits → config sent to backend API**
5. **Redirected to config dashboard**
6. **State persists on refresh via Redux Persist**
7. **Dark mode preference persists**

## 🎯 Key Features

✅ **Complete onboarding flow** — 4-step guided wizard
✅ **Configuration dashboard** — Manage agents, tools, platforms
✅ **Dark mode** — Full support with toggle and persistence
✅ **Responsive design** — Mobile, tablet, desktop optimized
✅ **Form validation** — Real-time error display
✅ **State management** — Redux Toolkit with persistence
✅ **Type safety** — Full TypeScript coverage
✅ **Accessibility** — Semantic HTML, ARIA labels
✅ **Component library** — 13+ reusable components
✅ **Storybook** — Interactive component documentation

## 🔧 Tech Stack

- **React 18** — UI library
- **TypeScript 5** — Type safety
- **Vite 5** — Build tool
- **Tailwind CSS 3** — Styling
- **Redux Toolkit** — State management
- **Redux Persist** — Local storage persistence
- **React Router 6** — Client routing
- **Vitest** — Testing
- **Storybook 7** — Component docs

## 📝 Component Development Guidelines

### Creating New Components

1. **Define types** in `src/types/index.ts`
2. **Create component** in appropriate folder (common, onboarding, config)
3. **Add PropTypes** or TypeScript interfaces
4. **Support dark mode** (use `dark:` classes)
5. **Make it responsive** (mobile-first)
6. **Add Storybook stories** (component.stories.tsx)
7. **Add tests** (component.test.tsx) — optional for Phase 1

### Styling Guidelines

- **Use Tailwind classes** — no CSS modules or styled-components
- **Support dark mode** — add `dark:` variants
- **Use design tokens** — colors from tailwind config
- **Consistent spacing** — use spacing scale (xs, s, m, l, xl, xxl)
- **Hover/focus states** — for all interactive elements

### Redux Integration

- **Create slice** in `store/slices/`
- **Define reducers** and async thunks
- **Use hooks** — `useAppDispatch()`, `useAppSelector()`
- **Type actions** — full TypeScript support
- **Persist if needed** — add to `persistConfig.whitelist`

## 🎓 Learning Resources

- **Tailwind Docs:** https://tailwindcss.com/docs
- **Redux Docs:** https://redux.js.org/
- **React Router:** https://reactrouter.com/
- **Storybook:** https://storybook.js.org/

## 📞 Support

Questions? Check the docs or create an issue on GitHub.

---

**Built with ❤️ for AOF Phase 1**

Ready for Claude integration. API client setup coming next! 🚀
