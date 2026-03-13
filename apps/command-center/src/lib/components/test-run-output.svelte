<script lang="ts">
	import { Trash2 } from 'lucide-svelte';

	let {
		output,
		running,
		onclear
	}: {
		output: string;
		running: boolean;
		onclear?: () => void;
	} = $props();

	let scrollContainer = $state<HTMLDivElement | undefined>(undefined);

	// Auto-scroll to bottom when output changes
	$effect(() => {
		// Depend on output
		const _ = output;
		if (scrollContainer) {
			scrollContainer.scrollTop = scrollContainer.scrollHeight;
		}
	});
</script>

<div class="flex flex-col overflow-hidden rounded-lg border border-border bg-[#0d1117]">

	<!-- Header bar -->
	<div class="flex items-center justify-between border-b border-white/10 px-4 py-2">
		<div class="flex items-center gap-2">
			<!-- Status indicator -->
			{#if running}
				<span class="relative flex h-2.5 w-2.5">
					<span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75"></span>
					<span class="relative inline-flex h-2.5 w-2.5 rounded-full bg-green-500"></span>
				</span>
				<span class="text-xs font-medium text-green-400">Running...</span>
			{:else if output}
				<span class="relative flex h-2.5 w-2.5">
					<span class="relative inline-flex h-2.5 w-2.5 rounded-full bg-gray-500"></span>
				</span>
				<span class="text-xs font-medium text-gray-400">Complete</span>
			{:else}
				<span class="text-xs text-gray-500">Test output will appear here</span>
			{/if}
		</div>

		<!-- Clear button -->
		{#if output || running}
			<button
				type="button"
				onclick={onclear}
				title="Clear output"
				class="flex items-center gap-1 rounded p-1.5 text-gray-500 hover:text-gray-300 hover:bg-white/5 transition-colors cursor-pointer"
				aria-label="Clear output"
			>
				<Trash2 class="h-3.5 w-3.5" />
			</button>
		{/if}
	</div>

	<!-- Output area -->
	<div
		bind:this={scrollContainer}
		class="min-h-32 max-h-80 overflow-y-auto p-4"
	>
		{#if output}
			<pre class="font-mono text-[13px] leading-6 text-green-300/90 whitespace-pre-wrap break-words">{output}</pre>
		{:else if running}
			<div class="flex items-center gap-2 text-green-400/70">
				<span class="font-mono text-sm">$</span>
				<span class="inline-block h-4 w-2 animate-pulse bg-green-400/70 rounded-sm"></span>
			</div>
		{:else}
			<p class="text-xs text-gray-600 italic">No output yet. Run the agent to see output here.</p>
		{/if}
	</div>
</div>
