# Milestone 2, Phase 1: Onboarding & Configuration UI

## Phase Goal
Users can set up AOF in 5 minutes with no YAML editing.

## Duration
1 week

## Dependencies  
- Phase 7 & 8 from Milestone 1 complete (API ready) ✅
- COMPLETE-API-SPECIFICATION.md available ✅
- WEB-APP-SPECIFICATION.md available ✅

## Frontend Specifications Available
- Complete API contracts in COMPLETE-API-SPECIFICATION.md (20+ endpoints)
- Full frontend spec in WEB-APP-SPECIFICATION.md (React 18, TypeScript, Redux, WebSocket)
- Design system: Colors (green #10b981, red #ef4444, blue #3b82f6), Typography, Spacing
- Components specified: 30+ components with detailed specifications
- Polling strategy: Metrics every 30s via REST, health/standup via WebSocket
- Builder.io integration checklist with 15+ items

## Key Requirements
1. Welcome page with setup flow
2. 4-step onboarding wizard (account, agent, platforms, review)
3. Conversational agent creation UI
4. Agent management dashboard (CRUD agents)
5. Platform configuration (connect Slack, Discord, etc.)
6. Tool discovery and management

## Success Criteria
1. First-time user can complete onboarding in <5 minutes
2. All form inputs validate with clear error messages
3. Configuration persists and survives daemon restart
4. Users can modify configuration after initial setup
5. Platform connections test successfully

## Approach
- React 18 with TypeScript, Redux Toolkit state management
- Vite build system
- Tailwind CSS for styling (design system predefined)
- Builder.io-ready component architecture
- Full integration with /api/config/* endpoints
- WebSocket connection health indicator
- 4 plans: Welcome+wizard, Config dashboard, Form validation, E2E integration testing
