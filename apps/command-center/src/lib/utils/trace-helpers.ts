import type { SpanKind, SpanRecord } from '$lib/api/types.js';

// ============================================================
// Tree structure
// ============================================================

export interface SpanNode {
	span: SpanRecord;
	children: SpanNode[];
	depth: number;
}

// ============================================================
// Time scale
// ============================================================

export interface TimeScale {
	startMs: number;
	endMs: number;
	totalMs: number;
}

// ============================================================
// Coordination edge
// ============================================================

export interface CoordinationEdge {
	from: string;
	to: string;
	runId?: string;
}

// ============================================================
// buildSpanTree — convert flat SpanRecord[] into a tree
// ============================================================

/**
 * Converts a flat array of SpanRecord into a tree of SpanNode.
 * Root spans have no parent_span_id. Sorts children by start_time.
 */
export function buildSpanTree(spans: SpanRecord[]): SpanNode[] {
	const nodeMap = new Map<string, SpanNode>();

	// Create all nodes first
	for (const span of spans) {
		nodeMap.set(span.span_id, { span, children: [], depth: 0 });
	}

	const roots: SpanNode[] = [];

	// Wire parent-child relationships
	for (const span of spans) {
		const node = nodeMap.get(span.span_id)!;
		if (span.parent_span_id && nodeMap.has(span.parent_span_id)) {
			const parent = nodeMap.get(span.parent_span_id)!;
			parent.children.push(node);
		} else {
			roots.push(node);
		}
	}

	// Assign depths and sort children
	function assignDepths(nodes: SpanNode[], depth: number): void {
		for (const node of nodes) {
			node.depth = depth;
			// Sort children by start_time
			node.children.sort(
				(a, b) =>
					new Date(a.span.start_time).getTime() - new Date(b.span.start_time).getTime()
			);
			assignDepths(node.children, depth + 1);
		}
	}

	// Sort roots by start_time
	roots.sort(
		(a, b) =>
			new Date(a.span.start_time).getTime() - new Date(b.span.start_time).getTime()
	);
	assignDepths(roots, 0);

	return roots;
}

// ============================================================
// calculateTimeScale — find timeline bounds across all spans
// ============================================================

/**
 * Finds the earliest start_time and latest end_time across all spans,
 * returning a TimeScale for positioning bars on the waterfall.
 */
export function calculateTimeScale(spans: SpanRecord[]): TimeScale {
	if (spans.length === 0) {
		return { startMs: 0, endMs: 0, totalMs: 0 };
	}

	const now = Date.now();
	let startMs = Infinity;
	let endMs = -Infinity;

	for (const span of spans) {
		const s = new Date(span.start_time).getTime();
		startMs = Math.min(startMs, s);

		if (span.end_time) {
			const e = new Date(span.end_time).getTime();
			endMs = Math.max(endMs, e);
		} else {
			// Still running — extend to now
			endMs = Math.max(endMs, now);
		}
	}

	if (!isFinite(startMs)) startMs = 0;
	if (!isFinite(endMs)) endMs = startMs;

	const totalMs = Math.max(endMs - startMs, 1);
	return { startMs, endMs, totalMs };
}

// ============================================================
// spanPosition — compute left offset + width as percentages
// ============================================================

/**
 * Returns the left offset and width of a span bar as percentages
 * of the total timeline duration.
 */
export function spanPosition(
	span: SpanRecord,
	scale: TimeScale
): { leftPercent: number; widthPercent: number } {
	if (scale.totalMs <= 0) return { leftPercent: 0, widthPercent: 100 };

	const spanStart = new Date(span.start_time).getTime();
	const spanEnd = span.end_time ? new Date(span.end_time).getTime() : scale.endMs;

	const leftPercent = Math.max(0, ((spanStart - scale.startMs) / scale.totalMs) * 100);
	const widthPercent = Math.max(0.5, ((spanEnd - spanStart) / scale.totalMs) * 100);

	return { leftPercent, widthPercent };
}

// ============================================================
// spanKindIcon — map SpanKind to Lucide icon name
// ============================================================

/**
 * Maps span kinds to Lucide icon component names.
 */
export function spanKindIcon(kind: SpanKind): string {
	const iconMap: Record<SpanKind, string> = {
		run: 'PlayCircle',
		iteration: 'RotateCw',
		llm_call: 'Brain',
		tool_call: 'Wrench',
		research: 'Search',
		memory_recall: 'Database',
		approval_wait: 'ShieldCheck'
	};
	return iconMap[kind] ?? 'Circle';
}

// ============================================================
// spanKindColor — map SpanKind to color hex
// ============================================================

/**
 * Maps span kinds to hex colors for use in SVG/CSS backgrounds.
 */
export function spanKindColor(kind: SpanKind): string {
	const colorMap: Record<SpanKind, string> = {
		run: '#64748b',        // slate-500
		iteration: '#3b82f6',  // blue-500
		llm_call: '#a855f7',   // purple-500
		tool_call: '#14b8a6',  // teal-500
		research: '#f59e0b',   // amber-500
		memory_recall: '#10b981', // emerald-500
		approval_wait: '#f97316'  // orange-500
	};
	return colorMap[kind] ?? '#94a3b8';
}

/**
 * Maps span kinds to Tailwind bg color classes (for DOM elements).
 */
export function spanKindBgClass(kind: SpanKind): string {
	const classMap: Record<SpanKind, string> = {
		run: 'bg-slate-500',
		iteration: 'bg-blue-500',
		llm_call: 'bg-purple-500',
		tool_call: 'bg-teal-500',
		research: 'bg-amber-500',
		memory_recall: 'bg-emerald-500',
		approval_wait: 'bg-orange-500'
	};
	return classMap[kind] ?? 'bg-slate-400';
}

// ============================================================
// detectCoordination — find multi-agent delegation edges
// ============================================================

/**
 * Scans spans for delegation patterns (tool_call spans with delegate_to
 * or target_agent attributes) and returns edges for the coordination graph.
 */
export function detectCoordination(spans: SpanRecord[]): CoordinationEdge[] {
	const edges: CoordinationEdge[] = [];
	const seen = new Set<string>();

	for (const span of spans) {
		if (span.kind === 'tool_call') {
			const target =
				span.attributes['delegate_to'] ??
				span.attributes['target_agent'] ??
				null;

			if (target) {
				const from =
					span.attributes['agent_name'] ??
					span.attributes['from_agent'] ??
					'unknown';
				const runId = span.attributes['run_id'] ?? undefined;
				const key = `${from}->${target}`;

				if (!seen.has(key)) {
					seen.add(key);
					edges.push({ from, to: target, runId });
				}
			}
		}
	}

	return edges;
}

// ============================================================
// Flatten tree back to ordered list (for rendering)
// ============================================================

/**
 * Flattens a SpanNode tree to an in-order array for rendering.
 */
export function flattenTree(nodes: SpanNode[]): SpanNode[] {
	const result: SpanNode[] = [];
	function walk(n: SpanNode[]) {
		for (const node of n) {
			result.push(node);
			walk(node.children);
		}
	}
	walk(nodes);
	return result;
}

// ============================================================
// Duration formatting helper
// ============================================================

/**
 * Formats a millisecond duration to a human-readable string.
 */
export function formatDuration(ms: number): string {
	if (ms < 1) return '<1ms';
	if (ms < 1000) return `${Math.round(ms)}ms`;
	if (ms < 60_000) return `${(ms / 1000).toFixed(2)}s`;
	const mins = Math.floor(ms / 60_000);
	const secs = ((ms % 60_000) / 1000).toFixed(0);
	return `${mins}m ${secs}s`;
}
