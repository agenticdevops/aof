<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: string;
		description: string;
		stepNumber: number;
		totalSteps: number;
		active: boolean;
		children?: Snippet;
		onBack?: () => void;
		onNext?: () => void;
		onFinish?: () => void;
		isFirst?: boolean;
		isLast?: boolean;
		nextDisabled?: boolean;
	}

	let {
		title,
		description,
		stepNumber,
		totalSteps,
		active,
		children,
		onBack,
		onNext,
		onFinish,
		isFirst = false,
		isLast = false,
		nextDisabled = false
	}: Props = $props();
</script>

{#if active}
	<div class="flex flex-col gap-4">
		<!-- Step indicator row -->
		<div class="flex items-center gap-3">
			<div
				class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary text-sm font-bold text-primary-foreground"
			>
				{stepNumber}
			</div>
			<div>
				<p class="text-xs text-muted-foreground">Step {stepNumber} of {totalSteps}</p>
				<h2 class="text-lg font-semibold leading-tight">{title}</h2>
			</div>
		</div>

		<p class="text-sm text-muted-foreground">{description}</p>

		<!-- Slot content -->
		{#if children}
			<div class="py-2">
				{@render children()}
			</div>
		{/if}

		<!-- Footer buttons -->
		<div class="flex justify-between pt-2">
			<button
				onclick={onBack}
				disabled={isFirst}
				class="rounded-md border px-4 py-2 text-sm font-medium transition-colors hover:bg-accent disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer"
			>
				Back
			</button>

			{#if isLast}
				<button
					onclick={onFinish}
					class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 cursor-pointer"
				>
					Open Dashboard
				</button>
			{:else}
				<button
					onclick={onNext}
					disabled={nextDisabled}
					class="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
				>
					Next
				</button>
			{/if}
		</div>
	</div>
{/if}
