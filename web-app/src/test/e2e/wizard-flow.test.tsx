import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Provider } from 'react-redux'
import { PersistGate } from 'redux-persist/integration/react'
import { store, persistor } from '@/store/store'
import OnboardingWizard from '@/components/onboarding/OnboardingWizard'

/**
 * E2E tests for the complete onboarding wizard flow
 */
describe('Onboarding Wizard E2E Flow', () => {
  beforeEach(() => {
    // Reset Redux store to initial state
    store.dispatch({ type: 'onboarding/resetWizard' })
    vi.clearAllMocks()
  })

  /**
   * Test 1: Wizard initialization
   */
  it('renders wizard with Step 1 of 4', () => {
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    expect(screen.getByText('Welcome to Xops')).toBeInTheDocument()
    expect(screen.getByText('Step 1 of 4')).toBeInTheDocument()
    expect(screen.getByText('Connect Channels')).toBeInTheDocument()
  })

  /**
   * Test 2: Back button is disabled on step 1
   */
  it('disables Back button on step 1', () => {
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    const backButton = screen.getByRole('button', { name: /back/i })
    expect(backButton).toBeDisabled()
  })

  /**
   * Test 3: Next button is disabled when no channels selected
   */
  it('disables Next button when no channels selected on Step 1', () => {
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    const nextButton = screen.getByRole('button', { name: /next/i })
    expect(nextButton).toBeDisabled()
  })

  /**
   * Test 4: Channel selection enables Next button
   */
  it('enables Next button after selecting a channel', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    // Find and check the Slack checkbox
    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })
    await user.click(slackCheckbox)

    // Next button should now be enabled
    const nextButton = screen.getByRole('button', { name: /next/i })
    expect(nextButton).not.toBeDisabled()
  })

  /**
   * Test 5: Step validation - selecting channel and moving to Step 2
   */
  it('allows moving to Step 2 after selecting channel', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    // Select Slack
    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })
    await user.click(slackCheckbox)

    // Click Next
    const nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    // Should move to Step 2
    await waitFor(() => {
      expect(screen.getByText('Step 2 of 4')).toBeInTheDocument()
      expect(screen.getByText('Select AI Model')).toBeInTheDocument()
    })
  })

  /**
   * Test 6: AI Model selection on Step 2
   */
  it('displays all AI model providers on Step 2', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    // Move to Step 2
    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })
    await user.click(slackCheckbox)
    const nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    // Wait for Step 2 and check for providers
    await waitFor(() => {
      expect(screen.getByText('Anthropic')).toBeInTheDocument()
      expect(screen.getByText('OpenAI')).toBeInTheDocument()
      expect(screen.getByText('Google')).toBeInTheDocument()
    })
  })

  /**
   * Test 7: Step 2 -> Step 3 navigation
   */
  it('moves from Step 2 to Step 3 (Tools)', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    // Complete Step 1: Select channel
    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })
    await user.click(slackCheckbox)

    // Move to Step 2
    let nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    // Wait for Step 2
    await waitFor(() => {
      expect(screen.getByText('Step 2 of 4')).toBeInTheDocument()
    })

    // Move to Step 3
    nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    // Should show Step 3
    await waitFor(() => {
      expect(screen.getByText('Step 3 of 4')).toBeInTheDocument()
      expect(screen.getByText('Choose Tools')).toBeInTheDocument()
    })
  })

  /**
   * Test 8: Back navigation works correctly
   */
  it('allows going back from Step 2 to Step 1', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    // Move to Step 2
    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })
    await user.click(slackCheckbox)
    let nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    // Wait for Step 2
    await waitFor(() => {
      expect(screen.getByText('Step 2 of 4')).toBeInTheDocument()
    })

    // Click Back
    const backButton = screen.getByRole('button', { name: /back/i })
    await user.click(backButton)

    // Should return to Step 1
    await waitFor(() => {
      expect(screen.getByText('Step 1 of 4')).toBeInTheDocument()
      expect(screen.getByText('Connect Channels')).toBeInTheDocument()
    })
  })

  /**
   * Test 9: Progress bar updates correctly
   */
  it('updates progress bar as user navigates steps', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    // Step 1
    expect(screen.getByText('Step 1 of 4')).toBeInTheDocument()

    // Move to Step 2
    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })
    await user.click(slackCheckbox)
    let nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    await waitFor(() => {
      expect(screen.getByText('Step 2 of 4')).toBeInTheDocument()
    })

    // Move to Step 3
    nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    await waitFor(() => {
      expect(screen.getByText('Step 3 of 4')).toBeInTheDocument()
    })

    // Move to Step 4
    nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    await waitFor(() => {
      expect(screen.getByText('Step 4 of 4')).toBeInTheDocument()
    })
  })

  /**
   * Test 10: Review step shows configuration summary
   */
  it('displays configuration summary on Step 4 (Review)', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    // Complete all steps
    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })
    await user.click(slackCheckbox)

    let nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    await waitFor(() => {
      expect(screen.getByText('Step 2 of 4')).toBeInTheDocument()
    })

    nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    await waitFor(() => {
      expect(screen.getByText('Step 3 of 4')).toBeInTheDocument()
    })

    nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    // Wait for Step 4 and verify configuration is shown
    await waitFor(() => {
      expect(screen.getByText('Step 4 of 4')).toBeInTheDocument()
      expect(screen.getByText('Launch Xops')).toBeInTheDocument()
    })

    // Check configuration summary
    expect(screen.getByText('Configuration Summary')).toBeInTheDocument()
  })

  /**
   * Test 11: Error alert displays when validation fails
   */
  it('shows error message when step validation fails', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    // Try clicking Next without selecting a channel
    const nextButton = screen.getByRole('button', { name: /next/i })
    expect(nextButton).toBeDisabled()

    // Validation helper text should be visible
    expect(
      screen.getByText('Please select at least one communication channel')
    ).toBeInTheDocument()
  })

  /**
   * Test 12: Multiple channels can be selected
   */
  it('allows selecting multiple communication channels', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    // Select Slack, Telegram, and Discord
    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })
    const telegramCheckbox = screen.getByRole('checkbox', { name: /telegram/i })
    const discordCheckbox = screen.getByRole('checkbox', { name: /discord/i })

    await user.click(slackCheckbox)
    await user.click(telegramCheckbox)
    await user.click(discordCheckbox)

    // All should be checked
    expect(slackCheckbox).toBeChecked()
    expect(telegramCheckbox).toBeChecked()
    expect(discordCheckbox).toBeChecked()

    // Next button should be enabled
    const nextButton = screen.getByRole('button', { name: /next/i })
    expect(nextButton).not.toBeDisabled()
  })

  /**
   * Test 13: Tool selection shows tool count
   */
  it('displays tool count on Step 3', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    // Move to Step 3
    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })
    await user.click(slackCheckbox)

    let nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    await waitFor(() => {
      expect(screen.getByText('Step 2 of 4')).toBeInTheDocument()
    })

    nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    // Wait for Step 3
    await waitFor(() => {
      expect(screen.getByText('Step 3 of 4')).toBeInTheDocument()
    })

    // Should show selected tools count
    expect(screen.getByText(/Selected tools:/i)).toBeInTheDocument()
  })

  /**
   * Test 14: Can deselect channels
   */
  it('allows deselecting previously selected channels', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })

    // Select Slack
    await user.click(slackCheckbox)
    expect(slackCheckbox).toBeChecked()

    // Next button enabled
    let nextButton = screen.getByRole('button', { name: /next/i })
    expect(nextButton).not.toBeDisabled()

    // Deselect Slack
    await user.click(slackCheckbox)
    expect(slackCheckbox).not.toBeChecked()

    // Next button disabled again
    nextButton = screen.getByRole('button', { name: /next/i })
    expect(nextButton).toBeDisabled()
  })

  /**
   * Test 15: Default AI model is pre-selected
   */
  it('pre-selects Anthropic as default AI model', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <PersistGate loading={null} persistor={persistor}>
          <OnboardingWizard />
        </PersistGate>
      </Provider>
    )

    // Move to Step 2
    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })
    await user.click(slackCheckbox)

    const nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    // Wait for Step 2 and check Anthropic is selected
    await waitFor(() => {
      const anthropicRadio = screen.getByRole('radio', { name: /anthropic/i })
      expect(anthropicRadio).toBeChecked()
    })
  })
})
