<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { ChevronLeft, Bot } from 'lucide-svelte';
	import AgentDetail from '$lib/components/agent-detail.svelte';
	import Skeleton from '$lib/components/ui/skeleton.svelte';
	import {
		selectedAgent,
		agentDetailLoading,
		agentDetailError,
		agentRuns,
		agentRunsLoading,
		loadAgent,
		loadAgentRuns
	} from '$lib/stores/agents.js';

	const agentName = $derived(decodeURIComponent(page.params.name ?? ''));

	onMount(() => {
		if (agentName) {
			loadAgent(agentName);
			loadAgentRuns(agentName);
		}
	});
</script>

<div class="p-6 space-y-6">
	<!-- Back link -->
	<a
		href="/agents"
		class="inline-flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
	>
		<ChevronLeft class="h-4 w-4" />
		Back to Agents
	</a>

	<!-- Error state -->
	{#if $agentDetailError}
		<div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3">
			<p class="text-sm font-medium text-destructive">{$agentDetailError}</p>
			<button
				onclick={() => { loadAgent(agentName); loadAgentRuns(agentName); }}
				class="mt-2 text-xs underline text-destructive cursor-pointer"
			>Retry</button>
		</div>
	{/if}

	<!-- Loading skeleton -->
	{#if $agentDetailLoading}
		<div class="space-y-4">
			<Skeleton class="h-20 rounded-xl" />
			<Skeleton class="h-10 rounded-md w-64" />
			<Skeleton class="h-48 rounded-xl" />
		</div>

	<!-- Agent not found -->
	{:else if !$selectedAgent}
		<div class="flex flex-col items-center justify-center rounded-xl border border-dashed p-16 text-center">
			<Bot class="h-10 w-10 text-muted-foreground/50 mb-3" />
			<p class="text-sm font-medium text-muted-foreground">Agent "{agentName}" not found</p>
			<a
				href="/agents"
				class="mt-3 text-xs text-muted-foreground underline"
			>Back to agent list</a>
		</div>

	<!-- Agent detail -->
	{:else}
		<AgentDetail
			agent={$selectedAgent}
			runs={$agentRunsLoading ? [] : $agentRuns}
		/>
	{/if}
</div>
