<script lang="ts">
	import { X, CheckCircle, AlertCircle, Bot } from 'lucide-svelte';
	import WizardStep from './wizard-step.svelte';
	import { completeWizard } from '$lib/stores/settings.js';
	import { gatewayUrl, setGatewayUrl } from '$lib/stores/gateway.js';
	import { api } from '$lib/api/client.js';

	interface Props {
		onclose?: () => void;
	}

	let { onclose }: Props = $props();

	const TOTAL_STEPS = 4;
	let currentStep = $state(1);

	// Step 2 state
	let inputUrl = $state($gatewayUrl);
	let connectionStatus = $state<'idle' | 'testing' | 'ok' | 'error'>('idle');
	let connectionError = $state('');

	// Step 3 state
	let agentCount = $state<number | null>(null);
	let agentLoadError = $state('');

	const progress = $derived((currentStep / TOTAL_STEPS) * 100);

	function close() {
		completeWizard();
		onclose?.();
	}

	function goNext() {
		if (currentStep < TOTAL_STEPS) {
			if (currentStep === 2) {
				// load agent count for step 3
				loadAgentCount();
			}
			currentStep++;
		}
	}

	function goBack() {
		if (currentStep > 1) currentStep--;
	}

	async function testConnection() {
		connectionStatus = 'testing';
		connectionError = '';
		// Temporarily set URL so api client picks it up
		setGatewayUrl(inputUrl.trim());
		try {
			await api.health.check();
			connectionStatus = 'ok';
		} catch (err) {
			connectionStatus = 'error';
			connectionError = err instanceof Error ? err.message : 'Connection failed';
		}
	}

	async function loadAgentCount() {
		agentCount = null;
		agentLoadError = '';
		try {
			const agents = await api.agents.list();
			agentCount = agents.length;
		} catch {
			agentCount = 0;
			agentLoadError = 'Could not fetch agents from gateway.';
		}
	}

	function finish() {
		completeWizard();
		onclose?.();
	}
</script>

