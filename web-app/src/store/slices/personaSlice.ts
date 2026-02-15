/**
 * Persona Redux Slice
 *
 * Manages agent persona state, allowing customization and persistence.
 * Default personas are immutable; custom modifications stored separately.
 */

import { createSlice, type PayloadAction } from '@reduxjs/toolkit'
import type { RootState } from '@/store/store'
import type { AgentPersona, PersonaType, PersonaColor, PersonaState } from '@/types/personas'
import { DEFAULT_PERSONAS } from '@/types/personas'

/**
 * Convert default personas array to keyed record
 */
function getDefaultPersonasRecord(): Record<string, AgentPersona> {
  return DEFAULT_PERSONAS.reduce(
    (acc, persona) => {
      acc[persona.id] = persona
      return acc
    },
    {} as Record<string, AgentPersona>
  )
}

/**
 * Initial state
 */
const initialState: PersonaState = {
  personas: getDefaultPersonasRecord(),
  defaultPersonas: DEFAULT_PERSONAS,
  isLoading: false,
  error: null,
}

/**
 * Persona slice
 */
export const personaSlice = createSlice({
  name: 'persona',
  initialState,
  reducers: {
    /**
     * Replace all personas (used when loading from backend)
     */
    setPersonas: (state, action: PayloadAction<AgentPersona[]>) => {
      state.personas = action.payload.reduce(
        (acc, persona) => {
          acc[persona.id] = persona
          return acc
        },
        {} as Record<string, AgentPersona>
      )
      state.error = null
    },

    /**
     * Update color palette for a specific persona
     */
    updatePersonaColor: (
      state,
      action: PayloadAction<{ personaId: string; colors: PersonaColor }>
    ) => {
      const { personaId, colors } = action.payload
      if (state.personas[personaId]) {
        state.personas[personaId].colors = colors
      }
    },

    /**
     * Update persona icon
     */
    updatePersonaIcon: (
      state,
      action: PayloadAction<{ personaId: string; icon: string }>
    ) => {
      const { personaId, icon } = action.payload
      if (state.personas[personaId]) {
        state.personas[personaId].icon = icon
      }
    },

    /**
     * Reset all personas to defaults
     */
    resetPersonasToDefault: (state) => {
      state.personas = getDefaultPersonasRecord()
      state.error = null
    },

    /**
     * Add a custom persona
     */
    addCustomPersona: (state, action: PayloadAction<AgentPersona>) => {
      const persona = action.payload
      state.personas[persona.id] = persona
    },

    /**
     * Remove a custom persona (cannot remove defaults)
     */
    removeCustomPersona: (state, action: PayloadAction<string>) => {
      const personaId = action.payload
      const persona = state.personas[personaId]

      // Only allow removal of custom personas
      if (persona && persona.type === 'custom') {
        delete state.personas[personaId]
      }
    },

    /**
     * Set loading state
     */
    setLoading: (state, action: PayloadAction<boolean>) => {
      state.isLoading = action.payload
    },

    /**
     * Set error state
     */
    setError: (state, action: PayloadAction<string | null>) => {
      state.error = action.payload
      state.isLoading = false
    },
  },
})

/**
 * Actions
 */
export const {
  setPersonas,
  updatePersonaColor,
  updatePersonaIcon,
  resetPersonasToDefault,
  addCustomPersona,
  removeCustomPersona,
  setLoading,
  setError,
} = personaSlice.actions

/**
 * Selectors
 */

/**
 * Select all personas as array
 */
export const selectAllPersonas = (state: RootState): AgentPersona[] => {
  return Object.values(state.persona.personas)
}

/**
 * Select persona by ID
 */
export const selectPersonaById = (state: RootState, personaId: string): AgentPersona | undefined => {
  return state.persona.personas[personaId]
}

/**
 * Select persona by type (returns first match)
 */
export const selectPersonaByType = (state: RootState, type: PersonaType): AgentPersona | undefined => {
  return Object.values(state.persona.personas).find((p) => p.type === type)
}

/**
 * Select default personas
 */
export const selectDefaultPersonas = (state: RootState): AgentPersona[] => {
  return state.persona.defaultPersonas
}

/**
 * Select custom personas
 */
export const selectCustomPersonas = (state: RootState): AgentPersona[] => {
  return Object.values(state.persona.personas).filter((p) => p.type === 'custom')
}

/**
 * Select loading state
 */
export const selectPersonaLoading = (state: RootState): boolean => {
  return state.persona.isLoading
}

/**
 * Select error state
 */
export const selectPersonaError = (state: RootState): string | null => {
  return state.persona.error
}

/**
 * Reducer
 */
export default personaSlice.reducer
