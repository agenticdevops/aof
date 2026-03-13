<script lang="ts">
	import { onMount } from 'svelte';
	import { ShieldCheck, CheckCircle, ChevronDown, ChevronUp, RefreshCw } from 'lucide-svelte';
	import ApprovalCard from '$lib/components/approval-card.svelte';
	import Skeleton from '$lib/components/ui/skeleton.svelte';
	import {
		approvals,
		approvalsLoading,
		approvalsError,
		pendingApprovals,
		decidedApprovals,
		loadApprovals,
		approveRequest,
		denyRequest
	} from '$lib/stores/approvals.js';
	import type { ApprovalStatus } from '$lib/api/types.js';

	// --------------------------------------------------------
	// Local state
	// --------------------------------------------------------

	type FilterOption = 'all' | ApprovalStatus;

	let filter = $state<FilterOption>('all');
	let showHistory = $state(false);
	let toastMessage = $state<string | null>(null);
	let toastType = $state<'success' | 'error'>('success');

	const filterOptions: Array<{ value: FilterOption; label: string }> = [
		{ value: 'all', label: 'All' },
		{ value: 'pending', label: 'Pending' },
		{ value: 'approved', label: 'Approved' },
		{ value: 'denied', label: 'Denied' }
	];

	// --------------------------------------------------------
	// Derived: filtered views
	// --------------------------------------------------------

	const filteredPending = $derived(
		filter === 'all' || filter === 'pending' ? $pendingApprovals : []
	);

	const filteredDecided = $derived(
		filter === 'all' || filter === 'approved' || filter === 'denied'
			? $decidedApprovals.filter((r) => filter === 'all' || r.status === filter).sort((a, b) => {
					const da = a.decided_at ? new Date(a.decided_at).getTime() : 0;
					const db = b.decided_at ? new Date(b.decided_at).getTime() : 0;
					return db - da;
				})
			: []
	);

	// --------------------------------------------------------
	// Toast helpers
	// --------------------------------------------------------

	let toastTimer: ReturnType<typeof setTimeout> | null = null;

	function showToast(message: string, type: 'success' | 'error') {
		if (toastTimer) clearTimeout(toastTimer);
		toastMessage = message;
		toastType = type;
		toastTimer = setTimeout(() => {
			toastMessage = null;
		}, 3000);
	}

	// --------------------------------------------------------
	// Actions
	// --------------------------------------------------------

	async function handleApprove(id: string) {
		const ok = await approveRequest(id);
		if (ok) {
			showToast('Request approved — agent will resume.', 'success');
		} else {
			showToast($approvalsError ?? 'Failed to approve request.', 'error');
		}
	}

	async function handleDeny(id: string, reason?: string) {
		const ok = await denyRequest(id, reason);
		if (ok) {
			showToast('Request denied — agent has been stopped.', 'success');
		} else {
			showToast($approvalsError ?? 'Failed to deny request.', 'error');
		}
	}

	// --------------------------------------------------------
	// Lifecycle
	// --------------------------------------------------------

	onMount(() => {
		loadApprovals();
	});
</script>

