---
phase: 01-onboarding-refinement
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - web-app/src/components/onboarding/OnboardingWizard.tsx
  - web-app/src/components/onboarding/StepChannels.tsx
  - web-app/src/components/onboarding/StepAIModel.tsx
  - web-app/src/components/onboarding/StepTools.tsx
  - web-app/src/components/onboarding/StepReview.tsx
  - web-app/src/store/slices/onboardingSlice.ts
  - web-app/src/types/onboarding.ts
  - web-app/src/test/e2e/wizard-flow.test.tsx

autonomous: true

must_haves:
  truths:
    - "Wizard is 3 steps (Channels → AI Model → Tools) instead of 4"
    - "Channels step configures which platforms (Slack, Telegram, Discord) Xops connects to"
    - "AI Model step selects LLM provider with default Anthropic"
    - "Tools step auto-discovers local tools and lets users select which to enable"
    - "Review step shows Xops profile with pre-defined persona (auto-created, editable later)"
    - "User can launch Xops and start working in <3 minutes after completing wizard"
    - "Xops is auto-created on Review completion with orchestrator role"
    - "Form validation provides clear error messages for all invalid inputs"
    - "State persists across browser refresh and daemon restarts"
    - "All form data properly typed and validated before submission"

  artifacts:
    - path: "web-app/src/components/onboarding/OnboardingWizard.tsx"
      provides: "Main wizard container with 3-step flow and validation"
      contains: "currentStep, handleNext, handleBack, handleSubmit"
    - path: "web-app/src/store/slices/onboardingSlice.ts"
      provides: "Redux state for wizard (currentStep, selectedChannels, selectedModel, selectedTools, xopsConfig)"
      exports: "updateStep, updateChannels, updateModel, updateTools, submitWizard"
    - path: "web-app/src/types/onboarding.ts"
      provides: "TypeScript types for all wizard data (Channel, Model, Tool, XopsConfig)"
      exports: "types used by components and Redux"

  key_links:
    - from: "OnboardingWizard.tsx"
      to: "StepChannels.tsx, StepAIModel.tsx, StepTools.tsx, StepReview.tsx"
      via: "conditional rendering based on currentStep"
      pattern: "currentStep === 1"
    - from: "OnboardingWizard.tsx"
      to: "onboardingSlice.ts"
      via: "dispatch(updateStep), dispatch(submitWizard) on next/submit"
      pattern: "dispatch\\(update"
    - from: "StepReview.tsx"
      to: "configAPI.createAgent()"
      via: "Redux thunk on submit, creates Xops agent"
      pattern: "dispatch\\(submitWizard"
    - from: "onboardingSlice.ts"
      to: "localStorage"
      via: "Redux persist middleware, saves state on each change"
      pattern: "redux-persist"

---

<objective>
Redesign the onboarding wizard from 4 steps (Project → Agent → Platforms → Review) to 3 focused steps (Channels → AI Model → Tools) with Xops auto-creation. This addresses user friction with unnecessary project setup and aligns the experience with how users think about agent deployment.

**Purpose:** Reduce onboarding complexity and mental model misalignment. Users should think "I want Xops in Slack with my tools" not "I need to create a project first."

**Output:**
- 3-step wizard flow (Channels → AI Model → Tools → Review with Xops launch)
- Xops auto-created with pre-defined orchestrator persona on completion
- Type-safe Redux state management for all steps
- Clear form validation with actionable error messages
- Complete test coverage of wizard flow
</objective>

<execution_context>
@/Users/gshah/.claude/get-shit-done/workflows/execute-plan.md
@/Users/gshah/.claude/get-shit-done/templates/summary.md
@/Users/gshah/work/opsflow-sh/aof/.planning/phases/01-onboarding-refinement/RESEARCH-SUMMARY.md
@/Users/gshah/work/opsflow-sh/aof/.planning/phases/01-onboarding-refinement/RESEARCH.md
</execution_context>

<context>
@/Users/gshah/work/opsflow-sh/aof/.planning/PROJECT.md
@/Users/gshah/work/opsflow-sh/aof/.planning/STATE.md
@/Users/gshah/work/opsflow-sh/aof/.planning/phases/01-onboarding-config-ui/01-INTEGRATION-SUMMARY.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Define onboarding types and state schema</name>
  <files>
    web-app/src/types/onboarding.ts
  </files>
  <action>
Create a comprehensive TypeScript types file for the onboarding wizard with:

1. **Channel types:**
   - `Channel = "slack" | "telegram" | "discord" | "none"`
   - `ChannelConfig { platform: Channel, webhookUrl?: string, botToken?: string, validated: boolean }`

2. **AI Model types:**
   - `AIModelProvider = "anthropic" | "openai" | "google" | "groq" | "ollama"`
   - `AIModelConfig { provider: AIModelProvider, model: string, apiKey: string }`

3. **Tool types:**
   - `Tool { id: string, name: string, version: string, path: string, available: boolean, enabled: boolean }`
   - `ToolCategory = "kubectl" | "terraform" | "docker" | "git" | "aws" | "shell" | "custom"`

4. **Xops config types:**
   - `XopsConfig { name: string, persona: AgentPersona, role: "orchestrator", channels: ChannelConfig[], model: AIModelConfig, tools: Tool[] }`
   - `AgentPersona { id: string, name: string, description: string, traits: string[], communication_style: string }`

