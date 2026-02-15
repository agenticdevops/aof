import { createSlice, PayloadAction } from '@reduxjs/toolkit'

interface AppState {
  navigation: 'welcome' | 'wizard' | 'config' | 'missionControl'
  theme: 'light' | 'dark'
  firstVisit: boolean
  daemonUrl: string
}

const initialState: AppState = {
  navigation: 'welcome',
  theme: 'light',
  firstVisit: true,
  daemonUrl: import.meta.env.VITE_API_URL || 'http://localhost:7777',
}

const appSlice = createSlice({
  name: 'app',
  initialState,
  reducers: {
    setNavigation: (state, action: PayloadAction<AppState['navigation']>) => {
      state.navigation = action.payload
    },
    setTheme: (state, action: PayloadAction<'light' | 'dark'>) => {
      state.theme = action.payload
    },
    setFirstVisit: (state, action: PayloadAction<boolean>) => {
      state.firstVisit = action.payload
    },
    toggleTheme: (state) => {
      state.theme = state.theme === 'light' ? 'dark' : 'light'
    },
  },
})

export const { setNavigation, setTheme, setFirstVisit, toggleTheme } = appSlice.actions
export default appSlice.reducer
