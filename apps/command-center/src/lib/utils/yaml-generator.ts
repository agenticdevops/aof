import { parse as parseYamlLib, stringify as stringifyYaml } from 'yaml';
import type { AgentFormState, TriggerFormItem } from '$lib/stores/builder.js';

// ============================================================
// Types mirroring the agent.yaml spec
// ============================================================

interface AgentYaml {
	apiVersion: string;
	kind: string;
	metadata: {
		name: string;
		namespace?: string;
		version?: string;
	};
	spec: {
		model: {
			preferred: string;
			fallback?: string;
		};
		mode: string;
		skills?: string[];
		tools?: string[];
		triggers?: Array<{ type: string; expression?: string; channel?: string }>;
		budget?: {
			daily_limit?: number;
			max_tokens_per_run?: number;
		};
		telemetry?: {
			enabled: boolean;
		};
	};
}

// ============================================================
// generateAgentYaml
// ============================================================

/**
 * Convert AgentFormState into a valid agent.yaml string.
 * Omits empty/undefined optional fields to keep the output clean.
 */
export function generateAgentYaml(form: AgentFormState): string {
	const doc: AgentYaml = {
		apiVersion: 'openagentix.dev/v1',
		kind: 'Agent',
		metadata: {
			name: form.name || 'my-agent',
			...(form.namespace && form.namespace !== 'default' ? { namespace: form.namespace } : {}),
			...(form.version && form.version !== '1.0.0' ? { version: form.version } : {})
		},
		spec: {
			model: {
				preferred: form.modelPreferred || 'claude-sonnet-4-20250514',
				...(form.modelFallback ? { fallback: form.modelFallback } : {})
			},
			mode: form.mode,
			...(form.skills.length > 0 ? { skills: form.skills } : {}),
			...(form.tools.length > 0 ? { tools: form.tools } : {}),
			...(form.triggers.length > 0
				? {
						triggers: form.triggers.map((t) => {
							const trigger: { type: string; expression?: string; channel?: string } = {
								type: t.type
							};
							if (t.expression) trigger.expression = t.expression;
							if (t.channel) trigger.channel = t.channel;
							return trigger;
						})
					}
				: {})
		}
	};

	// Budget: only include if at least one field is set
	const dailyLimit = form.budgetDailyLimit.trim()
		? parseFloat(form.budgetDailyLimit)
		: undefined;
	const maxTokens = form.budgetMaxTokens.trim()
		? parseInt(form.budgetMaxTokens, 10)
		: undefined;

	if (
		(dailyLimit !== undefined && !isNaN(dailyLimit)) ||
		(maxTokens !== undefined && !isNaN(maxTokens))
	) {
		doc.spec.budget = {};
		if (dailyLimit !== undefined && !isNaN(dailyLimit)) {
			doc.spec.budget.daily_limit = dailyLimit;
		}
		if (maxTokens !== undefined && !isNaN(maxTokens)) {
			doc.spec.budget.max_tokens_per_run = maxTokens;
		}
	}

	// Telemetry
	doc.spec.telemetry = { enabled: form.telemetryEnabled };

	return stringifyYaml(doc, { lineWidth: 0 });
}

// ============================================================
// parseAgentYaml
// ============================================================

/**
 * Parse agent.yaml content back into a partial AgentFormState.
 * Used for "Edit mode" where an existing agent is loaded into the builder.
 */
export function parseAgentYaml(yamlContent: string): Partial<AgentFormState> {
	let doc: AgentYaml;

	try {
		doc = parseYamlLib(yamlContent) as AgentYaml;
	} catch {
		return {};
	}

	if (!doc || typeof doc !== 'object') return {};

	const result: Partial<AgentFormState> = {};

	// Metadata
	if (doc.metadata?.name) result.name = doc.metadata.name;
	if (doc.metadata?.namespace) result.namespace = doc.metadata.namespace;
	if (doc.metadata?.version) result.version = doc.metadata.version;

	// Model
	if (doc.spec?.model?.preferred) result.modelPreferred = doc.spec.model.preferred;
	if (doc.spec?.model?.fallback) result.modelFallback = doc.spec.model.fallback;

	// Mode
	if (doc.spec?.mode) {
		const mode = doc.spec.mode as AgentFormState['mode'];
		if (mode === 'manual' || mode === 'semi-autonomous' || mode === 'autonomous') {
			result.mode = mode;
		}
	}

	// Skills & Tools
	if (Array.isArray(doc.spec?.skills)) result.skills = doc.spec.skills;
	if (Array.isArray(doc.spec?.tools)) result.tools = doc.spec.tools;

	// Triggers
	if (Array.isArray(doc.spec?.triggers)) {
		result.triggers = doc.spec.triggers.map(
			(t): TriggerFormItem => ({
				type: t.type ?? 'cron',
				...(t.expression ? { expression: t.expression } : {}),
				...(t.channel ? { channel: t.channel } : {})
			})
		);
	}

	// Budget
	if (doc.spec?.budget?.daily_limit != null) {
		result.budgetDailyLimit = String(doc.spec.budget.daily_limit);
	}
	if (doc.spec?.budget?.max_tokens_per_run != null) {
		result.budgetMaxTokens = String(doc.spec.budget.max_tokens_per_run);
	}

	// Telemetry
	if (doc.spec?.telemetry != null) {
		result.telemetryEnabled = doc.spec.telemetry.enabled;
	}

	return result;
}
