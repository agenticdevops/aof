---
phase: 01-onboarding-refinement
plan: 03
type: execute
wave: 3
depends_on: ["01-onboarding-refinement-01"]
files_modified:
  - web-app/src/types/agents.ts
  - web-app/src/data/botTemplates.ts
  - web-app/src/components/config/BotTemplateSelector.tsx
  - web-app/src/components/config/SquadCompositionPanel.tsx
  - web-app/src/components/config/SquadCompositionUI.tsx
  - web-app/src/store/slices/configSlice.ts
  - web-app/src/api/config.ts
  - web-app/src/test/e2e/bot-templates.test.tsx

autonomous: true

must_haves:
  truths:
    - "3 specialist bot templates available (Kubernetes Ops, Infrastructure, SRE Observability)"
    - "Each template includes pre-configured agents with specific personas and skills"
    - "Users can select template to create squad of agents"
    - "Squad Composition UI shows agents in team and their roles"
    - "Users can customize agents in squad (add/remove/modify)"
    - "Squad configuration persists and can be loaded on daemon restart"
    - "Each template has clear description, use cases, and success metrics"
    - "Agents in squad properly coordinate with Xops orchestrator"
    - "Web UI allows viewing and modifying squad composition"
    - "All specialist bots created with proper personas from SOUL.md"

  artifacts:
    - path: "web-app/src/data/botTemplates.ts"
      provides: "BotTemplate definitions for 3 specialist squads"
      exports: "BotTemplate type, 3 template instances, template utilities"
    - path: "web-app/src/components/config/SquadCompositionUI.tsx"
      provides: "Component showing agents in squad with roles and capabilities"
      exports: "SquadCompositionUI component"
    - path: "web-app/src/api/config.ts"
      provides: "API methods for creating agents from templates"
      exports: "createAgentFromTemplate, getSquadConfig, updateSquadConfig"

  key_links:
    - from: "botTemplates.ts"
      to: "BotTemplateSelector.tsx"
      via: "Component displays templates and calls createAgentFromTemplate"
      pattern: "botTemplates.map.*template"
    - from: "BotTemplateSelector.tsx"
      to: "configAPI.createAgentFromTemplate()"
      via: "User selects template → API creates agents"
      pattern: "dispatch\\(createAgentFromTemplate"
    - from: "SquadCompositionUI.tsx"
      to: "Redux configSlice"
      via: "Shows agents from Redux state"
      pattern: "useAppSelector.*agents"

---

<objective>
Create 3 specialist bot templates (Kubernetes Ops, Infrastructure Automation, SRE Observability) that users can select during onboarding or from the configuration dashboard. Each template includes pre-configured agents with specific personas, skills, and coordination patterns designed for real DevOps use cases.

**Purpose:** Accelerate user productivity by providing battle-tested agent configurations for common DevOps scenarios. Users get working squads immediately instead of building from scratch.

**Output:**
- 3 specialist bot templates with pre-configured agents and personas
- BotTemplateSelector component for choosing templates
- SquadCompositionUI for viewing and editing team members
- API methods for creating agents from templates
- Persistent squad configuration
</objective>

<execution_context>
@/Users/gshah/.claude/get-shit-done/workflows/execute-plan.md
@/Users/gshah/.claude/get-shit-done/templates/summary.md
@/Users/gshah/work/opsflow-sh/aof/.planning/phases/01-onboarding-refinement/RESEARCH-SUMMARY.md
@/Users/gshah/work/opsflow-sh/aof/.planning/phases/01-onboarding-refinement/RESEARCH.md
</execution_context>

<context>
@/Users/gshah/work/opsflow-sh/aof/.planning/PROJECT.md
@/Users/gshah/work/opsflow-sh/aof/.planning/STATE.md
@/Users/gshah/work/opsflow-sh/aof/.planning/phases/01-onboarding-config-ui/01-INTEGRATION-SUMMARY.md
@/Users/gshah/work/opsflow-sh/aof/.planning/phases/01-onboarding-refinement/01-PLAN.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Define bot template types and data structures</name>
  <files>
    web-app/src/types/agents.ts
    web-app/src/data/botTemplates.ts
  </files>
  <action>
Create comprehensive types and data structures for specialist bot templates:

1. **Update agents.ts with template types:**
   ```typescript
   // Agent types
   export type AgentRole = "orchestrator" | "specialist" | "assistant"
   export type SkillCategory = "incident-response" | "deployment" | "provisioning" | "monitoring" | "optimization"

   export interface AgentSkill {
     id: string
     name: string
     description: string
     tools: string[] // Tool IDs: ["kubectl", "terraform", etc]
     category: SkillCategory
     confidence: number // 0.0 - 1.0
   }

   export interface BotTemplate {
     id: string
     name: string
     description: string
     icon: string
     category: "kubernetes" | "infrastructure" | "observability" | "custom"
     useCase: string
     agents: TemplateAgent[]
     coordinationPattern: "hub-and-spoke" | "peer-to-peer" | "hierarchical"
     requiredTools: string[]
     estimatedSetupTime: number // minutes
     successMetrics: SuccessMetric[]
     tags: string[]
   }

   export interface TemplateAgent {
     id: string
     name: string
     role: AgentRole
     persona: {
       personality: string
       traits: string[]
       communication: string
       boundaries: string[]
     }
     skills: AgentSkill[]
     responsibilities: string[]
     coordinationWith: string[] // Agent IDs this agent coordinates with
   }

   export interface SuccessMetric {
     name: string
     description: string
     target: string // "MTTR < 5 min", "100% uptime", etc
   }

   export interface SquadConfig {
     id: string
     name: string
     templateId: string
     agents: SquadAgent[]
     createdAt: Date
     lastModified: Date
   }

   export interface SquadAgent {
     id: string
     agentId: string
     name: string
     role: AgentRole
     status: "active" | "inactive" | "error"
     persona: AgentPersona
     skills: AgentSkill[]
   }
   ```

