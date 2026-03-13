<script lang="ts">
	import { cn } from '$lib/utils.js';
	import { X } from 'lucide-svelte';
	import type { Snippet } from 'svelte';

	type Width = 'md' | 'lg' | 'xl';

	let {
		open = false,
		onClose,
		title,
		width = 'lg',
		children
	}: {
		open?: boolean;
		onClose?: () => void;
		title?: string;
		width?: Width;
		children?: Snippet;
	} = $props();

	const widthClasses: Record<Width, string> = {
		md: 'max-w-md',
		lg: 'max-w-lg',
		xl: 'max-w-xl'
	};

	function handleBackdropClick() {
		onClose?.();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			onClose?.();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<!-- Backdrop -->
	<button
		class="fixed inset-0 z-40 bg-black/40 transition-opacity"
		onclick={handleBackdropClick}
		aria-label="Close panel"
	></button>

	<!-- Panel -->
	<div
		class={cn(
			'fixed inset-y-0 right-0 z-50 flex w-full flex-col bg-card shadow-xl transition-transform duration-300',
			widthClasses[width]
		)}
		role="dialog"
		aria-modal="true"
		aria-label={title}
	>
		<!-- Header -->
		<div class="flex h-14 items-center justify-between border-b px-4">
			{#if title}
				<h2 class="text-base font-semibold">{title}</h2>
			{/if}
			<button
				onclick={() => onClose?.()}
				class="ml-auto cursor-pointer rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors duration-200"
				aria-label="Close panel"
			>
				<X class="h-4 w-4" />
			</button>
		</div>

		<!-- Content -->
		<div class="flex-1 overflow-y-auto p-4">
			{#if children}
				{@render children()}
			{/if}
		</div>
	</div>
{/if}