<!-- Full-screen overlay -->
<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
	<div
		class="relative w-full max-w-2xl rounded-2xl border bg-card shadow-2xl mx-4"
		role="dialog"
		aria-modal="true"
		aria-label="Setup Wizard"
	>
		<!-- Progress bar -->
		<div class="h-1.5 w-full rounded-t-2xl bg-muted overflow-hidden">
			<div
				class="h-full bg-primary transition-all duration-300"
				style="width: {progress}%"
			></div>
		</div>

		<!-- Close button -->
		<button
			onclick={close}
			class="absolute right-4 top-4 rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer"
			aria-label="Close wizard"
		>
			<X class="h-4 w-4" />
		</button>

		<!-- Content area -->
		<div class="p-8">
			<!-- Step 1: Welcome -->
			<WizardStep
				title="Welcome to OpenAgentiX Command Center"
				description="Let's get you set up to monitor and manage your AI agents."
				stepNumber={1}
				totalSteps={TOTAL_STEPS}
				active={currentStep === 1}
				isFirst={true}
				onNext={goNext}
				onBack={goBack}
			>
				<div class="flex flex-col items-center gap-6 py-6">
					<div class="flex h-20 w-20 items-center justify-center rounded-2xl bg-primary/10">
						<Bot class="h-10 w-10 text-primary" />
					</div>
					<div class="text-center space-y-2 max-w-md">
						<p class="text-sm text-muted-foreground">
							OpenAgentiX Command Center gives you a real-time dashboard to monitor agent runs,
							review approval requests, analyze costs, and inspect traces.
						</p>
						<p class="text-sm text-muted-foreground">
							This setup wizard will connect you to a running gateway and get you oriented.
						</p>
					</div>
					<button
						onclick={goNext}
						class="rounded-md bg-primary px-6 py-2.5 text-sm font-semibold text-primary-foreground hover:bg-primary/90 transition-colors cursor-pointer"
					>
						Get Started
					</button>
				</div>
			</WizardStep>

			<!-- Step 2: Connect to Gateway -->
			<WizardStep
				title="Connect to Gateway"
				description="Enter the URL of your running agentix gateway (usually http://localhost:7777)."
				stepNumber={2}
				totalSteps={TOTAL_STEPS}
				active={currentStep === 2}
				onNext={goNext}
				onBack={goBack}
				nextDisabled={connectionStatus !== 'ok'}
			>
				<div class="space-y-4">
					<div class="space-y-2">
						<label for="gateway-url-input" class="text-sm font-medium">Gateway URL</label>
						<div class="flex gap-2">
							<input
								id="gateway-url-input"
								type="url"
								bind:value={inputUrl}
								placeholder="http://localhost:7777"
								class="flex-1 rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
							/>
							<button
								onclick={testConnection}
								disabled={connectionStatus === 'testing'}
								class="rounded-md border px-4 py-2 text-sm font-medium hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer"
							>
								{connectionStatus === 'testing' ? 'Testing...' : 'Test Connection'}
							</button>
						</div>
					</div>

					{#if connectionStatus === 'ok'}
						<div class="flex items-center gap-2 rounded-md bg-green-500/10 border border-green-500/20 px-3 py-2">
							<CheckCircle class="h-4 w-4 text-green-500 shrink-0" />
							<p class="text-sm text-green-600 dark:text-green-400">Connected successfully!</p>
						</div>
					{:else if connectionStatus === 'error'}
						<div class="flex items-center gap-2 rounded-md bg-destructive/10 border border-destructive/20 px-3 py-2">
							<AlertCircle class="h-4 w-4 text-destructive shrink-0" />
							<p class="text-sm text-destructive">{connectionError || 'Could not connect to gateway.'}</p>
						</div>
					{/if}
				</div>
			</WizardStep>

			<!-- Step 3: Explore -->
			<WizardStep
				title="Explore Your Agents"
				description="See what's registered on your gateway."
				stepNumber={3}
				totalSteps={TOTAL_STEPS}
				active={currentStep === 3}
				onNext={goNext}
				onBack={goBack}
			>
				<div class="space-y-4">
					{#if agentCount === null && !agentLoadError}
						<p class="text-sm text-muted-foreground">Loading agents...</p>
					{:else if agentLoadError}
						<div class="rounded-md bg-destructive/10 border border-destructive/20 px-3 py-2">
							<p class="text-sm text-destructive">{agentLoadError}</p>
						</div>
					{:else if agentCount === 0}
						<div class="rounded-xl border border-dashed p-6 text-center space-y-3">
							<Bot class="mx-auto h-8 w-8 text-muted-foreground/50" />
							<p class="text-sm font-medium">No agents registered yet.</p>
							<p class="text-xs text-muted-foreground">
								You can create one in the Agent Builder after setup.
							</p>
						</div>
					{:else}
						<div class="rounded-xl border bg-green-500/5 p-6 text-center space-y-2">
							<p class="text-3xl font-bold text-primary">{agentCount}</p>
							<p class="text-sm text-muted-foreground">
								agent{agentCount !== 1 ? 's' : ''} registered. Explore them on the dashboard.
							</p>
						</div>
					{/if}

					<div class="flex gap-3">
						<a
							href="/"
							onclick={finish}
							class="flex-1 rounded-md border px-4 py-2 text-sm font-medium text-center hover:bg-accent transition-colors"
						>
							Go to Dashboard
						</a>
						<a
							href="/builder"
							onclick={finish}
							class="flex-1 rounded-md bg-primary/10 px-4 py-2 text-sm font-medium text-center text-primary hover:bg-primary/20 transition-colors"
						>
							Open Agent Builder
						</a>
					</div>
				</div>
			</WizardStep>

			<!-- Step 4: Done -->
			<WizardStep
				title="You're All Set!"
				description="Here's a quick overview of everything available in the Command Center."
				stepNumber={4}
				totalSteps={TOTAL_STEPS}
				active={currentStep === 4}
				isLast={true}
				onBack={goBack}
				onFinish={finish}
			>
				<div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
					{#each [
						{ label: 'Dashboard', desc: 'Metrics and activity feed' },
						{ label: 'Agents', desc: 'Live status and details' },
						{ label: 'Costs', desc: 'Spending analytics' },
						{ label: 'Traces', desc: 'Execution waterfall' },
						{ label: 'Approvals', desc: 'Approve/deny actions' },
						{ label: 'Builder', desc: 'Create and edit agents' }
					] as item}
						<div class="rounded-lg border p-3 space-y-0.5">
							<p class="text-sm font-semibold">{item.label}</p>
							<p class="text-xs text-muted-foreground">{item.desc}</p>
						</div>
					{/each}
				</div>
			</WizardStep>
		</div>
	</div>
</div>
