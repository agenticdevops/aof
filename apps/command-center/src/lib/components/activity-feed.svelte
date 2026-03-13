<script lang="ts">
	import Card from '$lib/components/ui/card.svelte';
	import { Activity, Bot, PlayCircle, ShieldCheck, DollarSign, Wifi } from 'lucide-svelte';
	import type { GatewayEvent } from '$lib/stores/websocket.js';

	let { events = [] }: { events: GatewayEvent[] } = $props();

	function formatRelativeTime(ts: string): string {
		const diff = Date.now() - new Date(ts).getTime();
		const secs = Math.floor(diff / 1000);
		if (secs < 60) return `${secs}s ago`;
		const mins = Math.floor(secs / 60);
		if (mins < 60) return `${mins}m ago`;
		const hours = Math.floor(mins / 60);
		return `${hours}h ago`;
	}

	function eventDescription(e: GatewayEvent): string {
		switch (e.type) {
			case 'agent_status':
				return `Agent "${e.agent_name}" is now ${e.status}`;
			case 'run_started':
				return `Run started for "${e.agent_name}" (${e.trigger_source})`;
			case 'run_completed':
				return `Run ${e.run_id.slice(0, 8)} for "${e.agent_name}" ${e.status}${e.cost_usd ? ` — $${e.cost_usd.toFixed(4)}` : ''}`;
			case 'approval_requested':
				return `Approval requested: "${e.action_description}" (${e.agent_name})`;
			case 'approval_decided':
				return `Approval ${e.decision} for "${e.agent_name}"`;
			case 'cost_update':
				return `Cost update for "${e.agent_name}" — $${e.total_today_usd.toFixed(4)} today`;
			case 'connected':
				return 'Connected to gateway';
			default:
				return 'Unknown event';
		}
	}

	// Timestamp from event (events don't carry timestamps from server — use arrival time)
	// We store events with an arrival timestamp in the parent, so we accept a simple array here.
	// The parent should provide TimestampedEvent objects.
</script>

<Card class="flex flex-col overflow-hidden">
	<div class="border-b px-4 py-3">
		<div class="flex items-center gap-2">
			<Activity class="h-4 w-4 text-muted-foreground" />
			<h3 class="text-sm font-semibold">Activity Feed</h3>
		</div>
	</div>

	<div class="flex-1 overflow-y-auto divide-y">
		{#if events.length === 0}
			<div class="flex flex-col items-center justify-center py-12 text-center text-muted-foreground">
				<Wifi class="mb-2 h-6 w-6 opacity-40" />
				<p class="text-sm">No events yet</p>
				<p class="text-xs opacity-60">Events will appear here when agents are active</p>
			</div>
		{:else}
			{#each events as event, i (i)}
				<div class="flex items-start gap-3 px-4 py-3 hover:bg-muted/30 transition-colors">
					<div class="mt-0.5 shrink-0">
						{#if event.type === 'agent_status'}
							<Bot class="h-4 w-4 text-info" />
						{:else if event.type === 'run_started'}
							<PlayCircle class="h-4 w-4 text-success" />
						{:else if event.type === 'run_completed'}
							<PlayCircle class="h-4 w-4 text-muted-foreground" />
						{:else if event.type === 'approval_requested' || event.type === 'approval_decided'}
							<ShieldCheck class="h-4 w-4 text-warning" />
						{:else if event.type === 'cost_update'}
							<DollarSign class="h-4 w-4 text-muted-foreground" />
						{:else}
							<Wifi class="h-4 w-4 text-success" />
						{/if}
					</div>
					<p class="flex-1 text-xs leading-relaxed text-foreground">{eventDescription(event)}</p>
				</div>
			{/each}
		{/if}
	</div>
</Card>
