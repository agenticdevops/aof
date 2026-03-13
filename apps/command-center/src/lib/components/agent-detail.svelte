<script lang="ts">
	import Badge from '$lib/components/ui/badge.svelte';
	import RunRow from '$lib/components/run-row.svelte';
	import type { Agent, AgentRun, AgentStatus } from '$lib/api/types.js';
	import type { BadgeVariant } from '$lib/components/ui/badge.svelte';
	import { Zap, Wrench, Bot, DollarSign, Activity } from 'lucide-svelte';

	let {
		agent,
		runs = []
	}: {
		agent: Agent;
		runs?: AgentRun[];
	} = $props();

	type Tab = 'overview' | 'runs' | 'config';
	let activeTab = $state<Tab>('overview');

	function statusVariant(status: AgentStatus): BadgeVariant {
		switch (status) {
			case 'running': return 'success';
			case 'error': return 'destructive';
			case 'scheduled': return 'info';
			default: return 'secondary';
		}
	}

	function modeVariant(mode: string): BadgeVariant {
		switch (mode) {
			case 'autonomous': return 'default';
			case 'semi-autonomous': return 'info';
			default: return 'outline';
		}
	}

	const agentYaml = $derived(buildAgentYaml(agent));

	function buildAgentYaml(a: Agent): string {
		const lines: string[] = [
			`apiVersion: openagentix.dev/v1`,
			`kind: Agent`,
			`metadata:`,
			`  name: ${a.name}`,
			`spec:`,
			`  model:`,
			`    preferred: ${a.model.preferred}`
		];
		if (a.model.fallback) {
			lines.push(`    fallback: ${a.model.fallback}`);
		}
		lines.push(`  mode: ${a.mode}`);
		if (a.triggers && a.triggers.length > 0) {
			lines.push(`  triggers:`);
			for (const t of a.triggers) {
				lines.push(`    - type: ${t.type}`);
				if (t.expression) lines.push(`      expression: "${t.expression}"`);
				if (t.channel) lines.push(`      channel: ${t.channel}`);
			}
		}
		if (a.skills && a.skills.length > 0) {
			lines.push(`  skills:`);
			for (const s of a.skills) {
				lines.push(`    - ${s}`);
			}
		}
		if (a.budget) {
			lines.push(`  budget:`);
			if (a.budget.daily_limit !== undefined) lines.push(`    daily_limit: ${a.budget.daily_limit}`);
			if (a.budget.max_tokens_per_run !== undefined) lines.push(`    max_tokens_per_run: ${a.budget.max_tokens_per_run}`);
		}
		return lines.join('\n');
	}

	const totalCost = $derived(
		runs.reduce((sum, r) => sum + (r.cost?.total_cost_usd ?? 0), 0)
	);
</script>

