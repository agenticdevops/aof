/**
 * Redux slice for conversational agent creation.
 * Manages conversation sessions, messages, and file previews.
 */

import { createSlice, createAsyncThunk } from '@reduxjs/toolkit';
import type { PayloadAction } from '@reduxjs/toolkit';
import type {
  ConversationMessage,
  ConversationMessageRequest,
  ConversationMessageResponse,
  ConversationConfirmRequest,
  ConversationConfirmResponse,
  ConversationCancelRequest,
  CreateSessionResponse,
  OrchestratorResponse,
} from '../types/conversation';

/**
 * Conversation slice state.
 */
interface ConversationState {
  /** Current session ID */
  sessionId: string | null;

  /** Message history for current session */
  messages: ConversationMessage[];

  /** Loading state during API calls */
  isLoading: boolean;

  /** Pending files awaiting confirmation */
  pendingFiles: Record<string, string> | null;

  /** Error message if any */
  error: string | null;

  /** Last orchestrator response */
  lastResponse: OrchestratorResponse | null;
}

/**
 * Initial state.
 */
const initialState: ConversationState = {
  sessionId: null,
  messages: [],
  isLoading: false,
  pendingFiles: null,
  error: null,
  lastResponse: null,
};

/**
 * Create a new conversation session.
 */
export const createSession = createAsyncThunk(
  'conversation/createSession',
  async () => {
    const response = await fetch('/api/conversation/session', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({}),
    });

    if (!response.ok) {
      throw new Error(`Failed to create session: ${response.statusText}`);
    }

    const data: CreateSessionResponse = await response.json();
    return data.session_id;
  }
);

/**
 * Send a message in the conversation.
 */
export const sendMessage = createAsyncThunk(
  'conversation/sendMessage',
  async (req: ConversationMessageRequest) => {
    const response = await fetch('/api/conversation/message', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      throw new Error(`Failed to send message: ${response.statusText}`);
    }

    const data: ConversationMessageResponse = await response.json();
    return data;
  }
);

/**
 * Confirm and persist generated files.
 */
export const confirmFiles = createAsyncThunk(
  'conversation/confirmFiles',
  async (req: ConversationConfirmRequest) => {
    const response = await fetch('/api/conversation/confirm', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      throw new Error(`Failed to confirm files: ${response.statusText}`);
    }

    const data: ConversationConfirmResponse = await response.json();
    return data;
  }
);

/**
 * Cancel pending file generation.
 */
export const cancelPending = createAsyncThunk(
  'conversation/cancelPending',
  async (req: ConversationCancelRequest) => {
    const response = await fetch('/api/conversation/cancel', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      throw new Error(`Failed to cancel: ${response.statusText}`);
    }
  }
);

/**
 * Conversation slice with reducers.
 */
const conversationSlice = createSlice({
  name: 'conversation',
  initialState,
  reducers: {
    /**
     * Clear conversation state.
     */
    clearConversation: (state) => {
      state.sessionId = null;
      state.messages = [];
      state.pendingFiles = null;
      state.error = null;
      state.lastResponse = null;
    },

    /**
     * Clear error message.
     */
    clearError: (state) => {
      state.error = null;
    },
  },
  extraReducers: (builder) => {
    // Create session
    builder.addCase(createSession.pending, (state) => {
      state.isLoading = true;
      state.error = null;
    });
    builder.addCase(createSession.fulfilled, (state, action) => {
      state.isLoading = false;
      state.sessionId = action.payload;
    });
    builder.addCase(createSession.rejected, (state, action) => {
      state.isLoading = false;
      state.error = action.error.message || 'Failed to create session';
    });

    // Send message
    builder.addCase(sendMessage.pending, (state) => {
      state.isLoading = true;
      state.error = null;
    });
    builder.addCase(sendMessage.fulfilled, (state, action) => {
      state.isLoading = false;
      state.messages = action.payload.messages;
      state.lastResponse = action.payload.response;

      // Set pending files if specialist returned files
      if (action.payload.response.type === 'specialist_result') {
        state.pendingFiles = action.payload.response.files;
      }
    });
    builder.addCase(sendMessage.rejected, (state, action) => {
      state.isLoading = false;
      state.error = action.error.message || 'Failed to send message';
    });

    // Confirm files
    builder.addCase(confirmFiles.pending, (state) => {
      state.isLoading = true;
      state.error = null;
    });
    builder.addCase(confirmFiles.fulfilled, (state) => {
      state.isLoading = false;
      state.pendingFiles = null;
      // Add success message to chat
      const successMessage: ConversationMessage = {
        role: 'assistant',
        content: 'Files successfully created! Your new agent is ready.',
        timestamp: new Date().toISOString(),
      };
      state.messages.push(successMessage);
    });
    builder.addCase(confirmFiles.rejected, (state, action) => {
      state.isLoading = false;
      state.error = action.error.message || 'Failed to confirm files';
    });

    // Cancel pending
    builder.addCase(cancelPending.pending, (state) => {
      state.isLoading = true;
      state.error = null;
    });
    builder.addCase(cancelPending.fulfilled, (state) => {
      state.isLoading = false;
      state.pendingFiles = null;
      // Add cancellation message
      const cancelMessage: ConversationMessage = {
        role: 'assistant',
        content: 'File generation cancelled. How else can I help?',
        timestamp: new Date().toISOString(),
      };
      state.messages.push(cancelMessage);
    });
    builder.addCase(cancelPending.rejected, (state, action) => {
      state.isLoading = false;
      state.error = action.error.message || 'Failed to cancel';
    });
  },
});

export const { clearConversation, clearError } = conversationSlice.actions;

export default conversationSlice.reducer;
