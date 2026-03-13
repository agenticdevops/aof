import { writable, derived } from 'svelte/store';
import { api } from '$lib/api/client.js';
import type { SpanRecord } from '$lib/api/types.js';
import {
	buildSpanTree,
	calculateTimeScale,
	detectCoordination,
	type SpanNode,
	type TimeScale,
	type CoordinationEdge
} from '$lib/utils/trace-helpers.js';

// ============================================================
// Primitive stores
// ============================================================

/** All spans for the currently loaded trace. */
export const traceSpans = writable<SpanRecord[]>([]);

/** Currently selected span (expanded inline detail). */
export const selectedSpan = writable<SpanRecord | null>(null);

/** Loading state for trace fetch. */
export const traceLoading = writable<boolean>(false);

/** Error message if trace fetch fails. */
export const traceError = writable<string | null>(null);

// ============================================================
// Derived stores
// ============================================================

/** Tree of SpanNode built from flat traceSpans. */
export const spanTree = derived<typeof traceSpans, SpanNode[]>(
	traceSpans,
	($spans) => buildSpanTree($spans)
);

/** TimeScale (startMs, endMs, totalMs) for positioning bars. */
export const timeScale = derived<typeof traceSpans, TimeScale>(
	traceSpans,
	($spans) => calculateTimeScale($spans)
);

/** Multi-agent coordination edges detected from spans. */
export const coordinationEdges = derived<typeof traceSpans, CoordinationEdge[]>(
	traceSpans,
	($spans) => detectCoordination($spans)
);

// ============================================================
// Actions
// ============================================================

/**
 * Fetches trace spans for a specific agent run and populates the store.
 * Clears any previous trace data and selected span before loading.
 */
export async function loadTrace(agentName: string, runId: string): Promise<void> {
	traceLoading.set(true);
	traceError.set(null);
	selectedSpan.set(null);
	traceSpans.set([]);

	try {
		const spans = await api.traces.getSpans(agentName, runId);
		traceSpans.set(spans);
	} catch (err) {
		traceError.set(err instanceof Error ? err.message : 'Failed to load trace');
	} finally {
		traceLoading.set(false);
	}
}

/**
 * Clears all trace state (call when navigating away).
 */
export function clearTrace(): void {
	traceSpans.set([]);
	selectedSpan.set(null);
	traceError.set(null);
	traceLoading.set(false);
}
