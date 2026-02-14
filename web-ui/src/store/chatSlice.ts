/**
 * Redux slice for chat messages.
 * Manages squad chat state with message deduplication and WebSocket sync.
 */

import { createSlice, type PayloadAction } from '@reduxjs/toolkit';
import type { ChatMessage } from '../types/chat';
import type { RootState } from './index';

/**
 * Chat slice state structure.
 */
interface ChatState {
  /** All chat messages (deduped by ID) */
  messages: ChatMessage[];

  /** Currently selected agent ID for filtering */
  selectedAgentId: string | null;

  /** Loading state */
  loading: boolean;

  /** Error message (if any) */
  error: Error | null;

  /** Unread message count */
  unreadCount: number;

  /** Last message ID received (for reconnection recovery) */
  lastMessageId: string;
}

/**
 * Initial chat slice state.
 */
const initialState: ChatState = {
  messages: [],
  selectedAgentId: null,
  loading: false,
  error: null,
  unreadCount: 0,
  lastMessageId: '',
};

/**
 * Chat slice - manages chat message state with deduplication.
 */
const chatSlice = createSlice({
  name: 'chat',
  initialState,
  reducers: {
    /**
     * Add single message with deduplication.
     * If message.id already exists, skip append.
     */
    addMessage: (state, action: PayloadAction<ChatMessage>) => {
      const message = action.payload;

      // Deduplication: check if message ID already exists
      const exists = state.messages.some((m) => m.id === message.id);
      if (exists) {
        // If this is a temp ID being replaced by real ID, update the message
        const tempIndex = state.messages.findIndex(
          (m) => m.id.startsWith('temp_') && m.content === message.content
        );
        if (tempIndex !== -1 && !message.id.startsWith('temp_')) {
          // Replace temp message with server-confirmed message
          state.messages[tempIndex] = message;
          state.lastMessageId = message.id;
        }
        return;
      }

      // Append new message
      state.messages.push(message);
      state.lastMessageId = message.id;

      // Increment unread count (UI will handle marking as read)
      state.unreadCount += 1;
    },

    /**
     * Set all messages (batch load from API).
     */
    setMessages: (state, action: PayloadAction<ChatMessage[]>) => {
      state.messages = action.payload;
      state.loading = false;
      state.error = null;

      // Update last message ID
      if (action.payload.length > 0) {
        state.lastMessageId = action.payload[action.payload.length - 1].id;
      }
    },

    /**
     * Clear all messages.
     */
    clearMessages: (state) => {
      state.messages = [];
      state.lastMessageId = '';
      state.unreadCount = 0;
    },

    /**
     * Mark messages as read (reset unread count).
     */
    markAsRead: (state) => {
      state.unreadCount = 0;
    },

    /**
     * Select agent for filtering.
     */
    selectAgent: (state, action: PayloadAction<string | null>) => {
      state.selectedAgentId = action.payload;
    },

    /**
     * Set loading state.
     */
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.loading = action.payload;
    },

    /**
     * Set error state.
     */
    setError: (state, action: PayloadAction<Error | null>) => {
      state.error = action.payload;
    },
  },
});

/**
 * Actions.
 */
export const {
  addMessage,
  setMessages,
  clearMessages,
  markAsRead,
  selectAgent,
  setLoading,
  setError,
} = chatSlice.actions;

/**
 * Selectors.
 */

/**
 * Select all messages.
 */
export const selectAllMessages = (state: RootState): ChatMessage[] => state.chat.messages;

/**
 * Select messages by agent (filter by senderId).
 */
export const selectMessagesByAgent = (agentId: string) => (state: RootState): ChatMessage[] =>
  state.chat.messages.filter((m) => m.senderId === agentId);

/**
 * Select unread messages (all messages after last read, simplified for now).
 */
export const selectUnreadMessages = (state: RootState): ChatMessage[] => {
  // For now, return all messages (future: track read position)
  return state.chat.messages.slice(-state.chat.unreadCount);
};

/**
 * Select messages since timestamp.
 */
export const selectMessagesSince = (timestamp: string) => (state: RootState): ChatMessage[] =>
  state.chat.messages.filter((m) => m.timestamp > timestamp);

/**
 * Select unread count.
 */
export const selectUnreadCount = (state: RootState): number => state.chat.unreadCount;

/**
 * Select last message ID.
 */
export const selectLastMessageId = (state: RootState): string => state.chat.lastMessageId;

/**
 * Select loading state.
 */
export const selectChatLoading = (state: RootState): boolean => state.chat.loading;

/**
 * Select error state.
 */
export const selectChatError = (state: RootState): Error | null => state.chat.error;

/**
 * Select currently selected agent ID.
 */
export const selectSelectedAgentId = (state: RootState): string | null =>
  state.chat.selectedAgentId;

/**
 * Default export.
 */
export default chatSlice.reducer;