2. **Create botTemplates.ts with 3 specialist templates:**
   ```typescript
   export const KUBERNETES_OPS_TEMPLATE: BotTemplate = {
     id: "kubernetes-ops-squad",
     name: "Kubernetes Ops Squad",
     description: "Incident response and deployment automation for Kubernetes clusters",
     icon: "⚙️",
     category: "kubernetes",
     useCase: "Production Kubernetes cluster management with <5 min incident response",
     agents: [
       {
         id: "incident-commander",
         name: "Incident Commander",
         role: "specialist",
         persona: {
           personality: "Decisive, calm under pressure, orchestrates response",
           traits: ["analytical", "leadership", "decisive"],
           communication: "Clear commands, escalation alerts, status updates",
           boundaries: ["No auto-remediation without approval", "Escalates after 10min"]
         },
         skills: [
           {
             id: "triage",
             name: "Incident Triage",
             description: "Classify incident severity and assign responders",
             tools: ["kubectl", "prometheus"],
             category: "incident-response",
             confidence: 0.95
           },
           {
             id: "escalation",
             name: "Escalation Management",
             description: "Route to on-call based on incident type",
             tools: [],
             category: "incident-response",
             confidence: 0.90
           }
         ],
         responsibilities: [
           "Detect incidents from alerts",
           "Triage and assess impact",
           "Coordinate specialist agents",
           "Escalate if needed"
         ],
         coordinationWith: ["log-analyzer", "metrics-checker", "k8s-diagnostician"]
       },
       {
         id: "log-analyzer",
         name: "Log Analyzer",
         role: "specialist",
         persona: {
           personality: "Detail-oriented, finds patterns in chaos",
           traits: ["analytical", "methodical"],
           communication: "Structured findings, evidence-based conclusions",
           boundaries: ["No data modification", "Privacy aware"]
         },
         skills: [
           {
             id: "log-analysis",
             name: "Log Aggregation & Analysis",
             description: "Search and analyze logs from Loki/ELK",
             tools: ["loki", "shell"],
             category: "incident-response",
             confidence: 0.93
           }
         ],
         responsibilities: [
           "Query logs for incident context",
           "Find error patterns",
           "Extract stack traces"
         ],
         coordinationWith: ["incident-commander", "metrics-checker"]
       },
       // ... more agents
     ],
     coordinationPattern: "hub-and-spoke",
     requiredTools: ["kubectl", "prometheus", "loki"],
     estimatedSetupTime: 5,
     successMetrics: [
       { name: "MTTR", description: "Mean time to resolution", target: "< 5 minutes" },
       { name: "Detection Accuracy", description: "Correct incident classification", target: "> 90%" },
       { name: "False Positive Rate", description: "Unnecessary escalations", target: "< 5%" }
     ],
     tags: ["kubernetes", "incident-response", "production"]
   }

   export const INFRASTRUCTURE_TEMPLATE: BotTemplate = {
     id: "infrastructure-squad",
     name: "Infrastructure Automation Squad",
     description: "IaC provisioning, cost optimization, and compliance management",
     icon: "🏗️",
     category: "infrastructure",
     useCase: "Terraform provisioning and AWS resource management",
     agents: [
       {
         id: "iac-provisioner",
         name: "IaC Provisioner",
         role: "specialist",
         // ... persona and skills
       },
       {
         id: "cost-optimizer",
         name: "Cost Optimizer",
         role: "specialist",
         // ... persona and skills
       },
       // ... more agents
     ],
     coordinationPattern: "hub-and-spoke",
     requiredTools: ["terraform", "aws-cli"],
     estimatedSetupTime: 10,
     successMetrics: [
       { name: "Provisioning Time", description: "Time to deploy infrastructure", target: "< 20 minutes" },
       { name: "Cost Savings", description: "Annual cost optimization", target: "> $10k identified" },
       { name: "Success Rate", description: "Successful deployments", target: "> 95%" }
     ],
     tags: ["infrastructure", "terraform", "aws"]
   }

   export const SRE_OBSERVABILITY_TEMPLATE: BotTemplate = {
     id: "sre-observability-squad",
     name: "SRE Observability Squad",
     description: "Continuous health monitoring, anomaly detection, and trend analysis",
     icon: "📊",
     category: "observability",
     useCase: "Daily health checks and anomaly detection for production systems",
     agents: [
       {
         id: "health-monitor",
         name: "Health Monitor",
         role: "specialist",
         // ... persona and skills
       },
       {
         id: "anomaly-detector",
         name: "Anomaly Detector",
         role: "specialist",
         // ... persona and skills
       },
       // ... more agents
     ],
     coordinationPattern: "peer-to-peer",
     requiredTools: ["prometheus", "kubectl"],
     estimatedSetupTime: 8,
     successMetrics: [
       { name: "Detection Latency", description: "Time to detect anomalies", target: "< 2 minutes" },
       { name: "Uptime", description: "System uptime", target: "99.9%" },
       { name: "False Positive Rate", description: "Unnecessary alerts", target: "< 10%" }
     ],
     tags: ["observability", "monitoring", "srp"]
   }

   // Export all templates as array for easy iteration
   export const BOT_TEMPLATES: BotTemplate[] = [
     KUBERNETES_OPS_TEMPLATE,
     INFRASTRUCTURE_TEMPLATE,
     SRE_OBSERVABILITY_TEMPLATE,
   ]

   // Utility functions
   export const getTemplateById = (id: string): BotTemplate | undefined => {
     return BOT_TEMPLATES.find(t => t.id === id)
   }

   export const getTemplatesByCategory = (category: string): BotTemplate[] => {
     return BOT_TEMPLATES.filter(t => t.category === category)
   }

   export const getRequiredToolsForTemplate = (templateId: string): string[] => {
     const template = getTemplateById(templateId)
     return template?.requiredTools || []
   }
   ```