5. **Wizard state types:**
   - `OnboardingState { currentStep: 1|2|3|4, selectedChannels: Channel[], selectedModel: AIModelProvider, selectedTools: Tool[], xopsConfig: Partial<XopsConfig>, loading: boolean, error?: string }`
   - `WizardStep = "channels" | "model" | "tools" | "review"`

6. **Form validation types:**
   - `ValidationError { field: string, message: string }`
   - `FormErrors { [key: string]: ValidationError[] }`

All types must have clear JSDoc comments explaining purpose and constraints. Use strict null checking.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm type-check
Should show 0 TypeScript errors and proper type exports.
  </verify>
  <done>
✅ onboarding.ts file exists with all required types
✅ All types properly exported and documented
✅ No TypeScript compilation errors
✅ Types cover: channels, models, tools, xops config, wizard state, validation
  </done>
</task>

<task type="auto">
  <name>Task 2: Create Redux slice for onboarding state management</name>
  <files>
    web-app/src/store/slices/onboardingSlice.ts
  </files>
  <action>
Update the Redux slice with proper state management for 3-step wizard:

1. **Initial state:**
   ```typescript
   const initialState: OnboardingState = {
     currentStep: 1,
     selectedChannels: [],
     selectedModel: "anthropic",
     selectedTools: [],
     xopsConfig: {},
     loading: false,
     error: null
   }
   ```

2. **Synchronous reducers:**
   - `setCurrentStep(state, action: number)` - Move to specific step
   - `updateChannels(state, action: Channel[])` - Update selected channels
   - `updateModel(state, action: AIModelProvider)` - Update selected model
   - `updateTools(state, action: Tool[])` - Update selected tools
   - `updateXopsConfig(state, action: Partial<XopsConfig>)` - Update Xops config
   - `clearErrors(state)` - Clear error messages
   - `resetWizard(state)` - Reset to initial state

3. **Async thunks with proper error handling:**
   - `submitWizard: AsyncThunk` - Submit completed wizard, calls configAPI.createAgent() to create Xops
     - On success: Set xopsConfig, clear errors
     - On error: Set error message with user-friendly text
     - Loading states: Set loading = true during API call

   - `validateChannels: AsyncThunk` - Test channel connectivity (optional webhook validation)
   - `validateModel: AsyncThunk` - Test API key by making small LLM call
   - `validateTools: AsyncThunk` - Verify tools are accessible at reported paths

4. **Middleware setup:**
   - Enable Redux Persist to save wizard state to localStorage
   - Selectors for: current step, form validation state, selected data, loading/error states
   - Memoized selectors for derived data (e.g., ready to proceed to next step)

5. **Error handling:**
   - All thunks catch errors and format as user-friendly messages
   - No sensitive data (API keys) logged
   - Track error source (validation, API, network)

Ensure all async thunks properly integrate with existing configAPI and conversationAPI clients.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- onboardingSlice
Should run Redux slice tests showing:
- Initial state correct
- Actions update state as expected
- Async thunks handle success and error paths
- State persists to localStorage correctly
  </verify>
  <done>
✅ onboardingSlice.ts properly implements all reducers and thunks
✅ Async thunks integrate with existing API clients (configAPI, conversationAPI)
✅ Redux Persist configured for state persistence
✅ All error handling includes user-friendly messages
✅ Selectors properly memoized
✅ Tests pass for state management logic
  </done>
</task>

<task type="auto">
  <name>Task 3: Update OnboardingWizard component with 3-step flow</name>
  <files>
    web-app/src/components/onboarding/OnboardingWizard.tsx
  </files>
  <action>
Refactor the OnboardingWizard component to implement the 3-step flow (Channels → AI Model → Tools → Review):

1. **Component structure:**
   - Use Redux hooks (useAppDispatch, useAppSelector) to access wizard state
   - Render correct step component based on currentStep (1-4)
   - Show progress indicator: "Step X of 4" at top
   - Show "Back" button (disabled on step 1) and "Next" / "Launch Xops" button

2. **Step-by-step validation:**
   - Step 1 (Channels): At least one channel must be selected
   - Step 2 (AI Model): Model and API key must be provided
   - Step 3 (Tools): At least one tool must be enabled
   - Step 4 (Review): Show Xops profile, allow edits before launching
   - Next button disabled until current step passes validation

3. **Navigation logic:**
   - `handleNext()`: Validate current step, call async validators (optional), move to next step
   - `handleBack()`: Move to previous step (no validation needed)
   - `handleSubmit()`: Dispatch submitWizard thunk, show loading spinner, navigate to dashboard on success
   - `handleSkipStep()`: Allow skipping optional steps (tools can skip if no custom tools needed)

4. **Progress and state management:**
   - Show current step progress (e.g., "2 of 4")
   - Display validation errors for current step in alert at top
   - Show loading spinner during API calls (especially on Review step)
   - Maintain form data as user navigates back/forward (Redux state persists)

5. **Error handling:**
   - Display form validation errors inline on each step
   - Show API errors (failed to create Xops) with retry button
   - Show network errors with option to retry

6. **Accessibility:**
   - Use semantic HTML (form, section, fieldset elements)
   - ARIA labels on form inputs
   - Keyboard navigation (Tab through inputs, Enter to submit)
   - Screen reader friendly error messages

