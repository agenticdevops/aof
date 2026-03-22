<script lang="ts">
	import { X, CheckCircle, AlertCircle, Bot, Key, RefreshCw, Wifi, WifiOff } from 'lucide-svelte';
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

	// Provider auth state
	let providerKeys = $state<Record<string, string>>({});
	let providerModes = $state<Record<string, 'api' | 'subscription'>>({});
	let providerSaving = $state(false);
	let providerSaved = $state(false);
	let anthropicWizardToken = $state('');
	let anthropicWizardSaving = $state(false);
	let authLoading = $state<Record<string, boolean>>({});

	const PROVIDERS = [
		{ id: 'anthropic', label: 'Anthropic', placeholder: 'sk-ant-...' },
		{ id: 'openai', label: 'OpenAI', placeholder: 'sk-...' },
		{ id: 'google', label: 'Google (Gemini)', placeholder: 'AIza...' }
	];

	function setWizardProviderMode(providerId: string, mode: 'api' | 'subscription') {
		providerModes[providerId] = mode;
	}

	async function saveProviderKeys() {
		providerSaving = true;
		try {
			for (const [name, key] of Object.entries(providerKeys)) {
				if (key.trim()) {
					await api.providers.update(name, key.trim());
				}
			}
			providerSaved = true;
			setTimeout(() => (providerSaved = false), 2500);
		} catch {
			// Silently handle — gateway may not support this yet
		} finally {
			providerSaving = false;
		}
	}

	async function startWizardAuth(providerId: string) {
		authLoading[providerId] = true;
		let popup: Window | null = null;
		try {
			const resp = await api.auth.start(providerId);
			if (resp.auth_url) {
				popup = window.open(resp.auth_url, 'agentix-auth', 'width=600,height=700,left=200,top=100');
				const pollStart = Date.now();
				await new Promise<void>((resolve, reject) => {
					const interval = setInterval(async () => {
						if (Date.now() - pollStart > 5 * 60 * 1000) {
							clearInterval(interval);
							reject(new Error('Timed out'));
							return;
						}
						try {
							const status = await api.auth.status(providerId);
							if (status.authenticated) {
								clearInterval(interval);
								if (popup && !popup.closed) popup.close();
								resolve();
							}
						} catch { /* keep polling */ }
					}, 2000);
				});
			}
		} catch {
			if (popup && !popup.closed) popup.close();
		} finally {
			authLoading[providerId] = false;
		}
	}

	async function submitWizardAnthropicToken() {
		const token = anthropicWizardToken.trim();
		if (!token) return;
		anthropicWizardSaving = true;
		try {
			await api.auth.submitToken('anthropic', token);
			anthropicWizardToken = '';
		} catch {
			// Silently handle
		} finally {
			anthropicWizardSaving = false;
		}
	}

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

					<!-- LLM Provider Setup (shown after successful connection) -->
					{#if connectionStatus === 'ok'}
						<div class="space-y-3 pt-2 border-t">
							<div class="flex items-center gap-2">
								<Key class="h-4 w-4 text-muted-foreground" />
								<p class="text-sm font-medium">LLM Providers <span class="text-muted-foreground font-normal">(optional)</span></p>
							</div>
							<p class="text-xs text-muted-foreground">
								Use your existing subscription or an API key — both options work equally well.
							</p>
							{#each PROVIDERS as provider}
								{@const mode = providerModes[provider.id]}
								{@const loading = authLoading[provider.id]}
								<div class="rounded-lg border p-3 space-y-2">
									<!-- Provider header + mode toggle -->
									<div class="flex items-center justify-between gap-2">
										<span class="text-xs font-medium">{provider.label}</span>
										<div class="inline-flex rounded-full border bg-muted/40 p-0.5 gap-0.5 text-xs font-medium shrink-0">
											<button
												onclick={() => setWizardProviderMode(provider.id, 'api')}
												class="rounded-full px-2.5 py-0.5 transition-colors cursor-pointer {mode === 'api'
													? 'bg-background text-foreground shadow-sm'
													: 'text-muted-foreground hover:text-foreground'}"
											>
												API Key
											</button>
											<button
												onclick={() => setWizardProviderMode(provider.id, 'subscription')}
												class="rounded-full px-2.5 py-0.5 transition-colors cursor-pointer {mode === 'subscription'
													? 'bg-background text-foreground shadow-sm'
													: 'text-muted-foreground hover:text-foreground'}"
											>
												Subscription
											</button>
										</div>
									</div>
									<!-- API Key input -->
									{#if mode === 'api'}
										<input
											id="wizard-key-{provider.id}"
											type="password"
											bind:value={providerKeys[provider.id]}
											placeholder={provider.placeholder}
											class="w-full rounded-md border bg-background px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-primary font-mono"
										/>
									{/if}
									<!-- Subscription flow -->
									{#if mode === 'subscription'}
										{#if provider.id === 'anthropic'}
											<div class="space-y-1.5">
												<p class="text-xs text-muted-foreground">
													Paste your token from <code class="font-mono bg-muted px-1 rounded">claude setup-token</code>
												</p>
												<div class="flex gap-2">
													<input
														type="password"
														bind:value={anthropicWizardToken}
														placeholder="Bearer token or sk-ant-..."
														class="flex-1 rounded-md border bg-background px-3 py-1.5 text-xs font-mono focus:outline-none focus:ring-2 focus:ring-primary"
													/>
													<button
														onclick={submitWizardAnthropicToken}
														disabled={anthropicWizardSaving || !anthropicWizardToken.trim()}
														class="rounded-md border px-3 py-1.5 text-xs font-medium hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer shrink-0"
													>
														{anthropicWizardSaving ? 'Saving...' : 'Save'}
													</button>
												</div>
											</div>
										{:else}
											<button
												onclick={() => startWizardAuth(provider.id)}
												disabled={loading}
												class="inline-flex items-center gap-1.5 rounded-md bg-primary/10 px-3 py-1.5 text-xs font-medium text-primary hover:bg-primary/20 transition-colors disabled:opacity-50 cursor-pointer"
											>
												{#if loading}
													<RefreshCw class="h-3 w-3 animate-spin" />
													Authorizing...
												{:else}
													<Wifi class="h-3 w-3" />
													Authorize with {provider.label}
												{/if}
											</button>
										{/if}
									{/if}
								</div>
							{/each}
							<!-- Save API keys button -->
							{#if Object.values(providerModes).some(m => m === 'api') || !Object.keys(providerModes).length}
								<div class="flex items-center gap-2">
									<button
										onclick={saveProviderKeys}
										disabled={providerSaving || !Object.values(providerKeys).some(k => k?.trim())}
										class="rounded-md border px-4 py-1.5 text-xs font-medium hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer"
									>
										{providerSaving ? 'Saving...' : 'Save API Keys'}
									</button>
									{#if providerSaved}
										<span class="text-xs text-green-600 dark:text-green-400">Saved!</span>
									{/if}
								</div>
							{/if}
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
