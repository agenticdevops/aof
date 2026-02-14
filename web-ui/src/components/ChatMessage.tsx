/**
 * ChatMessage component - displays a single chat message with sender info.
 */

import React from 'react';
import ReactMarkdown from 'react-markdown';
import type { ChatMessage as ChatMessageType } from '../types/chat';

/**
 * Props for ChatMessage component.
 */
interface ChatMessageProps {
  /** Message object */
  message: ChatMessageType;

  /** Whether markdown rendering is enabled */
  enableMarkdown?: boolean;

  /** Whether this is the current user's message (for styling) */
  isOwnMessage?: boolean;
}

/**
 * Format relative time from timestamp.
 */
function formatRelativeTime(timestamp: string): string {
  const now = new Date().getTime();
  const then = new Date(timestamp).getTime();
  const diffMs = now - then;
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) return 'Just now';
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHour < 24) return `${diffHour}h ago`;
  return `${diffDay}d ago`;
}

/**
 * ChatMessage component.
 */
export function ChatMessage({
  message,
  enableMarkdown = true,
  isOwnMessage = false,
}: ChatMessageProps): React.ReactElement {
  const relativeTime = formatRelativeTime(message.timestamp);

  return (
    <div
      className={`flex gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors ${
        isOwnMessage ? 'bg-blue-50 dark:bg-blue-900/20' : ''
      }`}
    >
      {/* Avatar */}
      <div className="flex-shrink-0">
        {message.senderAvatar ? (
          <div className="w-8 h-8 rounded-full bg-gray-200 dark:bg-gray-700 flex items-center justify-center text-lg">
            {message.senderAvatar}
          </div>
        ) : (
          <div className="w-8 h-8 rounded-full bg-gray-300 dark:bg-gray-600 flex items-center justify-center text-xs font-medium text-gray-700 dark:text-gray-300">
            {message.senderName.substring(0, 2).toUpperCase()}
          </div>
        )}
      </div>

      {/* Message Content */}
      <div className="flex-1 min-w-0">
        {/* Header: Sender name + timestamp */}
        <div className="flex items-baseline gap-2 mb-1">
          <span className="text-sm font-semibold text-gray-900 dark:text-white">
            {message.senderName}
          </span>
          <time className="text-xs text-gray-500 dark:text-gray-400" dateTime={message.timestamp}>
            {relativeTime}
          </time>
        </div>

        {/* Message content */}
        <div className="text-sm text-gray-700 dark:text-gray-300 break-words">
          {enableMarkdown ? (
            <ReactMarkdown
              components={{
                // Customize markdown rendering for safety
                a: ({ node, ...props }) => (
                  <a
                    {...props}
                    className="text-blue-600 dark:text-blue-400 hover:underline"
                    target="_blank"
                    rel="noopener noreferrer"
                  />
                ),
                code: ({ node, inline, ...props }) =>
                  inline ? (
                    <code
                      {...props}
                      className="bg-gray-100 dark:bg-gray-800 px-1 py-0.5 rounded text-xs font-mono"
                    />
                  ) : (
                    <code
                      {...props}
                      className="block bg-gray-100 dark:bg-gray-800 p-2 rounded text-xs font-mono overflow-x-auto"
                    />
                  ),
                p: ({ node, ...props }) => <p {...props} className="mb-1" />,
              }}
            >
              {message.content}
            </ReactMarkdown>
          ) : (
            <p>{message.content}</p>
          )}
        </div>
      </div>
    </div>
  );
}
