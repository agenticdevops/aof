<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Chart,
		BarController,
		BarElement,
		CategoryScale,
		LinearScale,
		Tooltip,
		Legend
	} from 'chart.js';
	import type { CostSummary } from '$lib/api/types.js';
	import {
		getAgentColor,
		getAgentHoverColor,
		formatCurrency,
		formatTokens,
		getChartTheme
	} from '$lib/utils/chart-helpers.js';

	Chart.register(BarController, BarElement, CategoryScale, LinearScale, Tooltip, Legend);

	interface Props {
		data: CostSummary[];
		onagentclick?: (name: string) => void;
	}

	let { data, onagentclick }: Props = $props();

	let canvas: HTMLCanvasElement = $state() as HTMLCanvasElement;
	let chart: Chart | undefined;

	function buildChart() {
		if (!canvas) return;

		const sorted = [...data].sort((a, b) => b.total_cost_usd - a.total_cost_usd);
		const theme = getChartTheme();

		const backgroundColors = sorted.map((_, i) => getAgentColor(i));
		const hoverColors = sorted.map((_, i) => getAgentHoverColor(i));

		chart = new Chart(canvas, {
			type: 'bar',
			data: {
				labels: sorted.map((s) => s.agent_name),
				datasets: [
					{
						label: 'Total Cost (USD)',
						data: sorted.map((s) => s.total_cost_usd),
						backgroundColor: backgroundColors,
						hoverBackgroundColor: hoverColors,
						borderRadius: 6,
						borderSkipped: false
					}
				]
			},
			options: {
				indexAxis: 'y',
				responsive: true,
				maintainAspectRatio: false,
				animation: {
					duration: 600,
					easing: 'easeOutQuart'
				},
				plugins: {
					legend: {
						display: false
					},
					tooltip: {
						backgroundColor: theme.backgroundColor,
						titleColor: theme.legendColor,
						bodyColor: theme.tickColor,
						borderColor: 'rgba(148,163,184,0.2)',
						borderWidth: 1,
						padding: 12,
						cornerRadius: 8,
						callbacks: {
							label: (ctx) => {
								const idx = ctx.dataIndex;
								const s = sorted[idx];
								return [
									` ${formatCurrency(s.total_cost_usd)}`,
									` ${s.total_runs} runs`,
									` ${formatTokens(s.total_input_tokens + s.total_output_tokens)} tokens`
								];
							}
						}
					}
				},
				scales: {
					x: {
						grid: {
							color: theme.gridColor
						},
						ticks: {
							color: theme.tickColor,
							font: { family: 'Plus Jakarta Sans', size: 11 },
							callback: (v) => formatCurrency(Number(v))
						},
						border: { display: false }
					},
					y: {
						grid: { display: false },
						ticks: {
							color: theme.tickColor,
							font: { family: 'Plus Jakarta Sans', size: 12, weight: 500 }
						},
						border: { display: false }
					}
				},
				onClick: (_evt, elements) => {
					if (elements.length > 0 && onagentclick) {
						const idx = elements[0].index;
						onagentclick(sorted[idx].agent_name);
					}
				},
				onHover: (evt, elements) => {
					const target = evt.native?.target as HTMLElement | undefined;
					if (target) {
						target.style.cursor = elements.length > 0 ? 'pointer' : 'default';
					}
				}
			}
		});
	}

	function destroyChart() {
		if (chart) {
			chart.destroy();
			chart = undefined;
		}
	}

	$effect(() => {
		// React to data changes
		void data;
		destroyChart();
		buildChart();
	});

	onDestroy(destroyChart);
</script>

<div class="relative h-full w-full">
	{#if data.length === 0}
		<div class="flex h-full items-center justify-center">
			<p class="text-sm text-muted-foreground">No cost data available</p>
		</div>
	{:else}
		<canvas bind:this={canvas}></canvas>
	{/if}
</div>
