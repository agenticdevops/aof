# Builder.io Project Setup Configuration

**Copy-paste this information into builder.io's "Setup Your Project" form**

---

## Step 1: App Root Folder

```
web-app
```

(Or if creating fresh: create a `/web-app` folder at the repository root)

---

## Step 2: Environment Variables

Add these two environment variables:

### Variable 1
- **Name:** `VITE_API_URL`
- **Value:** `http://localhost:7777`
- **Description:** API server URL (dev)

### Variable 2
- **Name:** `VITE_WS_URL`
- **Value:** `ws://localhost:7777`
- **Description:** WebSocket server URL (dev)

### Production Variables (for later)
- **Name:** `VITE_API_URL`
- **Value:** `https://aof-api.example.com`

- **Name:** `VITE_WS_URL`
- **Value:** `wss://aof-api.example.com`

---

## Step 3: Installation Commands

```bash
pnpm install
```

**Alternative (if using npm):**
```bash
npm install
```

**Alternative (if using yarn):**
```bash
yarn install
```

---

## Step 4: Dev Server Commands

```bash
pnpm dev
```

**This will:**
- Start Vite dev server on `http://localhost:5173`
- Proxy API calls to `http://localhost:7777`
- Proxy WebSocket to `ws://localhost:7777`
- Hot reload on file changes

---

## Step 5: Optional Tools

Select/Install:

### ✅ TypeScript
- Status: **Required**
- Version: `^5.3.0`
- Already configured in `tsconfig.json`

### ✅ ESLint
- Status: **Recommended**
- Install: `pnpm add -D eslint`
- Config: `.eslintrc.json` (template provided in DELIVERABLES.md)

### ✅ Prettier
- Status: **Recommended**
- Install: `pnpm add -D prettier`
- Config: `.prettierrc.json` (template provided)

### ✅ Storybook
- Status: **Required for Phase 1**
- Install: `pnpm add -D @storybook/react @storybook/addon-docs`
- Command: `pnpm storybook`

### ✅ Tailwind CSS
- Status: **Required**
- Already configured in `tailwind.config.ts`
- Version: `^3.3.0`

### ✅ React Router
- Status: **Required**
- Version: `^6.20.0`
- For routing between pages

### ✅ Redux Toolkit + Redux Persist
- Status: **Required**
- Redux: `^4.2.0`
- Redux Toolkit: `^1.9.0`
- Redux Persist: `^6.0.0`
- react-redux: `^8.1.0`

### ✅ Testing Libraries
- Status: **Required**
- Vitest: `^1.0.0`
- @testing-library/react: `^14.0.0`
- @testing-library/user-event: `^14.5.0`
- msw: `^2.0.0` (Mock Service Worker)

---

## Complete package.json Dependencies

Share this with builder.io:

```json
{
  "name": "aof-web",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "test": "vitest",
    "test:ui": "vitest --ui",
    "test:coverage": "vitest --coverage",
    "type-check": "tsc --noEmit",
    "lint": "eslint src",
    "format": "prettier --write src",
    "storybook": "storybook dev -p 6006",
    "build-storybook": "storybook build"
  },
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
    "@eslint/js": "^8.50.0",
    "eslint-plugin-react": "^7.33.0",
    "prettier": "^3.0.0",
    "vitest": "^1.0.0",
    "@testing-library/react": "^14.0.0",
    "@testing-library/user-event": "^14.5.0",
    "@testing-library/jest-dom": "^6.1.0",
    "msw": "^2.0.0",
    "@storybook/react": "^7.6.0",
    "@storybook/addon-docs": "^7.6.0",
    "@storybook/blocks": "^7.6.0"
  }
}
```

---

## Proxy Configuration (in vite.config.ts)

```typescript
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
}
```

---

## Before Starting Dev Server

1. **AOF Backend must be running:**
   ```bash
   aofctl serve
   # Should see: WebSocket server listening on 7777
   ```

2. **Then start the dev server:**
   ```bash
   pnpm dev
   # Should see: http://localhost:5173
   ```

3. **Browser:**
   - Visit `http://localhost:5173`
   - Should see Welcome page

---

## Storybook Setup

After components are built, run:

```bash
pnpm storybook
# Opens: http://localhost:6006
```

All components should have `.stories.tsx` files showing variants.

---

## Environment Setup Summary

| Item | Dev Value | Production Value |
|------|-----------|------------------|
| API URL | `http://localhost:7777` | `https://aof-api.example.com` |
| WS URL | `ws://localhost:7777` | `wss://aof-api.example.com` |
| Dev Server | `http://localhost:5173` | Built via `pnpm build` |
| Package Manager | pnpm | pnpm (recommended) |
| Node Version | 18+ | 18+ |

---

## Quick Start Commands (for builder.io to use)

```bash
# Install dependencies
pnpm install

# Start dev server (requires aofctl serve running)
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview

# Run tests
pnpm test

# Run tests with UI
pnpm test:ui

# Start Storybook
pnpm storybook

# Type check
pnpm type-check

# Lint code
pnpm lint

# Format code
pnpm format
```

---

## Notes for Builder.io

1. **Don't commit node_modules** — add to .gitignore
2. **Keep .env.local out of git** — it's local dev config only
3. **Component tests** — write .test.tsx alongside components
4. **Storybook stories** — write .stories.tsx alongside components
5. **Type safety** — strict TypeScript mode enabled
6. **Responsive design** — test at 320px, 768px, 1440px
7. **Dark mode** — all components should support both themes

---

## GitHub Integration

```bash
# After creating PR:
git checkout -b phase-1-builder-io
git add .
git commit -m "feat: Phase 1 - Welcome, wizard, config dashboard"
git push -u origin phase-1-builder-io

# Create PR on GitHub
# Reference: /docs/handoff/BUILDER.IO-DELIVERABLES.md for requirements
```

---

## Support

If builder.io has questions:
1. Check `/docs/frontend/WEB-APP-SPECIFICATION.md` (UI spec)
2. Check `/docs/api/COMPLETE-API-SPECIFICATION.md` (API spec)
3. Check `/docs/handoff/BUILDER.IO-BRIEF.md` (project overview)
4. Create a GitHub issue or PR comment

Claude will respond within 24 hours.

---

**You're all set! Ready to build.** 🚀