Example skeleton:
```typescript
export const OnboardingWizard: React.FC = () => {
  const dispatch = useAppDispatch()
  const currentStep = useAppSelector(s => s.onboarding.currentStep)
  const loading = useAppSelector(s => s.onboarding.loading)
  const error = useAppSelector(s => s.onboarding.error)

  const handleNext = async () => {
    // Validate current step
    if (!isCurrentStepValid()) return
    // Move to next step
    dispatch(setCurrentStep(currentStep + 1))
  }

  const handleSubmit = async () => {
    // Dispatch submitWizard thunk
    const result = await dispatch(submitWizard())
    if (submitWizard.fulfilled.match(result)) {
      // Navigate to dashboard
      window.location.hash = '#/dashboard'
    }
  }

  return (
    <div className="wizard-container">
      <ProgressBar current={currentStep} total={4} />
      {error && <AlertError message={error} />}

      {currentStep === 1 && <StepChannels />}
      {currentStep === 2 && <StepAIModel />}
      {currentStep === 3 && <StepTools />}
      {currentStep === 4 && <StepReview />}

      <div className="wizard-controls">
        <Button onClick={handleBack} disabled={currentStep === 1}>Back</Button>
        <Button onClick={currentStep === 4 ? handleSubmit : handleNext} loading={loading}>
          {currentStep === 4 ? 'Launch Xops' : 'Next'}
        </Button>
      </div>
    </div>
  )
}
```

