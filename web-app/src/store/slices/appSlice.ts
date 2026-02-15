import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import type { RootState } from '../store'

type NavigationState = 'welcome' | 'wizard' | 'config' | 'missionControl'
type ThemeMode = 'light' | 'dark'

interface AppState {
  navigation: NavigationState
  theme: ThemeMode
  firstVisit: boolean
  daemonUrl: string
}

const initialState: AppState = {
  navigation: 'welcome',
  theme: 'light',
  firstVisit: true,
  daemonUrl: 'http://localhost:7777',
}

const appSlice = createSlice({
  name: 'app',
  initialState,
  reducers: {
    setNavigation: (state, action: PayloadAction<NavigationState>) => {
      state.navigation = action.payload
    },
    setTheme: (state, action: PayloadAction<ThemeMode>) => {
      state.theme = action.payload
    },
    toggleTheme: (state) => {
      state.theme = state.theme === 'light' ? 'dark' : 'light'
    },
    setFirstVisit: (state, action: PayloadAction<boolean>) => {
      state.firstVisit = action.payload
    },
    setDaemonUrl: (state, action: PayloadAction<string>) => {
      state.daemonUrl = action.payload
    },
  },
})

export const { setNavigation, setTheme, toggleTheme, setFirstVisit, setDaemonUrl } = appSlice.actions
export default appSlice.reducer

// Selectors
export const selectNavigation = (state: RootState) => state.app.navigation
export const selectTheme = (state: RootState) => state.app.theme
export const selectIsDarkMode = (state: RootState) => state.app.theme === 'dark'
export const selectFirstVisit = (state: RootState) => state.app.firstVisit
export const selectDaemonUrl = (state: RootState) => state.app.daemonUrl
