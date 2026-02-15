import React from 'react'
import type { Message } from '@/types/chat'

interface MessageCardProps {
  message: Message
}

/**
 * Format timestamp to relative time (e.g., "5 min ago", "2h ago")
 */
const formatRelativeTime = (date: Date): string => {
  const now = new Date()
  const diff = now.getTime() - new Date(date).getTime()
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (seconds < 60) return 'just now'
  if (minutes < 60) return `${minutes} min ago`
  if (hours < 24) return `${hours}h ago`
  if (days < 7) return `${days}d ago`

  // For older messages, show date
  return new Date(date).toLocaleDateString()
}

/**
 * MessageCard - Individual message with persona styling
 *
 * Renders messages with different styling based on type:
 * - Agent messages: persona-colored with avatar
 * - User messages: gray with user avatar
 * - System messages: yellow/centered/italic
 * - Announcements: green/bold with broadcast icon
 */
export const MessageCard: React.FC<MessageCardProps> = ({ message }) => {
  const { type, sender, content, personaColor, personaIcon, timestamp } = message

  // System message styling (centered, italic, no avatar)
  if (type === 'system') {
    return (
      <div className="my-2 mx-0 py-3 px-4 border-t border-b border-yellow-300 dark:border-yellow-700 bg-yellow-50 dark:bg-yellow-900/30">
        <p className="text-sm text-gray-700 dark:text-gray-300 text-center italic">
          {content}
        </p>
        <p className="text-xs text-gray-500 dark:text-gray-400 text-center mt-1">
          {formatRelativeTime(timestamp)}
        </p>
      </div>
    )
  }

  // Announcement message styling (green, bold, broadcast icon)
  if (type === 'announcement') {
    return (
      <div className="my-2 mx-0 py-4 px-4 border-t-2 border-b-2 border-green-500 dark:border-green-600 bg-green-50 dark:bg-green-900/30 rounded-md hover:shadow-md transition-shadow">
        <div className="flex gap-3">
          <div className="flex-shrink-0 w-10 h-10 rounded-full bg-green-500 dark:bg-green-600 flex items-center justify-center text-lg">
            📢
          </div>
          <div className="flex-1">
            <div className="flex items-baseline gap-2 mb-1">
              <span className="font-semibold text-green-900 dark:text-green-100">
                Announcement from {sender.name}
              </span>
              <span className="text-xs text-gray-500 dark:text-gray-400">
                {formatRelativeTime(timestamp)}
              </span>
            </div>
            <p className="text-green-900 dark:text-green-100 font-semibold">
              {content}
            </p>
          </div>
        </div>
      </div>
    )
  }

  // Agent message styling (persona-colored, left border)
  if (type === 'agent') {
    return (
      <div
        className="my-2 mx-0 p-4 rounded-md bg-blue-50 dark:bg-blue-900/20 hover:shadow-md transition-shadow"
        style={{ borderLeft: `3px solid ${personaColor}` }}
      >
        <div className="flex gap-3">
          {/* Avatar with persona color */}
          <div
            className="flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center text-lg"
            style={{ backgroundColor: personaColor }}
          >
            {personaIcon}
          </div>

          {/* Message content */}
          <div className="flex-1">
            <div className="flex items-baseline gap-2 mb-1">
              <span className="font-bold text-gray-900 dark:text-white">
                {sender.name}
              </span>
              <span className="text-sm text-gray-600 dark:text-gray-400">
                {sender.role}
              </span>
              <span className="text-xs text-gray-500 dark:text-gray-400 ml-auto">
                {formatRelativeTime(timestamp)}
              </span>
            </div>
            <p className="text-gray-900 dark:text-white whitespace-pre-wrap">
              {content}
            </p>
          </div>
        </div>
      </div>
    )
  }

  // User message styling (gray, regular weight)
  return (
    <div
      className="my-2 mx-0 p-4 rounded-md bg-gray-100 dark:bg-gray-800 hover:shadow-md transition-shadow"
      style={{ borderLeft: '3px solid #9CA3AF' }}
    >
      <div className="flex gap-3">
        {/* User avatar */}
        <div
          className="flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center text-lg"
          style={{ backgroundColor: personaColor }}
        >
          {personaIcon}
        </div>

        {/* Message content */}
        <div className="flex-1">
          <div className="flex items-baseline gap-2 mb-1">
            <span className="font-normal text-gray-900 dark:text-white">
              {sender.name}
            </span>
            <span className="text-sm text-gray-600 dark:text-gray-400">
              {sender.role}
            </span>
            <span className="text-xs text-gray-500 dark:text-gray-400 ml-auto">
              {formatRelativeTime(timestamp)}
            </span>
          </div>
          <p className="text-gray-900 dark:text-white whitespace-pre-wrap">
            {content}
          </p>
        </div>
      </div>
    </div>
  )
}

export default MessageCard
