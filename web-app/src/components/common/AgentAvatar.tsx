/**
 * AgentAvatar Component
 *
 * Displays agent avatar with persona styling, icon, and online indicator.
 * Used consistently across all agent-related UI components.
 */

import React from 'react'
import type { PersonaType } from '@/types/personas'
import { getPersonaColors, getPersonaIcon, getTextColorOnPersona } from '@/utils/personaStyles'

export interface AgentAvatarProps {
  /** Persona type for styling */
  personaType: PersonaType
  /** Optional custom icon (overrides persona default) */
  icon?: string
  /** Size variant */
  size?: 'sm' | 'md' | 'lg'
  /** Show online indicator */
  isOnline?: boolean
  /** Optional custom className */
  className?: string
  /** Dark mode state */
  isDarkMode?: boolean
}

/**
 * Agent avatar with persona-based styling
 * Shows icon, persona colors, and optional online indicator
 */
export const AgentAvatar: React.FC<AgentAvatarProps> = ({
  personaType,
  icon,
  size = 'md',
  isOnline = false,
  className = '',
  isDarkMode = false,
}) => {
  // Get persona styling
  const colors = getPersonaColors(personaType, isDarkMode)
  const defaultIcon = getPersonaIcon(personaType)
  const displayIcon = icon || defaultIcon
  const textColor = getTextColorOnPersona(personaType, 'primary')

  // Size variants
  const sizeClasses = {
    sm: 'w-8 h-8 text-xs',
    md: 'w-12 h-12 text-lg',
    lg: 'w-16 h-16 text-2xl',
  }

  const sizeClass = sizeClasses[size]

  // Styles
  const avatarStyle: React.CSSProperties = {
    backgroundColor: colors.primary,
    color: textColor,
  }

  return (
    <div className={`relative inline-flex ${className}`}>
      {/* Avatar circle */}
      <div
        className={`${sizeClass} rounded-full flex items-center justify-center font-medium transition-all duration-200`}
        style={avatarStyle}
        title={`${personaType} avatar`}
        role="img"
        aria-label={`${personaType} avatar`}
      >
        {displayIcon}
      </div>

      {/* Online indicator */}
      {isOnline && (
        <div
          className="absolute bottom-0 right-0 w-3 h-3 rounded-full border-2 border-white dark:border-gray-900 bg-emerald-500"
          title="Online"
          role="status"
          aria-label="Online"
        />
      )}
    </div>
  )
}

export default AgentAvatar
