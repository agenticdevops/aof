/**
 * Chat input component for conversational agent creation.
 * Text input with send button, Enter key support.
 */

import { useState, useCallback, KeyboardEvent } from 'react';

interface ChatInputProps {
  /** Callback when message is sent */
  onSend: (message: string) => void;

  /** Disabled while loading */
  disabled?: boolean;

  /** Placeholder text */
  placeholder?: string;
}

/**
 * Chat input with send button.
 */
export function ChatInput({
  onSend,
  disabled = false,
  placeholder = 'Describe an agent, squad, schedule, or skill...',
}: ChatInputProps) {
  const [value, setValue] = useState('');

  /**
   * Handle send action.
   */
  const handleSend = useCallback(() => {
    const trimmed = value.trim();
    if (!trimmed) return;

    onSend(trimmed);
    setValue('');
  }, [value, onSend]);

  /**
   * Handle Enter key (Shift+Enter for newline).
   */
  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  return (
    <div className="flex gap-2 p-4 bg-gray-800 border-t border-gray-700">
      <textarea
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={handleKeyDown}
        disabled={disabled}
        placeholder={placeholder}
        className="flex-1 px-4 py-2 bg-gray-700 text-white rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
        rows={1}
        style={{
          minHeight: '44px',
          maxHeight: '120px',
        }}
      />
      <button
        onClick={handleSend}
        disabled={disabled || !value.trim()}
        className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        Send
      </button>
    </div>
  );
}
