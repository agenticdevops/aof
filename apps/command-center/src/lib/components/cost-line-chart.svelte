<script lang="ts">
	import { onDestroy } from 'svelte';
	import {
		Chart,
		LineController,
		LineElement,
		PointElement,
		CategoryScale,
		LinearScale,
		Filler,
		Tooltip,
		Legend,
		type ChartDataset,
		type ScriptableContext
	} from 'chart.js';
	import { formatCurrency, getChartTheme } from '$lib/utils/chart-helpers.js';

	Chart.register(
		LineController,
		LineElement,
		PointElement,
		CategoryScale,
		LinearScale,
		Filler,
		Tooltip,
		Legend
	);

	interface Props {
		data: { date: string; cost: number }[];
		agentData?: { date: string; cost: number }[];
		agentName?: string | null;
	}

	let { data, agentData = [], agentName = null }: Props = $props();

	let canvas: HTMLCanvasElement = $state() as HTMLCanvasElement;
	let chart: Chart | undefined;

	function buildChart() {
		if (!canvas) return;

		const theme = getChartTheme();
		const labels = data.map((d) => d.date);

		// Aggregate gradient (teal)
		const datasets: ChartDataset<'line'>[] = [
			{
				label: 'All Agents',
				data: data.map((d) => d.cost),
				borderColor: '#14b8a6',
				backgroundColor: (ctx: ScriptableContext<'line'>) => {
					const gradient = ctx.chart.ctx.createLinearGradient(0, 0, 0, ctx.chart.height);
					gradient.addColorStop(0, 'rgba(20,184,166,0.35)');
					gradient.addColorStop(1, 'rgba(20,184,166,0.00)');
					return gradient;
				},
				borderWidth: 2.5,
				pointRadius: 3,
				pointHoverRadius: 6,
				pointBackgroundColor: '#14b8a6',
				pointBorderColor: theme.backgroundColor,
				pointBorderWidth: 2,
				tension: 0.4,
				fill: true
			}
		];

		// Per-agent overlay (sky blue) when drill-down active
		if (agentName && agentData.length > 0) {
			datasets.push({
				label: agentName,
				data: labels.map((label) => {
					const match = agentData.find((d) => d.date === label);
					return match ? match.cost : 0;
				}),
				borderColor: '#0ea5e9',
				backgroundColor: (ctx: ScriptableContext<'line'>) => {
					const gradient = ctx.chart.ctx.createLinearGradient(0, 0, 0, ctx.chart.height);
					gradient.addColorStop(0, 'rgba(14,165,233,0.25)');
					gradient.addColorStop(1, 'rgba(14,165,233,0.00)');
					return gradient;
				},
				borderWidth: 2,
				pointRadius: 3,
				pointHoverRadius: 6,
				pointBackgroundColor: '#0ea5e9',
				pointBorderColor: theme.backgroundColor,
				pointBorderWidth: 2,
				tension: 0.4,
				fill: true
			});
		}

		chart = new Chart(canvas, {
			type: 'line',
			data: { labels, datasets },
			options: {
				responsive: true,
				maintainAspectRatio: false,
				animation: {
					duration: 800,
					easing: 'easeInOutQuart'
				},
				interaction: {
					mode: 'index',
					intersect: false
				},
				plugins: {
					legend: {
						display: agentName !== null,
						labels: {
							color: theme.legendColor,
							font: { family: 'Plus Jakarta Sans', size: 12 },
							usePointStyle: true,
							pointStyleWidth: 8
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
							title: ([ctx]) => ctx.label,
							label: (ctx) => ` ${ctx.dataset.label}: ${formatCurrency(Number(ctx.raw))}`
						}
					}
				},
				scales: {
					x: {
						grid: { color: theme.gridColor },
						ticks: {
							color: theme.tickColor,
							font: { family: 'Plus Jakarta Sans', size: 11 },
							maxTicksLimit: 8,
							maxRotation: 0
						},
						border: { display: false }
					},
					y: {
						grid: { color: theme.gridColor },
						ticks: {
							color: theme.tickColor,
							font: { family: 'Plus Jakarta Sans', size: 11 },
							callback: (v) => formatCurrency(Number(v))
						},
						border: { display: false }
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
		void agentData;
		void agentName;
		destroyChart();
		buildChart();
	});

	onDestroy(destroyChart);
</script>

<div class="relative h-full w-full">
	{#if data.length === 0}
		<div class="flex h-full items-center justify-center">
			<p class="text-sm text-muted-foreground">No data for selected date range</p>
		</div>
	{:else}
		<canvas bind:this={canvas}></canvas>
	{/if}
</div>
