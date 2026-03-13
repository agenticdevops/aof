<script lang="ts">
	import { onMount } from 'svelte';
	import { Bot, RefreshCw, ShieldCheck, PlayCircle, Clock } from 'lucide-svelte';
	import MetricsBar from '$lib/components/metrics-bar.svelte';
	import ActivityFeed from '$lib/components/activity-feed.svelte';
	import Card from '$lib/components/ui/card.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import Skeleton from '$lib/components/ui/skeleton.svelte';
	import { agents, agentsLoading, agentsError, loadAgents } from '$lib/stores/agents.js';
	import { wsEvents } from '$lib/stores/websocket.js';
	import type { GatewayEvent } from '$lib/stores/websocket.js';
	import { api } from '$lib/api/client.js';
	import type { CostSummary, ApprovalRequest, AgentRun, Agent } from '$lib/api/types.js';

	// Scheduled agents: agents with cron triggers
	const scheduledAgents = $derived(
		$agents.filter((a: Agent) => a.triggers?.some((t) => t.type === 'cron'))
	);

	function humanizeCron(expr: string): string {
		// Very simple humanizer for common cron patterns
		const parts = expr.trim().split(/\s+/);
		if (parts.length < 5) return expr;
		const [min, hour, dom, month, dow] = parts;
		if (min === '0' && dom === '*' && month === '*' && dow === '*') {
			return `Every day at ${hour}:00`;
		}
		if (dom === '*' && month === '*' && dow !== '*') {
			const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
			const dayName = days[parseInt(dow)] ?? `day ${dow}`;
			return `Every ${dayName} at ${hour}:${min.padStart(2, '0')}`;
		}
		if (min === '*' && hour === '*') return 'Every minute';
		return expr;
	}

	let costsToday = $state(0);
	let pendingApprovals = $state<ApprovalRequest[]>([]);
	let activeRuns = $state<AgentRun[]>([]);
	let pageLoading = $state(true);
	let pageError = $state<string | null>(null);

	// Keep last 20 events in an array for the activity feed
	let eventLog = $state<GatewayEvent[]>([]);

	// Subscribe to WebSocket events
	wsEvents.subscribe((event) => {
		if (!event) return;
		// Prepend newest event, keep last 20
		eventLog = [event, ...eventLog].slice(0, 20);
	});

	const activeRunCount = $derived($agents.filter((a) => a.status === 'running').length);
	const pendingApprovalCount = $derived(pendingApprovals.length);

	async function loadDashboard() {
		pageLoading = true;
		pageError = null;
		try {
			await loadAgents();
			// Costs
			try {
				const costs = await api.costs.listAll();
				costsToday = costs.reduce((sum: number, c: CostSummary) => sum + c.total_cost_usd, 0);
			} catch {
				costsToday = 0;
			}
			// Approvals
			try {
				const approvals = await api.approvals.list();
				pendingApprovals = approvals.filter((a: ApprovalRequest) => a.status === 'pending');
			} catch {
				pendingApprovals = [];
			}
			// Active runs
			try {
				const runs = await api.runs.list();
				activeRuns = runs.filter((r: AgentRun) => r.status === 'running');
			} catch {
				activeRuns = [];
			}
		} catch (err) {
			pageError = err instanceof Error ? err.message : 'Failed to load dashboard';
		} finally {
			pageLoading = false;
		}
	}

	onMount(() => {
		loadDashboard();
	});
</script>

