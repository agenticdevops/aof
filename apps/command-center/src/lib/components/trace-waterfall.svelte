<script lang="ts">
	import {
		PlayCircle,
		RotateCw,
		Brain,
		Wrench,
		Search,
		Database,
		ShieldCheck,
		Circle,
		AlertCircle,
		CheckCircle
	} from 'lucide-svelte';
	import type { SpanRecord, SpanKind } from '$lib/api/types.js';
	import {
		spanKindColor,
		spanPosition,
		flattenTree,
		formatDuration,
		type SpanNode,
		type TimeScale
	} from '$lib/utils/trace-helpers.js';
	import SpanDetail from './span-detail.svelte';
	import { selectedSpan } from '$lib/stores/traces.js';

	let { tree, scale }: { tree: SpanNode[]; scale: TimeScale } = $props();

	// Flattened ordered list for rendering
	const flatNodes = $derived(flattenTree(tree));

	// ============================================================
	// Time axis markers
	// ============================================================

	const MARKER_COUNT = 5;

	const timeMarkers = $derived.by(() => {
		const markers: Array<{ label: string; leftPercent: number }> = [];
		if (scale.totalMs <= 0) return markers;

		for (let i = 0; i <= MARKER_COUNT; i++) {
			const ms = (scale.totalMs * i) / MARKER_COUNT;
			const leftPercent = (i / MARKER_COUNT) * 100;
			markers.push({ label: formatDuration(ms), leftPercent });
		}
		return markers;
	});

	// ============================================================
	// Span icon resolver
	// ============================================================

	function getIcon(kind: SpanKind) {
		const map: Record<SpanKind, typeof PlayCircle> = {
			run: PlayCircle,
			iteration: RotateCw,
			llm_call: Brain,
			tool_call: Wrench,
			research: Search,
			memory_recall: Database,
			approval_wait: ShieldCheck
		};
		return map[kind] ?? Circle;
	}

	// ============================================================
	// Row click — toggle selected span
	// ============================================================

	function handleRowClick(span: SpanRecord) {
		if ($selectedSpan?.span_id === span.span_id) {
			selectedSpan.set(null);
		} else {
			selectedSpan.set(span);
		}
	}

	// ============================================================
	// Duration from span
	// ============================================================

	function getDuration(span: SpanRecord): number | null {
		if (span.duration_ms != null) return span.duration_ms;
		if (span.end_time) {
			return new Date(span.end_time).getTime() - new Date(span.start_time).getTime();
		}
		return null;
	}
</script>

