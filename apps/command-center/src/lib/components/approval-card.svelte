<script lang="ts">
	import { CheckCircle, XCircle, Clock, Bot } from 'lucide-svelte';
	import Badge from '$lib/components/ui/badge.svelte';
	import type { ApprovalRequest, ApprovalStatus } from '$lib/api/types.js';
	import type { BadgeVariant } from '$lib/components/ui/badge.svelte';

	interface Props {
		request: ApprovalRequest;
		onapprove: (id: string) => void;
		ondeny: (id: string, reason?: string) => void;
	}

	let { request, onapprove, ondeny }: Props = $props();

	// Deny flow state
	let showDenyInput = $state(false);
	let denyReason = $state('');
	let actioning = $state(false);

	function statusVariant(status: ApprovalStatus): BadgeVariant {
		switch (status) {
			case 'approved':
				return 'success';
			case 'denied':
				return 'destructive';
			case 'expired':
				return 'secondary';
			default:
				return 'warning';
		}
	}

	function statusLabel(status: ApprovalStatus): string {
		return status.charAt(0).toUpperCase() + status.slice(1);
	}

	/** Format a date string as a relative time label (e.g. "5 min ago"). */
	function timeAgo(isoString: string): string {
		const diff = Math.floor((Date.now() - new Date(isoString).getTime()) / 1000);
		if (diff < 60) return `${diff}s ago`;
		if (diff < 3600) return `${Math.floor(diff / 60)} min ago`;
		if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
		return `${Math.floor(diff / 86400)}d ago`;
	}

	async function handleApprove() {
		actioning = true;
		onapprove(request.id);
	}

	function handleDenyClick() {
		showDenyInput = true;
	}

	async function handleDenyConfirm() {
		actioning = true;
		ondeny(request.id, denyReason.trim() || undefined);
		showDenyInput = false;
		denyReason = '';
	}

	function handleDenyCancel() {
		showDenyInput = false;
		denyReason = '';
	}
</script>

<div
	class="rounded-xl border bg-card overflow-hidden transition-shadow duration-200 hover:shadow-md {request.status === 'pending'
		? 'border-amber-500/40'
		: ''}"
>
	<!-- Pending urgency: pulsing left border accent -->
	{#if request.status === 'pending'}
		<div class="h-0.5 w-full bg-amber-400 animate-pulse"></div>
	{/if}

	<div class="p-5">
		<!-- Header: Agent name + status badge -->
		<div class="flex items-start justify-between gap-3 mb-3">
			<div class="flex items-center gap-2 min-w-0">
				<div class="shrink-0 rounded-lg bg-muted p-1.5">
					<Bot class="h-4 w-4 text-muted-foreground" />
				</div>
				<span class="font-semibold text-sm truncate">{request.agent_name}</span>
			</div>
			<Badge variant={statusVariant(request.status)}>
				{statusLabel(request.status)}
			</Badge>
		</div>

		<!-- Action description — the main content -->
		<div class="rounded-lg bg-muted/50 border border-muted px-3 py-2.5 mb-3">
			<p class="text-sm leading-relaxed text-foreground">{request.action_description}</p>
		</div>

		<!-- Meta: time + run_id link -->
		<div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground mb-4">
			<span class="flex items-center gap-1">
				<Clock class="h-3 w-3" />
				{timeAgo(request.requested_at)}
			</span>
			<a
				href="/runs/{encodeURIComponent(request.agent_name)}/{encodeURIComponent(request.run_id)}"
				class="hover:text-foreground underline underline-offset-2 transition-colors font-mono truncate max-w-[120px]"
				title="View run trace"
			>
				{request.run_id.slice(0, 12)}...
			</a>
			{#if request.decided_by}
				<span>by <span class="text-foreground">{request.decided_by}</span></span>
			{/if}
			{#if request.decided_at}
				<span>decided {timeAgo(request.decided_at)}</span>
			{/if}
		</div>

		<!-- Footer: action buttons (only for pending) -->
		{#if request.status === 'pending'}
			{#if showDenyInput}
				<!-- Deny confirmation flow -->
				<div class="space-y-2">
					<p class="text-xs font-medium text-muted-foreground">Reason for denial (optional):</p>
					<input
						bind:value={denyReason}
						type="text"
						placeholder="e.g. Not authorized at this time"
						class="w-full rounded-md border bg-background px-3 py-1.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
						onkeydown={(e) => {
							if (e.key === 'Enter') handleDenyConfirm();
							if (e.key === 'Escape') handleDenyCancel();
						}}
					/>
					<div class="flex gap-2">
						<button
							onclick={handleDenyConfirm}
							disabled={actioning}
							class="inline-flex items-center gap-1.5 rounded-md bg-destructive px-3 py-1.5 text-xs font-medium text-destructive-foreground hover:bg-destructive/90 transition-colors disabled:opacity-50 disabled:pointer-events-none cursor-pointer"
						>
							<XCircle class="h-3.5 w-3.5" />
							Confirm Deny
						</button>
						<button
							onclick={handleDenyCancel}
							class="inline-flex items-center rounded-md border px-3 py-1.5 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
						>
							Cancel
						</button>
					</div>
				</div>
			{:else}
				<!-- Approve / Deny buttons -->
				<div class="flex gap-2">
					<button
						onclick={handleApprove}
						disabled={actioning}
						class="inline-flex flex-1 items-center justify-center gap-1.5 rounded-md bg-success px-4 py-2 text-sm font-medium text-white hover:bg-success/90 transition-colors disabled:opacity-50 disabled:pointer-events-none cursor-pointer"
					>
						<CheckCircle class="h-4 w-4" />
						Approve
					</button>
					<button
						onclick={handleDenyClick}
						disabled={actioning}
						class="inline-flex flex-1 items-center justify-center gap-1.5 rounded-md border border-destructive/60 bg-transparent px-4 py-2 text-sm font-medium text-destructive hover:bg-destructive/10 transition-colors disabled:opacity-50 disabled:pointer-events-none cursor-pointer"
					>
						<XCircle class="h-4 w-4" />
						Deny
					</button>
				</div>
			{/if}
		{:else}
			<!-- Decided outcome display -->
			<div class="flex items-center gap-2 text-sm">
				{#if request.status === 'approved'}
					<CheckCircle class="h-4 w-4 text-success shrink-0" />
					<span class="text-success font-medium">Approved</span>
				{:else if request.status === 'denied'}
					<XCircle class="h-4 w-4 text-destructive shrink-0" />
					<span class="text-destructive font-medium">Denied</span>
				{:else}
					<Clock class="h-4 w-4 text-muted-foreground shrink-0" />
					<span class="text-muted-foreground font-medium">Expired</span>
				{/if}
				{#if request.decided_at}
					<span class="text-xs text-muted-foreground ml-auto">{timeAgo(request.decided_at)}</span>
				{/if}
			</div>
		{/if}
	</div>
</div>
