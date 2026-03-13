<script lang="ts">
	import { Search, ChevronDown, ChevronUp, Check, X } from 'lucide-svelte';

	let {
		selectedSkills = $bindable([]),
		onchange
	}: {
		selectedSkills?: string[];
		onchange?: (skills: string[]) => void;
	} = $props();

	// ============================================================
	// Built-in skill catalog
	// ============================================================

	interface SkillDef {
		name: string;
		description: string;
		icon: string; // emoji icon for the skill domain
		tags: string[];
	}

	const BUILT_IN_SKILLS: SkillDef[] = [
		{
			name: 'aws',
			description: 'Manage AWS resources: EC2, S3, IAM, Lambda, CloudWatch, and more',
			icon: '',
			tags: ['cloud', 'infrastructure']
		},
		{
			name: 'kubernetes',
			description: 'Manage Kubernetes workloads, deployments, and cluster operations',
			icon: '',
			tags: ['cloud', 'orchestration', 'containers']
		},
		{
			name: 'terraform',
			description: 'Infrastructure as code with Terraform — plan, apply, and manage state',
			icon: '',
			tags: ['iac', 'infrastructure']
		},
		{
			name: 'docker',
			description: 'Build, run, and manage Docker containers and compose stacks',
			icon: '',
			tags: ['containers', 'devops']
		},
		{
			name: 'git',
			description: 'Interact with git repositories: commits, branches, PRs, and reviews',
			icon: '',
			tags: ['devops', 'vcs']
		},
		{
			name: 'database',
			description: 'Query and manage SQL and NoSQL databases with schema awareness',
			icon: '',
			tags: ['data', 'backend']
		},
		{
			name: 'security',
			description: 'Security scanning, vulnerability assessment, and compliance checks',
			icon: '',
			tags: ['security', 'compliance']
		},
		{
			name: 'observability',
			description: 'Query metrics, logs, and traces from Prometheus, Grafana, and Datadog',
			icon: '',
			tags: ['monitoring', 'observability']
		}
	];

	// ============================================================
	// State
	// ============================================================

	let expanded = $state(false);
	let searchQuery = $state('');

	const filteredSkills = $derived(
		BUILT_IN_SKILLS.filter(
			(s) =>
				searchQuery.trim() === '' ||
				s.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
				s.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
				s.tags.some((t) => t.toLowerCase().includes(searchQuery.toLowerCase()))
		)
	);

	function toggleSkill(name: string) {
		const next = selectedSkills.includes(name)
			? selectedSkills.filter((s) => s !== name)
			: [...selectedSkills, name];
		selectedSkills = next;
		onchange?.(next);
	}

	function removeSkill(name: string) {
		const next = selectedSkills.filter((s) => s !== name);
		selectedSkills = next;
		onchange?.(next);
	}

	function isSelected(name: string): boolean {
		return selectedSkills.includes(name);
	}
</script>

<div class="space-y-3">

	<!-- ============================================================ -->
	<!-- Attached skills badges                                       -->
	<!-- ============================================================ -->
	{#if selectedSkills.length > 0}
		<div class="flex flex-wrap gap-2">
			{#each selectedSkills as skill (skill)}
				<span class="flex items-center gap-1.5 rounded-full border border-primary/30 bg-primary/10 px-3 py-1 text-xs font-medium text-primary">
					{skill}
					<button
						type="button"
						onclick={() => removeSkill(skill)}
						class="rounded-full text-primary/70 hover:text-destructive transition-colors cursor-pointer"
						aria-label="Remove {skill}"
					>
						<X class="h-3 w-3" />
					</button>
				</span>
			{/each}
		</div>
	{:else}
		<p class="text-sm text-muted-foreground italic">No skills attached.</p>
	{/if}

	<!-- ============================================================ -->
	<!-- Toggle button                                                -->
	<!-- ============================================================ -->
	<button
		type="button"
		onclick={() => (expanded = !expanded)}
		class="flex items-center gap-2 rounded-md border border-dashed border-border px-3 py-1.5 text-xs font-medium text-muted-foreground hover:border-primary hover:text-primary transition-colors cursor-pointer"
	>
		{#if expanded}
			<ChevronUp class="h-3 w-3" />
			Hide Skill Browser
		{:else}
			<ChevronDown class="h-3 w-3" />
			Browse Skills
		{/if}
	</button>

	<!-- ============================================================ -->
	<!-- Skill catalog (expandable)                                  -->
	<!-- ============================================================ -->
	{#if expanded}
		<div class="rounded-lg border border-border bg-card p-4 space-y-4">
			<!-- Search -->
			<div class="relative">
				<Search class="absolute left-3 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground pointer-events-none" />
				<input
					type="search"
					bind:value={searchQuery}
					placeholder="Filter skills..."
					class="w-full rounded-md border border-input bg-background pl-9 pr-3 py-1.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
				/>
			</div>

			<!-- Grid of skill cards -->
			{#if filteredSkills.length === 0}
				<p class="text-sm text-muted-foreground text-center py-4">No skills match your search.</p>
			{:else}
				<div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
					{#each filteredSkills as skill (skill.name)}
						{@const selected = isSelected(skill.name)}
						<button
							type="button"
							onclick={() => toggleSkill(skill.name)}
							class={[
								'relative flex flex-col gap-1.5 rounded-lg border p-3 text-left transition-all cursor-pointer',
								selected
									? 'border-primary bg-primary/5 ring-1 ring-primary'
									: 'border-border hover:border-primary/40 hover:bg-accent/30'
							].join(' ')}
						>
							<!-- Check mark for selected -->
							{#if selected}
								<div class="absolute right-2 top-2 flex h-5 w-5 items-center justify-center rounded-full bg-primary">
									<Check class="h-3 w-3 text-primary-foreground" />
								</div>
							{/if}

							<span class="text-xl leading-none">{skill.icon}</span>
							<div>
								<p class="text-xs font-semibold">{skill.name}</p>
								<p class="text-[11px] leading-4 text-muted-foreground line-clamp-2 mt-0.5">
									{skill.description}
								</p>
							</div>
						</button>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>
