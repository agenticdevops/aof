<script lang="ts">
	import { Plus, Trash2 } from 'lucide-svelte';
	import type { AgentFormState, TriggerFormItem } from '$lib/stores/builder.js';

	let {
		form = $bindable(),
		errors = $bindable()
	}: {
		form: AgentFormState;
		errors: Record<string, string>;
	} = $props();

	// ============================================================
	// Model options
	// ============================================================

	const MODEL_OPTIONS = [
		{ value: 'claude-sonnet-4-20250514', label: 'Claude Sonnet 4 (2025-05-14)' },
		{ value: 'claude-haiku-35-20241022', label: 'Claude Haiku 3.5 (2024-10-22)' },
		{ value: 'gpt-4o', label: 'GPT-4o' },
		{ value: 'gpt-4o-mini', label: 'GPT-4o Mini' }
	];

	// ============================================================
	// Trigger types
	// ============================================================

	const TRIGGER_TYPES = [
		{ value: 'cron', label: 'Cron' },
		{ value: 'webhook', label: 'Webhook' },
		{ value: 'github', label: 'GitHub' },
		{ value: 'jira', label: 'Jira' },
		{ value: 'slack', label: 'Slack' },
		{ value: 'discord', label: 'Discord' },
		{ value: 'telegram', label: 'Telegram' }
	];

	const CRON_TRIGGER_TYPES = new Set(['cron']);
	const CHANNEL_TRIGGER_TYPES = new Set(['slack', 'discord', 'telegram']);

	function showsExpression(type: string): boolean {
		return CRON_TRIGGER_TYPES.has(type);
	}

	function showsChannel(type: string): boolean {
		return CHANNEL_TRIGGER_TYPES.has(type);
	}

	// ============================================================
	// Trigger management
	// ============================================================

	function addTrigger() {
		form.triggers = [...form.triggers, { type: 'cron', expression: '', channel: '' }];
	}

	function removeTrigger(index: number) {
		form.triggers = form.triggers.filter((_, i) => i !== index);
	}

	function updateTrigger(index: number, field: keyof TriggerFormItem, value: string) {
		form.triggers = form.triggers.map((t, i) => {
			if (i === index) return { ...t, [field]: value };
			return t;
		});
	}

	// ============================================================
	// Field error helper
	// ============================================================

	function fieldError(field: string): string {
		return errors[field] ?? '';
	}
</script>

