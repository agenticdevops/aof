<script lang="ts">
	import { dateRange } from '$lib/stores/costs.js';
	import { Calendar } from 'lucide-svelte';

	// Quick-select presets
	const presets = [
		{ label: '7 days', days: 7 },
		{ label: '30 days', days: 30 },
		{ label: '90 days', days: 90 },
		{ label: 'Year', days: 365 }
	];

	let startValue = $state(toInputValue($dateRange.start));
	let endValue = $state(toInputValue($dateRange.end));

	function toInputValue(date: Date): string {
		return date.toISOString().slice(0, 10);
	}

	function applyPreset(days: number) {
		const end = new Date();
		const start = new Date();
		start.setDate(start.getDate() - days);
		endValue = toInputValue(end);
		startValue = toInputValue(start);
		dateRange.set({ start, end });
	}

	function onStartChange(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		startValue = input.value;
		if (startValue && endValue) {
			dateRange.set({
				start: new Date(startValue),
				end: new Date(endValue)
			});
		}
	}

	function onEndChange(e: Event) {
		const input = e.currentTarget as HTMLInputElement;
		endValue = input.value;
		if (startValue && endValue) {
			dateRange.set({
				start: new Date(startValue),
				end: new Date(endValue)
			});
		}
	}

	// Detect which preset is currently active (if any)
	function isActivePreset(days: number): boolean {
		const end = new Date(endValue);
		const start = new Date(startValue);
		const diffDays = Math.round((end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24));
		return diffDays === days;
	}
</script>

<div class="flex flex-wrap items-center gap-3">
	<!-- Calendar icon + label -->
	<div class="flex items-center gap-1.5 text-muted-foreground">
		<Calendar class="h-4 w-4" />
		<span class="text-sm font-medium">Date Range</span>
	</div>

	<!-- Quick-select presets -->
	<div class="flex items-center gap-1">
		{#each presets as preset}
			<button
				type="button"
				onclick={() => applyPreset(preset.days)}
				class={[
					'rounded-md px-2.5 py-1 text-xs font-medium transition-colors duration-150 cursor-pointer',
					isActivePreset(preset.days)
						? 'bg-primary text-primary-foreground'
						: 'bg-muted text-muted-foreground hover:bg-accent hover:text-accent-foreground'
				].join(' ')}
			>
				{preset.label}
			</button>
		{/each}
	</div>

	<!-- Divider -->
	<div class="h-4 w-px bg-border"></div>

	<!-- Custom date inputs -->
	<div class="flex items-center gap-2">
		<input
			type="date"
			value={startValue}
			onchange={onStartChange}
			class="h-8 rounded-md border border-input bg-background px-2.5 py-1 text-xs text-foreground focus:outline-none focus:ring-2 focus:ring-ring cursor-pointer"
		/>
		<span class="text-xs text-muted-foreground">to</span>
		<input
			type="date"
			value={endValue}
			onchange={onEndChange}
			class="h-8 rounded-md border border-input bg-background px-2.5 py-1 text-xs text-foreground focus:outline-none focus:ring-2 focus:ring-ring cursor-pointer"
		/>
	</div>
</div>
