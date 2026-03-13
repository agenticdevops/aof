<script lang="ts">
	import { X, Clock, Hash, Tag, AlertCircle, CheckCircle, Circle } from 'lucide-svelte';
	import type { SpanRecord } from '$lib/api/types.js';
	import { spanKindColor, formatDuration } from '$lib/utils/trace-helpers.js';

	let { span, onclose }: { span: SpanRecord; onclose?: () => void } = $props();

	const borderColor = $derived(spanKindColor(span.kind));

	const durationMs = $derived(
		span.duration_ms ??
			(span.end_time
				? new Date(span.end_time).getTime() - new Date(span.start_time).getTime()
				: null)
	);

	const attributeEntries = $derived(Object.entries(span.attributes));
</script>

<!-- Inline panel — appears below the clicked span row -->
<div
	class="relative mx-2 my-1 rounded-lg border bg-muted/40 dark:bg-muted/20 shadow-sm"
	style="border-left: 3px solid {borderColor}"
	role="region"
	aria-label="Span detail"
>
	<!-- Close button -->
	<button
		type="button"
		onclick={onclose}
		class="absolute top-2 right-2 rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
		aria-label="Close span detail"
	>
		<X class="h-3.5 w-3.5" />
	</button>

	<div class="px-4 py-3 pr-10">
		<!-- Span name + status -->
		<div class="flex items-center gap-2 mb-3">
			{#if span.status === 'ok'}
				<CheckCircle class="h-4 w-4 text-emerald-500 shrink-0" />
			{:else if span.status === 'error'}
				<AlertCircle class="h-4 w-4 text-destructive shrink-0" />
			{:else}
				<Circle class="h-4 w-4 text-muted-foreground shrink-0" />
			{/if}
			<span class="text-sm font-semibold truncate">{span.name}</span>
			<span
				class="shrink-0 rounded-full px-2 py-0.5 text-[10px] font-medium uppercase tracking-wide"
				style="background: {borderColor}22; color: {borderColor}"
			>
				{span.kind}
			</span>
		</div>

		<!-- Core metadata row -->
		<div class="grid grid-cols-2 gap-x-6 gap-y-1.5 text-xs mb-3 sm:grid-cols-3 lg:grid-cols-4">
			<div class="flex items-center gap-1.5 text-muted-foreground">
				<Hash class="h-3 w-3 shrink-0" />
				<span class="truncate font-mono" title={span.span_id}>{span.span_id.slice(0, 16)}…</span>
			</div>
			<div class="flex items-center gap-1.5 text-muted-foreground">
				<Tag class="h-3 w-3 shrink-0" />
				<span class="truncate font-mono" title={span.trace_id}>{span.trace_id.slice(0, 16)}…</span>
			</div>
			{#if durationMs !== null}
				<div class="flex items-center gap-1.5 text-muted-foreground">
					<Clock class="h-3 w-3 shrink-0" />
					<span class="font-medium">{formatDuration(durationMs)}</span>
				</div>
			{/if}
			<div class="text-muted-foreground">
				<span class="text-[10px] uppercase tracking-wide">start</span>{' '}
				<span class="font-medium">{new Date(span.start_time).toLocaleTimeString()}</span>
			</div>
			{#if span.end_time}
				<div class="text-muted-foreground">
					<span class="text-[10px] uppercase tracking-wide">end</span>{' '}
					<span class="font-medium">{new Date(span.end_time).toLocaleTimeString()}</span>
				</div>
			{/if}
		</div>

		<!-- Attributes table -->
		{#if attributeEntries.length > 0}
			<div class="rounded-md border overflow-hidden">
				<table class="w-full text-xs">
					<thead>
						<tr class="bg-muted/60 border-b">
							<th class="px-3 py-1.5 text-left font-medium text-muted-foreground w-1/3">
								Attribute
							</th>
							<th class="px-3 py-1.5 text-left font-medium text-muted-foreground">Value</th>
						</tr>
					</thead>
					<tbody>
						{#each attributeEntries as [key, value], i}
							<tr class={['border-b last:border-0', i % 2 === 0 ? 'bg-background' : 'bg-muted/20'].join(' ')}>
								<td class="px-3 py-1.5 font-mono text-muted-foreground truncate max-w-0">
									{key}
								</td>
								<td class="px-3 py-1.5 font-mono break-all">
									{#if key === 'error' || key === 'error_message'}
										<span class="text-destructive">{value}</span>
									{:else if value.startsWith('http://') || value.startsWith('https://')}
										<a
											href={value}
											target="_blank"
											rel="noopener noreferrer"
											class="text-sky-500 hover:underline"
										>
											{value}
										</a>
									{:else}
										{value}
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{:else}
			<p class="text-xs text-muted-foreground italic">No attributes recorded for this span.</p>
		{/if}
	</div>
</div>
