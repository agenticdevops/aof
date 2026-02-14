# Developer Documentation Guide

**For AOF Internal Developer Docs**

This guide explains how to maintain, update, and properly wire the developer documentation.

---

## Overview

AOF's developer documentation is organized in `/docs/dev/` with:
- **INDEX.md** - Central entry point and navigation hub
- **ARCHITECTURE.md** - Core architecture and crate structure
- **Phase-specific docs** - Implementation guides for each phase
- **Subsystem docs** - Tools, skills, incident response, etc.
- **sidebar.js** - Docusaurus sidebar configuration
- **docusaurus.config.example.js** - Template for Docusaurus setup

**Total Coverage:** 20+ markdown files covering 6+ completed phases

---

## Documentation Structure

### By Phase

Each completed phase has 1-3 dedicated docs:

| Phase | Status | Documentation |
|-------|--------|-----------------|
| Phase 1 | ✅ Complete | `event-infrastructure.md` |
| Phase 2 | ✅ Complete | `skills-platform.md`, `incident-response.md` |
| Phase 3 | ✅ Complete | `docs/internal/design/` (3 files) |
| Phase 4 | ✅ Complete | `persona-system.md`, `persona-loaders.md`, `persona-ui-components.md` |
| Phase 5 | ✅ Complete | (Persona docs from Phase 4) |
| Phase 6 | ✅ Complete | `PHASE-6-IMPLEMENTATION-SUMMARY.md`, `conversational-architecture.md`, `conversation-api.md`, `squad-templates.md`, `agent-generation-pipeline.md` |
| Phase 7 | ⏰ Planning | (To be created) |
| Phase 8 | ⏰ Planning | (To be created) |

### By Type

**Architecture Documents:**
- `ARCHITECTURE.md` - Main architecture (all crates)
- `AGENTFLOW_DESIGN.md` - Workflow DAGs
- Phase 6 arch in `conversational-architecture.md`

**Subsystem Documents:**
- `persona-system.md` - Agent personas (SOUL.md)
- `skills-platform.md` - Skill definitions
- `event-infrastructure.md` - WebSocket, events
- `sandbox-isolation.md` - Execution isolation
- `resource-locking.md` - Concurrency management
- `decision-logging.md` - Audit trails

**API Reference:**
- `conversation-api.md` - REST API endpoints

**Contributing Guides:**
- `CONTRIBUTING.md` - Submission guidelines
- `TOOLS_DEVELOPMENT.md` - Adding new tools

---

## Updating Documentation for Phase Changes

**When you execute a phase (via `/gsd:execute-phase`), follow this process:**

### Step 1: Verify or Create Phase Documentation

**Before execution:**
1. Check if phase docs exist in `/docs/dev/`
2. If not, create `phase-X-overview.md` or update existing docs
3. Ensure documentation reflects the architecture and requirements

**Example:** Phase 6 has 5 dedicated files:
- `PHASE-6-IMPLEMENTATION-SUMMARY.md` (overview)
- `conversational-architecture.md` (detailed architecture)
- `conversation-api.md` (REST API reference)
- `squad-templates.md` (feature-specific)
- `agent-generation-pipeline.md` (feature-specific)

### Step 2: Update ARCHITECTURE.md

After phase execution, add a section to `ARCHITECTURE.md` describing:
- The new crate (if applicable)
- Key types and traits
- Architecture diagram
- Links to phase-specific docs

**Example (Phase 6):**
```markdown
## Phase 6: Conversational Configuration (aof-conversational)

### Overview
Phase 6 adds conversational interface...

### Architecture
[Diagram showing intent classification → orchestrator → specialists]

### Related Documentation
- [PHASE-6-IMPLEMENTATION-SUMMARY.md](...)
- [conversational-architecture.md](...)
```

### Step 3: Update INDEX.md

Add phase to the "Phase Implementation Guides" section:

```markdown
### Phase 6: Conversational Configuration ✅
- **[PHASE-6-IMPLEMENTATION-SUMMARY.md](...)**
- **[conversational-architecture.md](...)**
- **[conversation-api.md](...)**
```

### Step 4: Update sidebar.js

Add phase section to the sidebar configuration:

```javascript
{
  type: 'category',
  label: 'Phase 6: Conversational Configuration ✅',
  items: [
    'PHASE-6-IMPLEMENTATION-SUMMARY',
    'conversational-architecture',
    'conversation-api',
    'squad-templates',
    'agent-generation-pipeline',
  ],
},
```

### Step 5: Cross-Reference

