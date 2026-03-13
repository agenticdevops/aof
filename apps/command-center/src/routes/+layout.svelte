<script lang="ts">
	import '../app.css';
	import Sidebar from '$lib/components/sidebar.svelte';
	import { Menu } from 'lucide-svelte';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();

	let mobileOpen = $state(false);
</script>

<div class="flex h-screen overflow-hidden">
	<Sidebar bind:mobileOpen />

	{#if mobileOpen}
		<button
			class="fixed inset-0 z-30 bg-black/40 lg:hidden"
			onclick={() => (mobileOpen = false)}
			aria-label="Close sidebar"
		></button>
	{/if}

	<main class="flex-1 overflow-y-auto">
		<div class="sticky top-0 z-20 flex h-14 items-center border-b bg-card px-4 lg:hidden">
			<button
				onclick={() => (mobileOpen = true)}
				class="rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors duration-200 cursor-pointer"
				aria-label="Open sidebar"
			>
				<Menu class="h-5 w-5" />
			</button>
			<span class="ml-3 text-lg font-semibold tracking-tight">OpenAgentiX</span>
		</div>
		{@render children()}
	</main>
</div>
