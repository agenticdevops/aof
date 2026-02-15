/**
 * Color Scheme System
 *
 * Central color palette definitions for all persona types.
 * Provides consistent color mappings for light/dark modes, status indicators,
 * and text contrast.
 */

import type { PersonaType, PersonaColor } from '@/types/personas'
import type { AgentStatus } from '@/types/dashboard'

/**
 * Complete color palette map for all persona types
 */
export const personaColorMap: Record<PersonaType, PersonaColor> = {
  analyst: {
    primary: '#3b82f6',      // blue-500
    secondary: '#dbeafe',    // blue-100
    accent: '#1e40af',       // blue-800
    darkPrimary: '#60a5fa',  // blue-400
    darkSecondary: '#1e3a8a', // blue-900
    darkAccent: '#93c5fd',   // blue-300
  },
  coordinator: {
    primary: '#10b981',      // emerald-500
    secondary: '#d1fae5',    // emerald-100
    accent: '#047857',       // emerald-700
    darkPrimary: '#34d399',  // emerald-400
    darkSecondary: '#064e3b', // emerald-900
    darkAccent: '#6ee7b7',   // emerald-300
  },
  specialist: {
    primary: '#8b5cf6',      // violet-500
    secondary: '#ede9fe',    // violet-100
    accent: '#5b21b6',       // violet-800
    darkPrimary: '#a78bfa',  // violet-400
    darkSecondary: '#2e1065', // violet-900
    darkAccent: '#c4b5fd',   // violet-300
  },
  responder: {
    primary: '#ef4444',      // red-500
    secondary: '#fee2e2',    // red-100
    accent: '#991b1b',       // red-800
    darkPrimary: '#f87171',  // red-400
    darkSecondary: '#7f1d1d', // red-900
    darkAccent: '#fca5a5',   // red-300
  },
  custom: {
    primary: '#6b7280',      // gray-500
    secondary: '#f3f4f6',    // gray-100
    accent: '#374151',       // gray-700
    darkPrimary: '#9ca3af',  // gray-400
    darkSecondary: '#1f2937', // gray-800
    darkAccent: '#d1d5db',   // gray-300
  },
}

/**
 * Status color mappings (universal, overrides persona colors)
 */
export const statusColorMap: Record<AgentStatus, { light: string; dark: string }> = {
  active: {
    light: '#10b981', // emerald-500
    dark: '#34d399',  // emerald-400
  },
  idle: {
    light: '#f59e0b', // amber-500
    dark: '#fbbf24',  // amber-400
  },
  error: {
    light: '#ef4444', // red-500
    dark: '#f87171',  // red-400
  },
}

/**
 * Text color mappings for persona backgrounds
 * Ensures WCAG AA contrast compliance
 */
export const personaTextColorMap: Record<PersonaType, { onPrimary: string; onSecondary: string; onAccent: string }> = {
  analyst: {
    onPrimary: '#ffffff',   // white text on blue-500
    onSecondary: '#1e40af', // blue-800 text on blue-100
    onAccent: '#ffffff',    // white text on blue-800
  },
  coordinator: {
    onPrimary: '#ffffff',   // white text on emerald-500
    onSecondary: '#047857', // emerald-700 text on emerald-100
    onAccent: '#ffffff',    // white text on emerald-700
  },
  specialist: {
    onPrimary: '#ffffff',   // white text on violet-500
    onSecondary: '#5b21b6', // violet-800 text on violet-100
    onAccent: '#ffffff',    // white text on violet-800
  },
  responder: {
    onPrimary: '#ffffff',   // white text on red-500
    onSecondary: '#991b1b', // red-800 text on red-100
    onAccent: '#ffffff',    // white text on red-800
  },
  custom: {
    onPrimary: '#ffffff',   // white text on gray-500
    onSecondary: '#374151', // gray-700 text on gray-100
    onAccent: '#ffffff',    // white text on gray-700
  },
}

/**
 * Dark mode color variants
 * (for components that need explicit dark mode colors)
 */
export const darkModeColorMap: Record<PersonaType, PersonaColor> = {
  analyst: {
    primary: '#60a5fa',      // blue-400
    secondary: '#1e3a8a',    // blue-900
    accent: '#93c5fd',       // blue-300
    darkPrimary: '#60a5fa',  // same as primary
    darkSecondary: '#1e3a8a',
    darkAccent: '#93c5fd',
  },
  coordinator: {
    primary: '#34d399',      // emerald-400
    secondary: '#064e3b',    // emerald-900
    accent: '#6ee7b7',       // emerald-300
    darkPrimary: '#34d399',
    darkSecondary: '#064e3b',
    darkAccent: '#6ee7b7',
  },
  specialist: {
    primary: '#a78bfa',      // violet-400
    secondary: '#2e1065',    // violet-900
    accent: '#c4b5fd',       // violet-300
    darkPrimary: '#a78bfa',
    darkSecondary: '#2e1065',
    darkAccent: '#c4b5fd',
  },
  responder: {
    primary: '#f87171',      // red-400
    secondary: '#7f1d1d',    // red-900
    accent: '#fca5a5',       // red-300
    darkPrimary: '#f87171',
    darkSecondary: '#7f1d1d',
    darkAccent: '#fca5a5',
  },
  custom: {
    primary: '#9ca3af',      // gray-400
    secondary: '#1f2937',    // gray-800
    accent: '#d1d5db',       // gray-300
    darkPrimary: '#9ca3af',
    darkSecondary: '#1f2937',
    darkAccent: '#d1d5db',
  },
}

/**
 * Utility: Get status badge background color
 */
export function getStatusBadgeColor(status: AgentStatus, isDarkMode: boolean): string {
  return isDarkMode ? statusColorMap[status].dark : statusColorMap[status].light
}

/**
 * Utility: Get text color for persona background
 */
export function getTextOnPersonaColor(
  personaType: PersonaType,
  surface: 'primary' | 'secondary' | 'accent'
): string {
  return personaTextColorMap[personaType][`on${surface.charAt(0).toUpperCase() + surface.slice(1)}` as keyof typeof personaTextColorMap[PersonaType]]
}