3. **Export complete types:**
   - All types properly documented with JSDoc
   - No any types (strict typing)
   - Compatible with Redux and API serialization

Ensure types match backend agent/template schema if backend has one. Otherwise, these types define the contract.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm type-check
Should show 0 TypeScript errors.

Also verify template structure:
- 3 templates defined (Kubernetes, Infrastructure, SRE)
- Each template has agents array with personas and skills
- Success metrics defined for each
- All required fields present
  </verify>
  <done>
✅ Agent and template types defined in agents.ts
✅ 3 specialist bot templates created in botTemplates.ts
✅ Each template includes: name, description, agents, tools, success metrics
✅ Agent personas defined for each specialist agent
✅ Skills properly structured with tool mappings
✅ Utility functions for template lookup and filtering
✅ All types properly typed (no any)
✅ No TypeScript errors
  </done>
</task>

<task type="auto">
  <name>Task 2: Create BotTemplateSelector component</name>
  <files>
    web-app/src/components/config/BotTemplateSelector.tsx
  </files>
  <action>
Create a component for selecting and viewing bot templates:

1. **Component structure:**
   ```typescript
   interface BotTemplateSelectorProps {
     onSelectTemplate: (template: BotTemplate) => Promise<void>
     onClose?: () => void
     showDescription?: boolean
   }

   export const BotTemplateSelector: React.FC<BotTemplateSelectorProps> = ({
     onSelectTemplate,
     onClose,
     showDescription = true
   }) => {
     const [selectedTemplate, setSelectedTemplate] = useState<BotTemplate | null>(null)
     const [loading, setLoading] = useState(false)
     const [error, setError] = useState<string | null>(null)

     const handleSelectTemplate = async (template: BotTemplate) => {
       setLoading(true)
       setError(null)

       try {
         await onSelectTemplate(template)
         // Success feedback
       } catch (err) {
         setError(err instanceof Error ? err.message : 'Failed to create squad')
       } finally {
         setLoading(false)
       }
     }

     return (
       <div className="bot-template-selector">
         <h2>Choose a Specialist Squad</h2>
         <p>Get started with pre-configured agent teams for common DevOps tasks</p>

         <div className="templates-grid">
           {BOT_TEMPLATES.map(template => (
             <TemplateCard
               key={template.id}
               template={template}
               selected={selectedTemplate?.id === template.id}
               onSelect={setSelectedTemplate}
               loading={loading}
               onApply={() => handleSelectTemplate(template)}
             />
           ))}
         </div>

         {selectedTemplate && showDescription && (
           <div className="template-details">
             <TemplateDetailsPanel
               template={selectedTemplate}
               onApply={() => handleSelectTemplate(selectedTemplate)}
               loading={loading}
             />
           </div>
         )}

         {error && <AlertError message={error} />}
       </div>
     )
   }
   ```

2. **TemplateCard sub-component:**
   ```typescript
   interface TemplateCardProps {
     template: BotTemplate
     selected: boolean
     onSelect: (template: BotTemplate) => void
     loading: boolean
     onApply: () => void
   }

   const TemplateCard: React.FC<TemplateCardProps> = ({
     template,
     selected,
     onSelect,
     loading,
     onApply
   }) => {
     return (
       <div
         className={`template-card ${selected ? 'selected' : ''}`}
         onClick={() => onSelect(template)}
       >
         <div className="template-icon">{template.icon}</div>
         <h3>{template.name}</h3>
         <p className="description">{template.description}</p>

         <div className="template-meta">
           <span className="agents-count">{template.agents.length} agents</span>
           <span className="setup-time">{template.estimatedSetupTime} min setup</span>
         </div>

         <div className="tools-required">
           <span className="label">Tools:</span>
           {template.requiredTools.map(tool => (
             <span key={tool} className="tool-tag">{tool}</span>
           ))}
         </div>

         {selected && (
           <button
             onClick={e => {
               e.stopPropagation()
               onApply()
             }}
             disabled={loading}
             className="btn-apply"
           >
             {loading ? 'Creating...' : 'Create Squad'}
           </button>
         )}
       </div>
     )
   }
   ```

3. **TemplateDetailsPanel sub-component:**
   - Shows detailed description of selected template
   - Lists all agents in template with roles and skills
   - Shows success metrics and targets
   - Displays use cases and recommended scenarios
   - "Create Squad" button at bottom

4. **Integration:**
   - Dispatch createAgentFromTemplate(template) on "Create Squad"
   - Handle loading state during creation
   - Show success message when agents created
   - Can show in modal or full page

5. **Styling:**
   - Grid layout for template cards (3 columns on desktop, 1 on mobile)
   - Card hover effects
   - Selected state highlighted
   - Icon/emoji prominent
   - Responsive design

