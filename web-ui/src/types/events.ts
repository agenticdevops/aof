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

  /** Optional agent introduction data (present for introduction events) */
  introduction?: AgentIntroductionData;
}

/**
 * Agent introduction event data.
 * Present when the event is an agent introduction (daemon startup, squad join).
 */
export interface AgentIntroductionData {
  /** Agent unique identifier */
  agent_id: string;

  /** Display name */
  agent_name: string;

  /** Role description */
  role: string;

  /** Emoji avatar */
  avatar: string;

  /** Introduction message from SOUL.md */
  intro_message: string;

  /** One-line personality summary */
  personality_summary: string;

  /** Agent skills/capabilities */
  skills: string[];
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
 * Activity type matching Phase 1 ActivityType.
 */
export type ActivityType =
  | "agent_started"
  | "agent_completed"
  | "tool_called"
  | "tool_executing"
  | "tool_completed"
  | "tool_failed"
  | "thinking"
  | "error"
  | "info"
  | "warning"
  | "debug";

/**
 * Agent status for UI display.
 */
export type AgentStatus = "idle" | "working" | "blocked" | "error";

/**
 * Agent configuration interface.
 * Extended with persona fields from AGENTS.md (Phase 5).
 */
export interface Agent {
  /** Agent unique identifier */
  id: string;

  /** Human-readable agent name */
  name: string;

  /** Agent role/persona (e.g., "Infrastructure Specialist") */
  role: string;

  /** Personality description (free-form text) */
  personality?: string;

  /** Avatar emoji (from AGENTS.md, e.g., "🤖") */
  avatar?: string;

  /** Agent skills/capabilities */
  skills: string[];

  /** Current agent status */
  status: AgentStatus;

  /** Personality trait keywords (e.g., ["methodical", "proactive", "detail-oriented"]) */
  personality_traits?: string[];

  /** Actions the agent CAN perform (from AGENTS.md) */
  can?: string[];

  /** Actions the agent CANNOT perform (from AGENTS.md) */
  cannot?: string[];

  /** Communication style (e.g., "calm-professional") */
  communication_style?: string;

  /** Tone descriptor (e.g., "formal", "friendly") */
  tone?: string;

  /** Introduction message shown on first appearance */
  intro_message?: string;

  /** Uptime percentage (computed from event history, 0-100) */
  uptime_percent?: number;

  /** Success rate percentage (computed from event history, 0-100) */
  success_rate?: number;
}

/**
 * Persona information subset of Agent.
 * Used when only persona-related fields are needed.
 */
export interface PersonaInfo {
  /** Personality trait keywords */
  personality_traits: string[];

  /** Actions the agent CAN perform */
  can: string[];

  /** Actions the agent CANNOT perform */
  cannot: string[];

  /** Communication style */
  communication_style?: string;

  /** Tone descriptor */
  tone?: string;
}

/**
 * Introduction message event data.
 * Matches Phase 1 CoordinationEvent::AgentIntroduction structure.
 */
export interface IntroductionMessage {
  /** Agent identifier */
  agent_name: string;

  /** Introduction message text */
  intro_message: string;

  /** Agent skills for display */
  skills: string[];

  /** Agent avatar emoji */
  avatar?: string;
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
