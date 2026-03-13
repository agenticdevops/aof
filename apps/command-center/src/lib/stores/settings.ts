import { writable } from 'svelte/store';
import { browser } from '$app/environment';

const WIZARD_KEY = 'agentix-wizard-complete';
const SETTINGS_KEY = 'agentix-settings';

export interface AppSettings {
	gatewayUrl: string;
	theme: 'light' | 'dark' | 'system';
	refreshInterval: number;
}

const DEFAULT_SETTINGS: AppSettings = {
	gatewayUrl: 'http://localhost:7777',
	theme: 'system',
	refreshInterval: 30
};

function readSettings(): AppSettings {
	if (!browser) return DEFAULT_SETTINGS;
	try {
		const raw = localStorage.getItem(SETTINGS_KEY);
		if (raw) return { ...DEFAULT_SETTINGS, ...JSON.parse(raw) };
	} catch {
		// ignore parse errors
	}
	return DEFAULT_SETTINGS;
}

function readWizardComplete(): boolean {
	if (!browser) return false;
	return localStorage.getItem(WIZARD_KEY) === 'true';
}

export const settings = writable<AppSettings>(readSettings());

export const hasCompletedWizard = writable<boolean>(readWizardComplete());

export function completeWizard(): void {
	hasCompletedWizard.set(true);
	if (browser) {
		localStorage.setItem(WIZARD_KEY, 'true');
	}
}

export function resetWizard(): void {
	hasCompletedWizard.set(false);
	if (browser) {
		localStorage.removeItem(WIZARD_KEY);
	}
}

export function saveSettings(s: AppSettings): void {
	settings.set(s);
	if (browser) {
		localStorage.setItem(SETTINGS_KEY, JSON.stringify(s));
	}
}
