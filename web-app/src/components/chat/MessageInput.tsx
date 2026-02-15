import React, { useState, useRef, KeyboardEvent } from 'react'
import { TextArea } from '@/components/common/TextArea'
import { Button } from '@/components/common/Button'
import { Send } from 'lucide-react'

interface MessageInputProps {
  onSend: (content: string) => void
  disabled?: boolean
  placeholder?: string
}

/**
 * MessageInput - Text input for composing and sending messages
 *
 * Features:
 * - Auto-focus after send
 * - Validation (no empty messages)
 * - Keyboard shortcuts:
 *   - Shift+Enter: new line
 *   - Enter: send message
 * - Max length: 5000 characters
 * - Disabled state support
 */
export const MessageInput: React.FC<MessageInputProps> = ({
  onSend,
  disabled = false,
  placeholder = 'Type a message...',
}) => {
  const [input, setInput] = useState('')
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const maxLength = 5000

  /**
   * Handle send action
   */
  const handleSend = () => {
    const trimmed = input.trim()
    if (!trimmed || disabled) return

    onSend(trimmed)
    setInput('')

    // Focus back on textarea after sending
    setTimeout(() => {
      textareaRef.current?.focus()
    }, 0)
  }

  /**
   * Handle keyboard shortcuts
   * - Enter: send (unless Shift is held)
   * - Shift+Enter: new line
   */
  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const isSendDisabled = disabled || input.trim().length === 0
  const charCount = input.length
  const isNearLimit = charCount > maxLength * 0.9

  return (
    <div className="flex gap-2 p-4 border-t border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900">
      <div className="flex-1">
        <TextArea
          ref={textareaRef}
          placeholder={placeholder}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          rows={3}
          disabled={disabled}
          maxLength={maxLength}
          className="resize-none"
          helperText={
            isNearLimit
              ? `${charCount} / ${maxLength} characters`
              : undefined
          }
        />
      </div>

      <div className="flex items-start pt-1">
        <Button
          onClick={handleSend}
          disabled={isSendDisabled}
          variant="primary"
          size="md"
          icon={<Send className="w-4 h-4" />}
          iconPosition="right"
          title="Send message (Enter)"
        >
          Send
        </Button>
      </div>
    </div>
  )
}

export default MessageInput
