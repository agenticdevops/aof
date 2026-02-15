import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, vi } from 'vitest'
import { Provider } from 'react-redux'
import App from '@/App'
import { store } from '@/store/store'

describe('Complete Onboarding Flow', () => {
  it('renders app with Redux store', () => {
    render(
      <Provider store={store}>
        <App />
      </Provider>
    )

    // Verify the app renders
    const appElement = screen.getByRole('main', { hidden: true }).parentElement
    expect(appElement).toBeDefined()
  })

  it('has Redux store with onboarding state', () => {
    render(
      <Provider store={store}>
        <App />
      </Provider>
    )

    // Check that store is initialized
    const state = store.getState()
    expect(state).toBeDefined()
    expect(state.onboarding).toBeDefined()
    expect(state.config).toBeDefined()
  })

  it('initial onboarding state is correct', () => {
    render(
      <Provider store={store}>
        <App />
      </Provider>
    )

    const state = store.getState()
    // New wizard state structure
    expect(state.onboarding.currentStep).toBe(1)
    expect(state.onboarding.selectedChannels).toBeDefined()
    expect(Array.isArray(state.onboarding.selectedChannels)).toBe(true)
    expect(state.onboarding.selectedModel).toBe('anthropic')
    expect(state.onboarding.selectedTools).toBeDefined()
    expect(Array.isArray(state.onboarding.selectedTools)).toBe(true)
    expect(state.onboarding.loading).toBe(false)
  })
})
