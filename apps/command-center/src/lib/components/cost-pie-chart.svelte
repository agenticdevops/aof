<script lang="ts">
	import { onDestroy } from 'svelte';
	import { Chart, DoughnutController, ArcElement, Tooltip, Legend } from 'chart.js';
	import {
		getAgentColor,
		getAgentHoverColor,
		formatCurrency,
		getChartTheme
	} from '$lib/utils/chart-helpers.js';

	Chart.register(DoughnutController, ArcElement, Tooltip, Legend);

	interface AgentCostShare {
		agent: string;
		cost: number;
		percentage: number;
	}

	interface Props {
		data: AgentCostShare[];
		onagentclick?: (name: string) => void;
	}

	let { data, onagentclick }: Props = $props();

	let canvas: HTMLCanvasElement = $state() as HTMLCanvasElement;
	let chart: Chart | undefined;

	// Total for center text
	let totalCost = $derived(data.reduce((sum, d) => sum + d.cost, 0));

	function buildChart() {
		if (!canvas) return;

		const theme = getChartTheme();
		const colors = data.map((_, i) => getAgentColor(i));
		const hoverColors = data.map((_, i) => getAgentHoverColor(i));

		chart = new Chart(canvas, {
			type: 'doughnut',
			data: {
				labels: data.map((d) => d.agent),
				datasets: [
					{
						data: data.map((d) => d.cost),
						backgroundColor: colors,
						hoverBackgroundColor: hoverColors,
						borderColor: theme.backgroundColor,
						borderWidth: 2,
						hoverOffset: 8
					}
				]
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				cutout: '62%',
				animation: {
					animateRotate: true,
					animateScale: false,
					duration: 700,
					easing: 'easeOutQuart'
				},
				plugins: {
					legend: {
						position: 'bottom',
						labels: {
							color: theme.legendColor,
							font: { family: 'Plus Jakarta Sans', size: 11 },
							usePointStyle: true,
							pointStyleWidth: 8,
							padding: 12,
							// Truncate long agent names
							generateLabels: (chart) => {
								const ds = chart.data.datasets[0];
								return (chart.data.labels as string[]).map((label, i) => ({
									text: label.length > 18 ? label.slice(0, 16) + '…' : label,
									fillStyle: (ds.backgroundColor as string[])[i],
									strokeStyle: (ds.backgroundColor as string[])[i],
									pointStyle: 'circle' as const,
									index: i,
									hidden: false
								}));
							}
						}
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
								const d = data[ctx.dataIndex];
								return [
									` ${formatCurrency(d.cost)}`,
									` ${d.percentage.toFixed(1)}% of total`
								];
							}
						}
					}
				},
				onClick: (_evt, elements) => {
					if (elements.length > 0 && onagentclick) {
						const idx = elements[0].index;
						onagentclick(data[idx].agent);
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
		<!-- Wrapper to position center text over the doughnut hole -->
		<div class="relative h-full w-full">
			<canvas bind:this={canvas}></canvas>
			<!-- Center label — positioned absolutely in the upper center of the chart canvas -->
			{#if totalCost > 0}
				<div
					class="pointer-events-none absolute inset-0 flex flex-col items-center justify-center pb-16"
				>
					<p class="text-xs font-medium text-muted-foreground">Total</p>
					<p class="text-base font-bold tracking-tight text-foreground">
						{formatCurrency(totalCost)}
					</p>
				</div>
			{/if}
		</div>
	{/if}
</div>
