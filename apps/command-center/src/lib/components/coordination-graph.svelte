<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { CoordinationEdge } from '$lib/utils/trace-helpers.js';

	let {
		agents,
		edges,
		onnodeclick
	}: {
		agents: string[];
		edges: CoordinationEdge[];
		onnodeclick?: (agentName: string) => void;
	} = $props();

	// ============================================================
	// SVG layout
	// ============================================================

	const SVG_WIDTH = 700;
	const SVG_HEIGHT = 280;
	const NODE_W = 140;
	const NODE_H = 44;
	const NODE_RX = 10;

	interface NodeLayout {
		id: string;
		x: number;
		y: number;
	}

	/**
	 * Simple hierarchical layout:
	 * - Coordinator (most outgoing edges) at top center
	 * - Remaining agents spread across bottom row
	 */
	function computeLayout(agentNames: string[], edgeList: CoordinationEdge[]): NodeLayout[] {
		if (agentNames.length === 0) return [];

		// Count outgoing edges per agent to identify coordinators
		const outDegree = new Map<string, number>();
		for (const name of agentNames) outDegree.set(name, 0);
		for (const edge of edgeList) {
			outDegree.set(edge.from, (outDegree.get(edge.from) ?? 0) + 1);
		}

		// Sort descending by out-degree
		const sorted = [...agentNames].sort(
			(a, b) => (outDegree.get(b) ?? 0) - (outDegree.get(a) ?? 0)
		);

		const coordinator = sorted[0];
		const specialists = sorted.slice(1);

		const layout: NodeLayout[] = [];

		if (specialists.length === 0) {
			// Single agent — center it
			layout.push({
				id: coordinator,
				x: SVG_WIDTH / 2 - NODE_W / 2,
				y: SVG_HEIGHT / 2 - NODE_H / 2
			});
		} else {
			// Coordinator at top center
			layout.push({
				id: coordinator,
				x: SVG_WIDTH / 2 - NODE_W / 2,
				y: 30
			});

			// Specialists spread across bottom
			const gap = Math.min(180, (SVG_WIDTH - 40) / specialists.length);
			const totalWidth = gap * (specialists.length - 1) + NODE_W;
			const startX = (SVG_WIDTH - totalWidth) / 2;

			specialists.forEach((name, i) => {
				layout.push({
					id: name,
					x: startX + i * gap,
					y: SVG_HEIGHT - NODE_H - 30
				});
			});
		}

		return layout;
	}

	const layout = $derived(computeLayout(agents, edges));

	// Map agent name -> layout position for edge drawing
	const posMap = $derived(
		new Map<string, NodeLayout>(layout.map((n) => [n.id, n]))
	);

	// ============================================================
	// Edge path computation (cubic bezier arrow)
	// ============================================================

	interface EdgePath {
		d: string;
		midX: number;
		midY: number;
		key: string;
	}

	function computeEdgePaths(edgeList: CoordinationEdge[], pm: Map<string, NodeLayout>): EdgePath[] {
		return edgeList.map((edge, i) => {
			const from = pm.get(edge.from);
			const to = pm.get(edge.to);
			if (!from || !to) return { d: '', midX: 0, midY: 0, key: String(i) };

			// Connect bottom-center of from to top-center of to (or side if same row)
			const fromX = from.x + NODE_W / 2;
			const fromY = from.y + NODE_H;
			const toX = to.x + NODE_W / 2;
			const toY = to.y;

			// Control points for smooth curve
			const cy = (fromY + toY) / 2;
			const d = `M ${fromX} ${fromY} C ${fromX} ${cy}, ${toX} ${cy}, ${toX} ${toY}`;

			return {
				d,
				midX: (fromX + toX) / 2,
				midY: cy,
				key: `${edge.from}->${edge.to}-${i}`
			};
		});
	}

	const edgePaths = $derived(computeEdgePaths(edges, posMap));

	// ============================================================
	// Hover state
	// ============================================================

	let hoveredNode = $state<string | null>(null);

	// ============================================================
	// Click handler
	// ============================================================

	function handleNodeClick(agentName: string) {
		onnodeclick?.(agentName);
	}
</script>

