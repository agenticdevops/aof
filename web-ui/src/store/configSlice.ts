/**
 * Redux slice for configuration data (agents, tools).
 * Manages data from Phase 1 configuration API endpoints.
 * Extended with persona and introduction event support (Phase 5).
 */

import { createSlice } from '@reduxjs/toolkit';
import type { PayloadAction } from '@reduxjs/toolkit';
import type { Agent, Tool, IntroductionMessage } from '../types/events';

/**
 * Config slice state.
 */
interface ConfigState {
  /** Configured agents */
  agents: Agent[];

  /** Available tools */
  tools: Tool[];

  /** Configuration version (from X-Config-Version header) */
  configVersion: string;

  /** Introduction messages received from agents (Phase 5) */
  introductions: IntroductionMessage[];

  /** Set of agent IDs that have already been introduced (to avoid duplicate toasts) */
  introducedAgentIds: string[];
}

/**
 * Initial state.
 */
const initialState: ConfigState = {
  agents: [],
  tools: [],
  configVersion: '',
  introductions: [],
  introducedAgentIds: [],
};

/**
 * Config slice with reducers.
 */
const configSlice = createSlice({
  name: 'config',
  initialState,
  reducers: {
    /**
     * Set agents list (includes persona fields from AGENTS.md).
     */
    setAgents: (state, action: PayloadAction<Agent[]>) => {
      state.agents = action.payload;
    },

    /**
     * Set tools list.
     */
    setTools: (state, action: PayloadAction<Tool[]>) => {
      state.tools = action.payload;
    },

    /**
     * Set configuration version.
     */
    setConfigVersion: (state, action: PayloadAction<string>) => {
      state.configVersion = action.payload;
    },

    /**
     * Add an introduction message from an agent.
     * Only adds if the agent hasn't been introduced yet.
     */
    addIntroduction: (state, action: PayloadAction<IntroductionMessage>) => {
      const agentName = action.payload.agent_name;
      if (!state.introducedAgentIds.includes(agentName)) {
        state.introductions.push(action.payload);
        state.introducedAgentIds.push(agentName);
      }
    },

    /**
     * Mark an introduction as consumed (toast displayed).
     */
    consumeIntroduction: (state, action: PayloadAction<string>) => {
      state.introductions = state.introductions.filter(
        (intro) => intro.agent_name !== action.payload,
      );
    },

    /**
     * Clear all introductions (reset state).
     */
    clearIntroductions: (state) => {
      state.introductions = [];
      state.introducedAgentIds = [];
    },
  },
});

export const {
  setAgents,
  setTools,
  setConfigVersion,
  addIntroduction,
  consumeIntroduction,
  clearIntroductions,
} = configSlice.actions;
export default configSlice.reducer;
