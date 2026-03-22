<script lang="ts">
	import { Settings, CheckCircle, AlertCircle, RefreshCw, Info, Key, Eye, EyeOff } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { settings, saveSettings, resetWizard, hasCompletedWizard } from '$lib/stores/settings.js';
	import { setGatewayUrl, gatewayUrl } from '$lib/stores/gateway.js';
	import { api } from '$lib/api/client.js';
	import type { AppSettings } from '$lib/stores/settings.js';
	import type { ProviderStatus } from '$lib/api/types.js';

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

	const KNOWN_PROVIDERS = [
		{ id: 'anthropic', label: 'Anthropic', placeholder: 'sk-ant-...' },
		{ id: 'openai', label: 'OpenAI', placeholder: 'sk-...' },
		{ id: 'google', label: 'Google (Gemini)', placeholder: 'AIza...' }
	];

	onMount(() => {
		// Sync form with gateway URL store
		form.gatewayUrl = $gatewayUrl;
		loadProviders();
	});

	async function loadProviders() {
		try {
			providers = await api.providers.list();
		} catch {
			// Gateway may not support this endpoint yet
			providers = [];
		}
	}

	function getProviderStatus(id: string): ProviderStatus | undefined {
		return providers.find(p => p.name === id);
	}

	async function saveProviderKey(name: string) {
		const key = providerKeys[name]?.trim();
		if (!key) return;

		providerSaving = true;
		try {
			await api.providers.update(name, key);
			providerSaved = true;
			setTimeout(() => (providerSaved = false), 2500);
			// Reload to reflect changes
			await loadProviders();
			// Clear the input after save
			providerKeys[name] = '';
		} catch {
			// Silently handle
		} finally {
			providerSaving = false;
		}
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
		// Navigate to root — wizard will auto-show
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

	<!-- LLM Provider Keys -->
	<section class="space-y-4">
		<div class="flex items-center gap-2 border-b pb-2">
			<Key class="h-4 w-4 text-muted-foreground" />
			<h2 class="text-base font-semibold">LLM Provider Keys</h2>
		</div>
		<p class="text-xs text-muted-foreground">
			Configure API keys for LLM providers. Runtime keys override values from agentix.yaml and environment variables.
		</p>
		<div class="space-y-4">
			{#each KNOWN_PROVIDERS as provider}
				{@const status = getProviderStatus(provider.id)}
				<div class="space-y-2 rounded-lg border p-4">
					<div class="flex items-center justify-between">
						<label for="settings-key-{provider.id}" class="text-sm font-medium">{provider.label}</label>
						{#if status?.configured}
							<span class="inline-flex items-center gap-1 rounded-full bg-green-500/10 px-2 py-0.5 text-xs text-green-600 dark:text-green-400">
								<CheckCircle class="h-3 w-3" />
								Configured
								{#if status.source === 'config'}
									<span class="text-muted-foreground">(from config)</span>
								{:else if status.source === 'runtime'}
									<span class="text-muted-foreground">(runtime)</span>
								{/if}
							</span>
						{:else}
							<span class="text-xs text-muted-foreground">Not configured</span>
						{/if}
					</div>
					{#if status?.api_key_masked}
						<p class="text-xs font-mono text-muted-foreground">Current: {status.api_key_masked}</p>
					{/if}
					<div class="flex gap-2">
						<div class="relative flex-1">
							<input
								id="settings-key-{provider.id}"
								type={showKeys[provider.id] ? 'text' : 'password'}
								bind:value={providerKeys[provider.id]}
								placeholder={status?.configured ? 'Enter new key to replace...' : provider.placeholder}
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
