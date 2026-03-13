<script lang="ts">
	import Badge from '$lib/components/ui/badge.svelte';
	import type { AgentRun, RunStatus } from '$lib/api/types.js';
	import type { BadgeVariant } from '$lib/components/ui/badge.svelte';

	let { run }: { run: AgentRun } = $props();

	function statusVariant(status: RunStatus): BadgeVariant {
		switch (status) {
			case 'completed':
				return 'success';
			case 'running':
				return 'info';
			case 'failed':
				return 'destructive';
			case 'stopped':
				return 'warning';
			default:
				return 'secondary';
		}
	}

	function formatRelativeTime(isoStr: string): string {
		const diff = Date.now() - new Date(isoStr).getTime();
		const secs = Math.floor(diff / 1000);
		if (secs < 60) return `${secs}s ago`;
		const mins = Math.floor(secs / 60);
		if (mins < 60) return `${mins}m ago`;
		const hours = Math.floor(mins / 60);
		if (hours < 24) return `${hours}h ago`;
		const days = Math.floor(hours / 24);
		return `${days}d ago`;
	}

	function formatDuration(ms?: number): string {
		if (ms === undefined || ms === null) return '—';
		if (ms < 1000) return `${ms}ms`;
		const secs = Math.floor(ms / 1000);
		if (secs < 60) return `${secs}s`;
		const mins = Math.floor(secs / 60);
		const remainSecs = secs % 60;
		return `${mins}m ${remainSecs}s`;
	}

	function formatCost(cost?: { total_cost_usd: number }): string {
		if (!cost) return '—';
		return `$${cost.total_cost_usd.toFixed(4)}`;
	}

	const shortRunId = $derived(run.run_id.length > 12 ? run.run_id.slice(0, 12) + '…' : run.run_id);
</script>

<a
	href="/traces/{encodeURIComponent(run.agent_name)}/{encodeURIComponent(run.run_id)}"
	class="group flex items-center gap-4 px-4 py-3 hover:bg-muted/40 transition-colors cursor-pointer border-b last:border-0"
	title="View trace for {run.run_id}"
>
	<!-- Agent name -->
	<span class="w-36 shrink-0 truncate text-sm font-medium">{run.agent_name}</span>

	<!-- Run ID -->
	<span class="w-32 shrink-0 font-mono text-xs text-muted-foreground">{shortRunId}</span>

	<!-- Status -->
	<div class="w-24 shrink-0">
		<Badge variant={statusVariant(run.status)}>{run.status}</Badge>
	</div>

	<!-- Trigger source -->
	<span class="hidden flex-1 truncate text-xs text-muted-foreground sm:block">{run.trigger_source}</span>

	<!-- Started at -->
	<span class="w-20 shrink-0 text-xs text-muted-foreground text-right">{formatRelativeTime(run.started_at)}</span>

	<!-- Duration -->
	<span class="w-16 shrink-0 text-xs text-muted-foreground text-right">{formatDuration(run.duration_ms)}</span>

	<!-- Cost -->
	<span class="w-16 shrink-0 text-xs text-muted-foreground text-right">{formatCost(run.cost)}</span>
</a>
