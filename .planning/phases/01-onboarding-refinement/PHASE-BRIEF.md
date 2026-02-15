# Phase 1.5: Onboarding Refinement

**Goal:** Redesign onboarding wizard based on real user mental model (channels → tools → agents → launch)

**Duration:** 1 week (research + planning + execution)

**User Feedback Input:**
- Project/agent creation friction → unnecessary
- Users want: channels first, then default coordinator (Xops), then tools
- Xops as single interface in channels, coordinates fleet behind scenes
- Local tools auto-detection with install capability
- Predefined agent fleet + conversational agent creation

**Key Decisions Needed (Research Phase):**

1. **Agent Fleet Architecture**
   - One Xops in channel + internal agent coordination?
   - Multiple agents in same channel (how to distinguish)?
   - Agent-to-channel binding model (direct vs coordinator-routed)?
   - Scalability limits and patterns

2. **Real Use Cases**
   - DevOps squad example
   - Support bot fleet example
   - Research/analysis example
   - What agents? How many? What roles?

3. **Agent Templates**
   - What starter packs to ship? (DevOps, Support, Research, Custom)
   - How opinionated vs flexible?
   - Conversational agent creation vs templates?

4. **Local Tools Strategy**
   - Auto-scan on startup or manual trigger?
   - Install capability (docker run, pkg mgr, etc.)?
   - Tool discovery service?

5. **Onboarding UX Changes**
   - New flow: Channels → AI Model → Tools → Review & Launch Xops
   - Xops persona (pre-defined, editable later)
   - When to present agent fleet options?
   - Progressive vs upfront configuration?

**Deliverables:**
- RESEARCH.md → Architecture patterns, use cases, scalability analysis
- PLAN.md → New wizard flow, agent management patterns, local tools strategy
- Execute → Updated wizard, Xops auto-creation, tool detection, agent templates
- Verify → User can launch with Xops in <3 minutes, add agents from config dashboard

**Blockers/Risks:**
- Channel integration complexity (Slack API, Telegram API, etc.)
- Agent routing at scale (1 Xops ↔ 100 agents = ??)
- Tool installation across environments (Docker, K8s, local)

**Next Action:**
1. Research agent fleet architectures
2. Document 3+ real use cases with agent requirements
3. Analyze scalability constraints
4. Plan wizard UX changes
5. Execute implementation
