<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import {
		Activity,
		ArrowLeft,
		RefreshCw,
		AlertCircle,
		CheckCircle,
		Clock,
		Layers,
		Network
	} from 'lucide-svelte';
	import TraceWaterfall from '$lib/components/trace-waterfall.svelte';
	import CoordinationGraph from '$lib/components/coordination-graph.svelte';
	import {
		traceSpans,
		spanTree,
		timeScale,
		coordinationEdges,
		traceLoading,
		traceError,
		loadTrace,
		clearTrace
	} from '$lib/stores/traces.js';
	import { formatDuration } from '$lib/utils/trace-helpers.js';
	import { api } from '$lib/api/client.js';
	import type { AgentRun } from '$lib/api/types.js';

	// Route params (guaranteed to exist by SvelteKit route matching)
	const agentName = $derived(page.params.agent ?? '');
	const runId = $derived(page.params.runId ?? '');

	// Run metadata (fetched separately)
	let run = $state<AgentRun | null>(null);
	let runLoading = $state(true);

	// ============================================================
	// Derived: all agents involved (from spans + edges)
	// ============================================================

	const involvedAgents = $derived.by(() => {
		const names = new Set<string>();
		// Try attributes for agent name
		for (const span of $traceSpans) {
			const n = span.attributes['agent_name'];
			if (n) names.add(n);
		}
		// Add agents from edges
		for (const edge of $coordinationEdges) {
			names.add(edge.from);
			names.add(edge.to);
		}
		// If nothing found, use the route agent name
		if (names.size === 0) names.add(agentName);
		return [...names];
	});

	// ============================================================
	// Derived: run summary stats from spans
	// ============================================================

	const spanCount = $derived($traceSpans.length);

	const rootSpan = $derived($traceSpans.find((s) => !s.parent_span_id));

	const overallStatus = $derived.by(() => {
		if ($traceSpans.some((s) => s.status === 'error')) return 'error';
		if ($traceSpans.every((s) => s.status === 'ok')) return 'ok';
		return 'unset';
	});

	const totalDurationMs = $derived.by(() => {
		if (run?.duration_ms != null) return run.duration_ms;
		return $timeScale.totalMs;
	});

	// ============================================================
	// Load data
	// ============================================================

	async function load() {
		runLoading = true;
		try {
			run = await api.runs.get(agentName, runId);
		} catch {
			run = null;
		} finally {
			runLoading = false;
		}
		await loadTrace(agentName, runId);
	}

	// ============================================================
	// Coordination graph navigation
	// ============================================================

	function handleNodeClick(clickedAgent: string) {
		// Navigate to that agent's trace (we don't know the runId so go to list)
		goto(`/traces?agent=${encodeURIComponent(clickedAgent)}`);
	}

	// ============================================================
	// Lifecycle
	// ============================================================

	onMount(() => {
		load();
	});

	onDestroy(() => {
		clearTrace();
	});
</script>

<svelte:head>
	<title>Trace: {agentName}/{runId} — OpenAgentiX</title>
</svelte:head>

