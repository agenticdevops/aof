import { get } from 'svelte/store';
import { gatewayUrl } from '$lib/stores/gateway.js';
import type {
	Agent,
	AgentRun,
	CostSummary,
	RunCostDetail,
	SpanRecord,
	StructuredLogEntry,
	ApprovalRequest,
	AuditEntry,
	ChannelInfo
} from './types.js';

// ============================================================
// Internal helpers
// ============================================================

function baseUrl(): string {
	return get(gatewayUrl).replace(/\/$/, '');
}

async function request<T>(
	method: string,
	path: string,
	body?: unknown
): Promise<T> {
	const url = `${baseUrl()}${path}`;
	const init: RequestInit = {
		method,
		headers: { 'Content-Type': 'application/json' }
	};
	if (body !== undefined) {
		init.body = JSON.stringify(body);
	}

	const response = await fetch(url, init);

	if (!response.ok) {
		let message: string;
		try {
			message = await response.text();
		} catch {
			message = response.statusText;
		}
		throw new Error(`Gateway error ${response.status}: ${message}`);
	}

	// For void responses (204 No Content)
	if (response.status === 204 || response.headers.get('content-length') === '0') {
		return undefined as unknown as T;
	}

	return response.json() as Promise<T>;
}

async function del(path: string): Promise<void> {
	const url = `${baseUrl()}${path}`;
	const response = await fetch(url, {
		method: 'DELETE',
		headers: { 'Content-Type': 'application/json' }
	});

	if (!response.ok) {
		let message: string;
		try {
			message = await response.text();
		} catch {
			message = response.statusText;
		}
		throw new Error(`Gateway error ${response.status}: ${message}`);
	}
}

// ============================================================
// API client
// ============================================================

