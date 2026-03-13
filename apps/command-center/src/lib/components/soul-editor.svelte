<script lang="ts">
	import { marked } from 'marked';
	import { Bold, Italic, Heading1, Link, Code, List } from 'lucide-svelte';

	let {
		value = $bindable(''),
		oninput
	}: {
		value?: string;
		oninput?: (v: string) => void;
	} = $props();

	type ViewMode = 'edit' | 'split' | 'preview';
	let viewMode = $state<ViewMode>('split');

	let textarea = $state<HTMLTextAreaElement | undefined>(undefined);

	// Rendered HTML for preview
	let renderedHtml = $derived.by(() => {
		if (!value) return '<p class="text-muted-foreground italic">Nothing to preview yet.</p>';
		try {
			return marked.parse(value) as string;
		} catch {
			return '<p class="text-destructive">Error rendering markdown.</p>';
		}
	});

	// ============================================================
	// Toolbar actions
	// ============================================================

	function insertMarkdown(before: string, after = '') {
		if (!textarea) return;

		const start = textarea.selectionStart;
		const end = textarea.selectionEnd;
		const selected = value.slice(start, end);
		const newText = value.slice(0, start) + before + selected + after + value.slice(end);

		value = newText;
		oninput?.(newText);

		// Restore cursor position after reactive update
		requestAnimationFrame(() => {
			if (!textarea) return;
			textarea.focus();
			const newCursor = start + before.length + selected.length + after.length;
			textarea.setSelectionRange(newCursor, newCursor);
		});
	}

	function insertHeading() {
		if (!textarea) return;
		const lineStart = value.lastIndexOf('\n', textarea.selectionStart - 1) + 1;
		const lineContent = value.slice(lineStart);
		if (lineContent.startsWith('# ')) {
			// Already H1 — remove heading
			const newText = value.slice(0, lineStart) + lineContent.slice(2);
			value = newText;
			oninput?.(newText);
		} else {
			insertMarkdown('# ', '');
		}
	}

	function handleInput(e: Event) {
		const v = (e.target as HTMLTextAreaElement).value;
		value = v;
		oninput?.(v);
	}
</script>

<div class="flex h-full flex-col overflow-hidden rounded-lg border border-border">

	<!-- ============================================================ -->
	<!-- Toolbar                                                      -->
	<!-- ============================================================ -->
	<div class="flex items-center gap-1 border-b border-border bg-muted/50 px-2 py-1.5">
		<!-- Formatting buttons -->
		<button
			type="button"
			onclick={() => insertMarkdown('**', '**')}
			title="Bold"
			class="rounded p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
		>
			<Bold class="h-3.5 w-3.5" />
		</button>
		<button
			type="button"
			onclick={() => insertMarkdown('_', '_')}
			title="Italic"
			class="rounded p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
		>
			<Italic class="h-3.5 w-3.5" />
		</button>
		<button
			type="button"
			onclick={insertHeading}
			title="Heading"
			class="rounded p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
		>
			<Heading1 class="h-3.5 w-3.5" />
		</button>
		<button
			type="button"
			onclick={() => insertMarkdown('[', '](url)')}
			title="Link"
			class="rounded p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
		>
			<Link class="h-3.5 w-3.5" />
		</button>
		<button
			type="button"
			onclick={() => insertMarkdown('\n```\n', '\n```\n')}
			title="Code Block"
			class="rounded p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
		>
			<Code class="h-3.5 w-3.5" />
		</button>
		<button
			type="button"
			onclick={() => insertMarkdown('\n- ', '')}
			title="List"
			class="rounded p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
		>
			<List class="h-3.5 w-3.5" />
		</button>

		<div class="mx-1 h-4 w-px bg-border"></div>

		<!-- View mode toggle -->
		<div class="ml-auto flex rounded-md border border-border overflow-hidden">
			{#each [
				{ mode: 'edit' as ViewMode, label: 'Edit' },
				{ mode: 'split' as ViewMode, label: 'Split' },
				{ mode: 'preview' as ViewMode, label: 'Preview' }
			] as tab}
				<button
					type="button"
					onclick={() => (viewMode = tab.mode)}
					class={[
						'px-2.5 py-1 text-xs font-medium transition-colors cursor-pointer',
						viewMode === tab.mode
							? 'bg-primary text-primary-foreground'
							: 'bg-background text-muted-foreground hover:text-foreground hover:bg-accent'
					].join(' ')}
				>
					{tab.label}
				</button>
			{/each}
		</div>
	</div>

	<!-- ============================================================ -->
	<!-- Editor area                                                  -->
	<!-- ============================================================ -->
	<div class="flex flex-1 overflow-hidden">

		<!-- Editor pane -->
		{#if viewMode === 'edit' || viewMode === 'split'}
			<div class={['flex flex-col', viewMode === 'split' ? 'w-1/2 border-r border-border' : 'w-full'].join(' ')}>
				<textarea
					bind:this={textarea}
					{value}
					oninput={handleInput}
					placeholder="Describe your agent's identity, expertise, and behavior..."
					spellcheck={false}
					class="h-full w-full resize-none bg-background p-3 font-mono text-sm leading-6 text-foreground placeholder:text-muted-foreground focus:outline-none"
					style="tab-size: 2;"
				></textarea>
			</div>
		{/if}

		<!-- Preview pane -->
		{#if viewMode === 'preview' || viewMode === 'split'}
			<div class={['flex flex-col overflow-y-auto', viewMode === 'split' ? 'w-1/2' : 'w-full'].join(' ')}>
				<div
					class="prose prose-sm dark:prose-invert max-w-none p-4 text-sm leading-6"
					style="font-size: 0.875rem;"
				>
					<!-- eslint-disable-next-line svelte/no-at-html-tags -->
					{@html renderedHtml}
				</div>
			</div>
		{/if}
	</div>
</div>
