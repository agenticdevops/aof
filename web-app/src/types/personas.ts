/**
 * Persona System Type Definitions
 *
 * Defines agent personalities with distinct visual identities:
 * - Analysts: Blue theme (📊) - analytical, data-focused
 * - Coordinators: Green theme (⚙️) - leadership, orchestration
 * - Specialists: Purple theme (🔧) - technical expertise
 * - Responders: Red theme (🚨) - incident response
 */

/**
 * Persona type taxonomy
 */
export type PersonaType = 'analyst' | 'coordinator' | 'specialist' | 'responder' | 'custom'

/**
 * Color palette for a persona (light and dark mode variants)
 */
export interface PersonaColor {
  /** Primary color (light mode) */
  primary: string
  /** Secondary/background color (light mode) */
  secondary: string
  /** Accent color for highlights (light mode) */
  accent: string
  /** Primary color (dark mode) */
  darkPrimary: string
  /** Secondary/background color (dark mode) */
  darkSecondary: string
  /** Accent color for highlights (dark mode) */
  darkAccent: string
}

/**
 * Font weight variant for persona
 */
export type PersonaFont = 'regular' | 'bold' | 'decorative'

/**
 * Complete agent persona definition
 */
export interface AgentPersona {
  /** Unique persona identifier */
  id: string
  /** Persona type category */
  type: PersonaType
  /** Display name */
  name: string
  /** Icon representation (emoji or URL) */
  icon: string
  /** Color palette (light + dark mode) */
  colors: PersonaColor
  /** Typography style */
  font: PersonaFont
  /** Description of personality traits */
  description: string
}

/**
 * Redux state for persona management
 */
export interface PersonaState {
  /** Active personas keyed by ID */
  personas: Record<string, AgentPersona>
  /** Default personas (immutable) */
  defaultPersonas: AgentPersona[]
  /** Loading state */
  isLoading: boolean
  /** Error state */
  error: string | null
}

/**
 * Default persona configurations
 * Immutable presets for the 4 core agent types
 */
export const DEFAULT_PERSONAS: AgentPersona[] = [
  {
    id: 'analyst',
    type: 'analyst',
    name: 'Analyst',
    icon: '📊',
    font: 'regular',
    description: 'Analytical, data-focused, investigative',
    colors: {
      primary: '#3b82f6',      // blue-500
      secondary: '#dbeafe',    // blue-100
      accent: '#1e40af',       // blue-800
      darkPrimary: '#60a5fa',  // blue-400
      darkSecondary: '#1e3a8a', // blue-900
      darkAccent: '#93c5fd',   // blue-300
    },
  },
  {
    id: 'coordinator',
    type: 'coordinator',
    name: 'Coordinator',
    icon: '⚙️',
    font: 'bold',
    description: 'Leadership, orchestration, team management',
    colors: {
      primary: '#10b981',      // emerald-500
      secondary: '#d1fae5',    // emerald-100
      accent: '#047857',       // emerald-700
      darkPrimary: '#34d399',  // emerald-400
      darkSecondary: '#064e3b', // emerald-900
      darkAccent: '#6ee7b7',   // emerald-300
    },
  },
  {
    id: 'specialist',
    type: 'specialist',
    name: 'Specialist',
    icon: '🔧',
    font: 'regular',
    description: 'Technical expertise, precision, execution',
    colors: {
      primary: '#8b5cf6',      // violet-500
      secondary: '#ede9fe',    // violet-100
      accent: '#5b21b6',       // violet-800
      darkPrimary: '#a78bfa',  // violet-400
      darkSecondary: '#2e1065', // violet-900
      darkAccent: '#c4b5fd',   // violet-300
    },
  },
  {
    id: 'responder',
    type: 'responder',
    name: 'Responder',
    icon: '🚨',
    font: 'regular',
    description: 'Incident response, urgency, problem-solving',
    colors: {
      primary: '#ef4444',      // red-500
      secondary: '#fee2e2',    // red-100
      accent: '#991b1b',       // red-800
      darkPrimary: '#f87171',  // red-400
      darkSecondary: '#7f1d1d', // red-900
      darkAccent: '#fca5a5',   // red-300
    },
  },
]

/**
 * Get default persona by type
 */
export function getDefaultPersona(type: PersonaType): AgentPersona | undefined {
  return DEFAULT_PERSONAS.find((p) => p.type === type)
}

/**
 * Map legacy agent type to persona type
 * (handles backward compatibility with existing agent type field)
 */
export function mapAgentTypeToPersonaType(agentType: string): PersonaType {
  switch (agentType.toLowerCase()) {
    case 'analyst':
      return 'analyst'
    case 'coordinator':
    case 'orchestrator':
      return 'coordinator'
    case 'specialist':
      return 'specialist'
    case 'responder':
      return 'responder'
    default:
      return 'custom'
  }
}