Example usage in Dashboard:
```typescript
const [showTemplateSelector, setShowTemplateSelector] = useState(false)

const handleSelectTemplate = async (template: BotTemplate) => {
  const agents = await configAPI.createAgentFromTemplate(template)
  dispatch(addAgents(agents))
  setShowTemplateSelector(false)
  showToast('Squad created successfully', 'success')
}

return (
  <>
    <button onClick={() => setShowTemplateSelector(true)}>
      Add Specialist Squad
    </button>

    {showTemplateSelector && (
      <BotTemplateSelector
        onSelectTemplate={handleSelectTemplate}
        onClose={() => setShowTemplateSelector(false)}
        showDescription={true}
      />
    )}
  </>
)
```
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- BotTemplateSelector
Should show:
- Component renders with all 3 templates visible
- Clicking template card selects it
- Selected template shows details
- "Create Squad" button visible on selected template
- Clicking Create triggers onSelectTemplate callback
- Loading state shows during creation
- Error state displays error message

Browser test:
- Visit configuration dashboard
- Click "Add Specialist Squad"
- Should see 3 template cards
- Click one to see details
- Click "Create Squad" to create agents
  </verify>
  <done>
✅ BotTemplateSelector component created with 3 templates visible
✅ TemplateCard shows icon, name, description, agents count, setup time
✅ Required tools displayed for each template
✅ Template details panel shows agents and metrics
✅ Create Squad button triggers agent creation
✅ Loading state during creation
✅ Error handling with retry
✅ Responsive design (grid layout works on mobile)
  </done>
</task>

<task type="auto">
  <name>Task 3: Implement SquadCompositionUI component</name>
  <files>
    web-app/src/components/config/SquadCompositionUI.tsx
  </files>
  <action>
Create a component for viewing and managing squad composition:

1. **Component structure:**
   ```typescript
   interface SquadCompositionUIProps {
     squad: SquadConfig
     onAddAgent?: (agent: TemplateAgent) => void
     onRemoveAgent?: (agentId: string) => void
     onUpdateAgent?: (agentId: string, updates: Partial<SquadAgent>) => void
     readOnly?: boolean
   }

   export const SquadCompositionUI: React.FC<SquadCompositionUIProps> = ({
     squad,
     onAddAgent,
     onRemoveAgent,
     onUpdateAgent,
     readOnly = false
   }) => {
     const [expandedAgent, setExpandedAgent] = useState<string | null>(null)

     return (
       <div className="squad-composition">
         <div className="squad-header">
           <h3>{squad.name}</h3>
           <span className="agent-count">{squad.agents.length} agents</span>
         </div>

         <div className="agents-grid">
           {squad.agents.map(agent => (
             <AgentCard
               key={agent.id}
               agent={agent}
               expanded={expandedAgent === agent.id}
               onToggleExpand={() => setExpandedAgent(
                 expandedAgent === agent.id ? null : agent.id
               )}
               onRemove={!readOnly ? () => onRemoveAgent?.(agent.id) : undefined}
               onUpdate={!readOnly ? (updates) => onUpdateAgent?.(agent.id, updates) : undefined}
             />
           ))}
         </div>

         {!readOnly && onAddAgent && (
           <div className="add-agent-section">
             <button onClick={() => {}} className="btn-add-agent">
               + Add Agent
             </button>
           </div>
         )}
       </div>
     )
   }
   ```

2. **AgentCard sub-component:**
   ```typescript
   interface AgentCardProps {
     agent: SquadAgent
     expanded: boolean
     onToggleExpand: () => void
     onRemove?: () => void
     onUpdate?: (updates: Partial<SquadAgent>) => void
   }

   const AgentCard: React.FC<AgentCardProps> = ({
     agent,
     expanded,
     onToggleExpand,
     onRemove,
     onUpdate
   }) => {
     return (
       <div className={`agent-card ${expanded ? 'expanded' : ''}`}>
         <div className="card-header" onClick={onToggleExpand}>
           <div className="agent-identity">
             <div className="agent-avatar">{getAvatarEmoji(agent.persona)}</div>
             <div className="agent-info">
               <h4>{agent.name}</h4>
               <span className="role-badge">{agent.role}</span>
               <span className={`status-badge ${agent.status}`}>{agent.status}</span>
             </div>
           </div>
           <ChevronIcon rotated={expanded} />
         </div>

         {expanded && (
           <div className="card-details">
             <section className="persona-section">
               <h5>Personality</h5>
               <p>{agent.persona.personality}</p>
               <div className="traits">
                 {agent.persona.personality_traits?.map(trait => (
                   <span key={trait} className="trait-tag">{trait}</span>
                 ))}
               </div>
             </section>

             <section className="communication-section">
               <h5>Communication Style</h5>
               <p>{agent.persona.communication_style}</p>
             </section>

             <section className="skills-section">
               <h5>Skills ({agent.skills.length})</h5>
               <ul className="skills-list">
                 {agent.skills.map(skill => (
                   <li key={skill.id}>
                     <span className="skill-name">{skill.name}</span>
                     <span className="skill-tools">
                       {skill.tools.join(', ')}
                     </span>
                   </li>
                 ))}
               </ul>
             </section>

             {onRemove && (
               <div className="card-actions">
                 <button onClick={onRemove} className="btn-remove">
                   Remove from Squad
                 </button>
               </div>
             )}
           </div>
         )}
       </div>
     )
   }
   ```

3. **SquadOverview panel:**
   - Show squad name, template, creation date
   - Total agents and their roles
   - Quick stats: success rate, avg response time
   - Coordination pattern visualization

4. **Squad coordination visualization:**
   - Simple diagram showing how agents communicate
   - "Xops (Orchestrator)" at center
   - Specialist agents around with connection lines
   - Click agent to show details

5. **Styling:**
   - Card layout with hover effects
   - Persona traits color-coded by category
   - Role badges with colors (orchestrator, specialist)
   - Status indicator (active = green, inactive = gray)
   - Expandable details
   - Responsive grid