export const api = {
	// ----------------------------------------------------------
	// Agents
	// ----------------------------------------------------------
	agents: {
		/** List all registered agents. */
		list(): Promise<Agent[]> {
			return request<Agent[]>('GET', '/api/v1/agents');
		},

		/** Get details for a specific agent. */
		get(name: string): Promise<Agent> {
			return request<Agent>('GET', `/api/v1/agents/${encodeURIComponent(name)}`);
		},

		/** Register a new agent from YAML content. */
		register(yaml: string): Promise<Agent> {
			return request<Agent>('POST', '/api/v1/agents', { yaml });
		},

		/** Update an existing agent from YAML content. */
		update(name: string, yaml: string): Promise<Agent> {
			return request<Agent>('PUT', `/api/v1/agents/${encodeURIComponent(name)}`, { yaml });
		},

		/**
		 * Run an agent. Returns a Server-Sent Events stream.
		 * The caller is responsible for closing the EventSource when done.
		 */
		run(name: string, message?: string): EventSource {
			const params = new URLSearchParams();
			if (message) params.set('message', message);
			const query = params.size > 0 ? `?${params.toString()}` : '';
			return new EventSource(`${baseUrl()}/api/v1/agents/${encodeURIComponent(name)}/run${query}`);
		},

		/** Trigger an agent via a named trigger. */
		trigger(name: string, payload?: unknown): Promise<void> {
			return request<void>('POST', `/api/v1/agents/${encodeURIComponent(name)}/trigger`, payload);
		},

		/** Delegate work to another agent. */
		delegate(name: string, payload?: unknown): Promise<void> {
			return request<void>('POST', `/api/v1/agents/${encodeURIComponent(name)}/delegate`, payload);
		}
	},

	// ----------------------------------------------------------
	// Runs
	// ----------------------------------------------------------
	runs: {
		/** List all runs, optionally filtered by agent name. */
		list(agentName?: string): Promise<AgentRun[]> {
			const params = agentName ? `?agent=${encodeURIComponent(agentName)}` : '';
			return request<AgentRun[]>('GET', `/api/v1/runs${params}`);
		},

		/** List all runs for a specific agent. */
		listForAgent(name: string): Promise<AgentRun[]> {
			return request<AgentRun[]>('GET', `/api/v1/agents/${encodeURIComponent(name)}/runs`);
		},

		/** Get details for a specific run. */
		get(name: string, runId: string): Promise<AgentRun> {
			return request<AgentRun>(
				'GET',
				`/api/v1/agents/${encodeURIComponent(name)}/runs/${encodeURIComponent(runId)}`
			);
		},

		/** Get the log output for a run. */
		getLogs(name: string, runId: string): Promise<string> {
			return request<string>(
				'GET',
				`/api/v1/agents/${encodeURIComponent(name)}/runs/${encodeURIComponent(runId)}/logs`
			);
		},

		/** Stop a running agent run. */
		async stop(name: string, runId: string): Promise<void> {
			await del(
				`/api/v1/agents/${encodeURIComponent(name)}/runs/${encodeURIComponent(runId)}`
			);
		}
	},

	// ----------------------------------------------------------
	// Costs
	// ----------------------------------------------------------
	costs: {
		/** List cost summaries for all agents. */
		listAll(): Promise<CostSummary[]> {
			return request<CostSummary[]>('GET', '/api/v1/costs');
		},

		/** Get cost summary for a specific agent. */
		getAgent(name: string): Promise<CostSummary> {
			return request<CostSummary>('GET', `/api/v1/costs/agents/${encodeURIComponent(name)}`);
		},

		/** Get per-run cost breakdown for a specific agent. */
		getAgentRuns(name: string): Promise<RunCostDetail[]> {
			return request<RunCostDetail[]>(
				'GET',
				`/api/v1/costs/agents/${encodeURIComponent(name)}/runs`
			);
		}
	},

	// ----------------------------------------------------------
	// Traces
	// ----------------------------------------------------------
	traces: {
		/** Get trace spans for a specific run. */
		getSpans(name: string, runId: string): Promise<SpanRecord[]> {
			return request<SpanRecord[]>(
				'GET',
				`/api/v1/agents/${encodeURIComponent(name)}/runs/${encodeURIComponent(runId)}/trace`
			);
		},

		/** Get structured logs for a specific run. */
		getLogs(name: string, runId: string): Promise<StructuredLogEntry[]> {
			return request<StructuredLogEntry[]>(
				'GET',
				`/api/v1/agents/${encodeURIComponent(name)}/runs/${encodeURIComponent(runId)}/structured-logs`
			);
		}
	},

	// ----------------------------------------------------------
	// Memory
	// ----------------------------------------------------------
	memory: {
		/** List memory entries for an agent. */
		list(name: string): Promise<unknown[]> {
			return request<unknown[]>('GET', `/api/v1/agents/${encodeURIComponent(name)}/memory`);
		},

		/** Clear all memory for an agent. */
		async clear(name: string): Promise<void> {
			await del(`/api/v1/agents/${encodeURIComponent(name)}/memory`);
		}
	},

	// ----------------------------------------------------------
	// Approvals
	// ----------------------------------------------------------
	approvals: {
		/** List all pending approval requests. */
		list(): Promise<ApprovalRequest[]> {
			return request<ApprovalRequest[]>('GET', '/api/v1/approvals');
		},

		/** Get details for a specific approval request. */
		get(id: string): Promise<ApprovalRequest> {
			return request<ApprovalRequest>('GET', `/api/v1/approvals/${encodeURIComponent(id)}`);
		},

		/** Approve a pending request. */
		approve(id: string, approver?: string): Promise<void> {
			return request<void>('POST', `/api/v1/approvals/${encodeURIComponent(id)}/approve`, {
				approver
			});
		},

		/** Deny a pending request. */
		deny(id: string, approver?: string, reason?: string): Promise<void> {
			return request<void>('POST', `/api/v1/approvals/${encodeURIComponent(id)}/deny`, {
				approver,
				reason
			});
		}
	},

	// ----------------------------------------------------------
	// Audit
	// ----------------------------------------------------------
	audit: {
		/** Get audit trail for a specific agent. */
		getAgent(name: string): Promise<AuditEntry[]> {
			return request<AuditEntry[]>(
				'GET',
				`/api/v1/agents/${encodeURIComponent(name)}/audit`
			);
		},

		/** Get security audit events. */
		getSecurity(): Promise<AuditEntry[]> {
			return request<AuditEntry[]>('GET', '/api/v1/audit/security');
		}
	},

	// ----------------------------------------------------------
	// Channels
	// ----------------------------------------------------------
	channels: {
		/** List all configured channels. */
		list(): Promise<ChannelInfo[]> {
			return request<ChannelInfo[]>('GET', '/api/v1/channels');
		},

		/** Send a notification via a channel. */
		notify(target: unknown): Promise<void> {
			return request<void>('POST', '/api/v1/notify', target);
		}
	},

	// ----------------------------------------------------------
	// Health
	// ----------------------------------------------------------
	health: {
		/** Check gateway health. Returns true if healthy. */
		async check(): Promise<boolean> {
			try {
				const response = await fetch(`${baseUrl()}/healthz`);
				return response.ok;
			} catch {
				return false;
			}
		}
	}
};
