<script lang="ts">
	import { onMount } from 'svelte';
	import { DollarSign, TrendingUp, Zap, BarChart2, ChevronDown, X } from 'lucide-svelte';
	import {
		costSummaries,
		costLoading,
		costError,
		selectedAgent,
		agentRunCosts,
		agentCostLoading,
		filteredRunCosts,
		dateRange,
		loadCosts,
		loadAgentCosts,
		clearSelectedAgent
	} from '$lib/stores/costs.js';
	import {
		aggregateCostsByDate,
		aggregateCostsByAgent,
		formatCurrency,
		formatTokens
	} from '$lib/utils/chart-helpers.js';
	import DateRangePicker from '$lib/components/date-range-picker.svelte';
	import CostBarChart from '$lib/components/cost-bar-chart.svelte';
	import CostLineChart from '$lib/components/cost-line-chart.svelte';
	import CostPieChart from '$lib/components/cost-pie-chart.svelte';

	onMount(() => {
		loadCosts();
	});

	// ============================================================
	// Derived data for charts
	// ============================================================

	let agentDistribution = $derived(aggregateCostsByAgent($costSummaries));

	// Aggregate all runs across all agents by date (from agentRunCosts when drill-down active)
	// For the "all agents" aggregate line we use cost summaries data (synthetic daily from total)
	// We generate a daily distribution from run data if available; fall back to a flat line.
	let aggregateByDate = $derived.by(() => {
		const runs = $filteredRunCosts;
		if (runs.length > 0) {
			return aggregateCostsByDate(runs);
		}
		// No run data — synthesize from summaries (flat lines showing totals only)
		return [];
	});

	let agentRunsByDate = $derived(aggregateCostsByDate($filteredRunCosts));

	// ============================================================
	// Summary cards
	// ============================================================

	let totalCost = $derived(
		$costSummaries.reduce((sum, s) => sum + s.total_cost_usd, 0)
	);

	let totalRuns = $derived($costSummaries.reduce((sum, s) => sum + s.total_runs, 0));

	let mostExpensive = $derived(
		$costSummaries.length > 0
			? $costSummaries.reduce((max, s) =>
					s.total_cost_usd > max.total_cost_usd ? s : max
				)
			: null
	);

	let avgCostPerRun = $derived(totalRuns > 0 ? totalCost / totalRuns : 0);

	// ============================================================
	// Drill-down
	// ============================================================

	function handleAgentClick(name: string) {
		if ($selectedAgent === name) {
			clearSelectedAgent();
		} else {
			loadAgentCosts(name);
		}
	}

	// Budget % helper
	function budgetPct(s: (typeof $costSummaries)[0]): number | null {
		if (!s.daily_budget || s.daily_budget <= 0) return null;
		// Approximate: assume 30-day window
		const dailySpend = s.total_cost_usd / 30;
		return Math.min((dailySpend / s.daily_budget) * 100, 999);
	}

	function budgetColor(pct: number): string {
		if (pct >= 90) return 'text-destructive';
		if (pct >= 70) return 'text-warning';
		return 'text-success';
	}
</script>

<svelte:head>
	<title>Cost Dashboard — OpenAgentiX</title>
</svelte:head>