<div class="overflow-x-auto rounded-xl border bg-card shadow-sm">
	<!-- Time axis header -->
	<div class="flex border-b bg-muted/40 dark:bg-muted/20">
		<!-- Label column header -->
		<div class="w-[30%] min-w-48 shrink-0 border-r px-3 py-2 text-xs font-medium text-muted-foreground uppercase tracking-wide">
			Span
		</div>
		<!-- Time axis -->
		<div class="relative flex-1 px-2 py-2 text-[10px] text-muted-foreground">
			{#each timeMarkers as marker}
				<span
					class="absolute -translate-x-1/2 select-none"
					style="left: {marker.leftPercent}%"
				>
					{marker.label}
				</span>
			{/each}
			<!-- Invisible height anchor -->
			<span class="invisible">0ms</span>
		</div>
	</div>

	<!-- Span rows -->
	<div class="divide-y">
		{#each flatNodes as node, idx}
			{@const span = node.span}
			{@const pos = spanPosition(span, scale)}
			{@const color = spanKindColor(span.kind)}
			{@const Icon = getIcon(span.kind)}
			{@const isSelected = $selectedSpan?.span_id === span.span_id}
			{@const durationMs = getDuration(span)}
			{@const isError = span.status === 'error'}

			<!-- Row wrapper (span row + optional detail) -->
			<div
				class="span-row-wrapper"
				style="animation-delay: {idx * 30}ms"
			>
				<!-- Span row -->
				<button
					type="button"
					onclick={() => handleRowClick(span)}
					class={[
						'flex w-full cursor-pointer items-stretch text-left transition-colors hover:bg-accent/40',
						isSelected ? 'bg-accent/60' : '',
						isError ? 'bg-destructive/5 hover:bg-destructive/10' : ''
					].join(' ')}
					aria-expanded={isSelected}
					title="{span.name} ({span.kind}){durationMs != null ? ' — ' + formatDuration(durationMs) : ''}"
				>
					<!-- Left: span label (30% width) -->
					<div class="flex w-[30%] min-w-48 shrink-0 items-center gap-2 border-r px-3 py-2">
						<!-- Indent by depth -->
						<span
							class="shrink-0"
							style="width: {node.depth * 20}px; min-width: {node.depth * 20}px"
						></span>

						<!-- Tree connector dots for non-root spans -->
						{#if node.depth > 0}
							<span class="shrink-0 h-px w-3 bg-border"></span>
						{/if}

						<!-- Status dot -->
						<span class="shrink-0">
							{#if span.status === 'ok'}
								<CheckCircle class="h-3 w-3 text-emerald-500" />
							{:else if span.status === 'error'}
								<AlertCircle class="h-3 w-3 text-destructive" />
							{:else}
								<Circle class="h-3 w-3 text-muted-foreground" />
							{/if}
						</span>

						<!-- Kind icon (color-coded) -->
						<span class="shrink-0" style="color: {color}">
							<Icon class="h-3.5 w-3.5" />
						</span>

						<!-- Span name -->
						<span class="truncate text-xs font-medium leading-tight">
							{span.name}
						</span>
					</div>

					<!-- Right: waterfall bar (70% width) -->
					<div class="relative flex-1 py-1.5 px-2 flex items-center">
						<!-- Grid lines -->
						{#each timeMarkers as marker}
							{#if marker.leftPercent > 0 && marker.leftPercent < 100}
								<span
									class="pointer-events-none absolute inset-y-0 w-px bg-border/40"
									style="left: {marker.leftPercent}%"
								></span>
							{/if}
						{/each}

						<!-- Duration bar -->
						<span
							class={[
								'absolute h-5 rounded-sm flex items-center overflow-hidden',
								isError ? 'border border-destructive/60' : ''
							].join(' ')}
							style="
								left: calc({pos.leftPercent}% + 0.5rem);
								width: calc({pos.widthPercent}% - 1rem);
								max-width: calc(100% - 1rem);
								min-width: 4px;
								background-color: {color};
								opacity: {isSelected ? 1 : 0.85};
							"
						>
							{#if pos.widthPercent > 10 && durationMs != null}
								<span class="px-1.5 text-[10px] font-semibold text-white truncate leading-none">
									{formatDuration(durationMs)}
								</span>
							{/if}
						</span>

						<!-- Duration label outside bar (when bar is narrow) -->
						{#if pos.widthPercent <= 10 && durationMs != null}
							<span
								class="absolute text-[10px] text-muted-foreground whitespace-nowrap"
								style="left: calc({pos.leftPercent + pos.widthPercent}% + 0.75rem)"
							>
								{formatDuration(durationMs)}
							</span>
						{/if}
					</div>
				</button>

				<!-- Inline span detail (shown below the span row when selected) -->
				{#if isSelected}
					<SpanDetail
						span={span}
						onclose={() => selectedSpan.set(null)}
					/>
				{/if}
			</div>
		{:else}
			<div class="px-4 py-12 text-center text-sm text-muted-foreground">
				No spans to display.
			</div>
		{/each}
	</div>
</div>

<style>
	.span-row-wrapper {
		opacity: 0;
		animation: fadeInRow 0.2s ease-out forwards;
	}

	@keyframes fadeInRow {
		from {
			opacity: 0;
			transform: translateY(4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>
