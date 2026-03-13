<script lang="ts">
	import { onMount } from 'svelte';
	import { Bot, Search } from 'lucide-svelte';
	import AgentCard from '$lib/components/agent-card.svelte';
	import Skeleton from '$lib/components/ui/skeleton.svelte';
	import { agents, agentsLoading, agentsError, loadAgents } from '$lib/stores/agents.js';
	import type { AgentStatus } from '$lib/api/types.js';

	let searchQuery = $state('');
	let statusFilter = $state<AgentStatus | 'all'>('all');

	const statusOptions: Array<{ value: AgentStatus | 'all'; label: string }> = [
		{ value: 'all', label: 'All statuses' },
		{ value: 'running', label: 'Running' },
		{ value: 'idle', label: 'Idle' },
		{ value: 'error', label: 'Error' },
		{ value: 'scheduled', label: 'Scheduled' }
	];

	const filteredAgents = $derived(
		$agents.filter((a) => {
			const matchesSearch = searchQuery.trim() === '' || a.name.toLowerCase().includes(searchQuery.toLowerCase());
			const matchesStatus = statusFilter === 'all' || a.status === statusFilter;
			return matchesSearch && matchesStatus;
		})
	);

	onMount(() => {
		loadAgents();
	});
</script>

<div class="p-6 space-y-6">
	<!-- Header -->
	<div>
		<h1 class="text-2xl font-bold tracking-tight">Agents</h1>
		<p class="text-sm text-muted-foreground mt-1">All registered agents with live status.</p>
	</div>

	<!-- Error state -->
	{#if $agentsError}
		<div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3">
			<p class="text-sm font-medium text-destructive">{$agentsError}</p>
			<button
				onclick={loadAgents}
				class="mt-2 text-xs underline text-destructive cursor-pointer"
			>Retry</button>
		</div>
	{/if}

	<!-- Filter bar -->
	<div class="flex flex-col gap-3 sm:flex-row sm:items-center">
		<div class="relative flex-1">
			<Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none" />
			<input
				bind:value={searchQuery}
				type="search"
				placeholder="Search agents..."
				class="w-full rounded-md border bg-card pl-9 pr-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
			/>
		</div>
		<select
			bind:value={statusFilter}
			class="rounded-md border bg-card px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
		>
			{#each statusOptions as opt}
				<option value={opt.value}>{opt.label}</option>
			{/each}
		</select>
	</div>

	<!-- Loading skeleton -->
	{#if $agentsLoading}
		<div class="grid grid-cols-1 gap-4 lg:grid-cols-2 xl:grid-cols-3">
			{#each { length: 6 } as _}
				<Skeleton class="h-36 rounded-xl" />
			{/each}
		</div>

	<!-- Agent grid -->
	{:else if filteredAgents.length > 0}
		<div class="grid grid-cols-1 gap-4 lg:grid-cols-2 xl:grid-cols-3">
			{#each filteredAgents as agent (agent.name)}
				<AgentCard {agent} />
			{/each}
		</div>

	<!-- Empty state — no agents at all -->
	{:else if $agents.length === 0}
		<div class="flex flex-col items-center justify-center rounded-xl border border-dashed p-16 text-center">
			<Bot class="h-10 w-10 text-muted-foreground/50 mb-3" />
			<p class="text-sm font-medium text-muted-foreground">No agents registered</p>
			<p class="text-xs text-muted-foreground/70 mt-1 max-w-sm">
				Create your first agent to get started.
			</p>
			<a
				href="/builder"
				class="mt-4 inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
			>
				<Bot class="h-4 w-4" />
				Open Agent Builder
			</a>
		</div>

	<!-- Empty state — filtered to nothing -->
	{:else}
		<div class="flex flex-col items-center justify-center rounded-xl border border-dashed p-12 text-center">
			<Search class="h-8 w-8 text-muted-foreground/50 mb-3" />
			<p class="text-sm font-medium text-muted-foreground">No agents match your filter</p>
			<button
				onclick={() => { searchQuery = ''; statusFilter = 'all'; }}
				class="mt-3 text-xs text-muted-foreground underline cursor-pointer"
			>Clear filters</button>
		</div>
	{/if}
</div>