In phase-specific docs, link to:
- `ARCHITECTURE.md` section
- `INDEX.md` navigation
- Related subsystems (e.g., persona-system for Phase 6)
- User-facing docs (e.g., `/docs/features/conversational-interface.md`)

**Example in `conversational-architecture.md`:**
```markdown
## Related Documentation
- [ARCHITECTURE.md - Phase 6 Section](./ARCHITECTURE.md#phase-6-conversational-configuration)
- [INDEX.md Navigation](./INDEX.md)
- [Persona System](./persona-system.md)
```

---

## Creating Phase Documentation

When starting a new phase, create a template file:

```markdown
# Phase X: [Phase Name] - Implementation Summary

**Status:** ✅ COMPLETE / ⏰ Planning
**Completion Date:** YYYY-MM-DD
**Total Tasks:** N
**Tests Passing:** N/M

## Phase Overview
[One paragraph describing the phase goal and deliverables]

## Plans Executed
[List each plan with duration, tasks, deliverables]

### Plan X-01: [Name]
**Duration:** XXXs | **Tasks:** N/N | **Files:** N

**Deliverables:**
- [Bullet points]

## Key Design Decisions
[Explain architectural choices]

## Testing Coverage
- [List of test areas]

## Related Documentation
- [Links to detailed docs]

## Next Steps
- [What comes next]
```

---

## Docusaurus Integration

### Quick Setup

1. **Install Docusaurus** (if not already done):
   ```bash
   cd docs
   npm install -D docusaurus@latest @docusaurus/core @docusaurus/preset-classic
   ```

2. **Copy docusaurus.config.js**:
   ```bash
   cp docs/dev/docusaurus.config.example.js docs/docusaurus.config.js
   # Edit to match your setup
   ```

3. **Use sidebar.js**:
   ```bash
   cp docs/dev/sidebar.js docs/sidebars.js
   ```

4. **Run documentation server**:
   ```bash
   cd docs
   npm run start
   ```

5. **Build static site**:
   ```bash
   npm run build
   ```

### File Structure

```
docs/
├── docusaurus.config.js          # Main config (copy from example)
├── sidebars.js                   # Sidebar structure
├── dev/
│   ├── INDEX.md                  # Start here
│   ├── ARCHITECTURE.md            # Core architecture
│   ├── PHASE-6-IMPLEMENTATION-SUMMARY.md
│   ├── conversational-architecture.md
│   ├── ... (20+ more docs)
│   ├── sidebar.js                # (Optional - dev-specific sidebar)
│   └── docusaurus.config.example.js
├── features/                     # User-facing features
├── tutorials/                    # User guides
└── internal/                     # Internal design docs
```

### Sidebar Configuration

The sidebar maps documentation hierarchy. Update `docs/sidebars.js`:

```javascript
module.exports = {
  developers: [
    {
      type: 'doc',
      id: 'dev/INDEX',
      label: '📚 Developer Docs Index',
    },
    {
      type: 'category',
      label: 'Phase 6: Conversational Config ✅',
      items: [
        'dev/PHASE-6-IMPLEMENTATION-SUMMARY',
        'dev/conversational-architecture',
        'dev/conversation-api',
      ],
    },
    // ... more phases
  ],
};
```

### Markdown Frontmatter

Add to each doc for better organization:

```markdown
---
sidebar_position: 1
title: Developer Documentation Index
description: Central index for AOF internal development docs
---

# Developer Documentation Index
...
```

---

## Best Practices

### 1. Keep INDEX.md Updated
- Add new phases as they complete
- Update metrics (% complete, plan counts)
- Maintain reading paths as documentation grows

### 2. Link Liberally
- Use relative links: `[Phase 6](./conversational-architecture.md)`
- Cross-reference related docs
- Link from ARCHITECTURE.md to phase-specific docs
- Link from phase docs back to ARCHITECTURE.md

### 3. Maintain Consistency
- Use same structure for all phase summaries
- Use same table formats
- Consistent emoji usage (✅ Complete, ⏰ Planning, 🔄 In Progress)
- Keep sidebar structure parallel to documentation organization

### 4. Update Last-Updated Dates
- Top of each doc: `**Last Updated:** YYYY-MM-DD`
- Especially important for ARCHITECTURE.md and INDEX.md

### 5. Document Decisions
- Every phase summary should have "Key Design Decisions" section
- Explain the rationale, not just the what
- Note alternatives considered

### 6. Include Test Coverage
- Every phase doc should mention test counts
- Link to test patterns used (e.g., MockModel for Phase 6)
- Call out integration test scenarios