<div class="space-y-8">

	<!-- ============================================================ -->
	<!-- Identity                                                      -->
	<!-- ============================================================ -->
	<section>
		<h2 class="text-xs font-semibold uppercase tracking-widest text-muted-foreground mb-4">
			Identity
		</h2>
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
			<!-- Name -->
			<div class="sm:col-span-1">
				<label class="block text-sm font-medium mb-1" for="agent-name">
					Name <span class="text-destructive">*</span>
				</label>
				<input
					id="agent-name"
					type="text"
					bind:value={form.name}
					placeholder="my-agent"
					class={[
						'w-full rounded-md border bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2',
						fieldError('name') ? 'border-destructive focus:ring-destructive/30' : 'border-input focus:ring-ring'
					].join(' ')}
				/>
				{#if fieldError('name')}
					<p class="mt-1 text-xs text-destructive">{fieldError('name')}</p>
				{/if}
			</div>

			<!-- Namespace -->
			<div>
				<label class="block text-sm font-medium mb-1" for="agent-namespace">Namespace</label>
				<input
					id="agent-namespace"
					type="text"
					bind:value={form.namespace}
					placeholder="default"
					class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
				/>
			</div>

			<!-- Version -->
			<div>
				<label class="block text-sm font-medium mb-1" for="agent-version">Version</label>
				<input
					id="agent-version"
					type="text"
					bind:value={form.version}
					placeholder="1.0.0"
					class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
				/>
			</div>
		</div>
	</section>

	<div class="border-t border-border/60"></div>

	<!-- ============================================================ -->
	<!-- Model                                                        -->
	<!-- ============================================================ -->
	<section>
		<h2 class="text-xs font-semibold uppercase tracking-widest text-muted-foreground mb-4">
			Model
		</h2>
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
			<!-- Preferred model -->
			<div>
				<label class="block text-sm font-medium mb-1" for="model-preferred">
					Preferred Model <span class="text-destructive">*</span>
				</label>
				<select
					id="model-preferred"
					bind:value={form.modelPreferred}
					class={[
						'w-full rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2',
						fieldError('modelPreferred')
							? 'border-destructive focus:ring-destructive/30'
							: 'border-input focus:ring-ring'
					].join(' ')}
				>
					<option value="">Select model...</option>
					{#each MODEL_OPTIONS as opt}
						<option value={opt.value}>{opt.label}</option>
					{/each}
				</select>
				{#if fieldError('modelPreferred')}
					<p class="mt-1 text-xs text-destructive">{fieldError('modelPreferred')}</p>
				{/if}
			</div>

			<!-- Fallback model -->
			<div>
				<label class="block text-sm font-medium mb-1" for="model-fallback">
					Fallback Model
					<span class="text-xs font-normal text-muted-foreground ml-1">(optional)</span>
				</label>
				<select
					id="model-fallback"
					bind:value={form.modelFallback}
					class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
				>
					<option value="">None</option>
					{#each MODEL_OPTIONS as opt}
						<option value={opt.value}>{opt.label}</option>
					{/each}
				</select>
			</div>
		</div>
	</section>

	<div class="border-t border-border/60"></div>

	<!-- ============================================================ -->
	<!-- Mode                                                         -->
	<!-- ============================================================ -->
	<section>
		<h2 class="text-xs font-semibold uppercase tracking-widest text-muted-foreground mb-4">
			Execution Mode
		</h2>
		<div class="grid grid-cols-1 gap-3 sm:grid-cols-3">
			{#each [
				{ value: 'manual', label: 'Manual', desc: 'Triggered explicitly by a human or API call' },
				{ value: 'semi-autonomous', label: 'Semi-Autonomous', desc: 'Runs on triggers, pauses for approval on sensitive actions' },
				{ value: 'autonomous', label: 'Autonomous', desc: 'Fully automated — acts without human approval' }
			] as opt}
				<label
					class={[
						'flex cursor-pointer flex-col gap-1 rounded-lg border p-4 transition-colors',
						form.mode === opt.value
							? 'border-primary bg-primary/5 ring-1 ring-primary'
							: 'border-border hover:border-border/80 hover:bg-accent/30'
					].join(' ')}
				>
					<div class="flex items-center gap-2">
						<input
							type="radio"
							name="agent-mode"
							value={opt.value}
							bind:group={form.mode}
							class="text-primary accent-primary"
						/>
						<span class="text-sm font-medium">{opt.label}</span>
					</div>
					<p class="text-xs text-muted-foreground pl-5">{opt.desc}</p>
				</label>
			{/each}
		</div>
	</section>

	<div class="border-t border-border/60"></div>

	<!-- ============================================================ -->
	<!-- Triggers                                                     -->
	<!-- ============================================================ -->
	<section>
		<div class="flex items-center justify-between mb-4">
			<h2 class="text-xs font-semibold uppercase tracking-widest text-muted-foreground">
				Triggers
			</h2>
			<button
				type="button"
				onclick={addTrigger}
				class="flex items-center gap-1.5 rounded-md border border-dashed border-border px-3 py-1.5 text-xs font-medium text-muted-foreground hover:border-primary hover:text-primary transition-colors cursor-pointer"
			>
				<Plus class="h-3 w-3" />
				Add Trigger
			</button>
		</div>

		{#if form.triggers.length === 0}
			<p class="text-sm text-muted-foreground italic">
				No triggers configured — agent runs only on explicit API calls.
			</p>
		{:else}
			<div class="space-y-3">
				{#each form.triggers as trigger, i}
					<div class="flex flex-wrap items-start gap-3 rounded-lg border border-border/60 bg-muted/30 p-3">
						<!-- Trigger type -->
						<div class="w-36">
							<label for="trigger-type-{i}" class="block text-xs font-medium text-muted-foreground mb-1">Type</label>
							<select
								id="trigger-type-{i}"
								value={trigger.type}
								onchange={(e) => updateTrigger(i, 'type', (e.target as HTMLSelectElement).value)}
								class="w-full rounded-md border border-input bg-background px-2 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
							>
								{#each TRIGGER_TYPES as t}
									<option value={t.value}>{t.label}</option>
								{/each}
							</select>
						</div>

						<!-- Expression (cron) -->
						{#if showsExpression(trigger.type)}
							<div class="flex-1 min-w-40">
								<label for="trigger-expr-{i}" class="block text-xs font-medium text-muted-foreground mb-1">
									Cron Expression
								</label>
								<input
									id="trigger-expr-{i}"
									type="text"
									value={trigger.expression ?? ''}
									oninput={(e) => updateTrigger(i, 'expression', (e.target as HTMLInputElement).value)}
									placeholder="0 9 * * 1-5"
									class="w-full rounded-md border border-input bg-background px-2 py-1.5 text-sm font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
								/>
							</div>
						{/if}

						<!-- Channel (messaging) -->
						{#if showsChannel(trigger.type)}
							<div class="flex-1 min-w-40">
								<label for="trigger-channel-{i}" class="block text-xs font-medium text-muted-foreground mb-1">
									Channel ID
								</label>
								<input
									id="trigger-channel-{i}"
									type="text"
									value={trigger.channel ?? ''}
									oninput={(e) => updateTrigger(i, 'channel', (e.target as HTMLInputElement).value)}
									placeholder="#ops-alerts"
									class="w-full rounded-md border border-input bg-background px-2 py-1.5 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
								/>
							</div>
						{/if}

						<!-- Remove -->
						<button
							type="button"
							onclick={() => removeTrigger(i)}
							class="mt-5 rounded-md p-1.5 text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors cursor-pointer"
							aria-label="Remove trigger"
						>
							<Trash2 class="h-4 w-4" />
						</button>
					</div>
				{/each}
			</div>
		{/if}
	</section>

	<div class="border-t border-border/60"></div>

	<!-- ============================================================ -->
	<!-- Budget                                                       -->
	<!-- ============================================================ -->
	<section>
		<h2 class="text-xs font-semibold uppercase tracking-widest text-muted-foreground mb-4">
			Budget
			<span class="text-xs font-normal normal-case tracking-normal text-muted-foreground/70 ml-1">
				(optional)
			</span>
		</h2>
		<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
			<!-- Daily limit -->
			<div>
				<label class="block text-sm font-medium mb-1" for="budget-daily">
					Daily Limit (USD)
				</label>
				<div class="relative">
					<span class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground text-sm pointer-events-none">$</span>
					<input
						id="budget-daily"
						type="number"
						min="0"
						step="0.01"
						bind:value={form.budgetDailyLimit}
						placeholder="5.00"
						class={[
							'w-full rounded-md border bg-background pl-7 pr-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2',
							fieldError('budgetDailyLimit')
								? 'border-destructive focus:ring-destructive/30'
								: 'border-input focus:ring-ring'
						].join(' ')}
					/>
				</div>
				{#if fieldError('budgetDailyLimit')}
					<p class="mt-1 text-xs text-destructive">{fieldError('budgetDailyLimit')}</p>
				{/if}
			</div>

			<!-- Max tokens per run -->
			<div>
				<label class="block text-sm font-medium mb-1" for="budget-tokens">
					Max Tokens per Run
				</label>
				<input
					id="budget-tokens"
					type="number"
					min="0"
					step="1000"
					bind:value={form.budgetMaxTokens}
					placeholder="100000"
					class={[
						'w-full rounded-md border bg-background px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2',
						fieldError('budgetMaxTokens')
							? 'border-destructive focus:ring-destructive/30'
							: 'border-input focus:ring-ring'
					].join(' ')}
				/>
				{#if fieldError('budgetMaxTokens')}
					<p class="mt-1 text-xs text-destructive">{fieldError('budgetMaxTokens')}</p>
				{/if}
			</div>
		</div>
	</section>

	<div class="border-t border-border/60"></div>

	<!-- ============================================================ -->
	<!-- Telemetry                                                    -->
	<!-- ============================================================ -->
	<section>
		<h2 class="text-xs font-semibold uppercase tracking-widest text-muted-foreground mb-4">
			Telemetry
		</h2>
		<label class="flex cursor-pointer items-center gap-3">
			<div class="relative">
				<input
					type="checkbox"
					bind:checked={form.telemetryEnabled}
					class="sr-only"
					id="telemetry-toggle"
				/>
				<!-- Toggle track -->
				<div
					class={[
						'h-6 w-11 rounded-full transition-colors',
						form.telemetryEnabled ? 'bg-primary' : 'bg-muted'
					].join(' ')}
				></div>
				<!-- Toggle thumb -->
				<div
					class={[
						'absolute top-1 h-4 w-4 rounded-full bg-white shadow transition-transform',
						form.telemetryEnabled ? 'left-6' : 'left-1'
					].join(' ')}
				></div>
			</div>
			<div>
				<p class="text-sm font-medium">Enable Telemetry</p>
				<p class="text-xs text-muted-foreground">
					Collect spans and cost data for this agent
				</p>
			</div>
		</label>
	</section>

</div>