<div class="p-6 space-y-6">
	<!-- Page header -->
	<div class="flex items-start justify-between gap-4">
		<div>
			<div class="flex items-center gap-2">
				<ShieldCheck class="h-6 w-6 text-primary" />
				<h1 class="text-2xl font-bold tracking-tight">
					Approval Queue
					{#if $pendingApprovals.length > 0}
						<span class="ml-2 text-base font-medium text-amber-500">
							({$pendingApprovals.length} pending)
						</span>
					{/if}
				</h1>
			</div>
			<p class="text-sm text-muted-foreground mt-1">
				Human-in-the-loop control for semi-autonomous agents. Review and action requests before
				agents proceed.
			</p>
		</div>
		<button
			onclick={loadApprovals}
			disabled={$approvalsLoading}
			class="inline-flex items-center gap-1.5 rounded-md border px-3 py-1.5 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors disabled:opacity-50 disabled:pointer-events-none cursor-pointer"
		>
			<RefreshCw class="h-4 w-4 {$approvalsLoading ? 'animate-spin' : ''}" />
			Refresh
		</button>
	</div>

	<!-- Toast notification -->
	{#if toastMessage}
		<div
			class="rounded-lg border px-4 py-3 text-sm font-medium transition-all {toastType === 'success'
				? 'border-success/30 bg-success/10 text-success'
				: 'border-destructive/30 bg-destructive/10 text-destructive'}"
		>
			{toastMessage}
		</div>
	{/if}

	<!-- Error state -->
	{#if $approvalsError}
		<div class="rounded-xl border border-destructive/30 bg-destructive/10 px-4 py-3">
			<p class="text-sm font-medium text-destructive">{$approvalsError}</p>
			<button
				onclick={loadApprovals}
				class="mt-2 text-xs underline text-destructive cursor-pointer"
			>
				Retry
			</button>
		</div>
	{/if}

	<!-- Filter bar -->
	<div class="flex flex-wrap gap-2">
		{#each filterOptions as opt}
			<button
				onclick={() => (filter = opt.value)}
				class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors cursor-pointer {filter ===
				opt.value
					? 'bg-primary text-primary-foreground'
					: 'border bg-card text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
			>
				{opt.label}
			</button>
		{/each}
	</div>

	<!-- Loading skeleton -->
	{#if $approvalsLoading}
		<div class="space-y-4">
			{#each { length: 3 } as _}
				<Skeleton class="h-40 rounded-xl" />
			{/each}
		</div>

	<!-- Main content -->
	{:else}
		<!-- Pending section -->
		<section class="space-y-4">
			<div class="flex items-center gap-2">
				<h2 class="text-base font-semibold">
					Pending
					{#if filteredPending.length > 0}
						<span
							class="ml-1.5 inline-flex items-center rounded-full bg-amber-500/20 px-2 py-0.5 text-xs font-semibold text-amber-600 dark:text-amber-400"
						>
							{filteredPending.length}
						</span>
					{/if}
				</h2>
			</div>

			{#if filteredPending.length > 0}
				<div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
					{#each filteredPending as request (request.id)}
						<div class="animate-in fade-in-0 slide-in-from-top-2 duration-300">
							<ApprovalCard
								{request}
								onapprove={handleApprove}
								ondeny={handleDeny}
							/>
						</div>
					{/each}
				</div>
			{:else if filter === 'all' || filter === 'pending'}
				<!-- All clear empty state -->
				<div
					class="flex flex-col items-center justify-center rounded-xl border border-dashed p-14 text-center"
				>
					<div class="mb-3 rounded-full bg-success/10 p-3">
						<CheckCircle class="h-8 w-8 text-success" />
					</div>
					<p class="text-sm font-semibold text-foreground">No pending approvals — all clear</p>
					<p class="text-xs text-muted-foreground mt-1">
						Agents are running autonomously or waiting for triggers.
					</p>
				</div>
			{/if}
		</section>

		<!-- History section -->
		{#if (filter === 'all' || filter === 'approved' || filter === 'denied') && $decidedApprovals.length > 0}
			<section class="space-y-3">
				<button
					onclick={() => (showHistory = !showHistory)}
					class="flex items-center gap-2 text-sm font-semibold text-muted-foreground hover:text-foreground transition-colors cursor-pointer"
				>
					{#if showHistory}
						<ChevronUp class="h-4 w-4" />
					{:else}
						<ChevronDown class="h-4 w-4" />
					{/if}
					History
					<span class="text-xs font-normal">({filteredDecided.length} decisions)</span>
				</button>

				{#if showHistory}
					<div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
						{#each filteredDecided as request (request.id)}
							<ApprovalCard
								{request}
								onapprove={handleApprove}
								ondeny={handleDeny}
							/>
						{/each}
					</div>
				{/if}
			</section>
		{/if}

		<!-- Empty state when approvals list is empty overall -->
		{#if $approvals.length === 0 && !$approvalsLoading}
			<div
				class="flex flex-col items-center justify-center rounded-xl border border-dashed p-14 text-center"
			>
				<div class="mb-3 rounded-full bg-muted p-3">
					<ShieldCheck class="h-8 w-8 text-muted-foreground/50" />
				</div>
				<p class="text-sm font-medium text-muted-foreground">No approval requests yet</p>
				<p class="text-xs text-muted-foreground/70 mt-1">
					Requests appear here when semi-autonomous agents need authorization to proceed.
				</p>
			</div>
		{/if}
	{/if}
</div>
