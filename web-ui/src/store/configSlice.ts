/**
 * Redux slice for configuration data (agents, tools).
 * Manages data from Phase 1 configuration API endpoints.
 */

import { createSlice } from '@reduxjs/toolkit';
import type { PayloadAction } from '@reduxjs/toolkit';
import type { Agent, Tool } from '../types/events';

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
}

/**
 * Initial state.
 */
const initialState: ConfigState = {
  agents: [],
  tools: [],
  configVersion: '',
};

/**
 * Config slice with reducers.
 */
const configSlice = createSlice({
  name: 'config',
  initialState,
  reducers: {
    /**
     * Set agents list.
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
  },
});

export const { setAgents, setTools, setConfigVersion } = configSlice.actions;
export default configSlice.reducer;
