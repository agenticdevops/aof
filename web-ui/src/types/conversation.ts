// Conversation API types for natural language agent creation

// Intent types
export const INTENT_TYPES = {
  CREATE_AGENT: 'create_agent',
  BUILD_SQUAD: 'build_squad',
  CONFIGURE_SCHEDULE: 'configure_schedule',
  TEACH_SKILL: 'teach_skill',
  UNKNOWN: 'unknown',
} as const;

export type IntentType = typeof INTENT_TYPES[keyof typeof INTENT_TYPES];

// Message roles
export const MESSAGE_ROLES = {
  USER: 'user',
  ASSISTANT: 'assistant',
  SYSTEM: 'system',
} as const;

export type MessageRole = typeof MESSAGE_ROLES[keyof typeof MESSAGE_ROLES];

// Message structure
export interface ConversationMessage {
  role: MessageRole;
  content: string;
  timestamp: string;
  filePreview?: FilePreviewData;
}

export interface FilePreviewData {
  files: Record<string, string>;  // path -> content
  summary: string;
}

// API request/response types
export interface CreateSessionRequest {
  // Empty for now - session created with defaults
}

export interface CreateSessionResponse {
  session_id: string;
}

export interface ConversationMessageRequest {
  session_id: string;
  message: string;
}

export interface ConversationMessageResponse {
  session_id: string;
  response: OrchestratorResponse;
  messages: ConversationMessage[];
}

export interface ConversationConfirmRequest {
  session_id: string;
}

export interface ConversationConfirmResponse {
  files_written: string[];
  message: string;
}

export interface ConversationCancelRequest {
  session_id: string;
}

export interface SessionResponse {
  session_id: string;
  messages: ConversationMessage[];
  created_at: string;
}

// Orchestrator response variants
export type OrchestratorResponse =
  | {
      type: 'clarifying_questions';
      questions: string[];
      partial_intent: IntentType;
    }
  | {
      type: 'specialist_result';
      intent: IntentType;
      files: Record<string, string>;
      message: string;
    }
  | {
      type: 'error';
      message: string;
    }
  | {
      type: 'confirmation';
      session_id: string;
      files: Record<string, string>;
      summary: string;
    };

// Helper type guards
export function isClarifyingQuestions(
  response: OrchestratorResponse
): response is Extract<OrchestratorResponse, { type: 'clarifying_questions' }> {
  return response.type === 'clarifying_questions';
}

export function isSpecialistResult(
  response: OrchestratorResponse
): response is Extract<OrchestratorResponse, { type: 'specialist_result' }> {
  return response.type === 'specialist_result';
}

export function isConfirmation(
  response: OrchestratorResponse
): response is Extract<OrchestratorResponse, { type: 'confirmation' }> {
  return response.type === 'confirmation';
}

export function isError(
  response: OrchestratorResponse
): response is Extract<OrchestratorResponse, { type: 'error' }> {
  return response.type === 'error';
}
