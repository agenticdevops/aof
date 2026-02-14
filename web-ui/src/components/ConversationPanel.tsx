/**
 * ConversationPanel - Main chat UI for conversational agent creation.
 * Shows message history, typing indicator, and handles user input.
 */

import { useEffect, useRef } from 'react';
import { useConversation } from '../hooks/useConversation';
import { ChatInput } from './ChatInput';
import { MESSAGE_ROLES } from '../types/conversation';
import type { ConversationMessage } from '../types/conversation';

/**
 * Main conversation panel component.
 */
export function ConversationPanel() {
  const { messages, isLoading, error, sendMessage, dismissError } = useConversation();
  const messagesEndRef = useRef<HTMLDivElement>(null);

  /**
   * Auto-scroll to bottom on new messages.
   */
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  /**
   * Welcome message when conversation is empty.
   */
  const welcomeMessage = (
    <div className="flex-1 flex items-center justify-center p-8">
      <div className="max-w-2xl text-center">
        <h2 className="text-2xl font-bold text-white mb-4">
          Create Agents Conversationally
        </h2>
        <p className="text-gray-400 mb-6">
          I can help you create agents, build squads, configure schedules, and teach skills.
        </p>
        <div className="text-left bg-gray-800 rounded-lg p-4 space-y-2">
          <p className="text-sm text-gray-300 font-semibold mb-2">Try these examples:</p>
          <div className="space-y-1 text-sm text-gray-400">
            <p>"I need a K8s monitoring agent"</p>
            <p>"Create a squad for incident response"</p>
            <p>"Schedule the backup agent to run daily at 2am"</p>
            <p>"Teach the agent how to restart services"</p>
          </div>
        </div>
      </div>
    </div>
  );

  return (
    <div className="flex flex-col h-full bg-gray-900">
      {/* Error banner */}
      {error && (
        <div className="bg-red-600 text-white px-4 py-3 flex justify-between items-center">
          <span>{error}</span>
          <button
            onClick={dismissError}
            className="text-white hover:text-gray-200 font-bold"
          >
            ×
          </button>
        </div>
      )}

      {/* Message history */}
      <div className="flex-1 overflow-y-auto">
        {messages.length === 0 ? (
          welcomeMessage
        ) : (
          <div className="max-w-4xl mx-auto py-4">
            {messages.map((msg, idx) => (
              <MessageBubble key={idx} message={msg} />
            ))}
            {isLoading && <TypingIndicator />}
            <div ref={messagesEndRef} />
          </div>
        )}
      </div>

      {/* Input */}
      <ChatInput onSend={sendMessage} disabled={isLoading} />
    </div>
  );
}

/**
 * Message bubble component.
 */
function MessageBubble({ message }: { message: ConversationMessage }) {
  const isUser = message.role === MESSAGE_ROLES.USER;

  return (
    <div className={`flex ${isUser ? 'justify-end' : 'justify-start'} mb-4 px-4`}>
      <div
        className={`max-w-3xl rounded-lg px-4 py-3 ${
          isUser
            ? 'bg-blue-600 text-white'
            : 'bg-gray-700 text-gray-100'
        }`}
      >
        <div className="text-sm whitespace-pre-wrap break-words">
          {message.content}
        </div>
        {message.filePreview && (
          <div className="mt-3 pt-3 border-t border-gray-600">
            <p className="text-xs text-gray-300 mb-2">{message.filePreview.summary}</p>
            <div className="space-y-1">
              {Object.keys(message.filePreview.files).map((path) => (
                <div key={path} className="text-xs text-gray-400">
                  📄 {path}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Typing indicator (animated dots).
 */
function TypingIndicator() {
  return (
    <div className="flex justify-start mb-4 px-4">
      <div className="bg-gray-700 rounded-lg px-4 py-3">
        <div className="flex gap-1">
          <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
          <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
          <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
        </div>
      </div>
    </div>
  );
}
