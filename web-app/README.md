# AOF Web App

Modern React 18 + TypeScript web application for AOF (Agentic Ops Framework).

## Quick Start

### Prerequisites
- Node.js 18+
- pnpm 8+

### Installation

```bash
pnpm install
```

### Development

```bash
pnpm dev
```

Starts dev server at http://localhost:5173

### Build

```bash
pnpm build
```

Creates optimized production build in `dist/`

### Preview

```bash
pnpm preview
```

Preview production build locally

## Project Structure

```
src/
├── components/     # Reusable UI components
├── pages/         # Page components
├── store/         # Redux state management
├── types/         # TypeScript type definitions
├── utils/         # Utility functions
├── services/      # API clients and services
└── App.tsx        # Root component
```

## Tech Stack

- **Framework**: React 18
- **Language**: TypeScript
- **Build**: Vite
- **Styling**: Tailwind CSS
- **State**: Redux Toolkit + Redux Persist
- **Routing**: React Router v6
- **Forms**: React Hook Form + Zod validation
- **Testing**: Vitest + React Testing Library
- **Documentation**: Storybook
- **APIs**: Axios
- **Real-time**: WebSocket

## API Configuration

- **Dev API**: http://localhost:7777
- **Dev WebSocket**: ws://localhost:7777

Set custom URLs in `.env.local`:
```env
VITE_API_URL=http://localhost:7777
VITE_WS_URL=ws://localhost:7777
```

## Testing

```bash
# Run tests
pnpm test

# Run tests with UI
pnpm test:ui

# Coverage
pnpm test:coverage
```

## Linting & Formatting

```bash
# Lint
pnpm lint

# Fix linting issues
pnpm lint:fix

# Format code
pnpm format
```

## Storybook

```bash
# Start Storybook
pnpm storybook

# Build Storybook
pnpm build-storybook
```

## Design System

### Colors
- **Success**: #10b981 (green)
- **Error**: #ef4444 (red)
- **Info**: #3b82f6 (blue)
- **Warning**: #f59e0b (amber)
- **Neutral**: #6b7280 (gray)

See `tailwind.config.ts` for complete design tokens.

## License

Apache-2.0
