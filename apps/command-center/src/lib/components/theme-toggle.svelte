<script lang="ts">
	import { Sun, Moon } from 'lucide-svelte';
	import { browser } from '$app/environment';

	let dark = $state(false);

	if (browser) {
		dark =
			localStorage.getItem('theme') === 'dark' ||
			(!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches);
		applyTheme();
	}

	function applyTheme() {
		if (browser) {
			document.documentElement.classList.toggle('dark', dark);
			localStorage.setItem('theme', dark ? 'dark' : 'light');
		}
	}

	function toggle() {
		dark = !dark;
		applyTheme();
	}
</script>

<button
	onclick={toggle}
	class="cursor-pointer rounded-md p-2 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors duration-200"
	aria-label={dark ? 'Switch to light mode' : 'Switch to dark mode'}
>
	{#if dark}
		<Sun class="h-4 w-4" />
	{:else}
		<Moon class="h-4 w-4" />
	{/if}
</button>
