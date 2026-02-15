/**
 * Persona Styling Utilities
 *
 * Utility functions for applying persona-based styling across components.
 * Provides color lookup, icon resolution, font selection, and Tailwind class generation.
 */

import type { PersonaType, PersonaColor } from '@/types/personas'
import type { AgentStatus } from '@/types/dashboard'
import { personaColorMap, statusColorMap, personaTextColorMap } from '@/data/colorSchemes'
import { DEFAULT_PERSONAS } from '@/types/personas'

/**
 * Get color palette for a persona type
 *
 * @param personaType - The agent persona type
 * @param isDarkMode - Whether dark mode is active
 * @returns PersonaColor object with all color values
 */
export function getPersonaColors(personaType: PersonaType, isDarkMode = false): PersonaColor {
  const colors = personaColorMap[personaType] || personaColorMap.custom

  if (isDarkMode) {
    // Return dark mode variant
    return {
      primary: colors.darkPrimary,
      secondary: colors.darkSecondary,
      accent: colors.darkAccent,
      darkPrimary: colors.darkPrimary,
      darkSecondary: colors.darkSecondary,
      darkAccent: colors.darkAccent,
    }
  }

  return colors
}

/**
 * Get persona icon emoji
 *
 * @param personaType - The agent persona type
 * @returns Emoji string or default robot emoji
 */
export function getPersonaIcon(personaType: PersonaType): string {
  const persona = DEFAULT_PERSONAS.find((p) => p.type === personaType)
  return persona?.icon || '🤖'
}

/**
 * Get Tailwind font class for persona
 *
 * @param personaType - The agent persona type
 * @returns Tailwind font weight class
 */
export function getPersonaFont(personaType: PersonaType): string {
  const persona = DEFAULT_PERSONAS.find((p) => p.type === personaType)
  const font = persona?.font || 'regular'

  switch (font) {
    case 'bold':
      return 'font-bold'
    case 'decorative':
      return 'font-medium'
    case 'regular':
    default:
      return 'font-normal'
  }
}

/**
 * Generate Tailwind CSS classes for persona styling
 *
 * @param personaType - The agent persona type
 * @param isDarkMode - Whether dark mode is active
 * @param variant - Color variant to use (primary, secondary, accent)
 * @returns Tailwind CSS class string
 */
export function getPersonaTailwindClasses(
  personaType: PersonaType,
  isDarkMode = false,
  variant: 'primary' | 'secondary' | 'accent' = 'secondary'
): string {
  const colors = getPersonaColors(personaType, isDarkMode)
  const textColor = getTextColorOnPersona(personaType, variant)

  // Map hex colors to Tailwind classes
  const bgColorMap: Record<string, string> = {
    // Analyst (blue)
    '#dbeafe': 'bg-blue-50',
    '#1e3a8a': 'bg-blue-900',
    '#3b82f6': 'bg-blue-500',
    '#60a5fa': 'bg-blue-400',

    // Coordinator (emerald)
    '#d1fae5': 'bg-emerald-50',
    '#064e3b': 'bg-emerald-900',
    '#10b981': 'bg-emerald-500',
    '#34d399': 'bg-emerald-400',

    // Specialist (violet)
    '#ede9fe': 'bg-violet-50',
    '#2e1065': 'bg-violet-900',
    '#8b5cf6': 'bg-violet-500',
    '#a78bfa': 'bg-violet-400',

    // Responder (red)
    '#fee2e2': 'bg-red-50',
    '#7f1d1d': 'bg-red-900',
    '#ef4444': 'bg-red-500',
    '#f87171': 'bg-red-400',

    // Custom (gray)
    '#f3f4f6': 'bg-gray-50',
    '#1f2937': 'bg-gray-800',
    '#6b7280': 'bg-gray-500',
    '#9ca3af': 'bg-gray-400',
  }

  const textColorMap: Record<string, string> = {
    '#ffffff': 'text-white',
    '#1e40af': 'text-blue-800',
    '#047857': 'text-emerald-700',
    '#5b21b6': 'text-violet-800',
    '#991b1b': 'text-red-800',
    '#374151': 'text-gray-700',
  }

  const colorKey = variant === 'primary' ? colors.primary : variant === 'accent' ? colors.accent : colors.secondary
  const bgClass = bgColorMap[colorKey.toLowerCase()] || 'bg-gray-100'
  const textClass = textColorMap[textColor.toLowerCase()] || 'text-gray-900'

  return `${bgClass} ${textClass}`
}

/**
 * Get status indicator color (universal, not persona-specific)
 *
 * @param status - Agent status
 * @param isDarkMode - Whether dark mode is active
 * @returns Hex color code
 */
export function getStatusColor(status: AgentStatus, isDarkMode = false): string {
  return isDarkMode ? statusColorMap[status].dark : statusColorMap[status].light
}

/**
 * Get text color that contrasts well on persona background
 *
 * @param personaType - The agent persona type
 * @param surface - Which persona color surface (primary, secondary, accent)
 * @returns Hex color code for text
 */
export function getTextColorOnPersona(
  personaType: PersonaType,
  surface: 'primary' | 'secondary' | 'accent' = 'primary'
): string {
  const textColors = personaTextColorMap[personaType] || personaTextColorMap.custom

  switch (surface) {
    case 'primary':
      return textColors.onPrimary
    case 'secondary':
      return textColors.onSecondary
    case 'accent':
      return textColors.onAccent
    default:
      return textColors.onPrimary
  }
}

/**
 * Generate inline style object for persona background
 * (useful when Tailwind classes aren't flexible enough)
 *
 * @param personaType - The agent persona type
 * @param isDarkMode - Whether dark mode is active
 * @param variant - Color variant to use
 * @returns React style object
 */
export function getPersonaBackgroundStyle(
  personaType: PersonaType,
  isDarkMode = false,
  variant: 'primary' | 'secondary' | 'accent' = 'primary'
): React.CSSProperties {
  const colors = getPersonaColors(personaType, isDarkMode)
  const colorKey = variant === 'primary' ? colors.primary : variant === 'accent' ? colors.accent : colors.secondary
  const textColor = getTextColorOnPersona(personaType, variant)

  return {
    backgroundColor: colorKey,
    color: textColor,
  }
}

/**
 * Get border color for persona (uses primary color)
 *
 * @param personaType - The agent persona type
 * @param isDarkMode - Whether dark mode is active
 * @returns Hex color code
 */
export function getPersonaBorderColor(personaType: PersonaType, isDarkMode = false): string {
  const colors = getPersonaColors(personaType, isDarkMode)
  return colors.primary
}

/**
 * Check if color is dark (for automatic text color selection)
 *
 * @param hexColor - Hex color code
 * @returns true if color is dark, false if light
 */
export function isColorDark(hexColor: string): boolean {
  // Remove # if present
  const hex = hexColor.replace('#', '')

  // Convert to RGB
  const r = parseInt(hex.substring(0, 2), 16)
  const g = parseInt(hex.substring(2, 4), 16)
  const b = parseInt(hex.substring(4, 6), 16)

  // Calculate relative luminance (WCAG formula)
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255

  // Return true if color is dark (luminance < 0.5)
  return luminance < 0.5
}