{#if agents.length === 0 || (agents.length === 1 && edges.length === 0)}
	<!-- Single agent — no coordination -->
	<div
		class="flex items-center justify-center rounded-xl border border-dashed bg-muted/20 py-10 text-center text-sm text-muted-foreground"
	>
		<div>
			<p class="font-medium">Single agent run</p>
			<p class="mt-1 text-xs">No multi-agent coordination detected in this trace.</p>
		</div>
	</div>
{:else}
	<div class="overflow-hidden rounded-xl border bg-card shadow-sm">
		<div class="px-4 py-3 border-b flex items-center gap-2">
			<span class="text-sm font-semibold">Agent Coordination Graph</span>
			<span class="rounded-full bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground">
				{agents.length} agent{agents.length !== 1 ? 's' : ''} · {edges.length} delegation{edges.length !== 1 ? 's' : ''}
			</span>
		</div>

		<svg
			viewBox="0 0 {SVG_WIDTH} {SVG_HEIGHT}"
			class="w-full"
			style="max-height: 300px"
			aria-label="Multi-agent coordination graph"
		>
			<defs>
				<!-- Arrow marker -->
				<marker
					id="arrowhead"
					markerWidth="8"
					markerHeight="6"
					refX="7"
					refY="3"
					orient="auto"
				>
					<polygon points="0 0, 8 3, 0 6" class="fill-sky-500" />
				</marker>

				<!-- Animated dash pattern for edges -->
				<style>
					.edge-path {
						stroke-dasharray: 6 4;
						animation: dashFlow 1.2s linear infinite;
					}
					@keyframes dashFlow {
						to { stroke-dashoffset: -20; }
					}

					.node-rect {
						transition: filter 0.15s ease;
						cursor: pointer;
					}
					.node-rect:hover {
						filter: drop-shadow(0 0 8px rgba(14, 165, 233, 0.6));
					}
				</style>
			</defs>

			<!-- Edge paths -->
			{#each edgePaths as ep (ep.key)}
				{#if ep.d}
					<path
						d={ep.d}
						class="edge-path"
						fill="none"
						stroke="#0ea5e9"
						stroke-width="1.5"
						stroke-opacity="0.7"
						marker-end="url(#arrowhead)"
					/>
				{/if}
			{/each}

			<!-- Agent nodes -->
			{#each layout as node}
				{@const isHovered = hoveredNode === node.id}
				<g
					role="button"
					tabindex="0"
					aria-label="Agent: {node.id}"
					onclick={() => handleNodeClick(node.id)}
					onkeydown={(e) => e.key === 'Enter' && handleNodeClick(node.id)}
					onmouseenter={() => (hoveredNode = node.id)}
					onmouseleave={() => (hoveredNode = null)}
					style="cursor: pointer"
				>
					<!-- Node background -->
					<rect
						x={node.x}
						y={node.y}
						width={NODE_W}
						height={NODE_H}
						rx={NODE_RX}
						ry={NODE_RX}
						class="node-rect"
						fill={isHovered ? '#0f172a' : '#1e293b'}
						stroke={isHovered ? '#38bdf8' : '#334155'}
						stroke-width={isHovered ? 2 : 1.5}
					/>

					<!-- Gradient accent top bar -->
					<rect
						x={node.x}
						y={node.y}
						width={NODE_W}
						height={4}
						rx={NODE_RX}
						ry={NODE_RX}
						fill="url(#nodeGradient)"
					/>

					<!-- Agent name text -->
					<text
						x={node.x + NODE_W / 2}
						y={node.y + NODE_H / 2 + 5}
						text-anchor="middle"
						dominant-baseline="middle"
						font-size="12"
						font-weight="600"
						font-family="system-ui, sans-serif"
						fill={isHovered ? '#38bdf8' : '#e2e8f0'}
					>
						{node.id.length > 14 ? node.id.slice(0, 13) + '…' : node.id}
					</text>
				</g>
			{/each}

			<!-- Gradient definition for node accent bar -->
			<defs>
				<linearGradient id="nodeGradient" x1="0%" y1="0%" x2="100%" y2="0%">
					<stop offset="0%" stop-color="#14b8a6" />
					<stop offset="100%" stop-color="#0ea5e9" />
				</linearGradient>
			</defs>
		</svg>

		<!-- Tooltip (shown below graph when node hovered) -->
		{#if hoveredNode}
			<div class="border-t px-4 py-2 text-xs text-muted-foreground">
				<span class="font-medium text-foreground">{hoveredNode}</span>
				— click to view this agent's trace
			</div>
		{/if}
	</div>
{/if}
