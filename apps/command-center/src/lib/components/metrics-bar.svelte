<script lang="ts">
	import Card from '$lib/components/ui/card.svelte';
	import { Bot, PlayCircle, DollarSign, ShieldCheck } from 'lucide-svelte';

	let {
		agents = 0,
		activeRuns = 0,
		costsToday = 0,
		pendingApprovals = 0
	}: {
		agents?: number;
		activeRuns?: number;
		costsToday?: number;
		pendingApprovals?: number;
	} = $props();

	const metrics = $derived([
		{
			label: 'Total Agents',
			value: agents.toString(),
			icon: Bot,
			iconClass: 'text-info'
		},
		{
			label: 'Active Runs',
			value: activeRuns.toString(),
			icon: PlayCircle,
			iconClass: 'text-success'
		},
		{
			label: 'Costs Today',
			value: `$${costsToday.toFixed(4)}`,
			icon: DollarSign,
			iconClass: 'text-warning'
		},
		{
			label: 'Pending Approvals',
			value: pendingApprovals.toString(),
			icon: ShieldCheck,
			iconClass: pendingApprovals > 0 ? 'text-destructive' : 'text-muted-foreground'
		}
	]);
</script>

<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
	{#each metrics as metric}
		<Card class="p-5">
			<div class="flex items-start justify-between">
				<div>
					<p class="text-xs font-medium uppercase tracking-wider text-muted-foreground">{metric.label}</p>
					<p class="mt-2 text-3xl font-bold tracking-tight">{metric.value}</p>
				</div>
				<div class="rounded-lg bg-muted p-2">
					<metric.icon class="h-5 w-5 {metric.iconClass}" />
				</div>
			</div>
		</Card>
	{/each}
</div>
