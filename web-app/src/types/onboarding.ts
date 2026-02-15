/**
 * Comprehensive TypeScript types for the onboarding wizard.
 * Defines types for channels, AI models, tools, Xops configuration, and wizard state.
 */

/**
 * Supported communication channels/platforms for Xops
 */
export type Channel = 'slack' | 'telegram' | 'discord' | 'none'

/**
 * Configuration for a communication channel
 */
export interface ChannelConfig {
  /** Platform identifier */
  platform: Channel
  /** Webhook URL for platform (Slack) */
  webhookUrl?: string
  /** Bot token for platform (Telegram, Discord) */
  botToken?: string
  /** Channel name or ID where Xops will appear */
  channelName?: string
  /** Server ID (Discord) */
  serverId?: string
  /** Channel ID (Discord) */
  channelId?: string
  /** Chat ID or Channel ID (Telegram) */
  chatId?: string
  /** Whether the channel has been validated */
  validated: boolean
}

/**
 * Supported AI model providers
 */
export type AIModelProvider = 'anthropic' | 'openai' | 'google' | 'groq' | 'ollama'

/**
 * Configuration for AI model selection and API key
 */
export interface AIModelConfig {
  /** LLM provider (Anthropic, OpenAI, etc.) */
  provider: AIModelProvider
  /** Specific model variant (e.g., "claude-haiku-4.5", "gpt-4o") */
  model: string
  /** API key for the provider */
  apiKey: string
}

/**
 * Tool categories for organization and discovery
 */
export type ToolCategory = 'kubectl' | 'terraform' | 'docker' | 'git' | 'aws' | 'shell' | 'custom' | 'other'

/**
 * Represents an available tool that Xops can use
 */
export interface Tool {
  /** Unique identifier for the tool */
  id: string
  /** Display name (e.g., "kubectl", "terraform") */
  name: string
  /** Semantic version (e.g., "1.29.0") */
  version: string
  /** File path where tool is located */
  path: string
  /** Whether the tool is available on the system */
  available: boolean
  /** Whether user has enabled this tool for Xops */
  enabled: boolean
  /** Tool category for grouping */
  category: ToolCategory
  /** Description of what the tool does */
  description?: string
}

/**
 * Personality traits and communication style for Xops agent
 */
export interface AgentPersona {
  /** Unique persona identifier */
  id: string
  /** Display name (e.g., "Xops", "Coordinator") */
  name: string
  /** Description of the persona and its role */
  description: string
  /** Personality traits (e.g., ["analytical", "leadership", "decisive"]) */
  traits: string[]
  /** Communication style description */
  communication_style: string
  /** Emoji or icon representing the persona */
  emoji?: string
}

/**
 * Complete Xops agent configuration
 */
export interface XopsConfig {
  /** Agent name (editable by user) */
  name: string
  /** Pre-defined persona */
  persona: AgentPersona
  /** Agent role (always "orchestrator" for Xops) */
  role: 'orchestrator'
  /** Selected communication channels */
  channels: ChannelConfig[]
  /** Selected AI model and API key */
  model: AIModelConfig
  /** Selected tools for Xops to use */
  tools: Tool[]
  /** Timestamp when Xops was created */
  createdAt?: string
  /** Agent ID assigned by backend */
  agentId?: string
}

/**
 * Wizard step identifiers (3 main steps + 1 review step)
 */
export type WizardStep = 'channels' | 'model' | 'tools' | 'review'

/**
 * Numeric step representation for internal tracking
 */
export type StepNumber = 1 | 2 | 3 | 4

/**
 * Redux state for the onboarding wizard
 */
export interface OnboardingState {
  /** Current step in the wizard (1-4) */
  currentStep: StepNumber
  /** Selected communication channels */
  selectedChannels: Channel[]
  /** Selected AI model provider */
  selectedModel: AIModelProvider
  /** Selected tools */
  selectedTools: Tool[]
  /** Complete Xops configuration (populated on review) */
  xopsConfig: Partial<XopsConfig>
  /** Whether an API call is in progress */
  loading: boolean
  /** Error message if something went wrong */
  error?: string | null
}

/**
 * Validation error for a specific field
 */
export interface ValidationError {
  /** Field name that failed validation */
  field: string
  /** User-friendly error message */
  message: string
}

