/**
 * SquadChat component - real-time chat panel with message history and input.
 */

import React, { useState, useRef, useEffect } from 'react';
import { useSelector } from 'react-redux';
import { ChatMessage } from './ChatMessage';
import { useChatMessages } from '../hooks/useChatMessages';
import { selectAllMessages, selectChatLoading } from '../store/chatSlice';
import type { RootState } from '../store';

/**
 * SquadChat component.
 */
export function SquadChat(): React.ReactElement {
  const messages = useSelector(selectAllMessages);
  const loading = useSelector(selectChatLoading);
  const connected = useSelector((state: RootState) => state.events.connected);
  const { sendMessage } = useChatMessages();

  const [inputValue, setInputValue] = useState('');
  const [sending, setSending] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  /**
   * Auto-scroll to bottom when new messages arrive.
   */
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages.length]);

  /**
   * Handle send message.
   */
  const handleSend = async () => {
    if (!inputValue.trim() || sending) return;

    setSending(true);
    try {
      // TODO: Get actual user ID and name (for now use placeholder)
      await sendMessage(inputValue, 'user_1', 'You', '👤');
      setInputValue('');
      inputRef.current?.focus();
    } finally {
      setSending(false);
    }
  };

  /**
   * Handle Enter key to send.
   */
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex flex-col h-full bg-white dark:bg-gray-800 border-l border-gray-200 dark:border-gray-700">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Squad Chat</h3>
        <div className="flex items-center gap-2">
          {/* Connection indicator */}
          <div
            className={`w-2 h-2 rounded-full ${
              connected ? 'bg-green-500' : 'bg-red-500'
            }`}
            title={connected ? 'Connected' : 'Disconnected'}
          />
          <span className="text-xs text-gray-500 dark:text-gray-400">
            {connected ? 'Live' : 'Offline'}
          </span>
        </div>
      </div>

      {/* Message History */}
      <div className="flex-1 overflow-y-auto" style={{ height: '400px' }}>
        {loading && messages.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-sm text-gray-500 dark:text-gray-400">Loading messages...</div>
          </div>
        ) : messages.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-sm text-gray-500 dark:text-gray-400">
              No messages yet. Start the conversation!
            </div>
          </div>
        ) : (
          <>
            {messages.map((message) => (
              <ChatMessage
                key={message.id}
                message={message}
                enableMarkdown={true}
                isOwnMessage={message.senderId === 'user_1'}
              />
            ))}
            <div ref={messagesEndRef} />
          </>
        )}
      </div>

      {/* Message Input */}
      <div className="border-t border-gray-200 dark:border-gray-700 p-4">
        <div className="flex gap-2">
          <input
            ref={inputRef}
            type="text"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Type a message..."
            disabled={sending || !connected}
            className="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white placeholder-gray-400 dark:placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
            aria-label="Message input"
          />
          <button
            onClick={handleSend}
            disabled={!inputValue.trim() || sending || !connected}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            aria-label="Send message"
          >
            {sending ? 'Sending...' : 'Send'}
          </button>
        </div>
        {!connected && (
          <p className="mt-2 text-xs text-yellow-600 dark:text-yellow-400">
            Chat is offline. Reconnecting...
          </p>
        )}
      </div>
    </div>
  );
}
