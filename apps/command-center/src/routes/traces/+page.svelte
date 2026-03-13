<script lang="ts">
	import { onMount } from 'svelte';
	import { Activity, Clock, Layers, ChevronRight, RefreshCw } from 'lucide-svelte';
	import { api } from '$lib/api/client.js';
	import { agents, loadAgents } from '$lib/stores/agents.js';
	import type { AgentRun } from '$lib/api/types.js';

	let allRuns = $state<AgentRun[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let agentFilter = $state('all');

	const agentOptions = $derived([
		{ value: 'all', label: 'All agents' },
		...[...new Set(allRuns.map((r) => r.agent_name))].map((name) => ({
			value: name,
			label: name
		}))
	]);

	const filteredRuns = $derived(
		agentFilter === 'all' ? allRuns : allRuns.filter((r) => r.agent_name === agentFilter)
	);

	async function loadRuns() {
		loading = true;
		error = null;
		try {
			allRuns = await api.runs.list();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load runs';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		loadRuns();
		loadAgents();
	});

	// ============================================================
	// Formatting helpers
	// ============================================================

	function formatDuration(ms: number | undefined): string {
		if (ms == null) return '—';
		if (ms < 1000) return `${Math.round(ms)}ms`;
		if (ms < 60_000) return `${(ms / 1000).toFixed(1)}s`;
		const mins = Math.floor(ms / 60_000);
		const secs = ((ms % 60_000) / 1000).toFixed(0);
		return `${mins}m ${secs}s`;
	}

	function formatRelative(iso: string): string {
		const diff = Date.now() - new Date(iso).getTime();
		if (diff < 60_000) return 'just now';
		if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`;
		if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`;
		return `${Math.floor(diff / 86_400_000)}d ago`;
	}

	function statusClass(status: string): string {
		switch (status) {
			case 'completed': return 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-400';
			case 'failed': return 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-400';
			case 'running': return 'bg-sky-100 text-sky-700 dark:bg-sky-900/40 dark:text-sky-400';
			case 'stopped': return 'bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-400';
			default: return 'bg-muted text-muted-foreground';
		}
	}
</script>

<svelte:head>
	<title>Execution Traces — OpenAgentiX</title>
</svelte:head>

<div class="p-6 space-y-6">
	<!-- ======================================================== -->
	<!-- Header                                                    -->
	<!-- ======================================================== -->
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div class="flex items-center gap-3">
			<div
				class="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-violet-500 to-sky-500 shadow-sm"
			>
				<Activity class="h-5 w-5 text-white" />
			</div>
			<div>
				<h1 class="text-2xl font-bold tracking-tight">Execution Traces</h1>
				<p class="text-sm text-muted-foreground">
					Inspect agent run timelines, span trees, and coordination
				</p>
			</div>
		</div>
		<button
			onclick={loadRuns}
			class="inline-flex items-center gap-2 rounded-md border px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
		>
			<RefreshCw class="h-4 w-4" />
			Refresh
		</button>
	</div>

	<!-- ======================================================== -->
	<!-- Error                                                    -->
	<!-- ======================================================== -->
	{#if error}
		<div class="rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3">
			<p class="text-sm text-destructive">{error}</p>
			<button onclick={loadRuns} class="mt-1 text-xs underline text-destructive cursor-pointer">
				Retry
			</button>
		</div>
	{/if}

	<!-- ======================================================== -->
	<!-- Filter bar                                               -->
	<!-- ======================================================== -->
	<div class="flex gap-3">
		<select
			bind:value={agentFilter}
			class="rounded-md border bg-card px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
		>
			{#each agentOptions as opt}
				<option value={opt.value}>{opt.label}</option>
			{/each}
		</select>
		{#if agentFilter !== 'all'}
			<button
				onclick={() => (agentFilter = 'all')}
				class="rounded-md border px-3 py-2 text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
			>
				Clear
			</button>
		{/if}
	</div>

	<!-- ======================================================== -->
	<!-- Loading skeleton                                         -->
	<!-- ======================================================== -->
	{#if loading}
		<div class="rounded-xl border overflow-hidden">
			<div class="bg-muted/40 h-10 border-b"></div>
			{#each { length: 6 } as _}
				<div class="flex items-center gap-4 px-4 py-3 border-b last:border-0">
					<div class="h-4 w-28 animate-pulse rounded bg-muted"></div>
					<div class="h-4 w-48 animate-pulse rounded bg-muted flex-1"></div>
					<div class="h-4 w-16 animate-pulse rounded bg-muted"></div>
					<div class="h-4 w-20 animate-pulse rounded bg-muted"></div>
				</div>
			{/each}
		</div>

	<!-- ======================================================== -->
	<!-- Empty state                                             -->
	<!-- ======================================================== -->
	{:else if filteredRuns.length === 0}
		<div
			class="flex flex-col items-center justify-center rounded-xl border border-dashed p-16 text-center"
		>
			<Activity class="h-10 w-10 text-muted-foreground/40 mb-3" />
			<p class="text-sm font-medium text-muted-foreground">
				{allRuns.length === 0 ? 'No runs recorded yet' : 'No runs match the selected agent'}
			</p>
			<p class="mt-1 text-xs text-muted-foreground/70 max-w-sm">
				Traces appear after agents have executed. Run an agent to see its execution timeline.
			</p>
		</div>

	<!-- ======================================================== -->
	<!-- Run list                                                -->
	<!-- ======================================================== -->
	{:else}
		<div class="rounded-xl border overflow-hidden bg-card shadow-sm">
			<!-- Table header -->
			<div
				class="grid grid-cols-[1fr_2fr_auto_auto_auto_auto] gap-4 px-4 py-2.5 bg-muted/40 border-b text-xs font-medium uppercase tracking-wider text-muted-foreground"
			>
				<span>Agent</span>
				<span>Run ID</span>
				<span class="text-right hidden sm:block">Status</span>
				<span class="text-right">Started</span>
				<span class="text-right hidden md:block">Duration</span>
				<span></span>
			</div>

			{#each filteredRuns as run (run.run_id)}
				<a
					href="/traces/{encodeURIComponent(run.agent_name)}/{encodeURIComponent(run.run_id)}"
					class="grid grid-cols-[1fr_2fr_auto_auto_auto_auto] gap-4 items-center px-4 py-3 border-b last:border-0 hover:bg-accent/40 transition-colors cursor-pointer"
				>
					<!-- Agent avatar + name -->
					<div class="flex items-center gap-2 min-w-0">
						<div
							class="h-6 w-6 shrink-0 rounded-full flex items-center justify-center text-[10px] font-bold text-white"
							style="background: linear-gradient(135deg, #8b5cf6, #0ea5e9)"
						>
							{run.agent_name[0]?.toUpperCase() ?? '?'}
						</div>
						<span class="text-sm font-medium truncate">{run.agent_name}</span>
					</div>

					<!-- Run ID -->
					<div class="min-w-0">
						<span class="font-mono text-xs text-muted-foreground truncate block">
							{run.run_id}
						</span>
					</div>

					<!-- Status badge -->
					<div class="hidden sm:block">
						<span
							class="rounded-full px-2 py-0.5 text-[10px] font-medium uppercase tracking-wide {statusClass(run.status)}"
						>
							{run.status}
						</span>
					</div>

					<!-- Started -->
					<div class="flex items-center gap-1 text-xs text-muted-foreground whitespace-nowrap">
						<Clock class="h-3 w-3 shrink-0" />
						{formatRelative(run.started_at)}
					</div>

					<!-- Duration -->
					<div class="hidden md:flex items-center gap-1 text-xs text-muted-foreground">
						<Layers class="h-3 w-3 shrink-0" />
						{formatDuration(run.duration_ms)}
					</div>

					<!-- Chevron -->
					<ChevronRight class="h-4 w-4 text-muted-foreground" />
				</a>
			{/each}
		</div>
	{/if}
</div>
