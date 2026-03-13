import { writable } from 'svelte/store';

// ============================================================
// Types
// ============================================================

export interface TriggerFormItem {
	type: string;
	expression?: string;
	channel?: string;
}

export interface AgentFormState {
	name: string;
	namespace: string;
	version: string;
	modelPreferred: string;
	modelFallback: string;
	mode: 'manual' | 'semi-autonomous' | 'autonomous';
	skills: string[];
	tools: string[];
	triggers: TriggerFormItem[];
	budgetDailyLimit: string; // string for form input, convert to number on export
	budgetMaxTokens: string;
	telemetryEnabled: boolean;
	soulMd: string; // SOUL.md content
}

// ============================================================
// Default state
// ============================================================

const DEFAULT_STATE: AgentFormState = {
	name: '',
	namespace: 'default',
	version: '1.0.0',
	modelPreferred: 'claude-sonnet-4-20250514',
	modelFallback: '',
	mode: 'semi-autonomous',
	skills: [],
	tools: [],
	triggers: [],
	budgetDailyLimit: '',
	budgetMaxTokens: '',
	telemetryEnabled: true,
	soulMd: ''
};

// ============================================================
// Stores
// ============================================================

export const builderForm = writable<AgentFormState>({ ...DEFAULT_STATE });
export const builderErrors = writable<Record<string, string>>({});
export const testRunOutput = writable<string>('');
export const testRunning = writable<boolean>(false);

// ============================================================
// Validation
// ============================================================

/** Returns true if form is valid, false otherwise. Populates builderErrors. */
export function validateForm(form: AgentFormState): boolean {
	const errors: Record<string, string> = {};

	// Name required
	if (!form.name.trim()) {
		errors.name = 'Agent name is required';
	} else if (!/^[a-z0-9]+(-[a-z0-9]+)*$/.test(form.name.trim())) {
		// kebab-case: lowercase letters, numbers, hyphens (not at start/end, no consecutive)
		errors.name = 'Name must be kebab-case (e.g. my-agent, k8s-ops-bot)';
	}

	// Model required
	if (!form.modelPreferred.trim()) {
		errors.modelPreferred = 'Preferred model is required';
	}

	// Budget validation (if provided, must be positive numbers)
	if (form.budgetDailyLimit.trim()) {
		const n = parseFloat(form.budgetDailyLimit);
		if (isNaN(n) || n <= 0) {
			errors.budgetDailyLimit = 'Must be a positive number (USD)';
		}
	}

	if (form.budgetMaxTokens.trim()) {
		const n = parseInt(form.budgetMaxTokens, 10);
		if (isNaN(n) || n <= 0) {
			errors.budgetMaxTokens = 'Must be a positive integer';
		}
	}

	builderErrors.set(errors);
	return Object.keys(errors).length === 0;
}

/** Reset form to defaults and clear errors/output. */
export function resetForm(): void {
	builderForm.set({ ...DEFAULT_STATE });
	builderErrors.set({});
	testRunOutput.set('');
	testRunning.set(false);
}
