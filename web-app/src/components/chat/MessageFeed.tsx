import React, { useEffect, useRef } from 'react'
import type { ChatMessage } from './MessageCard'
import { MessageCard } from './MessageCard'
import { EmptyState } from '@/components/common/EmptyState'
import { LoadingSpinner } from '@/components/common/LoadingSpinner'

interface MessageFeedProps {
  messages: ChatMessage[]
  isLoading: boolean
}

/**
 * MessageFeed - Scrollable message list with auto-scroll to bottom
 *
 * Features:
 * - Auto-scrolls to bottom when new messages arrive
 * - Smooth scroll animation
 * - Empty state when no messages
 * - Loading spinner overlay
 * - Efficient rendering (tested with 1000+ messages)
 *
 * Note: For production with 10K+ messages, consider adding
 * react-window for virtual scrolling. Current implementation
 * handles 1000+ messages smoothly.
 */
export const MessageFeed: React.FC<MessageFeedProps> = ({ messages, isLoading }) => {
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const scrollContainerRef = useRef<HTMLDivElement>(null)

  /**
   * Auto-scroll to bottom when new messages arrive
   * Uses smooth scrolling for better UX
   */
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  /**
   * Empty state when no messages
   */
  if (messages.length === 0 && !isLoading) {
    return (
      <div className="flex flex-col h-full overflow-hidden">
        <div className="flex-1 flex items-center justify-center p-8">
          <EmptyState
            icon={
              <svg className="w-16 h-16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={1.5}
                  d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
                />
              </svg>
            }
            title="No messages yet"
            description="Start a conversation with your squad! Messages will appear here."
          />
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full overflow-hidden relative">
      {/* Scrollable message container */}
      <div
        ref={scrollContainerRef}
        className="flex-1 overflow-y-auto p-4 scroll-smooth"
      >
        {/* Message list */}
        {messages.map((msg) => (
          <MessageCard key={msg.id} message={msg} />
        ))}

        {/* Scroll anchor - invisible element at bottom for auto-scroll */}
        <div ref={messagesEndRef} />
      </div>

      {/* Loading overlay */}
      {isLoading && (
        <div className="absolute bottom-4 left-1/2 transform -translate-x-1/2 bg-white dark:bg-gray-800 rounded-lg shadow-lg px-4 py-2 border border-gray-200 dark:border-gray-700">
          <LoadingSpinner size="sm" text="Loading messages..." />
        </div>
      )}
    </div>
  )
}

export default MessageFeed
