<script lang="ts">
	import { cn } from '$lib/utils.js';
	import type { Component, Snippet } from 'svelte';

	let {
		icon: Icon,
		title,
		description,
		class: className = '',
		children
	}: {
		icon?: Component<{ class?: string }>;
		title: string;
		description?: string;
		class?: string;
		children?: Snippet;
	} = $props();
</script>

<div
	class={cn(
		'flex flex-col items-center justify-center rounded-xl border border-dashed p-16 text-center',
		className
	)}
>
	{#if Icon}
		<Icon class="h-10 w-10 text-muted-foreground/50 mb-3" />
	{/if}
	<p class="text-sm font-medium text-muted-foreground">{title}</p>
	{#if description}
		<p class="text-xs text-muted-foreground/70 mt-1 max-w-sm">{description}</p>
	{/if}
	{#if children}
		<div class="mt-4">
			{@render children()}
		</div>
	{/if}
</div>
