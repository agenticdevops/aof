/**
 * Hook for managing conversational agent creation.
 * Provides clean API for components to interact with conversation state.
 */

import { useCallback } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import type { AppDispatch, RootState } from '../store';
import {
  createSession,
  sendMessage as sendMessageThunk,
  confirmFiles as confirmFilesThunk,
  cancelPending as cancelPendingThunk,
  clearConversation,
  clearError,
} from '../store/conversationSlice';

/**
 * Conversation management hook.
 */
export function useConversation() {
  const dispatch = useDispatch<AppDispatch>();
  const {
    sessionId,
    messages,
    isLoading,
    pendingFiles,
    error,
    lastResponse,
  } = useSelector((state: RootState) => state.conversation);

  /**
   * Start a new conversation session.
   */
  const startSession = useCallback(async () => {
    await dispatch(createSession());
  }, [dispatch]);

  /**
   * Send a message in the conversation.
   * Auto-creates session if none exists.
   */
  const sendMessage = useCallback(
    async (text: string) => {
      // Create session if needed
      let currentSessionId = sessionId;
      if (!currentSessionId) {
        const result = await dispatch(createSession());
        if (createSession.fulfilled.match(result)) {
          currentSessionId = result.payload;
        } else {
          return;
        }
      }

      // Send message
      await dispatch(
        sendMessageThunk({
          session_id: currentSessionId,
          message: text,
        })
      );
    },
    [dispatch, sessionId]
  );

  /**
   * Confirm and persist pending files.
   */
  const confirm = useCallback(async () => {
    if (!sessionId) return;
    await dispatch(confirmFilesThunk({ session_id: sessionId }));
  }, [dispatch, sessionId]);

  /**
   * Cancel pending file generation.
   */
  const cancel = useCallback(async () => {
    if (!sessionId) return;
    await dispatch(cancelPendingThunk({ session_id: sessionId }));
  }, [dispatch, sessionId]);

  /**
   * Clear conversation state.
   */
  const clear = useCallback(() => {
    dispatch(clearConversation());
  }, [dispatch]);

  /**
   * Clear error message.
   */
  const dismissError = useCallback(() => {
    dispatch(clearError());
  }, [dispatch]);

  return {
    // State
    sessionId,
    messages,
    isLoading,
    pendingFiles,
    error,
    lastResponse,

    // Actions
    startSession,
    sendMessage,
    confirm,
    cancel,
    clear,
    dismissError,
  };
}