Example integration in Configuration dashboard:
```typescript
const [squad, setSquad] = useState<SquadConfig | null>(null)

const handleAddAgent = async (agent: TemplateAgent) => {
  const newAgent = await configAPI.createAgent({...agent})
  setSquad(prev => prev ? {
    ...prev,
    agents: [...prev.agents, newAgent]
  } : null)
}

return (
  <div className="config-dashboard">
    {squad && (
      <SquadCompositionUI
        squad={squad}
        onAddAgent={handleAddAgent}
        readOnly={false}
      />
    )}
  </div>
)
```
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- SquadCompositionUI
Should show:
- Component renders squad with agents
- Each agent displays name, role, status
- Clicking agent expands details
- Expanded view shows personality, skills, communication
- Remove button visible on expanded agent
- Agent count displayed
- Responsive grid layout works

Browser test:
- Visit configuration dashboard
- View created squad
- Should see all agents as cards
- Click agent to expand
- See persona and skills
  </verify>
  <done>
✅ SquadCompositionUI component created
✅ Shows squad name, agents count, creation date
✅ AgentCard displays agent name, role, status with avatar
✅ Expandable details show persona, skills, communication style
✅ Remove agent button available (not read-only)
✅ Coordination diagram shows (hub-and-spoke pattern)
✅ Responsive grid layout
✅ Tests passing
  </done>
</task>

<task type="auto">
  <name>Task 4: Add API methods for template-based agent creation</name>
  <files>
    web-app/src/api/config.ts
  </files>
  <action>
Extend config API with methods for creating agents from templates:

1. **Add methods to ConfigAPI class:**
   ```typescript
   export class ConfigAPI {
     // ... existing methods ...

     async createAgentFromTemplate(template: BotTemplate): Promise<Agent[]> {
       try {
         const agents: Agent[] = []

         // Create each agent from template definition
         for (const templateAgent of template.agents) {
           const agentData = {
             name: templateAgent.name,
             role: templateAgent.role,
             persona: templateAgent.persona,
             skills: templateAgent.skills,
             tools: template.requiredTools,
             coordinationWith: templateAgent.coordinationWith,
             status: 'active'
           }

           const response = await this.client.post<Agent>(
             '/config/agents',
             agentData
           )
           agents.push(response.data)
         }

         return agents
       } catch (error) {
         if (axios.isAxiosError(error)) {
           throw new Error(`Failed to create agent squad: ${error.response?.data?.error}`)
         }
         throw error
       }
     }

     async getSquadConfig(squadId: string): Promise<SquadConfig> {
       try {
         const response = await this.client.get<SquadConfig>(
           `/config/squads/${squadId}`
         )
         return response.data
       } catch (error) {
         throw new Error('Failed to load squad configuration')
       }
     }

     async updateSquadConfig(squadId: string, updates: Partial<SquadConfig>): Promise<SquadConfig> {
       try {
         const response = await this.client.put<SquadConfig>(
           `/config/squads/${squadId}`,
           updates
         )
         return response.data
       } catch (error) {
         throw new Error('Failed to update squad configuration')
       }
     }

     async deleteAgentFromSquad(squadId: string, agentId: string): Promise<void> {
       try {
         await this.client.delete(`/config/squads/${squadId}/agents/${agentId}`)
       } catch (error) {
         throw new Error('Failed to remove agent from squad')
       }
     }

     async listAvailableTemplates(): Promise<BotTemplate[]> {
       // Return local templates (could be backend-driven in future)
       return BOT_TEMPLATES
     }

     async listSquads(): Promise<SquadConfig[]> {
       try {
         const response = await this.client.get<SquadConfig[]>('/config/squads')
         return response.data
       } catch (error) {
         return [] // Return empty list if endpoint doesn't exist yet
       }
     }
   }
   ```

2. **Add mock handlers for squad endpoints:**
   ```typescript
   export const squadHandlers = [
     http.post('*/config/agents', async ({ request }) => {
       const body = await request.json()
       const agent: Agent = {
         id: `agent-${Date.now()}`,
         ...body,
         createdAt: new Date().toISOString(),
         status: 'active'
       }
       return HttpResponse.json(agent, { status: 201 })
     }),

     http.get('*/config/squads', () => {
       const squads: SquadConfig[] = [
         {
           id: 'squad-1',
           name: 'Kubernetes Ops Squad',
           templateId: 'kubernetes-ops-squad',
           agents: [], // Populated after agent creation
           createdAt: new Date(),
           lastModified: new Date()
         }
       ]
       return HttpResponse.json(squads)
     }),

     http.get('*/config/squads/:squadId', ({ params }) => {
       // Return mock squad config
       return HttpResponse.json({
         id: params.squadId,
         name: 'Example Squad',
         templateId: 'kubernetes-ops-squad',
         agents: [],
         createdAt: new Date(),
         lastModified: new Date()
       })
     }),

     http.put('*/config/squads/:squadId', async ({ request }) => {
       const body = await request.json()
       return HttpResponse.json({
         ...body,
         lastModified: new Date()
       })
     }),

     http.delete('*/config/squads/:squadId/agents/:agentId', () => {
       return HttpResponse.json({ success: true }, { status: 204 })
     })
   ]
   ```

3. **Update MSW handlers to include squad handlers**

4. **Add Redux actions for squad operations:**
   - setSelectedSquad(squad)
   - createSquadFromTemplate(template)
   - updateSquadConfig(updates)
   - deleteAgentFromSquad(agentId)
   - listSquads()

Error handling should provide clear messages:
- "Agent creation failed: {reason}"
- "Squad configuration not found"
- "Failed to remove agent"

