/**
 * Hook for managing chat messages with WebSocket sync.
 */

import { useEffect, useCallback } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { addMessage, setMessages, setLoading, setError, selectLastMessageId } from '../store/chatSlice';
import type { ChatMessage } from '../types/chat';
import type { AppDispatch, RootState } from '../store';

/**
 * Hook for chat message management.
 */
export function useChatMessages() {
  const dispatch = useDispatch<AppDispatch>();
  const lastMessageId = useSelector(selectLastMessageId);
  const connected = useSelector((state: RootState) => state.events.connected);
  const messages = useSelector((state: RootState) => state.chat.messages);

  /**
   * Fetch initial chat history.
   */
  const fetchHistory = useCallback(async () => {
    try {
      dispatch(setLoading(true));
      const response = await fetch('/api/chat/messages');
      if (!response.ok) {
        throw new Error(`Failed to fetch messages: ${response.statusText}`);
      }
      const data: ChatMessage[] = await response.json();
      dispatch(setMessages(data));
    } catch (err) {
      dispatch(setError(err as Error));
    }
  }, [dispatch]);

  /**
   * Fetch messages since last message ID (for reconnection recovery).
   */
  const fetchSince = useCallback(
    async (sinceId: string) => {
      try {
        const response = await fetch(`/api/chat/messages?since=${sinceId}`);
        if (!response.ok) {
          throw new Error(`Failed to fetch messages: ${response.statusText}`);
        }
        const data: ChatMessage[] = await response.json();
        // Add each message (dedup will happen in reducer)
        data.forEach((msg) => dispatch(addMessage(msg)));
      } catch (err) {
        console.error('Failed to fetch messages since reconnect:', err);
      }
    },
    [dispatch]
  );

  /**
   * Send a new message.
   */
  const sendMessage = useCallback(
    async (content: string, senderId: string, senderName: string, senderAvatar?: string) => {
      const optimisticMessage: ChatMessage = {
        id: `temp_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`,
        senderId,
        senderName,
        senderAvatar,
        content,
        timestamp: new Date().toISOString(),
      };

      // Optimistic update
      dispatch(addMessage(optimisticMessage));

      try {
        const response = await fetch('/api/chat/messages', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            content,
            senderId,
            senderName,
            senderAvatar,
          }),
        });

        if (!response.ok) {
          throw new Error(`Failed to send message: ${response.statusText}`);
        }

        const confirmedMessage: ChatMessage = await response.json();
        // Replace optimistic message with server-confirmed message
        dispatch(addMessage(confirmedMessage));
      } catch (err) {
        dispatch(setError(err as Error));
        // TODO: Implement rollback for failed messages
      }
    },
    [dispatch]
  );

  /**
   * Fetch history on mount.
   */
  useEffect(() => {
    fetchHistory();
  }, [fetchHistory]);

  /**
   * On reconnect, fetch messages sent during disconnect.
   */
  useEffect(() => {
    if (connected && lastMessageId && messages.length > 0) {
      fetchSince(lastMessageId);
    }
  }, [connected, lastMessageId, messages.length, fetchSince]);

  return {
    sendMessage,
    fetchHistory,
  };
}
