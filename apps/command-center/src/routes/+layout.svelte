<script lang="ts">
	import '../app.css';
	import Sidebar from '$lib/components/sidebar.svelte';
	import FirstRunWizard from '$lib/components/first-run-wizard.svelte';
	import { Menu } from 'lucide-svelte';
	import type { Snippet } from 'svelte';
	import { onMount, onDestroy } from 'svelte';
	import { initWebSocket, disconnectWebSocket } from '$lib/stores/websocket.js';
	import { hasCompletedWizard, completeWizard } from '$lib/stores/settings.js';

	let { children }: { children: Snippet } = $props();

	let mobileOpen = $state(false);
	let showWizard = $state(false);

	onMount(() => {
		initWebSocket();
		// Show wizard on first visit
		const unsub = hasCompletedWizard.subscribe((done) => {
			showWizard = !done;
		});
		return unsub;
	});

	onDestroy(() => {
		disconnectWebSocket();
	});

	function handleWizardClose() {
		completeWizard();
		showWizard = false;
	}
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

{#if showWizard}
	<FirstRunWizard onclose={handleWizardClose} />
{/if}
