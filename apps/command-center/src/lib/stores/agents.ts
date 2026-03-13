import { writable, get } from 'svelte/store';
import { api } from '$lib/api/client.js';
import { wsEvents } from '$lib/stores/websocket.js';
import type { Agent, AgentRun } from '$lib/api/types.js';

// ============================================================
// Stores
// ============================================================

export const agents = writable<Agent[]>([]);
export const agentsError = writable<string | null>(null);
export const agentsLoading = writable<boolean>(false);

export const selectedAgent = writable<Agent | null>(null);
export const agentDetailError = writable<string | null>(null);
export const agentDetailLoading = writable<boolean>(false);

export const agentRuns = writable<AgentRun[]>([]);
export const agentRunsError = writable<string | null>(null);
export const agentRunsLoading = writable<boolean>(false);

// Track which agent's runs are currently displayed (for reactive refresh)
let currentRunsAgent: string | null = null;

// ============================================================
// Actions
// ============================================================

/**
 * Fetch all agents from the gateway REST API and update the store.
 */
export async function loadAgents(): Promise<void> {
	agentsLoading.set(true);
	agentsError.set(null);
	try {
		const list = await api.agents.list();
		agents.set(list);
	} catch (err) {
		agentsError.set(err instanceof Error ? err.message : 'Failed to load agents');
	} finally {
		agentsLoading.set(false);
	}
}

/**
 * Fetch a single agent by name and update the selectedAgent store.
 */
export async function loadAgent(name: string): Promise<void> {
	agentDetailLoading.set(true);
	agentDetailError.set(null);
	try {
		const agent = await api.agents.get(name);
		selectedAgent.set(agent);
	} catch (err) {
		agentDetailError.set(err instanceof Error ? err.message : 'Failed to load agent');
	} finally {
		agentDetailLoading.set(false);
	}
}

/**
 * Fetch runs for a specific agent and update the agentRuns store.
 */
export async function loadAgentRuns(name: string): Promise<void> {
	currentRunsAgent = name;
	agentRunsLoading.set(true);
	agentRunsError.set(null);
	try {
		const runs = await api.runs.listForAgent(name);
		agentRuns.set(runs);
	} catch (err) {
		agentRunsError.set(err instanceof Error ? err.message : 'Failed to load runs');
	} finally {
		agentRunsLoading.set(false);
	}
}

// ============================================================
// WebSocket reactive updates
// ============================================================

wsEvents.subscribe((event) => {
	if (!event) return;

	if (event.type === 'agent_status') {
		// Update the matching agent's status in-place
		agents.update((list) =>
			list.map((a) =>
				a.name === event.agent_name ? { ...a, status: event.status } : a
			)
		);
		// Also update selectedAgent if it's the same agent
		const selected = get(selectedAgent);
		if (selected && selected.name === event.agent_name) {
			selectedAgent.set({ ...selected, status: event.status });
		}
	} else if (event.type === 'run_completed') {
		// Reload runs if we are currently viewing this agent's runs
		if (currentRunsAgent && currentRunsAgent === event.agent_name) {
			loadAgentRuns(currentRunsAgent);
		}
	} else if (event.type === 'run_started') {
		// Mark the agent as running when a run starts
		agents.update((list) =>
			list.map((a) =>
				a.name === event.agent_name ? { ...a, status: 'running' } : a
			)
		);
	}
});
