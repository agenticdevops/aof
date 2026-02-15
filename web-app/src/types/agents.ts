/**
 * Agent Types and Specialist Bot Templates
 *
 * Defines TypeScript types for:
 * - Agent roles, skills, and personas
 * - Bot template definitions
 * - Squad configurations
 * - Success metrics tracking
 */

/**
 * Agent role in the squad
 */
export type AgentRole = 'orchestrator' | 'specialist' | 'assistant'

/**
 * Skill category for organization
 */
export type SkillCategory =
  | 'incident-response'
  | 'deployment'
  | 'provisioning'
  | 'monitoring'
  | 'optimization'
  | 'analysis'

/**
 * Represents a skill an agent possesses
 */
export interface AgentSkill {
  id: string
  name: string
  description: string
  tools: string[] // Tool IDs: ["kubectl", "terraform", etc]
  category: SkillCategory
  confidence: number // 0.0 - 1.0 confidence level
}

/**
 * Persona traits that define agent behavior
 */
export interface AgentPersona {
  personality: string
  personality_traits: string[]
  communication_style: string
  can?: string[] // Boundaries - what agent can do
  cannot?: string[] // Boundaries - what agent cannot do
}

/**
 * Template agent definition (before instantiation in squad)
 */
export interface TemplateAgent {
  id: string
  name: string
  role: AgentRole
  persona: AgentPersona
  skills: AgentSkill[]
  responsibilities: string[]
  coordinationWith: string[] // Agent IDs this coordinates with
}

/**
 * Success metric for a template
 */
export interface SuccessMetric {
  name: string
  description: string
  target: string // "MTTR < 5 min", "100% uptime", etc
}

/**
 * Bot template definition
 * Represents a pre-configured squad of agents for a specific use case
 */
export interface BotTemplate {
  id: string
  name: string
  description: string
  icon: string // Emoji or icon identifier
  category: 'kubernetes' | 'infrastructure' | 'observability' | 'custom'
  useCase: string
  agents: TemplateAgent[]
  coordinationPattern: 'hub-and-spoke' | 'peer-to-peer' | 'hierarchical'
  requiredTools: string[]
  estimatedSetupTime: number // minutes
  successMetrics: SuccessMetric[]
  tags: string[]
}

/**
 * Squad agent instance (agent deployed from template)
 */
export interface SquadAgent {
  id: string
  agentId: string // Reference to template agent ID
  name: string
  role: AgentRole
  status: 'active' | 'inactive' | 'error'
  persona: AgentPersona
  skills: AgentSkill[]
}

/**
 * Squad configuration
 * Represents a deployed squad of agents
 */
export interface SquadConfig {
  id: string
  name: string
  templateId: string
  agents: SquadAgent[]
  createdAt: Date
  lastModified: Date
}

/**
 * Agent definition (generic agent, not template or squad-specific)
 */
export interface Agent {
  id: string
  name: string
  role: AgentRole
  persona: AgentPersona
  skills: AgentSkill[]
  status: 'active' | 'inactive' | 'error'
  createdAt?: Date
  updatedAt?: Date
}
