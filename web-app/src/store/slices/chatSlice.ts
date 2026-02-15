import { createSlice, PayloadAction, createSelector } from '@reduxjs/toolkit'
import type { ChatState, Message, SquadMember } from '@/types/chat'
import type { RootState } from '../store'
import { useAppSelector } from '../hooks'

/**
 * Initial chat state
 */
const initialState: ChatState = {
  messages: [],
  squadMembers: [],
  searchQuery: '',
  isLoading: false,
  error: null,
  typingAgentId: null,
}

/**
 * Chat slice - manages Squad Chat state
 *
 * Note: WebSocket integration will be added in Plan 05.
 * This slice currently manages local state and mock data.
 */
export const chatSlice = createSlice({
  name: 'chat',
  initialState,
  reducers: {
    /**
     * Add a new message to the chat feed
     */
    addMessage: (state, action: PayloadAction<Message>) => {
      state.messages.push(action.payload)
      state.error = null
    },

    /**
     * Replace the entire messages array
     */
    setMessages: (state, action: PayloadAction<Message[]>) => {
      state.messages = action.payload
      state.error = null
    },

    /**
     * Set the squad member list
     */
    setSquadMembers: (state, action: PayloadAction<SquadMember[]>) => {
      state.squadMembers = action.payload
      state.error = null
    },

    /**
     * Update the search query for message filtering
     */
    setSearchQuery: (state, action: PayloadAction<string>) => {
      state.searchQuery = action.payload
    },

    /**
     * Set loading state
     */
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload
    },

    /**
     * Set error state
     */
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
      state.isLoading = false
    },

    /**
     * Update a squad member's online status
     */
    updateMemberStatus: (
      state,
      action: PayloadAction<{ id: string; isOnline: boolean }>
    ) => {
      const member = state.squadMembers.find(m => m.id === action.payload.id)
      if (member) {
        member.isOnline = action.payload.isOnline
      }
    },

    /**
     * Mark all messages as read
     */
    markAllAsRead: (state) => {
      state.messages.forEach(msg => {
        msg.isRead = true
      })
    },

    /**
     * Clear all messages and reset chat state
     */
    clearChat: (state) => {
      state.messages = []
      state.error = null
      state.isLoading = false
    },

    /**
     * Add a squad member (for WebSocket events)
     */
    addSquadMember: (state, action: PayloadAction<SquadMember>) => {
      const exists = state.squadMembers.some(m => m.id === action.payload.id)
      if (!exists) {
        state.squadMembers.push(action.payload)
      }
    },

    /**
     * Remove a squad member by ID (for WebSocket events)
     */
    removeSquadMember: (state, action: PayloadAction<string>) => {
      state.squadMembers = state.squadMembers.filter(m => m.id !== action.payload)
    },

    /**
     * Set the typing agent ID (for typing indicator)
     */
    setTypingAgent: (state, action: PayloadAction<string | null>) => {
      state.typingAgentId = action.payload
    },
  },
})

// Export actions
export const {
  addMessage,
  setMessages,
  setSquadMembers,
  setSearchQuery,
  setLoading,
  setError,
  updateMemberStatus,
  markAllAsRead,
  clearChat,
  addSquadMember,
  removeSquadMember,
  setTypingAgent,
} = chatSlice.actions

// Export reducer
export default chatSlice.reducer

// Selectors
export const selectMessages = (state: RootState) => state.chat.messages
export const selectSquadMembers = (state: RootState) => state.chat.squadMembers
export const selectSearchQuery = (state: RootState) => state.chat.searchQuery
export const selectChatLoading = (state: RootState) => state.chat.isLoading
export const selectChatError = (state: RootState) => state.chat.error

/**
 * Memoized selector for filtered messages based on search query
 * Performs case-insensitive substring matching on message content
 */
export const selectFilteredMessages = createSelector(
  [selectMessages, selectSearchQuery],
  (messages, searchQuery) => {
    if (!searchQuery.trim()) {
      return messages
    }

    const query = searchQuery.toLowerCase()
    return messages.filter(msg =>
      msg.content.toLowerCase().includes(query)
    )
  }
)

/**
 * Memoized selector for online squad members
 */
export const selectOnlineMembers = createSelector(
  [selectSquadMembers],
  (members) => members.filter(m => m.isOnline)
)

/**
 * Memoized selector for offline squad members
 */
export const selectOfflineMembers = createSelector(
  [selectSquadMembers],
  (members) => members.filter(m => !m.isOnline)
)

/**
 * Custom hooks for chat state
 */

/**
 * Get all chat state
 */
export const useChat = () => {
  return useAppSelector((state) => state.chat)
}

/**
 * Get chat messages array
 */
export const useChatMessages = () => {
  return useAppSelector(selectMessages)
}

/**
 * Get squad members array
 */
export const useSquadMembers = () => {
  return useAppSelector(selectSquadMembers)
}

/**
 * Get filtered messages based on search query
 */
export const useFilteredMessages = () => {
  return useAppSelector(selectFilteredMessages)
}

/**
 * Get online squad members
 */
export const useOnlineMembers = () => {
  return useAppSelector(selectOnlineMembers)
}

/**
 * Get search query
 */
export const useSearchQuery = () => {
  return useAppSelector(selectSearchQuery)
}