<div class="p-6 space-y-6">
	<!-- Page header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold tracking-tight">Dashboard</h1>
			<p class="text-sm text-muted-foreground mt-1">Mission control for your AI agents.</p>
		</div>
		<button
			onclick={loadDashboard}
			class="inline-flex items-center gap-2 rounded-md border px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
		>
			<RefreshCw class="h-4 w-4" />
			Refresh
		</button>
	</div>

	<!-- Error state -->
	{#if pageError}
		<div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3">
			<p class="text-sm font-medium text-destructive">{pageError}</p>
			<button
				onclick={loadDashboard}
				class="mt-2 text-xs underline text-destructive cursor-pointer"
			>Retry</button>
		</div>
	{/if}

	<!-- Metrics bar -->
	{#if pageLoading}
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
			{#each { length: 4 } as _}
				<Skeleton class="h-24 rounded-xl" />
			{/each}
		</div>
	{:else}
		<MetricsBar
			agents={$agents.length}
			activeRuns={activeRunCount}
			{costsToday}
			pendingApprovals={pendingApprovalCount}
		/>
	{/if}

	<!-- Empty state when no agents -->
	{#if !pageLoading && $agents.length === 0 && !pageError}
		<div class="flex flex-col items-center justify-center rounded-xl border border-dashed p-16 text-center">
			<Bot class="h-10 w-10 text-muted-foreground/50 mb-3" />
			<p class="text-sm font-medium text-muted-foreground">No agents registered</p>
			<p class="text-xs text-muted-foreground/70 mt-1 max-w-sm">
				Register your first agent to get started. Head to Agents to create and deploy AI agents.
			</p>
			<div class="mt-4">
				<a
					href="/agents"
					class="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors duration-200"
				>
					<Bot class="h-4 w-4" />
					Go to Agents
				</a>
			</div>
		</div>
	{:else if !pageLoading}
		<!-- Main 2-column grid -->
		<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
			<!-- Activity feed -->
			<div class="min-h-64">
				<ActivityFeed events={eventLog} />
			</div>

			<!-- Quick panels -->
			<div class="space-y-4">
				<!-- Active runs -->
				<Card class="p-4">
					<div class="flex items-center gap-2 mb-3 border-b pb-2">
						<PlayCircle class="h-4 w-4 text-muted-foreground" />
						<h3 class="text-sm font-semibold">Active Runs</h3>
					</div>
					{#if pageLoading}
						<div class="space-y-2">
							{#each { length: 3 } as _}
								<Skeleton class="h-8" />
							{/each}
						</div>
					{:else if activeRuns.length === 0}
						<p class="text-xs text-muted-foreground py-4 text-center">No runs currently active</p>
					{:else}
						<ul class="space-y-2">
							{#each activeRuns.slice(0, 5) as run (run.run_id)}
								<li class="flex items-center justify-between text-xs">
									<span class="font-medium truncate max-w-[60%]">{run.agent_name}</span>
									<Badge variant="info">running</Badge>
								</li>
							{/each}
						</ul>
						{#if activeRuns.length > 5}
							<a href="/runs" class="mt-2 block text-xs text-muted-foreground underline">
								View all {activeRuns.length} runs
							</a>
						{/if}
					{/if}
				</Card>

				<!-- Pending approvals -->
				<Card class="p-4">
					<div class="flex items-center gap-2 mb-3 border-b pb-2">
						<ShieldCheck class="h-4 w-4 text-muted-foreground" />
						<h3 class="text-sm font-semibold">Pending Approvals</h3>
					</div>
					{#if pendingApprovals.length === 0}
						<p class="text-xs text-muted-foreground py-4 text-center">No pending approvals</p>
					{:else}
						<p class="text-sm">
							<span class="font-bold text-warning">{pendingApprovals.length}</span>
							approval{pendingApprovals.length !== 1 ? 's' : ''} awaiting decision.
						</p>
						<a
							href="/approvals"
							class="mt-3 inline-flex items-center gap-1.5 rounded-md bg-warning/10 px-3 py-1.5 text-xs font-medium text-warning hover:bg-warning/20 transition-colors"
						>
							<ShieldCheck class="h-3.5 w-3.5" />
							Review Approvals
						</a>
					{/if}
				</Card>
			<!-- Scheduled agents -->
				<Card class="p-4">
					<div class="flex items-center gap-2 mb-3 border-b pb-2">
						<Clock class="h-4 w-4 text-muted-foreground" />
						<h3 class="text-sm font-semibold">Scheduled Agents</h3>
					</div>
					{#if scheduledAgents.length === 0}
						<p class="text-xs text-muted-foreground py-4 text-center">No agents with cron triggers</p>
					{:else}
						<ul class="space-y-2">
							{#each scheduledAgents.slice(0, 5) as agent (agent.name)}
								{@const cronTrigger = agent.triggers.find((t) => t.type === 'cron')}
								<li class="flex items-start justify-between text-xs gap-2">
									<span class="font-medium truncate max-w-[45%]">{agent.name}</span>
									<span class="text-muted-foreground text-right truncate">
										{cronTrigger?.expression ? humanizeCron(cronTrigger.expression) : 'Cron'}
									</span>
								</li>
							{/each}
						</ul>
					{/if}
				</Card>
			</div>
		</div>
	{/if}
</div>
