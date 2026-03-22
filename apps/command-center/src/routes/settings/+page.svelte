<script lang="ts">
	import { Settings, CheckCircle, AlertCircle, RefreshCw, Info, Key, Eye, EyeOff, Wifi, WifiOff } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { settings, saveSettings, resetWizard, hasCompletedWizard } from '$lib/stores/settings.js';
	import { setGatewayUrl, gatewayUrl } from '$lib/stores/gateway.js';
	import { api } from '$lib/api/client.js';
	import type { AppSettings } from '$lib/stores/settings.js';
	import type { ProviderStatus, AuthStatus } from '$lib/api/types.js';

	let form = $state<AppSettings>({ ...$settings });
	let connectionStatus = $state<'idle' | 'testing' | 'ok' | 'error'>('idle');
	let connectionError = $state('');
	let saved = $state(false);

	// Provider state
	let providers = $state<ProviderStatus[]>([]);
	let providerKeys = $state<Record<string, string>>({});
	let providerSaving = $state(false);
	let providerSaved = $state(false);
	let showKeys = $state<Record<string, boolean>>({});

	// Subscription auth state
	let providerModes = $state<Record<string, 'api' | 'subscription'>>({});
	let authStatuses = $state<Record<string, AuthStatus>>({});
	let authLoading = $state<Record<string, boolean>>({});
	let authError = $state<Record<string, string>>({});
	let anthropicToken = $state('');
	let anthropicTokenSaving = $state(false);

	const KNOWN_PROVIDERS = [
		{ id: 'anthropic', label: 'Anthropic', placeholder: 'sk-ant-...' },
		{ id: 'openai', label: 'OpenAI', placeholder: 'sk-...' },
		{ id: 'google', label: 'Google (Gemini)', placeholder: 'AIza...' }
	];

	onMount(() => {
		// Sync form with gateway URL store
		form.gatewayUrl = $gatewayUrl;
		loadProviders();
		loadAuthStatuses();
	});

	async function loadProviders() {
		try {
			providers = await api.providers.list();
		} catch {
			// Gateway may not support this endpoint yet
			providers = [];
		}
	}

	async function loadAuthStatuses() {
		for (const provider of KNOWN_PROVIDERS) {
			try {
				const status = await api.auth.status(provider.id);
				authStatuses[provider.id] = status;
				// If authenticated via OAuth, default to subscription mode
				if (status.authenticated) {
					providerModes[provider.id] = 'subscription';
				}
			} catch {
				// Auth endpoint may not exist yet — silently ignore
			}
		}
		// For providers without auth status, check API key config
		for (const provider of KNOWN_PROVIDERS) {
			if (providerModes[provider.id]) continue;
			const keyStatus = providers.find(p => p.name === provider.id);
			if (keyStatus?.configured) {
				providerModes[provider.id] = 'api';
			}
			// If neither, leave undefined — user must choose
		}
	}

	function getProviderStatus(id: string): ProviderStatus | undefined {
		return providers.find(p => p.name === id);
	}

	function setProviderMode(providerId: string, mode: 'api' | 'subscription') {
		providerModes[providerId] = mode;
		authError[providerId] = '';
	}

	async function saveProviderKey(name: string) {
		const key = providerKeys[name]?.trim();
		if (!key) return;

		providerSaving = true;
		try {
			await api.providers.update(name, key);
			providerSaved = true;
			setTimeout(() => (providerSaved = false), 2500);
			await loadProviders();
			providerKeys[name] = '';
		} catch {
			// Silently handle
		} finally {
			providerSaving = false;
		}
	}

	async function startAuth(providerId: string) {
		authLoading[providerId] = true;
		authError[providerId] = '';
		let popup: Window | null = null;

		try {
			const resp = await api.auth.start(providerId);

			if (!resp.auth_url) {
				authError[providerId] = 'No authorization URL returned.';
				return;
			}

			// Open OAuth popup
			popup = window.open(resp.auth_url, 'agentix-auth', 'width=600,height=700,left=200,top=100');

			// Poll for completion (every 2s, timeout after 5 min)
			const pollStart = Date.now();
			const TIMEOUT_MS = 5 * 60 * 1000;
			const POLL_INTERVAL_MS = 2000;

			await new Promise<void>((resolve, reject) => {
				const interval = setInterval(async () => {
					if (Date.now() - pollStart > TIMEOUT_MS) {
						clearInterval(interval);
						reject(new Error('Authorization timed out. Please try again.'));
						return;
					}

					try {
						const status = await api.auth.status(providerId);
						if (status.authenticated) {
							clearInterval(interval);
							authStatuses[providerId] = status;
							if (popup && !popup.closed) popup.close();
							resolve();
						}
					} catch {
						// Status call failed — keep polling
					}
				}, POLL_INTERVAL_MS);
			});
		} catch (err) {
			if (popup && !popup.closed) popup.close();
			authError[providerId] = err instanceof Error ? err.message : 'Authorization failed. Please try again.';
		} finally {
			authLoading[providerId] = false;
		}
	}

	async function disconnectAuth(providerId: string) {
		authLoading[providerId] = true;
		authError[providerId] = '';
		try {
			await api.auth.disconnect(providerId);
			authStatuses[providerId] = {
				provider: providerId,
				authenticated: false,
				mode: null,
				expires_at: null,
				account_id: null,
				needs_reauth: false
			};
		} catch (err) {
			authError[providerId] = err instanceof Error ? err.message : 'Failed to disconnect.';
		} finally {
			authLoading[providerId] = false;
		}
	}

	async function submitAnthropicToken() {
		const token = anthropicToken.trim();
		if (!token) return;

		anthropicTokenSaving = true;
		authError['anthropic'] = '';
		try {
			await api.auth.submitToken('anthropic', token);
			anthropicToken = '';
			// Refresh status
			const status = await api.auth.status('anthropic');
			authStatuses['anthropic'] = status;
		} catch (err) {
			authError['anthropic'] = err instanceof Error ? err.message : 'Failed to save token.';
		} finally {
			anthropicTokenSaving = false;
		}
	}

	function formatRelativeTime(isoDate: string | null): string {
		if (!isoDate) return '';
		const exp = new Date(isoDate).getTime();
		const now = Date.now();
		const diffMs = exp - now;

		if (diffMs <= 0) return 'expired';

		const totalMinutes = Math.floor(diffMs / 60000);
		const hours = Math.floor(totalMinutes / 60);
		const minutes = totalMinutes % 60;

		if (hours > 0) return `in ${hours}h ${minutes}m`;
		return `in ${minutes}m`;
	}

	async function testConnection() {
		connectionStatus = 'testing';
		connectionError = '';
		setGatewayUrl(form.gatewayUrl.trim());
		try {
			await api.health.check();
			connectionStatus = 'ok';
		} catch (err) {
			connectionStatus = 'error';
			connectionError = err instanceof Error ? err.message : 'Connection failed';
		}
	}

	function save() {
		setGatewayUrl(form.gatewayUrl.trim());
		saveSettings({ ...form, gatewayUrl: form.gatewayUrl.trim() });
		saved = true;
		setTimeout(() => (saved = false), 2500);
	}

	function relaunchWizard() {
		resetWizard();
		window.location.href = '/';
	}