Ensure API methods are type-safe and properly handle network errors.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- config.test
Should show:
- createAgentFromTemplate() callable and returns agents array
- Each agent in array has required fields (id, name, role, persona, skills)
- getSquadConfig() returns SquadConfig
- updateSquadConfig() updates and returns updated config
- deleteAgentFromSquad() removes agent
- Error handling works (returns error message)

Also test with mock API:
- POST /config/agents creates agent
- GET /config/squads lists squads
- PUT /config/squads/:squadId updates
- DELETE /config/squads/:squadId/agents/:agentId removes
  </verify>
  <done>
✅ configAPI.createAgentFromTemplate() creates all agents from template
✅ configAPI.getSquadConfig() fetches squad configuration
✅ configAPI.updateSquadConfig() updates squad
✅ configAPI.deleteAgentFromSquad() removes agent from squad
✅ configAPI.listSquads() lists all squads
✅ Mock handlers return proper JSON responses
✅ Error handling with user-friendly messages
✅ All API methods properly typed
  </done>
</task>

<task type="auto">
  <name>Task 5: Integrate specialist bots into configuration dashboard</name>
  <files>
    web-app/src/components/config/SquadCompositionPanel.tsx
    web-app/src/store/slices/configSlice.ts
  </files>
  <action>
Create SquadCompositionPanel component that brings everything together in the dashboard:

1. **SquadCompositionPanel component:**
   ```typescript
   export const SquadCompositionPanel: React.FC = () => {
     const dispatch = useAppDispatch()
     const squads = useAppSelector(s => s.config.squads)
     const loading = useAppSelector(s => s.config.loading)
     const [showTemplateSelector, setShowTemplateSelector] = useState(false)
     const [selectedSquadId, setSelectedSquadId] = useState<string | null>(null)

     useEffect(() => {
       // Load squads on mount
       dispatch(listSquads())
     }, [dispatch])

     const selectedSquad = squads.find(s => s.id === selectedSquadId)

     const handleSelectTemplate = async (template: BotTemplate) => {
       try {
         dispatch(createSquadFromTemplate(template))
         setShowTemplateSelector(false)
         // Show success toast
       } catch (error) {
         // Error already handled in Redux
       }
     }

     const handleRemoveAgent = async (agentId: string) => {
       if (selectedSquad) {
         dispatch(deleteAgentFromSquad(selectedSquad.id, agentId))
       }
     }

     return (
       <div className="squad-composition-panel">
         <div className="panel-header">
           <h2>Agent Squads</h2>
           <button
             onClick={() => setShowTemplateSelector(true)}
             className="btn-primary"
           >
             + Add Squad
           </button>
         </div>

         {loading && <LoadingSpinner message="Loading squads..." />}

         {squads.length === 0 ? (
           <EmptyState
             icon="👥"
             title="No squads yet"
             description="Start with a specialist squad template"
             action={{
               label: 'Add Your First Squad',
               onClick: () => setShowTemplateSelector(true)
             }}
           />
         ) : (
           <>
             <div className="squads-list">
               {squads.map(squad => (
                 <div
                   key={squad.id}
                   className={`squad-item ${selectedSquadId === squad.id ? 'selected' : ''}`}
                   onClick={() => setSelectedSquadId(squad.id)}
                 >
                   <span className="squad-name">{squad.name}</span>
                   <span className="agent-count">{squad.agents.length} agents</span>
                 </div>
               ))}
             </div>

             {selectedSquad && (
               <SquadCompositionUI
                 squad={selectedSquad}
                 onRemoveAgent={handleRemoveAgent}
                 readOnly={false}
               />
             )}
           </>
         )}

         {showTemplateSelector && (
           <Modal onClose={() => setShowTemplateSelector(false)}>
             <BotTemplateSelector
               onSelectTemplate={handleSelectTemplate}
               onClose={() => setShowTemplateSelector(false)}
             />
           </Modal>
         )}
       </div>
     )
   }
   ```

2. **Update Redux configSlice:**
   ```typescript
   interface ConfigState {
     agents: Agent[]
     squads: SquadConfig[]
     tools: Tool[]
     platforms: PlatformConfig[]
     loading: boolean
     error: string | null
   }

   // Add async thunks
   export const listSquads = createAsyncThunk(
     'config/listSquads',
     async (_, { rejectWithValue }) => {
       try {
         return await configAPI.listSquads()
       } catch (error) {
         return rejectWithValue((error as Error).message)
       }
     }
   )

   export const createSquadFromTemplate = createAsyncThunk(
     'config/createSquadFromTemplate',
     async (template: BotTemplate, { rejectWithValue }) => {
       try {
         const agents = await configAPI.createAgentFromTemplate(template)
         return {
           squad: {
             id: `squad-${Date.now()}`,
             name: template.name,
             templateId: template.id,
             agents: agents as SquadAgent[],
             createdAt: new Date(),
             lastModified: new Date()
           },
           agents
         }
       } catch (error) {
         return rejectWithValue((error as Error).message)
       }
     }
   )

   export const deleteAgentFromSquad = createAsyncThunk(
     'config/deleteAgentFromSquad',
     async ({ squadId, agentId }: { squadId: string; agentId: string }, { rejectWithValue }) => {
       try {
         await configAPI.deleteAgentFromSquad(squadId, agentId)
         return { squadId, agentId }
       } catch (error) {
         return rejectWithValue((error as Error).message)
       }
     }
   )

   // Add extraReducers
   builder
     .addCase(listSquads.fulfilled, (state, action) => {
       state.squads = action.payload
     })
     .addCase(createSquadFromTemplate.fulfilled, (state, action) => {
       state.squads.push(action.payload.squad)
       state.agents.push(...action.payload.agents)
     })
     .addCase(deleteAgentFromSquad.fulfilled, (state, action) => {
       const squad = state.squads.find(s => s.id === action.payload.squadId)
       if (squad) {
         squad.agents = squad.agents.filter(a => a.id !== action.payload.agentId)
       }
       state.agents = state.agents.filter(a => a.id !== action.payload.agentId)
     })
   ```

