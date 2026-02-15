/**
 * Dashboard domain types for Mission Control
 */

/**
 * Agent operational status
 */
export type AgentStatus = 'active' | 'idle' | 'error'

/**
 * Real-time agent performance metrics
 */
export interface AgentMetrics {
  /** Uptime percentage (0-100) */
  uptime: number
  /** Success rate percentage (0-100) */
  successRate: number
  /** Average response time in milliseconds */
  responseTime: number
  /** Total completed tasks */
  tasksCompleted: number
}

/**
 * Dashboard agent representation with live metrics and persona
 */
export interface DashboardAgent {
  /** Unique agent identifier */
  id: string
  /** Display name */
  name: string
  /** Agent role (e.g., "Orchestrator", "K8s Specialist") */
  role: string
  /** Current operational status */
  status: AgentStatus
  /** Real-time performance metrics */
  metrics: AgentMetrics
  /** Persona color (hex code) */
  personaColor: string
  /** Persona icon (emoji or symbol) */
  personaIcon: string
  /** Last status update timestamp */
  updatedAt: Date
}

/**
 * Dashboard Redux state
 */
export interface DashboardState {
  /** All agents visible in Mission Control */
  agents: DashboardAgent[]
  /** Currently selected agent for detail view */
  selectedAgent: DashboardAgent | null
  /** Loading state for async operations */
  isLoading: boolean
  /** Error message (if any) */
  error: string | null
}

/**
 * Extended metrics for detail modal view
 */
export interface ExtendedAgentMetrics extends AgentMetrics {
  /** Last active timestamp */
  lastActive: Date
  /** Total lifetime runs */
  totalRuns: number
  /** Error rate percentage (0-100) */
  errorRate: number
  /** Tasks per day average */
  tasksPerDay: number
}
