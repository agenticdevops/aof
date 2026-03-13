<script lang="ts">
	import { onMount } from 'svelte';
	import { PlayCircle, Search, ChevronLeft, ChevronRight } from 'lucide-svelte';
	import RunRow from '$lib/components/run-row.svelte';
	import Skeleton from '$lib/components/ui/skeleton.svelte';
	import { agents, loadAgents } from '$lib/stores/agents.js';
	import { api } from '$lib/api/client.js';
	import type { AgentRun, RunStatus } from '$lib/api/types.js';

	const PAGE_SIZE = 25;

	let allRuns = $state<AgentRun[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let currentPage = $state(1);

	// Filters
	let agentFilter = $state('all');
	let statusFilter = $state<RunStatus | 'all'>('all');
	let dateStart = $state('');
	let dateEnd = $state('');

	const statusOptions: Array<{ value: RunStatus | 'all'; label: string }> = [
		{ value: 'all', label: 'All statuses' },
		{ value: 'running', label: 'Running' },
		{ value: 'completed', label: 'Completed' },
		{ value: 'failed', label: 'Failed' },
		{ value: 'stopped', label: 'Stopped' },
		{ value: 'paused', label: 'Paused' }
	];

	const agentOptions = $derived([
		{ value: 'all', label: 'All agents' },
		...[...new Set(allRuns.map((r) => r.agent_name))].map((name) => ({ value: name, label: name }))
	]);

	const filteredRuns = $derived(
		allRuns.filter((r) => {
			if (agentFilter !== 'all' && r.agent_name !== agentFilter) return false;
			if (statusFilter !== 'all' && r.status !== statusFilter) return false;
			if (dateStart && r.started_at < dateStart) return false;
			if (dateEnd && r.started_at > dateEnd + 'T23:59:59Z') return false;
			return true;
		})
	);

	const totalPages = $derived(Math.max(1, Math.ceil(filteredRuns.length / PAGE_SIZE)));

	const pagedRuns = $derived(
		filteredRuns.slice((currentPage - 1) * PAGE_SIZE, currentPage * PAGE_SIZE)
	);

	// Reset page when filters change
	$effect(() => {
		// Touch filter values to create dependency
		agentFilter; statusFilter; dateStart; dateEnd;
		currentPage = 1;
	});

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
</script>

<div class="p-6 space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold tracking-tight">Runs</h1>
			<p class="text-sm text-muted-foreground mt-1">All agent run history with filtering.</p>
		</div>
		<button
			onclick={loadRuns}
			class="inline-flex items-center gap-2 rounded-md border px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
		>
			<Search class="h-4 w-4" />
			Refresh
		</button>
	</div>

	<!-- Error state -->
	{#if error}
		<div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3">
			<p class="text-sm font-medium text-destructive">{error}</p>
			<button onclick={loadRuns} class="mt-2 text-xs underline text-destructive cursor-pointer">Retry</button>
		</div>
	{/if}

	<!-- Filter bar -->
	<div class="flex flex-wrap gap-3">
		<select
			bind:value={agentFilter}
			class="rounded-md border bg-card px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
		>
			{#each agentOptions as opt}
				<option value={opt.value}>{opt.label}</option>
			{/each}
		</select>
		<select
			bind:value={statusFilter}
			class="rounded-md border bg-card px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
		>
			{#each statusOptions as opt}
				<option value={opt.value}>{opt.label}</option>
			{/each}
		</select>
		<input
			bind:value={dateStart}
			type="date"
			title="Start date"
			class="rounded-md border bg-card px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
		/>
		<input
			bind:value={dateEnd}
			type="date"
			title="End date"
			class="rounded-md border bg-card px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
		/>
		{#if agentFilter !== 'all' || statusFilter !== 'all' || dateStart || dateEnd}
			<button
				onclick={() => { agentFilter = 'all'; statusFilter = 'all'; dateStart = ''; dateEnd = ''; }}
				class="rounded-md border px-3 py-2 text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
			>
				Clear filters
			</button>
		{/if}
	</div>

	<!-- Loading skeleton -->
	{#if loading}
		<div class="rounded-xl border overflow-hidden">
			<div class="bg-muted/50 h-10 border-b"></div>
			{#each { length: 8 } as _}
				<div class="px-4 py-3 border-b">
					<Skeleton class="h-5 w-full" />
				</div>
			{/each}
		</div>

	<!-- Empty state -->
	{:else if filteredRuns.length === 0}
		<div class="flex flex-col items-center justify-center rounded-xl border border-dashed p-16 text-center">
			<PlayCircle class="h-10 w-10 text-muted-foreground/50 mb-3" />
			<p class="text-sm font-medium text-muted-foreground">
				{allRuns.length === 0 ? 'No runs recorded yet' : 'No runs match your filters'}
			</p>
			{#if allRuns.length > 0}
				<button
					onclick={() => { agentFilter = 'all'; statusFilter = 'all'; dateStart = ''; dateEnd = ''; }}
					class="mt-3 text-xs text-muted-foreground underline cursor-pointer"
				>Clear filters</button>
			{/if}
		</div>

	<!-- Run table -->
	{:else}
		<div>
			<p class="text-xs text-muted-foreground mb-3">
				Showing {(currentPage - 1) * PAGE_SIZE + 1}–{Math.min(currentPage * PAGE_SIZE, filteredRuns.length)} of {filteredRuns.length} run{filteredRuns.length !== 1 ? 's' : ''}
			</p>
			<div class="overflow-x-auto rounded-xl border">
				<!-- Table header -->
				<div class="flex items-center gap-4 px-4 py-2 bg-muted/50 text-xs font-medium uppercase tracking-wider text-muted-foreground border-b">
					<span class="w-36 shrink-0">Agent</span>
					<span class="w-32 shrink-0">Run ID</span>
					<span class="w-24 shrink-0">Status</span>
					<span class="hidden flex-1 sm:block">Trigger</span>
					<span class="w-20 shrink-0 text-right">Started</span>
					<span class="w-16 shrink-0 text-right">Duration</span>
					<span class="w-16 shrink-0 text-right">Cost</span>
				</div>
				{#each pagedRuns as run (run.run_id)}
					<RunRow {run} />
				{/each}
			</div>

			<!-- Pagination -->
			{#if totalPages > 1}
				<div class="flex items-center justify-between mt-4">
					<button
						onclick={() => (currentPage = Math.max(1, currentPage - 1))}
						disabled={currentPage === 1}
						class="inline-flex items-center gap-1.5 rounded-md border px-3 py-1.5 text-sm font-medium transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed hover:bg-accent"
					>
						<ChevronLeft class="h-4 w-4" />
						Previous
					</button>
					<span class="text-sm text-muted-foreground">Page {currentPage} of {totalPages}</span>
					<button
						onclick={() => (currentPage = Math.min(totalPages, currentPage + 1))}
						disabled={currentPage === totalPages}
						class="inline-flex items-center gap-1.5 rounded-md border px-3 py-1.5 text-sm font-medium transition-colors cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed hover:bg-accent"
					>
						Next
						<ChevronRight class="h-4 w-4" />
					</button>
				</div>
			{/if}
		</div>
	{/if}
</div>