### 7. Provide Examples
- Code examples for architecture patterns
- Step-by-step testing guides (like Phase 6's manual E2E test)
- Diagrams for complex flows

---

## Documentation Checklist for Phase Execution

When you execute a phase via `/gsd:execute-phase X`, ensure:

- [ ] Phase-specific documentation exists in `/docs/dev/`
  - [ ] Overview/summary document created
  - [ ] Feature-specific docs created (if applicable)

- [ ] ARCHITECTURE.md updated
  - [ ] New crate section added (if applicable)
  - [ ] Links to phase docs included
  - [ ] Phase 6 conversational section already included as example

- [ ] INDEX.md updated
  - [ ] Phase added to "Phase Implementation Guides"
  - [ ] Links to all phase docs included
  - [ ] Phase status (✅ Complete) updated

- [ ] sidebar.js updated
  - [ ] Phase category added
  - [ ] All phase docs referenced

- [ ] Phase docs cross-linked
  - [ ] Phase docs link to ARCHITECTURE.md
  - [ ] Phase docs link to INDEX.md
  - [ ] Phase docs link to related subsystems

- [ ] Last-updated dates
  - [ ] ARCHITECTURE.md updated
  - [ ] INDEX.md updated
  - [ ] Phase summaries dated

---

## Example: Phase 6 Documentation

Phase 6 demonstrates all best practices:

**Files Created:**
1. `PHASE-6-IMPLEMENTATION-SUMMARY.md` - Overview of all 5 plans
2. `conversational-architecture.md` - Detailed technical architecture
3. `conversation-api.md` - REST API reference with testing guide
4. `squad-templates.md` - Squad template system
5. `agent-generation-pipeline.md` - Agent creation pipeline

**Integration Points:**
- ARCHITECTURE.md: Added "Phase 6" section (400+ lines)
- INDEX.md: Added Phase 6 to navigation, reading paths, quick reference
- sidebar.js: Added Phase 6 category with 5 docs

**Cross-References:**
- All 5 Phase 6 docs link to each other
- Phase 6 docs link back to ARCHITECTURE.md
- ARCHITECTURE.md Phase 6 section links to all 5 Phase 6 docs
- INDEX.md has Phase 6 in "By Feature" section

**Documentation Coverage:**
- Intent classification: 4 intents, confidence thresholds
- Session management: LRU cache design, TTL behavior
- File generation: Atomic writes, validation
- REST API: 5 endpoints, manual testing steps
- UI: React components, Redux integration
- Testing: 47+ unit tests, MockModel pattern

---

## Troubleshooting

### Documentation Not Appearing in Docusaurus

**Check:**
1. File is in correct directory (`/docs/dev/` for dev docs)
2. Filename is referenced in sidebar.js
3. Markdown syntax is valid (run `npm run build`)
4. No special characters in filenames (use hyphens, not spaces)

### Links Not Working

**Check:**
1. Use relative paths: `./filename.md` or `../path/filename.md`
2. Include `.md` extension in links
3. Verify target file exists and is spelled correctly
4. Test locally before deploying: `npm run start`

### Sidebar Not Updating

**Check:**
1. Edit `docs/sidebars.js` (not `docs/dev/sidebar.js`)
2. Restart Docusaurus: `npm run start`
3. Check for JSON syntax errors in sidebars.js
4. Verify document ID matches filename (without .md)

---

## Maintenance Schedule

**After each phase execution:**
- [ ] Create phase summary (1-2 hours)
- [ ] Update ARCHITECTURE.md (~30 minutes)
- [ ] Update INDEX.md (~15 minutes)
- [ ] Update sidebar.js (~5 minutes)
- [ ] Test locally: `npm run start` (~2 minutes)

**Weekly:**
- [ ] Review INDEX.md for accuracy
- [ ] Check for broken links
- [ ] Update "Last Updated" dates if changes made

**Monthly:**
- [ ] Review all phase docs for consistency
- [ ] Update metrics (% complete, task counts)
- [ ] Verify sidebar structure matches documentation

---

## Contributing Documentation

To contribute developer docs:

1. Create a feature branch
2. Add/update markdown files in `/docs/dev/`
3. Update ARCHITECTURE.md if adding a new subsystem
4. Update INDEX.md if adding a new phase or major feature
5. Update sidebar.js if adding new sections
6. Test locally: `npm run start`
7. Submit PR with documentation changes

See [CONTRIBUTING.md](./CONTRIBUTING.md) for full contribution guidelines.

---

**Last Updated:** 2026-02-14
**Status:** Phase 6 Complete | Docusaurus Configuration Available
**Next Steps:** Phase 7 planning begins (Coordination Protocols)
