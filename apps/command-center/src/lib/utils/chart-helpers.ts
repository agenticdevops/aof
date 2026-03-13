import type { RunCostDetail, CostSummary } from '$lib/api/types.js';

// ============================================================
// Color palette — slate blues, teals, purples, ambers
// ============================================================

const PALETTE_MAIN = [
	'#0ea5e9', // sky-500
	'#14b8a6', // teal-500
	'#8b5cf6', // violet-500
	'#f59e0b', // amber-500
	'#06b6d4', // cyan-500
	'#a855f7', // purple-500
	'#10b981', // emerald-500
	'#f97316', // orange-500
	'#3b82f6', // blue-500
	'#ec4899', // pink-500
	'#84cc16', // lime-500
	'#6366f1'  // indigo-500
];

const PALETTE_HOVER = [
	'#38bdf8', // sky-400
	'#2dd4bf', // teal-400
	'#a78bfa', // violet-400
	'#fbbf24', // amber-400
	'#22d3ee', // cyan-400
	'#c084fc', // purple-400
	'#34d399', // emerald-400
	'#fb923c', // orange-400
	'#60a5fa', // blue-400
	'#f472b6', // pink-400
	'#a3e635', // lime-400
	'#818cf8'  // indigo-400
];

const PALETTE_BG = [
	'rgba(14,165,233,0.15)',
	'rgba(20,184,166,0.15)',
	'rgba(139,92,246,0.15)',
	'rgba(245,158,11,0.15)',
	'rgba(6,182,212,0.15)',
	'rgba(168,85,247,0.15)',
	'rgba(16,185,129,0.15)',
	'rgba(249,115,22,0.15)',
	'rgba(59,130,246,0.15)',
	'rgba(236,72,153,0.15)',
	'rgba(132,204,22,0.15)',
	'rgba(99,102,241,0.15)'
];

/** Returns the main hex color for an agent by palette index. */
export function getAgentColor(index: number): string {
	return PALETTE_MAIN[index % PALETTE_MAIN.length];
}

/** Returns the hover hex color for an agent by palette index. */
export function getAgentHoverColor(index: number): string {
	return PALETTE_HOVER[index % PALETTE_HOVER.length];
}

/** Returns the semi-transparent background color for gradient fills. */
export function getAgentBgColor(index: number): string {
	return PALETTE_BG[index % PALETTE_BG.length];
}

/** Returns all palette main colors (for multi-series charts). */
export function getAllColors(): string[] {
	return [...PALETTE_MAIN];
}

/** Returns all palette bg colors (for fills). */
export function getAllBgColors(): string[] {
	return [...PALETTE_BG];
}

// ============================================================
// Formatting helpers
// ============================================================

/**
 * Formats a USD cost value. Shows micro-cents for very small values,
 * cents for small values, and dollars for larger values.
 */
export function formatCurrency(usd: number): string {
	if (usd === 0) return '$0.00';
	if (usd < 0.001) return `$${usd.toFixed(6)}`;
	if (usd < 0.01) return `$${usd.toFixed(4)}`;
	if (usd < 1) return `$${usd.toFixed(4)}`;
	if (usd < 100) return `$${usd.toFixed(2)}`;
	return `$${usd.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

/**
 * Formats a token count. Displays as K/M suffixed for large values.
 */
export function formatTokens(count: number): string {
	if (count < 1000) return count.toString();
	if (count < 1_000_000) return `${(count / 1000).toFixed(1)}K`;
	return `${(count / 1_000_000).toFixed(1)}M`;
}

// ============================================================
// Data aggregation helpers
// ============================================================

/**
 * Groups RunCostDetail[] by calendar date (YYYY-MM-DD from started_at),
 * sums costs per day, and returns sorted chronologically.
 */
export function aggregateCostsByDate(
	runs: RunCostDetail[]
): { date: string; cost: number }[] {
	const map = new Map<string, number>();

	for (const run of runs) {
		// Extract YYYY-MM-DD from ISO timestamp
		const date = run.started_at.slice(0, 10);
		map.set(date, (map.get(date) ?? 0) + run.cost_usd);
	}

	return Array.from(map.entries())
		.sort(([a], [b]) => a.localeCompare(b))
		.map(([date, cost]) => ({ date, cost }));
}

/**
 * Computes cost distribution across agents including percentage share.
 * Sorted descending by cost.
 */
export function aggregateCostsByAgent(
	summaries: CostSummary[]
): { agent: string; cost: number; percentage: number }[] {
	const total = summaries.reduce((sum, s) => sum + s.total_cost_usd, 0);

	return summaries
		.map((s) => ({
			agent: s.agent_name,
			cost: s.total_cost_usd,
			percentage: total > 0 ? (s.total_cost_usd / total) * 100 : 0
		}))
		.sort((a, b) => b.cost - a.cost);
}

// ============================================================
// Chart.js theme helpers — adapts colors to dark/light mode
// ============================================================

export interface ChartTheme {
	gridColor: string;
	tickColor: string;
	legendColor: string;
	backgroundColor: string;
}

/**
 * Returns chart colors appropriate for the current dark/light theme.
 * Reads `.dark` class from document root.
 */
export function getChartTheme(): ChartTheme {
	const isDark =
		typeof document !== 'undefined' && document.documentElement.classList.contains('dark');

	return isDark
		? {
				gridColor: 'rgba(255,255,255,0.08)',
				tickColor: '#94a3b8',
				legendColor: '#cbd5e1',
				backgroundColor: '#0f172a'
			}
		: {
				gridColor: 'rgba(0,0,0,0.06)',
				tickColor: '#64748b',
				legendColor: '#334155',
				backgroundColor: '#ffffff'
			};
}
