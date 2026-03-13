<script lang="ts">
	import Card from '$lib/components/ui/card.svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import type { Agent, AgentStatus } from '$lib/api/types.js';
	import type { BadgeVariant } from '$lib/components/ui/badge.svelte';
	import { Bot, Zap, Wrench } from 'lucide-svelte';

	let { agent }: { agent: Agent } = $props();

	function statusVariant(status: AgentStatus): BadgeVariant {
		switch (status) {
			case 'running':
				return 'success';
			case 'error':
				return 'destructive';
			case 'scheduled':
				return 'info';
			default:
				return 'secondary';
		}
	}

	function statusLabel(status: AgentStatus): string {
		return status.charAt(0).toUpperCase() + status.slice(1);
	}

	function modeVariant(mode: string): BadgeVariant {
		switch (mode) {
			case 'autonomous':
				return 'default';
			case 'semi-autonomous':
				return 'info';
			default:
				return 'outline';
		}
	}
</script>

<a
	href="/agents/{encodeURIComponent(agent.name)}"
	class="block focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 rounded-xl"
>
	<Card class="p-5 hover:shadow-md transition-shadow duration-200 cursor-pointer h-full">
		<div class="flex items-start justify-between gap-3">
			<div class="flex min-w-0 items-center gap-3">
				<div class="shrink-0 rounded-lg bg-muted p-2">
					<Bot class="h-5 w-5 text-muted-foreground" />
				</div>
				<div class="min-w-0">
					<p class="truncate font-semibold text-sm leading-tight">{agent.name}</p>
					<p class="text-xs text-muted-foreground truncate">{agent.model.preferred}</p>
				</div>
			</div>
			<Badge variant={statusVariant(agent.status)}>
				{statusLabel(agent.status)}
			</Badge>
		</div>

		<div class="mt-4 flex flex-wrap gap-2">
			<Badge variant={modeVariant(agent.mode)}>
				{agent.mode}
			</Badge>
			{#if agent.triggers && agent.triggers.length > 0}
				<Badge variant="outline">
					<Zap class="mr-1 h-3 w-3" />
					{agent.triggers.length} trigger{agent.triggers.length !== 1 ? 's' : ''}
				</Badge>
			{/if}
			{#if agent.skills && agent.skills.length > 0}
				<Badge variant="outline">
					<Wrench class="mr-1 h-3 w-3" />
					{agent.skills.length} skill{agent.skills.length !== 1 ? 's' : ''}
				</Badge>
			{/if}
		</div>
	</Card>
</a>
