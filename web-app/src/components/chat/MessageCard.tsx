/**
 * MessageCard Component
 *
 * Displays chat messages with persona-based styling for agent messages.
 * Agent messages use persona colors; user messages use neutral gray.
 */

import React from 'react'
import AgentAvatar from '@/components/common/AgentAvatar'
import type { PersonaType } from '@/types/personas'
import { mapAgentTypeToPersonaType } from '@/types/personas'
import { getPersonaColors, getPersonaFont, getTextColorOnPersona } from '@/utils/personaStyles'
import { useAppSelector } from '@/store/hooks'
import { selectIsDarkMode } from '@/store/slices/appSlice'

/**
 * Message type
 */
export interface ChatMessage {
  /** Message ID */
  id: string
  /** Message content */
  content: string
  /** Sender type */
  senderType: 'agent' | 'user'
  /** Sender name */
  senderName: string
  /** Sender persona type (for agents) */
  senderPersonaType?: PersonaType
  /** Sender icon (emoji or URL) */
  senderIcon?: string
  /** Timestamp */
  timestamp: Date
}

export interface MessageCardProps {
  /** Message data */
  message: ChatMessage
  /** Optional className */
  className?: string
}

/**
 * Chat message card with persona styling
 * Agent messages styled with persona colors, user messages neutral
 */
export const MessageCard: React.FC<MessageCardProps> = ({ message, className = '' }) => {
  const isDarkMode = useAppSelector(selectIsDarkMode)

  const isAgent = message.senderType === 'agent'
  const personaType = isAgent && message.senderPersonaType
    ? message.senderPersonaType
    : 'custom'

  // Get persona styling for agent messages
  const colors = isAgent ? getPersonaColors(personaType, isDarkMode) : null
  const fontClass = isAgent ? getPersonaFont(personaType) : 'font-normal'
  const textColor = isAgent ? getTextColorOnPersona(personaType, 'secondary') : null

  // Format timestamp
  const formattedTime = message.timestamp.toLocaleTimeString([], {
    hour: '2-digit',
    minute: '2-digit',
  })

  return (
    <div className={`flex gap-3 ${className}`}>
      {/* Avatar */}
      {isAgent ? (
        <AgentAvatar
          personaType={personaType}
          icon={message.senderIcon}
          size="sm"
          isDarkMode={isDarkMode}
        />
      ) : (
        <div className="flex-shrink-0 w-8 h-8 rounded-full bg-gray-500 dark:bg-gray-600 flex items-center justify-center text-white text-sm">
          👤
        </div>
      )}

      {/* Message content */}
      <div className="flex-1 min-w-0">
        {/* Header: Name + Timestamp */}
        <div className="flex items-baseline gap-2 mb-1">
          <span
            className={`text-sm font-semibold ${isAgent ? fontClass : 'font-normal'}`}
            style={isAgent && colors ? { color: colors.primary } : undefined}
          >
            {message.senderName}
          </span>
          <span className="text-xs text-gray-500 dark:text-gray-400">
            {formattedTime}
          </span>
        </div>

        {/* Message bubble */}
        <div
          className={`rounded-lg p-3 transition-colors duration-200 border-l-3 ${
            isAgent ? fontClass : 'font-normal'
          }`}
          style={
            isAgent && colors
              ? {
                  backgroundColor: colors.secondary,
                  borderLeftColor: colors.primary,
                  borderLeftWidth: '3px',
                  color: textColor || undefined,
                }
              : undefined
          }
        >
          <p className={`text-sm ${isAgent ? '' : 'text-gray-900 dark:text-white'} whitespace-pre-wrap`}>
            {message.content}
          </p>
        </div>
      </div>
    </div>
  )
}

export default MessageCard