Ensure component integrates properly with existing Button, Alert, ProgressBar components from the design system.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm dev
Visit http://localhost:5173
- Onboarding wizard loads with "Step 1 of 4"
- Can navigate forward/back through all steps
- Next button disabled until validation passes for current step
- API errors displayed clearly
- Loading spinner shows during API calls
- URL hash changes correctly (#/wizard → #/dashboard on completion)
  </verify>
  <done>
✅ OnboardingWizard properly renders 3-step flow (Channels → AI Model → Tools → Review)
✅ Step navigation works (Forward, Back, submit)
✅ Validation blocks navigation to next step if current step invalid
✅ Loading states and error handling display correctly
✅ Redux state persists across browser refresh
✅ Accessibility features working (ARIA labels, keyboard nav, screen reader friendly)
✅ Integrates with existing design system components
  </done>
</task>

<task type="auto">
  <name>Task 4: Implement StepChannels component</name>
  <files>
    web-app/src/components/onboarding/StepChannels.tsx
  </files>
  <action>
Create the Channels step component (Step 1) that lets users select which platforms Xops connects to:

1. **UI structure:**
   - Show 3 channel options: Slack, Telegram, Discord
   - Each option is a checkbox with icon and brief description
   - Show "Configuration Required" label if channel selected
   - Optional "Validate Channel" button for each selected channel

2. **Slack configuration:**
   - Input for Slack Workspace URL (auto-detect domain from URL)
   - Input for Bot Token (starts with "xoxb-")
   - Input for Channel Name (where Xops will appear)
   - Validation: At least one field required, token format must be valid
   - Optional: Test connection button

3. **Telegram configuration:**
   - Input for Bot Token (format: "123456:ABC-DEF...")
   - Input for Chat ID or Channel (numeric or @-prefixed)
   - Validation: Token must contain colon, Chat ID must be numeric

4. **Discord configuration:**
   - Input for Server ID
   - Input for Bot Token
   - Input for Channel ID
   - Validation: All three required if Discord selected

5. **Validation logic:**
   - At least one channel must be selected
   - If channel selected, all required fields must be filled
   - Token format validation (regex checks)
   - Show real-time validation feedback (green checkmark = valid)

6. **State management:**
   - Dispatch updateChannels(selectedChannels) on selection change
   - Store channel config in Redux onboarding state
   - Show current selections if returning to this step

7. **Styling:**
   - Use Tailwind classes for responsive layout
   - Consistent with existing form components
   - Visual indication of required vs optional fields
   - Clear error messages next to fields (not just alert)

Example structure:
```typescript
export const StepChannels: React.FC = () => {
  const dispatch = useAppDispatch()
  const selectedChannels = useAppSelector(s => s.onboarding.selectedChannels)
  const [channelConfigs, setChannelConfigs] = useState({
    slack: { url: '', token: '', channel: '' },
    telegram: { token: '', chatId: '' },
    discord: { serverId: '', token: '', channelId: '' }
  })

  const handleChannelToggle = (channel: Channel) => {
    const updated = selectedChannels.includes(channel)
      ? selectedChannels.filter(c => c !== channel)
      : [...selectedChannels, channel]
    dispatch(updateChannels(updated))
  }

  const handleConfigChange = (channel: Channel, field: string, value: string) => {
    setChannelConfigs(prev => ({
      ...prev,
      [channel]: { ...prev[channel], [field]: value }
    }))
  }

  const validateSlackToken = (token: string) => token.startsWith('xoxb-')
  const validateTelegramToken = (token: string) => token.includes(':')
  // etc.

  return (
    <div className="step-channels">
      <h2 className="text-2xl font-bold mb-6">Where should Xops appear?</h2>
      <p className="text-gray-600 mb-8">Select the platforms where you want Xops to coordinate with your team</p>

      <div className="space-y-6">
        {['slack', 'telegram', 'discord'].map(channel => (
          <ChannelOption
            key={channel}
            channel={channel as Channel}
            selected={selectedChannels.includes(channel as Channel)}
            onToggle={handleChannelToggle}
            config={channelConfigs[channel]}
            onConfigChange={handleConfigChange}
          />
        ))}
      </div>

      {selectedChannels.length === 0 && (
        <p className="text-red-500 mt-4">At least one channel is required</p>
      )}
    </div>
  )
}
```

Ensure channel validation provides clear, actionable feedback (e.g., "Slack token must start with 'xoxb-'").
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- StepChannels
Should show:
- All three channel options render (Slack, Telegram, Discord)
- Selecting a channel shows configuration inputs
- Deselecting a channel hides configuration inputs
- Token validation works in real-time
- Error messages display for invalid tokens
- State updates in Redux correctly

Also test in browser:
- http://localhost:5173 → onboarding wizard
- Step 1 shows channel options
- Can select/deselect channels
- Can enter channel configs
- Next button disabled if no channels selected
  </verify>
  <done>
✅ StepChannels component renders all 3 channel options
✅ Channel selection toggles configuration inputs
✅ All required fields validated (Slack token, Telegram token format, Discord IDs)
✅ Real-time validation feedback (checkmarks, error messages)
✅ Redux state updates on channel selection/config changes
✅ Form data persists when navigating away and back
✅ Clear error messages guide users (no ambiguous errors)
  </done>
</task>

<task type="auto">
  <name>Task 5: Implement StepAIModel component</name>
  <files>
    web-app/src/components/onboarding/StepAIModel.tsx
  </files>
  <action>
Create the AI Model step component (Step 2) that lets users select LLM provider and configure API key:

1. **UI structure:**
   - Radio buttons for provider selection (Anthropic, OpenAI, Google, Groq, Ollama)
   - Default selected: Anthropic
   - Show provider description (e.g., "Best for complex reasoning, $0.003/1K input tokens")
   - Conditional inputs based on selected provider

2. **Provider-specific configurations:**

   **Anthropic:**
   - Input for API key (starts with "sk-ant-")
   - Model selector dropdown: claude-opus-4.6, claude-sonnet-4, claude-haiku-4.5 (default)
   - Cost estimate display ($X per 1M tokens)
   - Rate limit display (5+ req/min)
   - Help text: "Get key from console.anthropic.com"

   **OpenAI:**
   - Input for API key (starts with "sk-")
   - Model selector: gpt-4o, gpt-4-turbo, gpt-3.5-turbo (default)
   - Cost estimate display

   **Google:**
   - Input for API key
   - Model selector: gemini-pro, gemini-pro-vision

   **Groq:**
   - Input for API key
   - Model selector: mixtral-8x7b, llama2-70b (default)
   - Show as "Free tier available"

   **Ollama (Local):**
   - Input for base URL (default: http://localhost:11434)
   - Model selector with auto-discovery option
   - Show as "Free, runs locally"

3. **Validation logic:**
   - API key format validation (regex per provider)
   - At least one model must be selected
   - Optional: Test button that makes small API call to verify key works
   - Show validation status: loading spinner while testing, checkmark if success, error if fails

4. **State management:**
   - Dispatch updateModel(provider) on provider selection
   - Store API key and model choice in Redux (encrypted if possible, or sessionStorage)
   - Persist provider/model selection but NOT API key to localStorage (security)

5. **Styling and UX:**
   - Use cards or radio group styling
   - Show provider comparison table (optional): cost, speed, reasoning quality
   - Clear "Why choose X?" explanation for each provider
   - Sensible defaults (Anthropic + claude-haiku for cost/speed balance)

Example structure:
```typescript
export const StepAIModel: React.FC = () => {
  const dispatch = useAppDispatch()
  const selectedModel = useAppSelector(s => s.onboarding.selectedModel)
  const [apiKey, setApiKey] = useState('')
  const [selectedModelVariant, setSelectedModelVariant] = useState('claude-haiku-4.5')
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null)

  const handleProviderChange = (provider: AIModelProvider) => {
    dispatch(updateModel(provider))
    setTestResult(null)
  }

  const handleTestAPI = async () => {
    setTesting(true)
    try {
      // Make small test call to LLM API
      const result = await testLLMConnection(selectedModel, apiKey)
      setTestResult('success')
    } catch (error) {
      setTestResult('error')
    }
    setTesting(false)
  }

  const getProviderConfig = (provider: AIModelProvider) => {
    // Return { description, models, costEstimate, helpText }
  }

  return (
    <div className="step-ai-model">
      <h2>Which AI model should Xops use?</h2>
      <p>Choose your preferred LLM provider. You can switch later.</p>

      <div className="provider-options">
        {['anthropic', 'openai', 'google', 'groq', 'ollama'].map(provider => (
          <label key={provider} className="provider-option">
            <input
              type="radio"
              name="provider"
              value={provider}
              checked={selectedModel === provider}
              onChange={() => handleProviderChange(provider as AIModelProvider)}
            />
            <div className="provider-details">
              <h3>{PROVIDER_NAMES[provider]}</h3>
              <p>{PROVIDER_DESCRIPTIONS[provider]}</p>
            </div>
          </label>
        ))}
      </div>

      {selectedModel && (
        <div className="model-config">
          <ModelInput
            provider={selectedModel}
            apiKey={apiKey}
            onApiKeyChange={setApiKey}
            selectedVariant={selectedModelVariant}
            onVariantChange={setSelectedModelVariant}
          />

          <button onClick={handleTestAPI} disabled={!apiKey || testing}>
            {testing ? 'Testing...' : 'Test Connection'}
          </button>

          {testResult === 'success' && <p className="text-green-600">✓ Connection successful</p>}
          {testResult === 'error' && <p className="text-red-600">✗ Connection failed. Check your API key.</p>}
        </div>
      )}
    </div>
  )
}
```

Ensure API key is never logged or persisted to localStorage. Use sessionStorage only if needed.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- StepAIModel
Should show:
- All 5 provider options render
- Selecting a provider shows provider-specific inputs
- Model variant dropdown works
- API key input accepts text without validation errors
- Test button triggers API test
- Test result displays (success/error)
- Redux state updates on provider selection

Browser test:
- http://localhost:5173 → onboarding wizard
- Step 2 shows all providers
- Selecting Anthropic shows Claude model options
- Selecting OpenAI shows GPT models
- Can enter API key and test
  </verify>
  <done>
✅ StepAIModel renders all 5 LLM providers (Anthropic, OpenAI, Google, Groq, Ollama)
✅ Provider selection shows provider-specific model options
✅ API key input validates format per provider (starts with correct prefix)
✅ Test connection button makes API call and shows result
✅ Redux state updates on provider/model selection
✅ API key NOT persisted to localStorage (security)
✅ Sensible defaults: Anthropic + claude-haiku-4.5
  </done>
</task>

<task type="auto">
  <name>Task 6: Implement StepTools component with auto-discovery</name>
  <files>
    web-app/src/components/onboarding/StepTools.tsx
  </files>
  <action>
Create the Tools step component (Step 3) that shows auto-discovered tools and lets users enable/disable them:

1. **Auto-discovery integration:**
   - On component mount, call API endpoint `/api/tools/discover` (or new endpoint from Rust backend)
   - Backend returns list of available tools with: { id, name, version, path, available: bool, category }
   - Show loading spinner while discovering tools ("Scanning for available tools...")
   - Display error message if discovery fails (but allow continuing)

2. **Tool display:**
   - Group tools by category (kubectl, terraform, docker, git, aws, shell, custom)
   - Use collapsible sections per category
   - For each tool show:
     - Checkbox to enable/disable
     - Tool name and version
     - Icon or category badge
     - Path where tool was found
     - Status indicator: ✓ Found at /usr/bin/kubectl, ✗ Not found, ⚠ Version mismatch

3. **Tool selection logic:**
   - At least one tool should be enabled (validation)
   - Allow skipping this step if no custom tools needed (all tools optional)
   - Dispatch updateTools(selectedTools) on selection change
   - Store selection in Redux state

4. **Special handling:**
   - Pre-select most useful tools by default (kubectl, terraform, docker, git, aws-cli, helm)
   - Show "Basic Tools" section at top with most critical tools
   - Show "Advanced Tools" section with less common tools (vault, istio, argocd)
   - "Custom Tools" section for user-provided tools

5. **Validation:**
   - Show validation message "No tools selected - Xops will have limited capabilities"
   - Allow proceeding with no tools (they'll be added later in config dashboard)
   - Show warning if recommended tools (kubectl) are not available

6. **Styling:**
   - Use toggle switches or checkboxes for each tool
   - Visual grouping with category colors
   - "Found" status in green, "Not found" in gray (disabled)
   - Version info in smaller text below tool name

Example structure:
```typescript
export const StepTools: React.FC = () => {
  const dispatch = useAppDispatch()
  const selectedTools = useAppSelector(s => s.onboarding.selectedTools)
  const [availableTools, setAvailableTools] = useState<Tool[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const discoverTools = async () => {
      try {
        const tools = await configAPI.discoverTools()
        setAvailableTools(tools)
        // Pre-select recommended tools
        const recommended = tools.filter(t => RECOMMENDED_TOOLS.includes(t.name))
        dispatch(updateTools(recommended))
      } catch (err) {
        setError('Failed to discover tools. You can add them later.')
      } finally {
        setLoading(false)
      }
    }

    discoverTools()
  }, [dispatch])

  const handleToolToggle = (tool: Tool) => {
    const updated = selectedTools.find(t => t.id === tool.id)
      ? selectedTools.filter(t => t.id !== tool.id)
      : [...selectedTools, tool]
    dispatch(updateTools(updated))
  }

  const groupedTools = groupBy(availableTools, 'category')

  return (
    <div className="step-tools">
      <h2>Which tools can Xops use?</h2>
      <p>Select the tools you want Xops to have access to. We found these on your system.</p>

      {loading && <LoadingSpinner message="Discovering tools..." />}
      {error && <AlertWarning message={error} />}

      {!loading && (
        <div className="tools-selection">
          {Object.entries(groupedTools).map(([category, tools]) => (
            <ToolCategory
              key={category}
              category={category}
              tools={tools}
              selectedTools={selectedTools}
              onToggle={handleToolToggle}
            />
          ))}
        </div>
      )}

      <div className="tools-summary">
        <p>Selected tools: {selectedTools.length}</p>
        {selectedTools.length === 0 && (
          <AlertInfo message="No tools selected. You can add them later in the configuration dashboard." />
        )}
      </div>
    </div>
  )
}
```

Ensure backend `/api/tools/discover` endpoint exists and returns tools in correct format. If not, create a placeholder that returns empty array.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- StepTools
Should show:
- Loading spinner displays while discovering tools
- Tools from API populate correctly
- Tools grouped by category
- Recommended tools pre-selected by default
- Toggling tool selection updates Redux state
- At least one tool is recommended to be selected
- Can proceed with no tools selected (warning shown)

Browser test:
- http://localhost:5173 → onboarding wizard
- Step 3 shows "Discovering tools..." spinner briefly
- Tools display grouped by category (kubectl, terraform, etc)
- Can toggle tools on/off
- Selected count updates in real-time
- Recommended tools pre-checked
  </verify>
  <done>
✅ StepTools component loads available tools from backend `/api/tools/discover`
✅ Tools auto-discovered and displayed (kubectl, terraform, docker, git, aws-cli, helm, etc)
✅ Tools grouped by category with collapsible sections
✅ Each tool shows name, version, path, and availability status
✅ Recommended tools pre-selected (kubectl, terraform, docker, git, aws-cli)
✅ User can toggle tool selection
✅ Redux state updates on selection change
✅ Validation allows proceeding with no tools (warning shown)
✅ Loading and error states handled gracefully
  </done>
</task>

<task type="auto">
  <name>Task 7: Implement StepReview component with Xops profile preview</name>
  <files>
    web-app/src/components/onboarding/StepReview.tsx
  </files>
  <action>
Create the Review step component (Step 4) that shows the Xops agent profile and launches Xops:

1. **Xops profile display:**
   - Agent name: "Xops" (editable text input)
   - Role: "Orchestrator" (read-only badge)
   - Persona: Pre-defined with name, description, traits
   - Selected channels: Show list of connected platforms
   - Selected model: Show provider and model variant
   - Selected tools: Show count and categories

2. **Persona preview:**
   - Show Xops default persona (from RESEARCH.md):
     - Name: "Xops"
     - Description: "Orchestrator and coordinator of specialized agents"
     - Traits: ["analytical", "leadership", "decisive", "collaborative"]
     - Communication style: "Clear, concise, action-oriented"
   - Show as card with personality indicators (emoji, color based on traits)
   - Allow editing persona later (button: "Customize later in dashboard")

3. **Configuration summary:**
   ```
   Xops Configuration
   ├─ Channels: Slack, Telegram
   ├─ Model: Anthropic Claude Haiku
   └─ Tools: 8 tools selected
       ├─ kubectl
       ├─ terraform
       ├─ docker
       └─ ... (show first 5, "2 more" link to expand)
   ```

4. **Launch action:**
   - Large "Launch Xops" button that calls Redux submitWizard thunk
   - Button shows loading spinner during API call
   - Disable button during submission
   - Show error message if creation fails (with retry button)

5. **Success state:**
   - After successful creation, show success message
   - Display Xops details (agent ID, created timestamp)
   - Button changes to "Go to Dashboard"
   - Navigate to #/dashboard on click

6. **Validation before launch:**
   - Ensure all required data is present (channels, model, tools)
   - Show validation errors if any field missing
   - Prevent launch if validation fails

7. **Styling:**
   - Use cards and layout similar to Dashboard components
   - Show Xops avatar/icon prominently
   - Color-code channels and tools by type
   - Loading and error states clear

Example structure:
```typescript
export const StepReview: React.FC = () => {
  const dispatch = useAppDispatch()
  const onboarding = useAppSelector(s => s.onboarding)
  const loading = useAppSelector(s => s.onboarding.loading)
  const error = useAppSelector(s => s.onboarding.error)
  const [xopsName, setXopsName] = useState('Xops')
  const [submitted, setSubmitted] = useState(false)

  const handleLaunch = async () => {
    // Build config from Redux state
    const config = {
      name: xopsName,
      persona: DEFAULT_XOPS_PERSONA,
      role: 'orchestrator',
      channels: onboarding.selectedChannels,
      model: onboarding.selectedModel,
      tools: onboarding.selectedTools
    }

    const result = await dispatch(submitWizard())

    if (submitWizard.fulfilled.match(result)) {
      setSubmitted(true)
    }
  }

  return (
    <div className="step-review">
      <h2>Launch Xops</h2>
      <p>Review your configuration and launch your orchestrator agent</p>

      {submitted && !error && (
        <div className="success-message">
          <h3>✓ Xops is ready!</h3>
          <p>Your orchestrator agent has been created and is ready to coordinate.</p>
          <button onClick={() => window.location.hash = '#/dashboard'} className="btn-primary">
            Go to Dashboard
          </button>
        </div>
      )}

      {!submitted && (
        <>
          <div className="xops-profile-card">
            <div className="profile-header">
              <XopsAvatar />
              <div className="profile-info">
                <input
                  type="text"
                  value={xopsName}
                  onChange={e => setXopsName(e.target.value)}
                  className="profile-name-input"
                  placeholder="Agent name"
                />
                <span className="role-badge">Orchestrator</span>
              </div>
            </div>

            <div className="profile-details">
              <PersonaCard persona={DEFAULT_XOPS_PERSONA} />

              <ConfigSummary
                channels={onboarding.selectedChannels}
                model={onboarding.selectedModel}
                tools={onboarding.selectedTools}
              />
            </div>
          </div>

          {error && <AlertError message={error} onRetry={handleLaunch} />}

          <button
            onClick={handleLaunch}
            disabled={loading}
            className="btn-primary btn-large"
          >
            {loading ? 'Launching...' : 'Launch Xops'}
          </button>
        </>
      )}
    </div>
  )
}
```

Ensure that submitWizard thunk properly creates Xops agent via configAPI.createAgent() with correct schema. Use Xops orchestrator persona from RESEARCH.md.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- StepReview
Should show:
- Xops profile displays with pre-filled name "Xops"
- Xops role shows as "Orchestrator"
- Persona card displays with traits and communication style
- Configuration summary shows selected channels, model, tools
- Launch button is clickable
- Clicking Launch triggers Redux thunk
- Loading spinner shows during submission
- Success message appears after successful creation
- Error message displays if creation fails (with retry)
- "Go to Dashboard" button appears after success

Browser test:
- http://localhost:5173 → onboarding wizard
- Complete steps 1-3, reach step 4
- Review page shows all configuration
- Can edit Xops name
- Launch button works
- On success, can navigate to dashboard
  </verify>
  <done>
✅ StepReview component displays Xops profile with name, role, persona
✅ Configuration summary shows channels, model, and tools selected
✅ Launch button triggers submitWizard Redux thunk
✅ Loading spinner and error handling work correctly
✅ On success, shows success message and "Go to Dashboard" button
✅ Xops agent created via configAPI.createAgent() with correct schema
✅ Xops persona set to default orchestrator (from RESEARCH.md)
✅ All configuration data from previous steps included in submission
  </done>
</task>

<task type="auto">
  <name>Task 8: Write comprehensive E2E tests for wizard flow</name>
  <files>
    web-app/src/test/e2e/wizard-flow.test.tsx
  </files>
  <action>
Create end-to-end tests covering the complete onboarding wizard flow:

1. **Test suite structure using Vitest:**
   ```typescript
   describe('Onboarding Wizard E2E Flow', () => {
     // Test 1: Initial state
     // Test 2: Step 1 - Channels selection
     // Test 3: Step 2 - AI Model selection
     // Test 4: Step 3 - Tools discovery
     // Test 5: Step 4 - Review and launch
     // Test 6: Error handling
     // Test 7: State persistence
     // Test 8: Navigation (back button)
   })
   ```

2. **Test cases to implement:**

   **Test 1: Wizard initialization**
   - Wizard renders with Step 1 visible
   - Progress indicator shows "Step 1 of 4"
   - Back button is disabled
   - Next button is disabled (no channels selected)

   **Test 2: Channel selection and validation**
   - Selecting Slack shows Slack token input
   - Token validation works (invalid token → error message)
   - Selecting multiple channels works
   - Deselecting all channels disables Next button
   - Proceeding to Step 2 saves channel selection to Redux

   **Test 3: AI Model selection and API key validation**
   - Defaulting to Anthropic provider works
   - Changing provider updates displayed model options
   - API key input accepts text
   - Test connection button triggers API call (mocked)
   - Failed API test shows error message with feedback
   - Proceeding to Step 3 saves model and API key selection

   **Test 4: Tools discovery and selection**
   - Loading spinner shows during tool discovery
   - Tools populate from API (mocked via MSW)
   - Recommended tools pre-selected
   - Toggling tool selection works
   - Tool count updates in real-time
   - Can proceed with no tools selected (warning shown)

   **Test 5: Review and Xops launch**
   - All configuration from steps 1-3 displayed correctly
   - Launch button is clickable
   - Clicking Launch shows loading spinner
   - API call to create Xops fires correctly (mocked)
   - Success state shows with "Go to Dashboard" button
   - Navigating to dashboard works

   **Test 6: Error handling throughout flow**
   - Network error on tool discovery → shows error, allows continuing
   - API error on Xops creation → shows error with retry button
   - Invalid channel config → blocks advancement with clear error
   - Failed API key validation → shows specific error message

   **Test 7: State persistence**
   - Refreshing page retains wizard state (if on same step)
   - Redux persist correctly saves/restores state
   - API key not persisted to localStorage
   - Channel selections persist correctly

   **Test 8: Navigation**
   - Back button works on steps 2-4
   - Back button disabled on step 1
   - Navigating back doesn't lose data
   - Can navigate forward again to complete wizard

3. **Mocking setup:**
   - Mock `/api/config/agents` endpoints (GET, POST)
   - Mock `/api/tools/discover` endpoint
   - Mock `/api/config/platforms` endpoints
   - Use MSW handlers from `test/mocks/handlers.ts`
   - Mock Redux store with initial state

4. **Test utilities:**
   - Helper function to render wizard with Redux context
   - Helper to fill out each step
   - Helper to check validation error messages
   - Helper to verify Redux state updates

Example test structure:
```typescript
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { OnboardingWizard } from '@/components/onboarding/OnboardingWizard'
import { Provider } from 'react-redux'
import { store } from '@/store'

describe('Onboarding Wizard E2E Flow', () => {
  beforeEach(() => {
    // Reset Redux store to initial state
    // Clear localStorage
  })

  it('renders wizard with Step 1 of 4', () => {
    render(
      <Provider store={store}>
        <OnboardingWizard />
      </Provider>
    )

    expect(screen.getByText('Step 1 of 4')).toBeInTheDocument()
    expect(screen.getByText('Where should Xops appear?')).toBeInTheDocument()
  })

  it('allows channel selection and moves to next step', async () => {
    const user = userEvent.setup()
    render(
      <Provider store={store}>
        <OnboardingWizard />
      </Provider>
    )

    // Select Slack channel
    const slackCheckbox = screen.getByRole('checkbox', { name: /slack/i })
    await user.click(slackCheckbox)

    // Fill in Slack config
    const tokenInput = screen.getByPlaceholderText('Bot Token')
    await user.type(tokenInput, 'xoxb-1234567890-abcdef')

    // Click Next
    const nextButton = screen.getByRole('button', { name: /next/i })
    await user.click(nextButton)

    // Should move to Step 2
    await waitFor(() => {
      expect(screen.getByText('Step 2 of 4')).toBeInTheDocument()
    })
  })

  // ... more tests
})
```

Ensure all tests use the existing MSW server from `test/setup.ts`. Tests should be deterministic and not depend on timing.
  </action>
  <verify>
cd /Users/gshah/work/opsflow-sh/aof/web-app && pnpm test -- wizard-flow
All tests should pass:
- 8 test cases covering full wizard flow
- No TypeScript errors
- No console errors or warnings during tests
- Coverage report shows >80% coverage of wizard components

Also run full test suite:
pnpm test
Should show all tests passing (existing + new wizard tests)
  </verify>
  <done>
✅ E2E tests written for complete wizard flow (8 test cases)
✅ Tests cover: initialization, channels, model, tools, review, errors, persistence, navigation
✅ All tests passing (pnpm test → all passing)
✅ Tests use MSW for API mocking
✅ Redux state correctly updated and persisted in tests
✅ Error scenarios tested (network errors, validation errors)
✅ Navigation (back/forward) tested
✅ Test coverage >80% for wizard components
  </done>
</task>

</tasks>

<verification>
After completing all tasks:

1. **Wizard flow verification:**
   - [ ] Start at Step 1, can navigate through all 4 steps
   - [ ] Back button works on steps 2-4, disabled on step 1
   - [ ] Validation prevents advancing if current step invalid
   - [ ] Channel selection shows/hides config inputs correctly
   - [ ] Tools auto-discovered and displayed
   - [ ] Xops profile preview shows on Step 4 with correct data
   - [ ] Launch button creates Xops agent via API

2. **State management verification:**
   - [ ] Redux state tracks current step correctly
   - [ ] Form data persists when navigating back/forward
   - [ ] State persists to localStorage (pnpm dev → refresh → state retained)
   - [ ] No API keys logged or persisted unsafely

3. **Component integration:**
   - [ ] All components properly use Redux hooks
   - [ ] Types imported from onboarding.ts with no errors
   - [ ] Redux selectors properly memoized (no unnecessary re-renders)
   - [ ] All form inputs properly controlled (value + onChange)

4. **Testing:**
   - [ ] pnpm test → 8+ new wizard tests passing
   - [ ] pnpm test → all existing tests still passing (no regressions)
   - [ ] pnpm type-check → 0 TypeScript errors
   - [ ] pnpm build → build succeeds with no errors

5. **UX quality:**
   - [ ] Error messages are clear and actionable
   - [ ] Loading states visible during API calls
   - [ ] Form inputs properly labeled (accessibility)
   - [ ] Mobile responsiveness working (tested on mobile viewport)
   - [ ] No console errors or warnings

6. **Acceptance criteria:**
   - [ ] Wizard reduced from 4 steps to 3 (Channels → AI Model → Tools → Review)
   - [ ] Xops auto-created on wizard completion with orchestrator persona
   - [ ] Tools auto-discovered from backend and displayed
   - [ ] User can complete setup and launch Xops in <3 minutes
   - [ ] All configuration properly typed and validated
   - [ ] No regressions in existing Phase 1 Integration functionality
</verification>

<success_criteria>

**Plan 01 Complete When:**

1. ✅ **Wizard flow implemented:** 3-step flow (Channels → AI Model → Tools → Review) working end-to-end
2. ✅ **All types defined:** onboarding.ts contains all required TypeScript types with proper documentation
3. ✅ **Redux state management:** onboardingSlice properly manages wizard state with async thunks
4. ✅ **4 step components:** StepChannels, StepAIModel, StepTools, StepReview all implemented and working
5. ✅ **Validation working:** Each step validates inputs before allowing advancement to next step
6. ✅ **Auto-discovery integrated:** Tools step fetches and displays available tools from backend
7. ✅ **Xops auto-creation:** Submitting wizard creates Xops agent with orchestrator persona via API
8. ✅ **State persistence:** Redux state persists to localStorage and survives browser refresh
9. ✅ **Error handling:** All error scenarios handled gracefully (validation, API errors, network errors)
10. ✅ **Tests passing:** 8+ E2E tests pass covering complete wizard flow
11. ✅ **No regressions:** All existing Phase 1 tests still passing
12. ✅ **TypeScript clean:** pnpm type-check → 0 errors
13. ✅ **Build succeeds:** pnpm build completes without errors

**Verification Method:**
```bash
# Type checking
pnpm type-check

# Tests
pnpm test -- wizard-flow     # 8+ tests passing
pnpm test                    # All tests passing (no regressions)

# Build
pnpm build                   # Succeeds without errors

# Manual testing
pnpm dev
# Navigate to http://localhost:5173
# Complete wizard flow from Step 1 to Launch
# Verify Xops created in backend
# Refresh page and verify state persisted
```

**Deliverables:**
- 8 new/updated files (types, Redux, 4 components, E2E tests)
- 8 commits with atomic changes
- 100+ lines of test coverage
- Updated onboarding.ts, onboardingSlice.ts with full types and thunks
- 4 step components fully implemented and integrated
- All form validation working with clear error messages
</success_criteria>

<output>
After completion, create `.planning/phases/01-onboarding-refinement/01-SUMMARY.md` with:
- Executive summary of changes (wizard redesign from 4→3 steps)
- File list (8 files created/modified)
- Test results (8 E2E tests + all regression tests passing)
- Verification checklist (all 13 items passing)
- Performance metrics (no regressions, build time, test time)
- Next steps (Plan 02: Tool auto-discovery & installation)
</output>