/**
 * Form validation errors keyed by field
 */
export interface FormErrors {
  [key: string]: ValidationError[]
}

/**
 * Result of channel validation
 */
export interface ChannelValidationResult {
  /** Whether validation passed */
  success: boolean
  /** Channel that was validated */
  channel: Channel
  /** Error message if validation failed */
  error?: string
}

/**
 * Result of API key validation
 */
export interface ModelValidationResult {
  /** Whether validation passed */
  success: boolean
  /** Provider that was validated */
  provider: AIModelProvider
  /** Error message if validation failed */
  error?: string
  /** Usage information from test API call */
  usage?: {
    inputTokens: number
    outputTokens: number
  }
}

/**
 * Tool discovery result from backend
 */
export interface ToolDiscoveryResult {
  /** List of discovered tools */
  tools: Tool[]
  /** Error message if discovery failed */
  error?: string
  /** Timestamp when discovery occurred */
  discoveredAt: string
}

/**
 * Recommended tools that should be pre-selected
 */
export const RECOMMENDED_TOOLS = [
  'kubectl',
  'terraform',
  'docker',
  'git',
  'aws-cli',
  'helm',
]

/**
 * Default Xops persona (used for all deployments)
 */
export const DEFAULT_XOPS_PERSONA: AgentPersona = {
  id: 'xops-default',
  name: 'Xops',
  description: 'Orchestrator and coordinator of specialized agents. Focuses on coordination, delegation, and result synthesis.',
  traits: ['analytical', 'leadership', 'decisive', 'collaborative'],
  communication_style: 'Clear, concise, action-oriented. Speaks with authority but invites input.',
  emoji: '🤖',
}

/**
 * Provider information for UI display
 */
export const PROVIDER_INFO: Record<AIModelProvider, {
  name: string
  description: string
  models: { value: string; label: string }[]
  costEstimate: string
  helpUrl?: string
}> = {
  anthropic: {
    name: 'Anthropic',
    description: 'Best for complex reasoning and analysis. Fast and reliable.',
    models: [
      { value: 'claude-opus-4.6', label: 'Claude Opus 4.6 (Most capable)' },
      { value: 'claude-sonnet-4', label: 'Claude Sonnet 4 (Balanced)' },
      { value: 'claude-haiku-4.5', label: 'Claude Haiku 4.5 (Fast, cheap)' },
    ],
    costEstimate: '$0.003 per 1K input tokens',
    helpUrl: 'https://console.anthropic.com',
  },
  openai: {
    name: 'OpenAI',
    description: 'Industry standard with strong performance across tasks.',
    models: [
      { value: 'gpt-4o', label: 'GPT-4o (Latest)' },
      { value: 'gpt-4-turbo', label: 'GPT-4 Turbo' },
      { value: 'gpt-3.5-turbo', label: 'GPT-3.5 Turbo (Fast, cheap)' },
    ],
    costEstimate: '$0.005 per 1K input tokens',
    helpUrl: 'https://platform.openai.com',
  },
  google: {
    name: 'Google',
    description: 'Gemini models with multimodal capabilities.',
    models: [
      { value: 'gemini-2.0-flash', label: 'Gemini 2.0 Flash' },
      { value: 'gemini-pro', label: 'Gemini Pro' },
      { value: 'gemini-pro-vision', label: 'Gemini Pro Vision' },
    ],
    costEstimate: '$0.0005 per 1K input tokens',
    helpUrl: 'https://aistudio.google.com',
  },
  groq: {
    name: 'Groq',
    description: 'Very fast inference with free tier available.',
    models: [
      { value: 'mixtral-8x7b-32768', label: 'Mixtral 8x7B' },
      { value: 'llama2-70b-4096', label: 'Llama 2 70B' },
    ],
    costEstimate: 'Free tier available',
    helpUrl: 'https://console.groq.com',
  },
  ollama: {
    name: 'Ollama',
    description: 'Run models locally on your machine. Free and private.',
    models: [
      { value: 'llama2', label: 'Llama 2' },
      { value: 'mistral', label: 'Mistral' },
      { value: 'neural-chat', label: 'Neural Chat' },
    ],
    costEstimate: 'Free (runs locally)',
  },
}