3. **Integration in Configuration dashboard:**
   - Add "Agent Squads" tab or section
   - Use SquadCompositionPanel as main UI
   - Position near "Agents" tab for easy access

4. **Styling:**
   - Consistent with existing dashboard styling
   - Sidebar showing squad list
   - Main area showing selected squad details
   - Modal for template selection
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- SquadCompositionPanel
Should show:
- Component renders with "Add Squad" button
- Empty state shown when no squads
- Squads list populates after load
- Clicking squad selects it
- Selected squad shows composition UI
- Template selector modal opens/closes
- Creating squad adds to Redux state and list
- Removing agent updates both squad and list

Browser test:
- Visit configuration dashboard
- Click "Agent Squads" tab or section
- Should see "Add Squad" button
- Click to open template selector
- Select template to create squad
- Squad appears in list
- Click to view composition
- Can remove agents from squad
  </verify>
  <done>
✅ SquadCompositionPanel component created
✅ Shows list of existing squads
✅ "Add Squad" button opens template selector
✅ Template selection creates agents and squad
✅ Squad composition UI displays selected squad
✅ Can remove agents from squad
✅ Redux state properly manages squads
✅ Empty state shown when no squads
✅ Loading state displays during operations
  </done>
</task>

<task type="auto">
  <name>Task 6: Write comprehensive tests for specialist bots and squad management</name>
  <files>
    web-app/src/test/e2e/bot-templates.test.tsx
  </files>
  <action>
Create end-to-end tests for template selection and squad management:

1. **Test suite structure:**
   ```typescript
   describe('Bot Templates and Squad Management E2E', () => {
     // Test 1: Template discovery
     // Test 2: Template selection and agent creation
     // Test 3: Squad composition UI
     // Test 4: Add/remove agents
     // Test 5: Error handling
     // Test 6: State persistence
   })
   ```

2. **Test cases:**

   **Test 1: Bot templates available**
   - All 3 templates visible (Kubernetes, Infrastructure, SRE)
   - Each template shows correct info (icon, name, description, agents count)
   - Required tools listed correctly
   - Success metrics displayed

   **Test 2: Template selection and agent creation**
   - Selecting template shows details
   - "Create Squad" button creates all agents
   - Each agent created with correct persona
   - Redux state updated with new squad
   - Squad appears in dashboard list

   **Test 3: Squad composition display**
   - Squad card shows all agents
   - Agent card shows name, role, status
   - Clicking expands to show details
   - Details show persona, skills, communication style

   **Test 4: Agent management**
   - Can remove agent from squad (deletes from list)
   - Can modify agent status
   - Changes persist to Redux state

   **Test 5: Error handling**
   - Network error on squad creation shows error message
   - Retry button available
   - Invalid template data handled gracefully

   **Test 6: Multiple squads**
   - Can create multiple squads
   - Each squad independently managed
   - Selecting squad shows correct agents

3. **Example tests:**
   ```typescript
   import { render, screen, fireEvent, waitFor } from '@testing-library/react'
   import userEvent from '@testing-library/user-event'
   import { SquadCompositionPanel } from '@/components/config/SquadCompositionPanel'
   import { Provider } from 'react-redux'
   import { store } from '@/store'

   describe('Bot Templates and Squad Management', () => {
     it('displays all 3 bot templates', async () => {
       render(
         <Provider store={store}>
           <SquadCompositionPanel />
         </Provider>
       )

       const addButton = screen.getByRole('button', { name: /add squad/i })
       fireEvent.click(addButton)

       await waitFor(() => {
         expect(screen.getByText(/kubernetes ops squad/i)).toBeInTheDocument()
         expect(screen.getByText(/infrastructure automation/i)).toBeInTheDocument()
         expect(screen.getByText(/sre observability/i)).toBeInTheDocument()
       })
     })

     it('creates agents from template', async () => {
       const user = userEvent.setup()

       // Render and navigate to template selector
       render(
         <Provider store={store}>
           <SquadCompositionPanel />
         </Provider>
       )

       const addButton = screen.getByRole('button', { name: /add squad/i })
       await user.click(addButton)

       // Select Kubernetes template
       const kubeCard = screen.getByText(/kubernetes ops squad/i).closest('.template-card')
       await user.click(kubeCard)

       // Create squad
       const createButton = screen.getByRole('button', { name: /create squad/i })
       await user.click(createButton)

       // Verify squad appears in list
       await waitFor(() => {
         expect(screen.getByText(/kubernetes ops squad/i)).toBeInTheDocument()
       })
     })

     it('displays squad composition with agents', async () => {
       // ... create squad first ...

       // Select squad to view
       const squadItem = screen.getByText(/kubernetes ops squad/i)
       fireEvent.click(squadItem)

       // Should show agents from template
       await waitFor(() => {
         expect(screen.getByText(/incident commander/i)).toBeInTheDocument()
         expect(screen.getByText(/log analyzer/i)).toBeInTheDocument()
       })
     })
   })
   ```

4. **Test utilities:**
   - Helper to render panel with Redux context
   - Helper to select template and create squad
   - Helper to verify agent creation
   - Helper to check Redux state after actions

5. **Coverage goals:**
   - >80% coverage of template/squad components
   - All major user flows tested
   - Error scenarios covered
   - Redux state updates verified