<div class="flex flex-col gap-0">
	<!-- Header -->
	<div class="flex items-start gap-4 pb-4 border-b">
		<div class="rounded-lg bg-muted p-3">
			<Bot class="h-6 w-6 text-muted-foreground" />
		</div>
		<div class="flex-1 min-w-0">
			<h2 class="text-lg font-bold truncate">{agent.name}</h2>
			<p class="text-sm text-muted-foreground">{agent.model.preferred}</p>
			<div class="mt-2 flex flex-wrap gap-2">
				<Badge variant={statusVariant(agent.status)}>{agent.status}</Badge>
				<Badge variant={modeVariant(agent.mode)}>{agent.mode}</Badge>
				{#if agent.namespace}
					<Badge variant="outline">{agent.namespace}</Badge>
				{/if}
			</div>
		</div>
	</div>

	<!-- Tabs -->
	<div class="flex border-b mt-0">
		{#each (['overview', 'runs', 'config'] as Tab[]) as tab}
			<button
				onclick={() => (activeTab = tab)}
				class="px-4 py-2.5 text-sm font-medium capitalize border-b-2 -mb-px transition-colors cursor-pointer {activeTab === tab
					? 'border-primary text-foreground'
					: 'border-transparent text-muted-foreground hover:text-foreground'}"
			>
				{tab}
				{#if tab === 'runs' && runs.length > 0}
					<span class="ml-1 rounded-full bg-muted px-1.5 py-0.5 text-xs font-semibold">{runs.length}</span>
				{/if}
			</button>
		{/each}
	</div>

	<!-- Tab content -->
	<div class="pt-4">
		{#if activeTab === 'overview'}
			<dl class="space-y-4">
				<div>
					<dt class="text-xs font-medium uppercase tracking-wider text-muted-foreground mb-1">Model</dt>
					<dd class="text-sm font-mono">{agent.model.preferred}{agent.model.fallback ? ` / ${agent.model.fallback}` : ''}</dd>
				</div>
				<div>
					<dt class="text-xs font-medium uppercase tracking-wider text-muted-foreground mb-1">Mode</dt>
					<dd><Badge variant={modeVariant(agent.mode)}>{agent.mode}</Badge></dd>
				</div>
				{#if agent.triggers && agent.triggers.length > 0}
					<div>
						<dt class="flex items-center gap-1.5 text-xs font-medium uppercase tracking-wider text-muted-foreground mb-2">
							<Zap class="h-3.5 w-3.5" /> Triggers
						</dt>
						<dd class="space-y-1">
							{#each agent.triggers as trigger}
								<div class="flex items-center gap-2 text-sm">
									<Badge variant="outline">{trigger.type}</Badge>
									{#if trigger.expression}
										<span class="font-mono text-xs text-muted-foreground">{trigger.expression}</span>
									{/if}
									{#if trigger.channel}
										<span class="text-xs text-muted-foreground">#{trigger.channel}</span>
									{/if}
								</div>
							{/each}
						</dd>
					</div>
				{/if}
				{#if agent.skills && agent.skills.length > 0}
					<div>
						<dt class="flex items-center gap-1.5 text-xs font-medium uppercase tracking-wider text-muted-foreground mb-2">
							<Wrench class="h-3.5 w-3.5" /> Skills
						</dt>
						<dd class="flex flex-wrap gap-2">
							{#each agent.skills as skill}
								<Badge variant="secondary">{skill}</Badge>
							{/each}
						</dd>
					</div>
				{/if}
				{#if agent.budget}
					<div>
						<dt class="flex items-center gap-1.5 text-xs font-medium uppercase tracking-wider text-muted-foreground mb-2">
							<DollarSign class="h-3.5 w-3.5" /> Budget
						</dt>
						<dd class="space-y-1 text-sm">
							{#if agent.budget.daily_limit !== undefined}
								<p>Daily limit: <span class="font-semibold">${agent.budget.daily_limit}</span></p>
							{/if}
							{#if agent.budget.max_tokens_per_run !== undefined}
								<p>Max tokens/run: <span class="font-semibold">{agent.budget.max_tokens_per_run.toLocaleString()}</span></p>
							{/if}
						</dd>
					</div>
				{/if}
				{#if runs.length > 0}
					<div>
						<dt class="flex items-center gap-1.5 text-xs font-medium uppercase tracking-wider text-muted-foreground mb-1">
							<Activity class="h-3.5 w-3.5" /> Run Summary
						</dt>
						<dd class="text-sm">
							<span class="font-semibold">{runs.length}</span> runs · total cost <span class="font-semibold">${totalCost.toFixed(4)}</span>
						</dd>
					</div>
				{/if}
			</dl>

		{:else if activeTab === 'runs'}
			{#if runs.length === 0}
				<p class="text-sm text-muted-foreground py-6 text-center">No runs recorded yet.</p>
			{:else}
				<div class="overflow-x-auto rounded-lg border">
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
					{#each runs as run (run.run_id)}
						<RunRow {run} />
					{/each}
				</div>
			{/if}

		{:else if activeTab === 'config'}
			<div class="overflow-auto rounded-lg border">
				<pre class="p-4 text-xs font-mono leading-relaxed text-foreground whitespace-pre-wrap break-words">{agentYaml}</pre>
			</div>
		{/if}
	</div>
</div>
