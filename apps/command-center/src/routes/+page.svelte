<script lang="ts">
	import { Bot } from 'lucide-svelte';
</script>

<div class="p-6">
	<div class="mb-6">
		<h1 class="text-2xl font-bold tracking-tight">OpenAgentiX Command Center</h1>
		<p class="text-sm text-muted-foreground mt-1">
			Monitor and manage your AI agents from a single dashboard.
		</p>
	</div>

	<div
		class="flex flex-col items-center justify-center rounded-xl border border-dashed p-16 text-center"
	>
		<Bot class="h-10 w-10 text-muted-foreground/50 mb-3" />
		<p class="text-sm font-medium text-muted-foreground">No agents running</p>
		<p class="text-xs text-muted-foreground/70 mt-1 max-w-sm">
			Register your first agent to get started. Navigate to Agents to create and deploy AI agents.
		</p>
		<div class="mt-4">
			<a
				href="/agents"
				class="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors duration-200"
			>
				<Bot class="h-4 w-4" />
				Go to Agents
			</a>
		</div>
	</div>
</div>
