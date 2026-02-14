/**
 * TypeScript types for Phase 1 CoordinationEvent and extended schemas
 */

/**
 * Core coordination event from Phase 1 event infrastructure.
 * Represents a single observable state transition in agent execution.
 */
export interface CoordinationEvent {
  /** Unique event identifier */
  event_id: string;

  /** Agent that emitted this event */
  agent_id: string;

  /** Activity details */
  activity: AgentActivity;

  /** ISO 8601 timestamp when event occurred */
  timestamp: string;
}

/**
 * Activity details within a coordination event.
 * Matches Phase 1 ActivityEvent structure.
 */
export interface AgentActivity {
  /** Type of activity */
  type: ActivityType;

  /** Additional context and details */
  details: Record<string, unknown>;
}

/**
 * Activity type enum matching Phase 1 ActivityType.
 */
export enum ActivityType {
  AgentStarted = "agent_started",
  AgentCompleted = "agent_completed",
  ToolCalled = "tool_called",
  ToolExecuting = "tool_executing",
  ToolCompleted = "tool_completed",
  ToolFailed = "tool_failed",
  Thinking = "thinking",
  Error = "error",
  Info = "info",
  Warning = "warning",
  Debug = "debug",
}

/**
 * Agent status for UI display.
 */
export enum AgentStatus {
  Idle = "idle",
  Working = "working",
  Blocked = "blocked",
  Error = "error",
}

/**
 * Agent configuration interface.
 */
export interface Agent {
  /** Agent unique identifier */
  id: string;

  /** Human-readable agent name */
  name: string;

  /** Agent role/persona */
  role: string;

  /** Personality description */
  personality?: string;

  /** Avatar URL or emoji */
  avatar?: string;

  /** Agent skills/capabilities */
  skills: string[];

  /** Current agent status */
  status: AgentStatus;
}

/**
 * Tool configuration interface.
 */
export interface Tool {
  /** Tool name */
  name: string;

  /** Tool description */
  description: string;

  /** Tool category (e.g., "kubernetes", "system", "network") */
  category: string;

  /** Input JSON schema (optional) */
  input_schema?: Record<string, unknown>;

  /** Output JSON schema (optional) */
  output_schema?: Record<string, unknown>;
}
