<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { Wrench, Save, Play, RotateCcw, ChevronRight } from 'lucide-svelte';
	import {
		builderForm,
		builderErrors,
		testRunOutput,
		testRunning,
		validateForm,
		resetForm
	} from '$lib/stores/builder.js';
	import { generateAgentYaml, parseAgentYaml } from '$lib/utils/yaml-generator.js';
	import { api } from '$lib/api/client.js';
	import AgentForm from '$lib/components/agent-form.svelte';
	import SoulEditor from '$lib/components/soul-editor.svelte';
	import SkillBrowser from '$lib/components/skill-browser.svelte';
	import YamlPreview from '$lib/components/yaml-preview.svelte';
	import TestRunOutput from '$lib/components/test-run-output.svelte';
	import Skeleton from '$lib/components/ui/skeleton.svelte';

	// ============================================================
	// State
	// ============================================================

	let rightTab = $state<'soul' | 'yaml'>('soul');
	let loading = $state(false);
	let saving = $state(false);
	let saveError = $state('');
	let saveSuccess = $state('');
	let editMode = $state(false); // true when ?agent=name is set

	// Reactive YAML generated from form state
	let generatedYaml = $derived(generateAgentYaml($builderForm));

	// ============================================================
	// Edit mode — load existing agent
	// ============================================================

	onMount(async () => {
		const agentName = page.url.searchParams.get('agent');
		if (!agentName) return;

		editMode = true;
		loading = true;

		try {
			const agent = await api.agents.get(agentName);

			// Reconstruct YAML from the agent object so we can parse it
			// (the API returns an Agent object, not raw YAML)
			const partialForm = {
				name: agent.name,
				namespace: agent.namespace ?? 'default',
				version: agent.version ?? '1.0.0',
				modelPreferred: agent.model.preferred,
				modelFallback: agent.model.fallback ?? '',
				mode: agent.mode,
				skills: agent.skills ?? [],
				tools: agent.tools ?? [],
				triggers: (agent.triggers ?? []).map((t) => ({
					type: t.type,
					expression: t.expression ?? '',
					channel: t.channel ?? ''
				})),
				budgetDailyLimit: agent.budget?.daily_limit != null
					? String(agent.budget.daily_limit)
					: '',
				budgetMaxTokens: agent.budget?.max_tokens_per_run != null
					? String(agent.budget.max_tokens_per_run)
					: '',
				telemetryEnabled: agent.telemetry?.enabled ?? true,
				soulMd: ''
			};

			builderForm.update((f) => ({ ...f, ...partialForm }));
		} catch (err) {
			saveError = `Failed to load agent: ${err instanceof Error ? err.message : String(err)}`;
		} finally {
			loading = false;
		}
	});

	// ============================================================
	// Actions
	// ============================================================

	async function handleSave() {
		if (!validateForm($builderForm)) return;

		saving = true;
		saveError = '';
		saveSuccess = '';

		try {
			const yaml = generatedYaml;

			if (editMode) {
				await api.agents.update($builderForm.name, yaml);
				saveSuccess = `Agent "${$builderForm.name}" updated successfully.`;
			} else {
				await api.agents.register(yaml);
				saveSuccess = `Agent "${$builderForm.name}" registered successfully.`;
				editMode = true; // Switch to edit mode after first save
			}

			// Auto-dismiss success after 3s
			setTimeout(() => (saveSuccess = ''), 3000);
		} catch (err) {
			saveError = err instanceof Error ? err.message : String(err);
		} finally {
			saving = false;
		}
	}

	async function handleTestRun() {
		if (!validateForm($builderForm)) return;

		// Save agent first
		saving = true;
		saveError = '';

		try {
			const yaml = generatedYaml;

			if (editMode) {
				await api.agents.update($builderForm.name, yaml);
			} else {
				await api.agents.register(yaml);
				editMode = true;
			}
		} catch (err) {
			saveError = `Failed to save agent before run: ${err instanceof Error ? err.message : String(err)}`;
			saving = false;
			return;
		}

		saving = false;

		// Start SSE run
		testRunOutput.set('');
		testRunning.set(true);

		const es = api.agents.run($builderForm.name);

		es.onmessage = (event) => {
			testRunOutput.update((prev) => prev + event.data + '\n');
		};

		es.onerror = () => {
			testRunning.set(false);
			es.close();
		};

		// Also listen for explicit "done" events
		es.addEventListener('done', () => {
			testRunning.set(false);
			es.close();
		});

		es.addEventListener('end', () => {
			testRunning.set(false);
			es.close();
		});
	}

	function handleReset() {
		resetForm();
		editMode = false;
		saveError = '';
		saveSuccess = '';
	}

	function clearTestOutput() {
		testRunOutput.set('');
	}
</script>

<svelte:head>
	<title>Agent Builder — OpenAgentiX</title>
</svelte:head>

