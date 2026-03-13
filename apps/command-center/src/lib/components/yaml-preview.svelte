<script lang="ts">
	import { Copy, Check } from 'lucide-svelte';

	let { yaml }: { yaml: string } = $props();

	let copied = $state(false);

	async function copyToClipboard() {
		try {
			await navigator.clipboard.writeText(yaml);
			copied = true;
			setTimeout(() => (copied = false), 2000);
		} catch {
			// Fallback for browsers that don't support clipboard API
			const el = document.createElement('textarea');
			el.value = yaml;
			el.style.position = 'fixed';
			el.style.opacity = '0';
			document.body.appendChild(el);
			el.select();
			document.execCommand('copy');
			document.body.removeChild(el);
			copied = true;
			setTimeout(() => (copied = false), 2000);
		}
	}
</script>

<div class="relative h-full overflow-hidden rounded-lg border border-border bg-[#0d1117]">

	<!-- Copy button -->
	<button
		type="button"
		onclick={copyToClipboard}
		title="Copy YAML"
		class="absolute right-3 top-3 z-10 flex items-center gap-1.5 rounded-md border border-white/10 bg-white/5 px-2.5 py-1.5 text-xs text-gray-400 transition-colors hover:bg-white/10 hover:text-white cursor-pointer"
	>
		{#if copied}
			<Check class="h-3.5 w-3.5 text-green-400" />
			<span class="text-green-400">Copied!</span>
		{:else}
			<Copy class="h-3.5 w-3.5" />
			<span>Copy</span>
		{/if}
	</button>

	<!-- YAML content -->
	<div class="h-full overflow-auto p-4">
		<pre
			class="font-mono text-[13px] leading-6 text-green-300/90"
		><code>{yaml}</code></pre>
	</div>
</div>