</script>

<div class="p-6 space-y-8 max-w-2xl">
	<!-- Header -->
	<div class="flex items-center gap-3">
		<Settings class="h-6 w-6 text-muted-foreground" />
		<div>
			<h1 class="text-2xl font-bold tracking-tight">Settings</h1>
			<p class="text-sm text-muted-foreground mt-0.5">Configure your Command Center preferences.</p>
		</div>
	</div>

	<!-- Gateway Connection -->
	<section class="space-y-4">
		<h2 class="text-base font-semibold border-b pb-2">Gateway Connection</h2>
		<div class="space-y-2">
			<label for="settings-gateway-url" class="text-sm font-medium">Gateway URL</label>
			<div class="flex gap-2">
				<input
					id="settings-gateway-url"
					type="url"
					bind:value={form.gatewayUrl}
					placeholder="http://localhost:7777"
					class="flex-1 rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
				/>
				<button
					onclick={testConnection}
					disabled={connectionStatus === 'testing'}
					class="inline-flex items-center gap-2 rounded-md border px-4 py-2 text-sm font-medium hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer"
				>
					{#if connectionStatus === 'testing'}
						<RefreshCw class="h-4 w-4 animate-spin" />
						Testing...
					{:else}
						Test Connection
					{/if}
				</button>
			</div>
			{#if connectionStatus === 'ok'}
				<div class="flex items-center gap-2 rounded-md bg-green-500/10 border border-green-500/20 px-3 py-2">
					<CheckCircle class="h-4 w-4 text-green-500 shrink-0" />
					<p class="text-sm text-green-600 dark:text-green-400">Connected successfully.</p>
				</div>
			{:else if connectionStatus === 'error'}
				<div class="flex items-center gap-2 rounded-md bg-destructive/10 border border-destructive/20 px-3 py-2">
					<AlertCircle class="h-4 w-4 text-destructive shrink-0" />
					<p class="text-sm text-destructive">{connectionError}</p>
				</div>
			{/if}
		</div>
	</section>

	<!-- LLM Providers -->
	<section class="space-y-4">
		<div class="flex items-center gap-2 border-b pb-2">
			<Key class="h-4 w-4 text-muted-foreground" />
			<h2 class="text-base font-semibold">LLM Providers</h2>
		</div>
		<p class="text-xs text-muted-foreground">
			Choose how to authenticate with each LLM provider. Use your existing subscription or an API key — both options are available.
		</p>
		<div class="space-y-4">
			{#each KNOWN_PROVIDERS as provider}
				{@const keyStatus = getProviderStatus(provider.id)}
				{@const authStatus = authStatuses[provider.id]}
				{@const mode = providerModes[provider.id]}
				{@const loading = authLoading[provider.id]}
				{@const error = authError[provider.id]}
				<div class="rounded-lg border p-4 space-y-3">
					<!-- Card header: provider label + segmented control -->
					<div class="flex items-center justify-between gap-3">
						<span class="text-sm font-medium">{provider.label}</span>

						<!-- Segmented mode control -->
						<div class="inline-flex rounded-full border bg-muted/40 p-0.5 gap-0.5 text-xs font-medium shrink-0">
							<button
								onclick={() => setProviderMode(provider.id, 'api')}
								class="rounded-full px-3 py-1 transition-colors cursor-pointer {mode === 'api'
									? 'bg-background text-foreground shadow-sm'
									: 'text-muted-foreground hover:text-foreground'}"
							>
								API Key
							</button>
							<button
								onclick={() => setProviderMode(provider.id, 'subscription')}
								class="rounded-full px-3 py-1 transition-colors cursor-pointer {mode === 'subscription'
									? 'bg-background text-foreground shadow-sm'
									: 'text-muted-foreground hover:text-foreground'}"
							>
								Subscription
							</button>
						</div>
					</div>

					<!-- API Key mode -->
					{#if mode === 'api' || !mode}
						<div class="space-y-2">
							{#if keyStatus?.configured}
								<div class="flex items-center gap-2">
									<span class="inline-flex items-center gap-1 rounded-full bg-green-500/10 px-2 py-0.5 text-xs text-green-600 dark:text-green-400">
										<CheckCircle class="h-3 w-3" />
										Configured
										{#if keyStatus.source === 'config'}
											<span class="text-muted-foreground">(from config)</span>
										{:else if keyStatus.source === 'runtime'}
											<span class="text-muted-foreground">(runtime)</span>
										{/if}
									</span>
									{#if keyStatus.api_key_masked}
										<span class="text-xs font-mono text-muted-foreground">{keyStatus.api_key_masked}</span>
									{/if}
								</div>
							{:else}
								<span class="text-xs text-muted-foreground">No API key configured</span>
							{/if}
							<div class="flex gap-2">
								<div class="relative flex-1">
									<input
										id="settings-key-{provider.id}"
										type={showKeys[provider.id] ? 'text' : 'password'}
										bind:value={providerKeys[provider.id]}
										placeholder={keyStatus?.configured ? 'Enter new key to replace...' : provider.placeholder}
										class="w-full rounded-md border bg-background px-3 py-1.5 pr-10 text-sm focus:outline-none focus:ring-2 focus:ring-primary font-mono"
									/>
									<button
										type="button"
										onclick={() => (showKeys[provider.id] = !showKeys[provider.id])}
										class="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground cursor-pointer"
										aria-label={showKeys[provider.id] ? 'Hide key' : 'Show key'}
									>
										{#if showKeys[provider.id]}
											<EyeOff class="h-4 w-4" />
										{:else}
											<Eye class="h-4 w-4" />
										{/if}
									</button>
								</div>
								<button
									onclick={() => saveProviderKey(provider.id)}
									disabled={providerSaving || !providerKeys[provider.id]?.trim()}
									class="rounded-md border px-4 py-1.5 text-sm font-medium hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer shrink-0"
								>
									{providerSaving ? 'Saving...' : 'Save'}
								</button>
							</div>
						</div>
					{/if}

					<!-- Subscription mode -->
					{#if mode === 'subscription'}
						<div class="space-y-2">
							{#if authStatus?.authenticated}
								<!-- Connected state -->
								<div class="flex items-center justify-between gap-2">
									<div class="flex items-center gap-2">
										<span class="inline-flex items-center gap-1.5 text-sm text-green-600 dark:text-green-400">
											<span class="h-2 w-2 rounded-full bg-green-500 shrink-0"></span>
											Connected
										</span>
										{#if authStatus.account_id}
											<span class="text-xs text-muted-foreground font-mono">{authStatus.account_id}</span>
										{/if}
										{#if authStatus.expires_at}
											<span class="text-xs text-muted-foreground">
												{formatRelativeTime(authStatus.expires_at)}
											</span>
										{/if}
									</div>
									<div class="flex gap-2 shrink-0">
										<button
											onclick={() => startAuth(provider.id)}
											disabled={loading}
											class="rounded-md border px-3 py-1 text-xs font-medium hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer"
										>
											{loading ? 'Authorizing...' : 'Re-authorize'}
										</button>
										<button
											onclick={() => disconnectAuth(provider.id)}
											disabled={loading}
											class="rounded-md border border-destructive/40 px-3 py-1 text-xs font-medium text-destructive hover:bg-destructive/10 transition-colors disabled:opacity-50 cursor-pointer"
										>
											Disconnect
										</button>
									</div>
								</div>
							{:else}
								<!-- Not connected state -->
								{#if provider.id === 'anthropic'}
									<!-- Anthropic: token paste flow -->
									<div class="space-y-2">
										<p class="text-xs text-muted-foreground">
											Paste your token from <code class="font-mono bg-muted px-1 rounded">claude setup-token</code> or enter a Bearer token.
										</p>
										<div class="flex gap-2">
											<input
												type="password"
												bind:value={anthropicToken}
												placeholder="sk-ant-... or Bearer token"
												class="flex-1 rounded-md border bg-background px-3 py-1.5 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-primary"
											/>
											<button
												onclick={submitAnthropicToken}
												disabled={anthropicTokenSaving || !anthropicToken.trim()}
												class="rounded-md border px-4 py-1.5 text-sm font-medium hover:bg-accent transition-colors disabled:opacity-50 cursor-pointer shrink-0"
											>
												{anthropicTokenSaving ? 'Saving...' : 'Save'}
											</button>
										</div>
									</div>
								{:else}
									<!-- OpenAI / Gemini: OAuth popup flow -->
									<div class="flex items-center gap-3">
										<span class="inline-flex items-center gap-1.5 text-sm text-muted-foreground">
											<WifiOff class="h-4 w-4" />
											Not connected
										</span>
										<button
											onclick={() => startAuth(provider.id)}
											disabled={loading}
											class="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 cursor-pointer"
										>
											{#if loading}
												<RefreshCw class="h-3.5 w-3.5 animate-spin" />
												Authorizing...
											{:else}
												<Wifi class="h-3.5 w-3.5" />
												Authorize with {provider.label}
											{/if}
										</button>
									</div>
								{/if}
							{/if}

							<!-- Error message -->
							{#if error}
								<div class="flex items-center gap-2 rounded-md bg-destructive/10 border border-destructive/20 px-3 py-2">
									<AlertCircle class="h-4 w-4 text-destructive shrink-0" />
									<p class="text-xs text-destructive">{error}</p>
								</div>
							{/if}
						</div>
					{/if}
				</div>
			{/each}
		</div>
		{#if providerSaved}
			<div class="flex items-center gap-1 text-sm text-green-600 dark:text-green-400">
				<CheckCircle class="h-4 w-4" /> Provider key saved
			</div>
		{/if}
	</section>

	<!-- Appearance -->
	<section class="space-y-4">
		<h2 class="text-base font-semibold border-b pb-2">Appearance</h2>
		<div class="space-y-2">
			<label for="settings-theme" class="text-sm font-medium">Theme</label>
			<select
				id="settings-theme"
				bind:value={form.theme}
				class="w-48 rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-primary"
			>
				<option value="system">System</option>
				<option value="light">Light</option>
				<option value="dark">Dark</option>
			</select>
		</div>
	</section>

	<!-- Onboarding -->
	<section class="space-y-4">
		<h2 class="text-base font-semibold border-b pb-2">Onboarding</h2>
		<div class="flex items-start gap-4">
			<div class="flex-1">
				<p class="text-sm font-medium">Re-launch Setup Wizard</p>
				<p class="text-xs text-muted-foreground mt-0.5">
					Re-run the first-time setup to reconfigure your gateway connection.
				</p>
			</div>
			<button
				onclick={relaunchWizard}
				class="rounded-md border px-4 py-2 text-sm font-medium hover:bg-accent transition-colors cursor-pointer shrink-0"
			>
				Re-launch Wizard
			</button>
		</div>
	</section>

	<!-- About -->
	<section class="space-y-4">
		<h2 class="text-base font-semibold border-b pb-2">About</h2>
		<div class="space-y-2 text-sm text-muted-foreground">
			<div class="flex items-center gap-2">
				<Info class="h-4 w-4 shrink-0" />
				<span>OpenAgentiX Command Center — v2.0.0-alpha.10</span>
			</div>
			<div class="flex gap-4 text-xs">
				<a
					href="https://docs.aof.sh/command-center"
					target="_blank"
					rel="noopener noreferrer"
					class="underline hover:text-foreground"
				>Documentation</a>
				<a
					href="https://github.com/agenticdevops/aof"
					target="_blank"
					rel="noopener noreferrer"
					class="underline hover:text-foreground"
				>GitHub</a>
			</div>
		</div>
	</section>

	<!-- Save button -->
	<div class="flex items-center gap-3">
		<button
			onclick={save}
			class="rounded-md bg-primary px-5 py-2 text-sm font-semibold text-primary-foreground hover:bg-primary/90 transition-colors cursor-pointer"
		>
			Save Settings
		</button>
		{#if saved}
			<span class="text-sm text-green-600 dark:text-green-400 flex items-center gap-1">
				<CheckCircle class="h-4 w-4" /> Saved
			</span>
		{/if}
	</div>
</div>
