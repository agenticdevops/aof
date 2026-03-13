import { writable, derived } from 'svelte/store';
import { api } from '$lib/api/client.js';
import type { CostSummary, RunCostDetail } from '$lib/api/types.js';

// ============================================================
// Date range store — default last 30 days
// ============================================================

function defaultDateRange(): { start: Date; end: Date } {
	const end = new Date();
	const start = new Date();
	start.setDate(start.getDate() - 30);
	return { start, end };
}

export const dateRange = writable<{ start: Date; end: Date }>(defaultDateRange());

// ============================================================
// Cost summaries (all agents)
// ============================================================

export const costSummaries = writable<CostSummary[]>([]);

// ============================================================
// Drill-down state
// ============================================================

/** Currently selected agent name for drill-down (null = none selected). */
export const selectedAgent = writable<string | null>(null);

/** Per-run cost detail for the selected agent. */
export const agentRunCosts = writable<RunCostDetail[]>([]);

// ============================================================
// Error state
// ============================================================

export const costError = writable<string | null>(null);
export const costLoading = writable<boolean>(false);
export const agentCostLoading = writable<boolean>(false);

// ============================================================
// Derived: filter agentRunCosts by current dateRange
// ============================================================

export const filteredRunCosts = derived(
	[agentRunCosts, dateRange],
	([$runs, $range]) => {
		return $runs.filter((run) => {
			const runDate = new Date(run.started_at);
			return runDate >= $range.start && runDate <= $range.end;
		});
	}
);

// ============================================================
// Actions
// ============================================================

/**
 * Loads cost summaries for all agents from the gateway.
 */
export async function loadCosts(): Promise<void> {
	costLoading.set(true);
	costError.set(null);

	try {
		const data = await api.costs.listAll();
		costSummaries.set(data);
	} catch (err) {
		const message = err instanceof Error ? err.message : 'Failed to load costs';
		costError.set(message);
		costSummaries.set([]);
	} finally {
		costLoading.set(false);
	}
}

/**
 * Loads per-run cost details for a specific agent and sets selectedAgent.
 */
export async function loadAgentCosts(name: string): Promise<void> {
	selectedAgent.set(name);
	agentCostLoading.set(true);

	try {
		const data = await api.costs.getAgentRuns(name);
		agentRunCosts.set(data);
	} catch (err) {
		const message = err instanceof Error ? err.message : `Failed to load costs for ${name}`;
		costError.set(message);
		agentRunCosts.set([]);
	} finally {
		agentCostLoading.set(false);
	}
}

/**
 * Clears the drill-down selection.
 */
export function clearSelectedAgent(): void {
	selectedAgent.set(null);
	agentRunCosts.set([]);
}