<div class="p-6 space-y-6">
	<!-- ======================================================== -->
	<!-- Back link + header                                       -->
	<!-- ======================================================== -->
	<div class="flex items-start gap-4">
		<a
			href="/traces"
			class="mt-1 flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
		>
			<ArrowLeft class="h-4 w-4" />
			Traces
		</a>

		<div class="flex items-center gap-3 flex-1 min-w-0">
			<div
				class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-gradient-to-br from-violet-500 to-sky-500 shadow-sm"
			>
				<Activity class="h-5 w-5 text-white" />
			</div>
			<div class="min-w-0">
				<h1 class="text-xl font-bold tracking-tight truncate">
					{agentName}
					<span class="text-muted-foreground font-normal">/ {runId}</span>
				</h1>
				<p class="text-xs text-muted-foreground mt-0.5">Execution trace and span waterfall</p>
			</div>
		</div>

		<button
			onclick={load}
			class="shrink-0 inline-flex items-center gap-1.5 rounded-md border px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
		>
			<RefreshCw class="h-4 w-4" />
			Retry
		</button>
	</div>

	<!-- ======================================================== -->
	<!-- Error banner                                             -->
	<!-- ======================================================== -->
	{#if $traceError}
		<div class="rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 flex items-start gap-3">
			<AlertCircle class="h-4 w-4 text-destructive shrink-0 mt-0.5" />
			<div>
				<p class="text-sm font-medium text-destructive">Failed to load trace</p>
				<p class="text-xs text-destructive/80 mt-0.5">{$traceError}</p>
			</div>
			<button
				onclick={load}
				class="ml-auto shrink-0 text-xs font-medium text-destructive underline hover:no-underline cursor-pointer"
			>
				Retry
			</button>
		</div>
	{/if}

	<!-- ======================================================== -->
	<!-- Run summary bar                                         -->
	<!-- ======================================================== -->
	<div class="rounded-xl border bg-card shadow-sm px-5 py-4">
		{#if $traceLoading || runLoading}
			<div class="flex gap-6">
				{#each { length: 4 } as _}
					<div class="h-10 w-28 animate-pulse rounded-lg bg-muted"></div>
				{/each}
			</div>
		{:else}
			<div class="flex flex-wrap items-center gap-6">
				<!-- Status -->
				<div class="flex items-center gap-2">
					{#if overallStatus === 'ok'}
						<CheckCircle class="h-5 w-5 text-emerald-500" />
						<span class="text-sm font-semibold text-emerald-600 dark:text-emerald-400">
							Success
						</span>
					{:else if overallStatus === 'error'}
						<AlertCircle class="h-5 w-5 text-destructive" />
						<span class="text-sm font-semibold text-destructive">Failed</span>
					{:else}
						<Clock class="h-5 w-5 text-muted-foreground" />
						<span class="text-sm font-semibold text-muted-foreground">In progress</span>
					{/if}
				</div>

				<!-- Separator -->
				<span class="h-6 w-px bg-border"></span>

				<!-- Agent + Run ID -->
				<div>
					<p class="text-[10px] uppercase tracking-wide text-muted-foreground">Agent</p>
					<p class="text-sm font-semibold">{agentName}</p>
				</div>

				<!-- Separator -->
				<span class="h-6 w-px bg-border"></span>

				<!-- Duration -->
				<div class="flex items-center gap-2">
					<Clock class="h-4 w-4 text-muted-foreground" />
					<div>
						<p class="text-[10px] uppercase tracking-wide text-muted-foreground">Duration</p>
						<p class="text-sm font-semibold">
							{formatDuration(totalDurationMs)}
						</p>
					</div>
				</div>

				<!-- Span count -->
				<div class="flex items-center gap-2">
					<Layers class="h-4 w-4 text-muted-foreground" />
					<div>
						<p class="text-[10px] uppercase tracking-wide text-muted-foreground">Spans</p>
						<p class="text-sm font-semibold">{spanCount}</p>
					</div>
				</div>

				<!-- Coordination -->
				{#if $coordinationEdges.length > 0}
					<span class="h-6 w-px bg-border"></span>
					<div class="flex items-center gap-2">
						<Network class="h-4 w-4 text-violet-500" />
						<div>
							<p class="text-[10px] uppercase tracking-wide text-muted-foreground">
								Coordination
							</p>
							<p class="text-sm font-semibold text-violet-600 dark:text-violet-400">
								{$coordinationEdges.length} delegation{$coordinationEdges.length !== 1 ? 's' : ''}
							</p>
						</div>
					</div>
				{/if}

				<!-- Run cost if available -->
				{#if run?.cost}
					<span class="h-6 w-px bg-border"></span>
					<div>
						<p class="text-[10px] uppercase tracking-wide text-muted-foreground">Cost</p>
						<p class="text-sm font-semibold">
							${run.cost.total_cost_usd.toFixed(4)}
						</p>
					</div>
				{/if}
			</div>
		{/if}
	</div>

	<!-- ======================================================== -->
	<!-- Loading skeleton — waterfall                             -->
	<!-- ======================================================== -->
	{#if $traceLoading}
		<div class="rounded-xl border overflow-hidden">
			<!-- Header row -->
			<div class="flex items-center px-4 py-2.5 bg-muted/40 border-b gap-4">
				<div class="w-[30%] h-4 animate-pulse rounded bg-muted"></div>
				<div class="flex-1 h-4 animate-pulse rounded bg-muted"></div>
			</div>
			<!-- Skeleton span rows (different depths/widths for realism) -->
			{#each [
				{ indent: 0, left: 0, width: 90 },
				{ indent: 1, left: 5, width: 40 },
				{ indent: 2, left: 10, width: 15 },
				{ indent: 2, left: 30, width: 20 },
				{ indent: 1, left: 55, width: 35 }
			] as row}
				<div class="flex items-center gap-4 px-4 py-2.5 border-b last:border-0">
					<div class="w-[30%] flex items-center gap-2">
						<span style="width: {row.indent * 20}px" class="shrink-0"></span>
						<div class="h-3.5 w-3.5 animate-pulse rounded-full bg-muted shrink-0"></div>
						<div class="h-3.5 flex-1 animate-pulse rounded bg-muted"></div>
					</div>
					<div class="relative flex-1 h-6">
						<div
							class="absolute h-5 rounded-sm animate-pulse bg-muted"
							style="left: {row.left}%; width: {row.width - row.left}%"
						></div>
					</div>
				</div>
			{/each}
		</div>

	<!-- ======================================================== -->
	<!-- Content                                                 -->
	<!-- ======================================================== -->
	{:else if $traceSpans.length > 0}
		<!-- Coordination graph (only when multi-agent detected) -->
		{#if $coordinationEdges.length > 0}
			<div class="space-y-2">
				<div class="flex items-center gap-2">
					<Network class="h-4 w-4 text-violet-500" />
					<h2 class="text-sm font-semibold">Multi-Agent Coordination</h2>
				</div>
				<CoordinationGraph
					agents={involvedAgents}
					edges={$coordinationEdges}
					onnodeclick={handleNodeClick}
				/>
			</div>
		{/if}

		<!-- Waterfall -->
		<div class="space-y-2">
			<div class="flex items-center gap-2">
				<Activity class="h-4 w-4 text-sky-500" />
				<h2 class="text-sm font-semibold">Execution Waterfall</h2>
				<span class="text-xs text-muted-foreground">
					{spanCount} span{spanCount !== 1 ? 's' : ''} · {formatDuration(totalDurationMs)} total
				</span>
			</div>
			<TraceWaterfall tree={$spanTree} scale={$timeScale} />
		</div>

	<!-- ======================================================== -->
	<!-- Empty: no spans                                         -->
	<!-- ======================================================== -->
	{:else if !$traceError}
		<div
			class="flex flex-col items-center justify-center rounded-xl border border-dashed p-16 text-center"
		>
			<Activity class="h-10 w-10 text-muted-foreground/40 mb-3" />
			<p class="text-sm font-medium text-muted-foreground">No trace spans found</p>
			<p class="mt-1 text-xs text-muted-foreground/70 max-w-sm">
				This run may not have telemetry enabled, or the data has not been captured yet.
			</p>
		</div>
	{/if}
</div>
