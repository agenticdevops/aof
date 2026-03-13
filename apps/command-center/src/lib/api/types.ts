// ============================================================
// Agent types
// ============================================================

export interface Agent {
	name: string;
	status: AgentStatus;
	model: ModelConfig;
	mode: AgentMode;
	triggers: TriggerConfig[];
	skills: string[];
	tools: string[];
	budget?: BudgetConfig;
	telemetry?: TelemetryConfig;
	namespace?: string;
	version?: string;
}

export type AgentStatus = 'running' | 'idle' | 'error' | 'scheduled';

export type AgentMode = 'manual' | 'semi-autonomous' | 'autonomous';

export interface ModelConfig {
	preferred: string;
	fallback?: string;
}

export interface TriggerConfig {
	type: string;
	expression?: string;
	channel?: string;
}

export interface BudgetConfig {
	daily_limit?: number;
	max_tokens_per_run?: number;
}

export interface TelemetryConfig {
	enabled: boolean;
	export_endpoint?: string;
}

// ============================================================
// Run types
// ============================================================

export interface AgentRun {
	run_id: string;
	agent_name: string;
	status: RunStatus;
	started_at: string;
	completed_at?: string;
	duration_ms?: number;
	trigger_source: string;
	cost?: RunCost;
}

export type RunStatus = 'running' | 'completed' | 'failed' | 'stopped' | 'paused';

export interface RunCost {
	input_tokens: number;
	output_tokens: number;
	total_cost_usd: number;
}

// ============================================================
// Cost types
// ============================================================

export interface CostSummary {
	agent_name: string;
	total_runs: number;
	total_input_tokens: number;
	total_output_tokens: number;
	total_cost_usd: number;
	daily_budget?: number;
	budget_remaining?: number;
}

export interface RunCostDetail {
	run_id: string;
	started_at: string;
	input_tokens: number;
	output_tokens: number;
	cost_usd: number;
	model_used: string;
}

// ============================================================
// Trace types (matching agentix-core SpanRecord)
// ============================================================

export interface SpanRecord {
	span_id: string;
	parent_span_id?: string;
	trace_id: string;
	name: string;
	kind: SpanKind;
	status: SpanStatus;
	start_time: string;
	end_time?: string;
	duration_ms?: number;
	attributes: Record<string, string>;
}

export type SpanKind =
	| 'run'
	| 'iteration'
	| 'llm_call'
	| 'tool_call'
	| 'research'
	| 'memory_recall'
	| 'approval_wait';

export type SpanStatus = 'ok' | 'error' | 'unset';

// ============================================================
// Approval types
// ============================================================

export interface ApprovalRequest {
	id: string;
	agent_name: string;
	run_id: string;
	action_description: string;
	status: ApprovalStatus;
	requested_at: string;
	decided_at?: string;
	decided_by?: string;
}

export type ApprovalStatus = 'pending' | 'approved' | 'denied' | 'expired';

// ============================================================
// Audit types
// ============================================================

export interface AuditEntry {
	timestamp: string;
	agent_name: string;
	action: string;
	actor: string;
	outcome: string;
	details?: Record<string, string>;
}

// ============================================================
// Channel types
// ============================================================

export interface ChannelInfo {
	platform: string;
	agent_name: string;
	channel_id: string;
	active: boolean;
}

// ============================================================
// Structured log entry (from /structured-logs endpoint)
// ============================================================

export interface StructuredLogEntry {
	timestamp: string;
	level: string;
	message: string;
	span_id?: string;
	trace_id?: string;
	fields?: Record<string, string>;
}