<div class="p-6 space-y-6">
	<!-- ======================================================== -->
	<!-- Header                                                    -->
	<!-- ======================================================== -->
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div class="flex items-center gap-3">
			<div
				class="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-teal-500 to-sky-500 shadow-sm"
			>
				<DollarSign class="h-5 w-5 text-white" />
			</div>
			<div>
				<h1 class="text-2xl font-bold tracking-tight">Cost Dashboard</h1>
				<p class="text-sm text-muted-foreground">Monitor and analyze AI agent spend</p>
			</div>
		</div>
		<DateRangePicker />
	</div>

	<!-- ======================================================== -->
	<!-- Error banner                                              -->
	<!-- ======================================================== -->
	{#if $costError}
		<div
			class="flex items-start gap-3 rounded-lg border border-destructive/30 bg-destructive/10 p-4"
		>
			<p class="text-sm text-destructive">{$costError}</p>
			<button
				type="button"
				onclick={() => loadCosts()}
				class="ml-auto shrink-0 text-xs font-medium text-destructive underline hover:no-underline cursor-pointer"
			>
				Retry
			</button>
		</div>
	{/if}

	<!-- ======================================================== -->
	<!-- Summary cards                                            -->
	<!-- ======================================================== -->
	<div class="grid grid-cols-2 gap-4 lg:grid-cols-4">
		<!-- Total Cost -->
		<div
			class="rounded-xl border bg-card p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="flex items-center justify-between">
				<p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
					Total Cost
				</p>
				<DollarSign class="h-4 w-4 text-teal-500" />
			</div>
			{#if $costLoading}
				<div class="mt-3 h-8 w-24 animate-pulse rounded-md bg-muted"></div>
			{:else}
				<p class="mt-2 text-3xl font-bold tracking-tight">{formatCurrency(totalCost)}</p>
				<p class="mt-1 text-xs text-muted-foreground">across {$costSummaries.length} agents</p>
			{/if}
		</div>

		<!-- Most Expensive Agent -->
		<div
			class="rounded-xl border bg-card p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="flex items-center justify-between">
				<p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
					Top Agent
				</p>
				<TrendingUp class="h-4 w-4 text-violet-500" />
			</div>
			{#if $costLoading}
				<div class="mt-3 h-8 w-28 animate-pulse rounded-md bg-muted"></div>
			{:else if mostExpensive}
				<p class="mt-2 text-xl font-bold tracking-tight truncate" title={mostExpensive.agent_name}>
					{mostExpensive.agent_name}
				</p>
				<p class="mt-1 text-xs text-muted-foreground">
					{formatCurrency(mostExpensive.total_cost_usd)}
				</p>
			{:else}
				<p class="mt-2 text-xl font-bold text-muted-foreground">—</p>
			{/if}
		</div>

		<!-- Avg Cost / Run -->
		<div
			class="rounded-xl border bg-card p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="flex items-center justify-between">
				<p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
					Avg / Run
				</p>
				<Zap class="h-4 w-4 text-amber-500" />
			</div>
			{#if $costLoading}
				<div class="mt-3 h-8 w-20 animate-pulse rounded-md bg-muted"></div>
			{:else}
				<p class="mt-2 text-3xl font-bold tracking-tight">{formatCurrency(avgCostPerRun)}</p>
				<p class="mt-1 text-xs text-muted-foreground">per agent run</p>
			{/if}
		</div>

		<!-- Total Runs -->
		<div
			class="rounded-xl border bg-card p-5 shadow-sm transition-shadow hover:shadow-md"
		>
			<div class="flex items-center justify-between">
				<p class="text-xs font-medium uppercase tracking-wide text-muted-foreground">
					Total Runs
				</p>
				<BarChart2 class="h-4 w-4 text-sky-500" />
			</div>
			{#if $costLoading}
				<div class="mt-3 h-8 w-16 animate-pulse rounded-md bg-muted"></div>
			{:else}
				<p class="mt-2 text-3xl font-bold tracking-tight">
					{totalRuns.toLocaleString()}
				</p>
				<p class="mt-1 text-xs text-muted-foreground">all-time</p>
			{/if}
		</div>
	</div>

	<!-- ======================================================== -->
	<!-- Charts Row 1: Bar (2/3) + Pie (1/3)                     -->
	<!-- ======================================================== -->
	<div class="grid grid-cols-1 gap-4 lg:grid-cols-3">
		<!-- Bar chart — per-agent costs -->
		<div class="lg:col-span-2 rounded-xl border bg-card p-5 shadow-sm">
			<div class="mb-4">
				<h2 class="text-sm font-semibold">Cost by Agent</h2>
				<p class="text-xs text-muted-foreground">Sorted by total spend — click to drill down</p>
			</div>
			{#if $costLoading}
				<div class="space-y-2">
					{#each Array(4) as _}
						<div class="flex items-center gap-3">
							<div class="h-4 w-24 animate-pulse rounded bg-muted"></div>
							<div class="h-6 flex-1 animate-pulse rounded bg-muted"></div>
						</div>
					{/each}
				</div>
			{:else}
				<div
					class="h-64"
					style={$costSummaries.length > 6
						? `height: ${Math.max(200, $costSummaries.length * 40)}px`
						: ''}
				>
					<CostBarChart
						data={$costSummaries}
						onagentclick={handleAgentClick}
					/>
				</div>
			{/if}
		</div>

		<!-- Pie chart — cost distribution -->
		<div class="rounded-xl border bg-card p-5 shadow-sm">
			<div class="mb-4">
				<h2 class="text-sm font-semibold">Cost Distribution</h2>
				<p class="text-xs text-muted-foreground">Click segment to drill down</p>
			</div>
			{#if $costLoading}
				<div class="flex h-64 items-center justify-center">
					<div class="h-40 w-40 animate-pulse rounded-full bg-muted"></div>
				</div>
			{:else}
				<div class="h-64">
					<CostPieChart
						data={agentDistribution}
						onagentclick={handleAgentClick}
					/>
				</div>
			{/if}
		</div>
	</div>

	<!-- ======================================================== -->
	<!-- Chart Row 2: Line — aggregate cost over time             -->
	<!-- ======================================================== -->
	<div class="rounded-xl border bg-card p-5 shadow-sm">
		<div class="mb-4 flex items-center justify-between">
			<div>
				<h2 class="text-sm font-semibold">Cost Over Time</h2>
				<p class="text-xs text-muted-foreground">
					{#if $selectedAgent}
						Showing <span class="font-medium text-sky-500">{$selectedAgent}</span> vs aggregate
					{:else}
						Daily cost totals — select an agent to overlay
					{/if}
				</p>
			</div>
			{#if $selectedAgent}
				<button
					type="button"
					onclick={clearSelectedAgent}
					class="flex items-center gap-1 rounded-md px-2.5 py-1 text-xs font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
				>
					<X class="h-3 w-3" />
					Clear
				</button>
			{/if}
		</div>
		{#if $costLoading || $agentCostLoading}
			<div class="h-52 w-full animate-pulse rounded-lg bg-muted"></div>
		{:else}
			<div class="h-52">
				<CostLineChart
					data={aggregateByDate}
					agentData={agentRunsByDate}
					agentName={$selectedAgent}
				/>
			</div>
		{/if}
	</div>

	<!-- ======================================================== -->
	<!-- Agent Cost Breakdown Table                               -->
	<!-- ======================================================== -->
	<div class="rounded-xl border bg-card shadow-sm">
		<div class="px-5 py-4 border-b">
			<h2 class="text-sm font-semibold">Agent Cost Breakdown</h2>
		</div>

		{#if $costLoading}
			<div class="space-y-3 p-5">
				{#each Array(3) as _}
					<div class="h-10 w-full animate-pulse rounded bg-muted"></div>
				{/each}
			</div>
		{:else if $costSummaries.length === 0}
			<div class="flex flex-col items-center justify-center px-5 py-16 text-center">
				<DollarSign class="h-8 w-8 text-muted-foreground/40 mb-3" />
				<p class="text-sm font-medium text-muted-foreground">No cost data yet</p>
				<p class="mt-1 text-xs text-muted-foreground/70 max-w-sm">
					Cost data appears after agents have run. Register and run an agent to see spend
					analytics.
				</p>
			</div>
		{:else}
			<div class="overflow-x-auto">
				<table class="w-full text-sm">
					<thead>
						<tr class="border-b text-xs font-medium text-muted-foreground">
							<th class="px-5 py-3 text-left">Agent</th>
							<th class="px-4 py-3 text-right">Runs</th>
							<th class="px-4 py-3 text-right">Input Tokens</th>
							<th class="px-4 py-3 text-right">Output Tokens</th>
							<th class="px-4 py-3 text-right">Total Cost</th>
							<th class="px-4 py-3 text-right">Daily Budget</th>
							<th class="px-4 py-3 text-right">Budget Used</th>
						</tr>
					</thead>
					<tbody>
						{#each $costSummaries.sort((a, b) => b.total_cost_usd - a.total_cost_usd) as summary}
							{@const pct = budgetPct(summary)}
							<tr
								class={[
									'border-b last:border-0 transition-colors cursor-pointer hover:bg-accent/50',
									$selectedAgent === summary.agent_name
										? 'bg-sky-50 dark:bg-sky-950/30'
										: ''
								].join(' ')}
								onclick={() => handleAgentClick(summary.agent_name)}
							>
								<td class="px-5 py-3.5">
									<div class="flex items-center gap-2">
										<div
											class="flex h-6 w-6 items-center justify-center rounded-full text-xs font-bold text-white"
											style="background: linear-gradient(135deg, #14b8a6, #0ea5e9)"
										>
											{summary.agent_name[0]?.toUpperCase() ?? '?'}
										</div>
										<a
											href={`/agents/${encodeURIComponent(summary.agent_name)}`}
											onclick={(e) => e.stopPropagation()}
											class="font-medium hover:text-sky-500 hover:underline transition-colors"
										>
											{summary.agent_name}
										</a>
										{#if $selectedAgent === summary.agent_name}
											<span
												class="rounded-full bg-sky-100 px-2 py-0.5 text-[10px] font-medium text-sky-600 dark:bg-sky-900/50 dark:text-sky-400"
											>
												selected
											</span>
										{/if}
									</div>
								</td>
								<td class="px-4 py-3.5 text-right tabular-nums">{summary.total_runs}</td>
								<td class="px-4 py-3.5 text-right tabular-nums text-muted-foreground">
									{formatTokens(summary.total_input_tokens)}
								</td>
								<td class="px-4 py-3.5 text-right tabular-nums text-muted-foreground">
									{formatTokens(summary.total_output_tokens)}
								</td>
								<td class="px-4 py-3.5 text-right tabular-nums font-semibold">
									{formatCurrency(summary.total_cost_usd)}
								</td>
								<td class="px-4 py-3.5 text-right tabular-nums text-muted-foreground">
									{summary.daily_budget != null ? formatCurrency(summary.daily_budget) : '—'}
								</td>
								<td class="px-4 py-3.5 text-right">
									{#if pct !== null}
										<span class={`font-medium ${budgetColor(pct)}`}>
											{pct.toFixed(0)}%
										</span>
									{:else}
										<span class="text-muted-foreground">—</span>
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>

	<!-- ======================================================== -->
	<!-- Drill-down panel: per-run cost detail                    -->
	<!-- ======================================================== -->
	{#if $selectedAgent}
		<div class="rounded-xl border bg-card shadow-sm">
			<div class="flex items-center justify-between px-5 py-4 border-b">
				<div>
					<h2 class="text-sm font-semibold">
						Run Detail — <span class="text-sky-500">{$selectedAgent}</span>
					</h2>
					<p class="text-xs text-muted-foreground mt-0.5">
						Showing runs in selected date range
					</p>
				</div>
				<button
					type="button"
					onclick={clearSelectedAgent}
					class="rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
					aria-label="Close drill-down"
				>
					<X class="h-4 w-4" />
				</button>
			</div>

			{#if $agentCostLoading}
				<div class="space-y-3 p-5">
					{#each Array(4) as _}
						<div class="h-9 w-full animate-pulse rounded bg-muted"></div>
					{/each}
				</div>
			{:else if $filteredRunCosts.length === 0}
				<div class="flex flex-col items-center justify-center px-5 py-12 text-center">
					<ChevronDown class="h-8 w-8 text-muted-foreground/40 mb-3" />
					<p class="text-sm font-medium text-muted-foreground">No runs in this date range</p>
					<p class="mt-1 text-xs text-muted-foreground/70">
						Try expanding the date range filter above.
					</p>
				</div>
			{:else}
				<div class="overflow-x-auto">
					<table class="w-full text-sm">
						<thead>
							<tr class="border-b text-xs font-medium text-muted-foreground">
								<th class="px-5 py-3 text-left">Run ID</th>
								<th class="px-4 py-3 text-left">Started</th>
								<th class="px-4 py-3 text-right">Model</th>
								<th class="px-4 py-3 text-right">Input Tokens</th>
								<th class="px-4 py-3 text-right">Output Tokens</th>
								<th class="px-4 py-3 text-right">Cost</th>
							</tr>
						</thead>
						<tbody>
							{#each $filteredRunCosts as run}
								<tr class="border-b last:border-0 hover:bg-accent/50 transition-colors">
									<td class="px-5 py-3 font-mono text-xs text-muted-foreground truncate max-w-32">
										{run.run_id}
									</td>
									<td class="px-4 py-3 text-xs text-muted-foreground">
										{new Date(run.started_at).toLocaleString()}
									</td>
									<td class="px-4 py-3 text-right text-xs">
										{run.model_used}
									</td>
									<td class="px-4 py-3 text-right tabular-nums text-muted-foreground">
										{formatTokens(run.input_tokens)}
									</td>
									<td class="px-4 py-3 text-right tabular-nums text-muted-foreground">
										{formatTokens(run.output_tokens)}
									</td>
									<td class="px-4 py-3 text-right tabular-nums font-semibold text-teal-600 dark:text-teal-400">
										{formatCurrency(run.cost_usd)}
									</td>
								</tr>
							{/each}
						</tbody>
						<tfoot>
							<tr class="border-t bg-muted/30">
								<td colspan="5" class="px-5 py-3 text-sm font-medium text-right">
									Total ({$filteredRunCosts.length} runs)
								</td>
								<td class="px-4 py-3 text-right font-bold text-teal-600 dark:text-teal-400">
									{formatCurrency($filteredRunCosts.reduce((s, r) => s + r.cost_usd, 0))}
								</td>
							</tr>
						</tfoot>
					</table>
				</div>
			{/if}
		</div>
	{/if}
</div>
