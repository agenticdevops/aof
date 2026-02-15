/**
 * Persona Styling Utilities Tests
 *
 * Tests for persona color lookups, icon resolution, font selection,
 * and color contrast utilities.
 */

import { describe, it, expect } from 'vitest'
import {
  getPersonaColors,
  getPersonaIcon,
  getPersonaFont,
  getStatusColor,
  getTextColorOnPersona,
  getPersonaBorderColor,
  isColorDark,
} from '@/utils/personaStyles'
import type { PersonaType } from '@/types/personas'

describe('Persona Styling Utilities', () => {
  describe('getPersonaColors', () => {
    it('returns correct colors for analyst persona (light mode)', () => {
      const colors = getPersonaColors('analyst', false)
      expect(colors.primary).toBe('#3b82f6') // blue-500
      expect(colors.secondary).toBe('#dbeafe') // blue-100
      expect(colors.accent).toBe('#1e40af') // blue-800
    })

    it('returns dark mode colors when isDarkMode=true', () => {
      const colors = getPersonaColors('analyst', true)
      expect(colors.primary).toBe('#60a5fa') // blue-400
      expect(colors.secondary).toBe('#1e3a8a') // blue-900
    })

    it('returns coordinator (emerald) colors', () => {
      const colors = getPersonaColors('coordinator', false)
      expect(colors.primary).toBe('#10b981') // emerald-500
      expect(colors.secondary).toBe('#d1fae5') // emerald-100
    })

    it('returns specialist (violet) colors', () => {
      const colors = getPersonaColors('specialist', false)
      expect(colors.primary).toBe('#8b5cf6') // violet-500
      expect(colors.secondary).toBe('#ede9fe') // violet-100
    })

    it('returns responder (red) colors', () => {
      const colors = getPersonaColors('responder', false)
      expect(colors.primary).toBe('#ef4444') // red-500
      expect(colors.secondary).toBe('#fee2e2') // red-100
    })

    it('returns gray colors for custom/unknown persona', () => {
      const colors = getPersonaColors('custom', false)
      expect(colors.primary).toBe('#6b7280') // gray-500
    })
  })

  describe('getPersonaIcon', () => {
    it('returns correct icon for analyst', () => {
      expect(getPersonaIcon('analyst')).toBe('📊')
    })

    it('returns correct icon for coordinator', () => {
      expect(getPersonaIcon('coordinator')).toBe('⚙️')
    })

    it('returns correct icon for specialist', () => {
      expect(getPersonaIcon('specialist')).toBe('🔧')
    })

    it('returns correct icon for responder', () => {
      expect(getPersonaIcon('responder')).toBe('🚨')
    })

    it('returns default robot icon for unknown persona', () => {
      expect(getPersonaIcon('unknown' as PersonaType)).toBe('🤖')
    })
  })

  describe('getPersonaFont', () => {
    it('returns font-normal for analyst', () => {
      expect(getPersonaFont('analyst')).toBe('font-normal')
    })

    it('returns font-bold for coordinator', () => {
      expect(getPersonaFont('coordinator')).toBe('font-bold')
    })

    it('returns font-normal for specialist', () => {
      expect(getPersonaFont('specialist')).toBe('font-normal')
    })

    it('returns font-normal for responder', () => {
      expect(getPersonaFont('responder')).toBe('font-normal')
    })

    it('returns font-normal for unknown persona', () => {
      expect(getPersonaFont('unknown' as PersonaType)).toBe('font-normal')
    })
  })

  describe('getStatusColor', () => {
    it('returns green for active status (light mode)', () => {
      expect(getStatusColor('active', false)).toBe('#10b981') // emerald-500
    })

    it('returns green for active status (dark mode)', () => {
      expect(getStatusColor('active', true)).toBe('#34d399') // emerald-400
    })

    it('returns yellow/amber for idle status (light mode)', () => {
      expect(getStatusColor('idle', false)).toBe('#f59e0b') // amber-500
    })

    it('returns red for error status (light mode)', () => {
      expect(getStatusColor('error', false)).toBe('#ef4444') // red-500
    })

    it('returns red for error status (dark mode)', () => {
      expect(getStatusColor('error', true)).toBe('#f87171') // red-400
    })
  })

  describe('getTextColorOnPersona', () => {
    it('returns white for text on analyst primary', () => {
      expect(getTextColorOnPersona('analyst', 'primary')).toBe('#ffffff')
    })

    it('returns dark blue for text on analyst secondary', () => {
      expect(getTextColorOnPersona('analyst', 'secondary')).toBe('#1e40af')
    })

    it('returns white for text on coordinator primary', () => {
      expect(getTextColorOnPersona('coordinator', 'primary')).toBe('#ffffff')
    })

    it('returns dark emerald for text on coordinator secondary', () => {
      expect(getTextColorOnPersona('coordinator', 'secondary')).toBe('#047857')
    })
  })

  describe('getPersonaBorderColor', () => {
    it('returns primary color for border (light mode)', () => {
      expect(getPersonaBorderColor('analyst', false)).toBe('#3b82f6')
    })

    it('returns dark primary color for border (dark mode)', () => {
      expect(getPersonaBorderColor('analyst', true)).toBe('#60a5fa')
    })
  })

  describe('isColorDark', () => {
    it('returns true for dark colors', () => {
      expect(isColorDark('#1e40af')).toBe(true) // blue-800
      expect(isColorDark('#000000')).toBe(true) // black
      expect(isColorDark('#374151')).toBe(true) // gray-700
    })

    it('returns false for light colors', () => {
      expect(isColorDark('#ffffff')).toBe(false) // white
      expect(isColorDark('#dbeafe')).toBe(false) // blue-100
      expect(isColorDark('#f3f4f6')).toBe(false) // gray-100
    })

    it('handles colors with # prefix', () => {
      expect(isColorDark('#1e40af')).toBe(true)
    })

    it('handles colors without # prefix', () => {
      expect(isColorDark('1e40af')).toBe(true)
    })
  })
})