<div class="flex h-full flex-col overflow-hidden">

	<!-- ============================================================ -->
	<!-- Header                                                       -->
	<!-- ============================================================ -->
	<div class="flex shrink-0 items-center justify-between border-b border-border bg-card px-6 py-4">
		<div class="flex items-center gap-3">
			<div class="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-violet-500 to-indigo-500 shadow-sm">
				<Wrench class="h-5 w-5 text-white" />
			</div>
			<div>
				<h1 class="text-xl font-bold tracking-tight">Agent Builder</h1>
				<p class="text-xs text-muted-foreground">
					{#if editMode}
						Editing <span class="font-medium text-foreground">{$builderForm.name}</span>
					{:else}
						Create a new agent definition
					{/if}
				</p>
			</div>
		</div>

		<!-- Breadcrumb for edit mode -->
		{#if editMode}
			<a
				href="/agents/{encodeURIComponent($builderForm.name)}"
				class="flex items-center gap-1 rounded-md px-3 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
			>
				View Agent
				<ChevronRight class="h-3.5 w-3.5" />
			</a>
		{/if}
	</div>

	<!-- ============================================================ -->
	<!-- Toast notifications                                         -->
	<!-- ============================================================ -->
	{#if saveSuccess}
		<div class="shrink-0 mx-6 mt-4 flex items-center gap-2 rounded-lg border border-success/30 bg-success/10 px-4 py-3 text-sm font-medium text-success">
			{saveSuccess}
		</div>
	{/if}
	{#if saveError}
		<div class="shrink-0 mx-6 mt-4 flex items-center justify-between gap-2 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm font-medium text-destructive">
			{saveError}
			<button
				type="button"
				onclick={() => (saveError = '')}
				class="shrink-0 text-xs underline cursor-pointer"
			>
				Dismiss
			</button>
		</div>
	{/if}

	<!-- ============================================================ -->
	<!-- Main content area                                           -->
	<!-- ============================================================ -->
	{#if loading}
		<!-- Skeleton loading state -->
		<div class="flex flex-1 gap-6 overflow-hidden p-6">
			<div class="flex-1 space-y-4">
				{#each { length: 5 } as _}
					<Skeleton class="h-12 rounded-lg" />
				{/each}
			</div>
			<div class="flex-1 space-y-4">
				<Skeleton class="h-8 rounded-lg w-48" />
				<Skeleton class="h-64 rounded-lg" />
			</div>
		</div>
	{:else}
		<div class="flex flex-1 flex-col overflow-hidden lg:flex-row">

			<!-- ========================================================== -->
			<!-- Left panel: Form + Skill Browser                          -->
			<!-- ========================================================== -->
			<div class="flex flex-1 flex-col overflow-y-auto border-b border-border lg:border-b-0 lg:border-r lg:w-1/2">
				<div class="p-6 space-y-8">
					<!-- Agent form -->
					<AgentForm bind:form={$builderForm} bind:errors={$builderErrors} />

					<!-- Skill Browser -->
					<div>
						<div class="mb-3">
							<h2 class="text-xs font-semibold uppercase tracking-widest text-muted-foreground">
								Skills
							</h2>
							<p class="text-xs text-muted-foreground mt-1">
								Attach built-in skill packs to expand agent capabilities.
							</p>
						</div>
						<SkillBrowser
							bind:selectedSkills={$builderForm.skills}
							onchange={(skills) => builderForm.update((f) => ({ ...f, skills }))}
						/>
					</div>
				</div>
			</div>

			<!-- ========================================================== -->
			<!-- Right panel: SOUL.md editor + YAML preview               -->
			<!-- ========================================================== -->
			<div class="flex flex-1 flex-col overflow-hidden lg:w-1/2">

				<!-- Tab bar -->
				<div class="shrink-0 flex border-b border-border bg-muted/30">
					{#each [
						{ tab: 'soul' as const, label: 'SOUL.md' },
						{ tab: 'yaml' as const, label: 'YAML Preview' }
					] as t}
						<button
							type="button"
							onclick={() => (rightTab = t.tab)}
							class={[
								'px-5 py-3 text-sm font-medium transition-colors cursor-pointer border-b-2',
								rightTab === t.tab
									? 'border-primary text-foreground'
									: 'border-transparent text-muted-foreground hover:text-foreground'
							].join(' ')}
						>
							{t.label}
						</button>
					{/each}
				</div>

				<!-- Tab content -->
				<div class="flex-1 overflow-hidden p-4">
					{#if rightTab === 'soul'}
						<SoulEditor
							bind:value={$builderForm.soulMd}
							oninput={(v) => builderForm.update((f) => ({ ...f, soulMd: v }))}
						/>
					{:else}
						<YamlPreview yaml={generatedYaml} />
					{/if}
				</div>
			</div>
		</div>

		<!-- ============================================================ -->
		<!-- Test Run Output (shown below when output exists)            -->
		<!-- ============================================================ -->
		{#if $testRunOutput || $testRunning}
			<div class="shrink-0 border-t border-border p-4">
				<div class="mb-2 flex items-center gap-2">
					<Play class="h-3.5 w-3.5 text-muted-foreground" />
					<h3 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
						Test Run Output
					</h3>
				</div>
				<TestRunOutput
					output={$testRunOutput}
					running={$testRunning}
					onclear={clearTestOutput}
				/>
			</div>
		{/if}

		<!-- ============================================================ -->
		<!-- Sticky action bar                                           -->
		<!-- ============================================================ -->
		<div class="shrink-0 flex items-center justify-between gap-3 border-t border-border bg-card px-6 py-4">
			<!-- Reset -->
			<button
				type="button"
				onclick={handleReset}
				class="flex items-center gap-2 rounded-md border border-border px-4 py-2 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
			>
				<RotateCcw class="h-4 w-4" />
				Reset
			</button>

			<div class="flex items-center gap-3">
				<!-- Test Run -->
				<button
					type="button"
					onclick={handleTestRun}
					disabled={saving || $testRunning}
					class="flex items-center gap-2 rounded-md border border-border bg-background px-4 py-2 text-sm font-medium transition-colors cursor-pointer hover:bg-accent hover:text-accent-foreground disabled:opacity-50 disabled:cursor-not-allowed"
				>
					<Play class="h-4 w-4 text-green-500" />
					{$testRunning ? 'Running...' : 'Test Run'}
				</button>

				<!-- Save Agent -->
				<button
					type="button"
					onclick={handleSave}
					disabled={saving}
					class="flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
				>
					<Save class="h-4 w-4" />
					{saving ? 'Saving...' : editMode ? 'Update Agent' : 'Save Agent'}
				</button>
			</div>
		</div>
	{/if}

</div>
