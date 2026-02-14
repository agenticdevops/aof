/**
 * TypeScript types for coordination protocols UI (Phase 7-05).
 * Defines interfaces for heartbeat health, standup results, and coordination metrics.
 */

/**
 * Agent health status from heartbeat protocol.
 */
export type AgentHealthStatus = 'Healthy' | 'Degraded' | 'Unresponsive';

/**
 * Individual agent health record.
 */
export interface AgentHealthRecord {
  /** Agent identifier */
  agent_id: string;

  /** Current health status */
  status: AgentHealthStatus;

  /** Last successful heartbeat timestamp (ISO 8601, null if never responded) */
  last_heartbeat: string | null;

  /** Number of consecutive missed heartbeats */
  consecutive_misses: number;

  /** Last response time in milliseconds (null if no response) */
  last_response_ms: number | null;

  /** Reason for degraded status (present when status is Degraded) */
  degraded_reason?: string;
}

/**
 * Heartbeat health response from /api/coordination/health endpoint.
 */
export interface HeartbeatHealthResponse {
  /** Array of agent health records */
  agents: AgentHealthRecord[];

  /** Heartbeat configuration */
  heartbeat_config: {
    /** Heartbeat interval in seconds */
    frequency_secs: number;

    /** Response timeout in seconds */
    timeout_secs: number;
  };

  /** Whether coordination is enabled */
  coordination_enabled: boolean;
}

/**
 * Individual agent standup response.
 */
export interface StandupResponseRecord {
  /** Agent identifier */
  agent_id: string;

  /** What the agent did since last standup */
  what_i_did: string;

  /** What the agent is currently working on */
  what_im_doing: string;

  /** List of blockers (empty array if none) */
  blockers: string[];

  /** Token count for this response */
  token_count: number;

  /** Response timestamp (ISO 8601) */
  timestamp: string;
}

/**
 * Complete standup result with all agent responses.
 */
export interface StandupResult {
  /** Unique request identifier */
  request_id: string;

  /** Array of agent responses */
  responses: StandupResponseRecord[];

  /** Optional AI-generated summary (present when summarization enabled) */
  summary?: string;

  /** Standup trigger timestamp (ISO 8601) */
  triggered_at: string;
}

/**
 * Coordination mode enum.
 */
export type CoordinationMode =
  | 'Full'
  | 'Standard'
  | 'Reduced'
  | 'HeartbeatOnly'
  | 'Disabled';

/**
 * Coordination token metrics from /api/coordination/metrics endpoint.
 */
export interface CoordinationMetrics {
  /** Total tokens used for coordination */
  coordination_tokens: number;

  /** Total tokens used for production work */
  production_tokens: number;

  /** Coordination overhead as percentage (0-100) */
  overhead_percent: number;

  /** Tokens used for heartbeat protocol */
  heartbeat_tokens: number;

  /** Tokens used for standup protocol */
  standup_tokens: number;

  /** Current coordination mode */
  current_mode: CoordinationMode;

  /** Whether auto-degradation is enabled */
  auto_degrade_enabled: boolean;

  /** Maximum allowed overhead percentage before degradation */
  max_overhead_percent: number;

  /** Metrics window start timestamp (ISO 8601) */
  window_start: string;
}

/**
 * Redux coordination state shape.
 */
export interface CoordinationState {
  /** Current agent health records */
  health: AgentHealthRecord[];

  /** Latest standup result (null if none) */
  latestStandup: StandupResult | null;

  /** Current coordination metrics (null if not loaded) */
  metrics: CoordinationMetrics | null;

  /** Loading state for API calls */
  isLoading: boolean;

  /** Error message (null if no error) */
  error: string | null;

  /** Whether coordination is enabled in backend */
  coordinationEnabled: boolean;
}

/**
 * WebSocket coordination event types.
 * Extends CoordinationEvent with coordination-specific activities.
 */
export type CoordinationEventType =
  | 'HeartbeatResponse'
  | 'HeartbeatTimeout'
  | 'StandupResponse'
  | 'StandupSummary';

/**
 * Heartbeat response event payload.
 */
export interface HeartbeatResponsePayload {
  agent_id: string;
  response_time_ms: number;
  status: AgentHealthStatus;
  timestamp: string;
}

/**
 * Heartbeat timeout event payload.
 */
export interface HeartbeatTimeoutPayload {
  agent_ids: string[];
  consecutive_misses: number;
  timestamp: string;
}

/**
 * Standup response event payload.
 */
export interface StandupResponsePayload {
  agent_id: string;
  response: StandupResponseRecord;
  timestamp: string;
}

/**
 * Standup summary event payload.
 */
export interface StandupSummaryPayload {
  request_id: string;
  summary: string;
  agent_count: number;
  timestamp: string;
}