Ensure tests use MSW for API mocking and don't depend on external services.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- bot-templates
Should show:
- 6+ test cases all passing
- Template display test passes
- Agent creation test passes
- Squad composition test passes
- Error handling test passes

Also run full suite:
pnpm test
All tests passing (no regressions).
  </verify>
  <done>
✅ E2E tests for bot templates and squads (6+ test cases)
✅ Tests cover: template display, selection, agent creation, composition UI, error handling
✅ All tests passing (pnpm test → all passing)
✅ Tests use MSW for API mocking
✅ Redux state properly verified in tests
✅ Error scenarios tested
✅ No regressions in existing tests
  </done>
</task>

</tasks>

<verification>
After completing all tasks:

1. **Bot template definitions:**
   - [ ] 3 specialist templates defined (Kubernetes, Infrastructure, SRE)
   - [ ] Each template has agents with personas and skills
   - [ ] Success metrics defined for each template
   - [ ] Required tools properly specified
   - [ ] Templates exported and importable

2. **BotTemplateSelector component:**
   - [ ] Renders all 3 templates as cards
   - [ ] Shows template icon, name, description
   - [ ] Displays agent count and setup time
   - [ ] Shows required tools
   - [ ] "Create Squad" button functional
   - [ ] Details panel shows on selection
   - [ ] Loading state during creation
   - [ ] Error handling with retry

3. **SquadCompositionUI component:**
   - [ ] Displays squad with all agents
   - [ ] AgentCard shows name, role, status, avatar
   - [ ] Expandable details show persona and skills
   - [ ] Remove agent button available
   - [ ] Coordination diagram visible (hub-and-spoke)
   - [ ] Responsive grid layout

4. **Configuration dashboard integration:**
   - [ ] SquadCompositionPanel shows in dashboard
   - [ ] Can add squads from templates
   - [ ] Squads list shows and selectable
   - [ ] Squad details visible when selected
   - [ ] Can manage agents in squad

5. **API integration:**
   - [ ] createAgentFromTemplate() creates agents
   - [ ] getSquadConfig() fetches squad
   - [ ] updateSquadConfig() updates squad
   - [ ] deleteAgentFromSquad() removes agent
   - [ ] listSquads() lists all squads
   - [ ] Mock handlers return proper responses

6. **Redux integration:**
   - [ ] Squads properly managed in Redux state
   - [ ] createSquadFromTemplate thunk works
   - [ ] deleteAgentFromSquad thunk works
   - [ ] State updates on agent operations
   - [ ] State persists via Redux Persist

7. **Testing:**
   - [ ] pnpm test → 6+ template tests passing
   - [ ] pnpm test → all tests passing (no regressions)
   - [ ] pnpm type-check → 0 TypeScript errors
   - [ ] pnpm build → succeeds without errors

8. **UX quality:**
   - [ ] Template cards visually appealing
   - [ ] Agent cards clearly show information
   - [ ] Persona traits visible in expanded view
   - [ ] Loading states shown during operations
   - [ ] Error messages clear
   - [ ] Mobile responsive

9. **Acceptance criteria:**
   - [ ] 3 specialist templates available (Kubernetes, Infrastructure, SRE)
   - [ ] Users can select template to create squad
   - [ ] Squad shows all agents with personas and skills
   - [ ] Users can view and modify squad composition
   - [ ] Squad configuration persists
   - [ ] Agents properly coordinated with Xops
   - [ ] All specialist bots created with correct personas
</verification>

<success_criteria>

**Plan 03 Complete When:**

1. ✅ **3 specialist templates:** Kubernetes Ops, Infrastructure Automation, SRE Observability
2. ✅ **Template structure:** Each has agents, personas, skills, success metrics
3. ✅ **BotTemplateSelector:** Component shows all templates and allows selection
4. ✅ **Squad creation:** Selecting template creates all agents via API
5. ✅ **SquadCompositionUI:** Displays team members with personas and skills
6. ✅ **Dashboard integration:** Squad management available in config dashboard
7. ✅ **Agent management:** Can add/remove agents from squad
8. ✅ **Redux integration:** Squads properly managed in state
9. ✅ **API integration:** All squad operations (CRUD) working
10. ✅ **Error handling:** Network errors, creation failures handled gracefully
11. ✅ **Testing:** 6+ E2E tests all passing
12. ✅ **No regressions:** All existing tests still passing
13. ✅ **TypeScript clean:** 0 compilation errors

**Verification Method:**
```bash
# Tests
pnpm test -- bot-templates    # 6+ tests passing
pnpm test                     # All tests passing

# Type checking
pnpm type-check              # 0 errors

# Build
pnpm build                   # Succeeds

# Manual testing
pnpm dev
# Visit config dashboard
# Click "Add Squad"
# Select template
# Verify squad created with agents
```

**Deliverables:**
- 8 new/updated files (types, templates, components, API, tests)
- 6+ commits with atomic changes
- 3 specialist bot templates
- BotTemplateSelector and SquadCompositionUI components
- API methods for squad management
- Redux integration for squad state
- 6+ E2E tests
- 0 regressions
</success_criteria>

<output>
After completion, create `.planning/phases/01-onboarding-refinement/03-SUMMARY.md` with:
- Executive summary of specialist bot templates
- File list (8 files created/updated)
- Template definitions (3 templates with agent counts)
- Component inventory (BotTemplateSelector, SquadCompositionUI, SquadCompositionPanel)
- Test results (6+ tests passing)
- Integration verification (templates working in dashboard)
- Next steps (Plan 04: Safety/Approval gates)
</output>
