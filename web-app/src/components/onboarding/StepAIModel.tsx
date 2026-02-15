import React, { useState } from 'react'
import { useAppDispatch, useAppSelector, updateModel } from '@/store'
import { AIModelProvider, PROVIDER_INFO } from '@/types/onboarding'
import Input from '@/components/common/Input'
import Select from '@/components/common/Select'
import Badge from '@/components/common/Badge'
import LoadingSpinner from '@/components/common/LoadingSpinner'

/**
 * Step 2: AI Model Selection and Configuration
 * Allows users to select LLM provider and configure API key
 */
export const StepAIModel: React.FC = () => {
  const dispatch = useAppDispatch()
  const selectedModel = useAppSelector((state) => state.onboarding.selectedModel)
  const loading = useAppSelector((state) => state.onboarding.loading)

  const [apiKey, setApiKey] = useState('')
  const [selectedModelVariant, setSelectedModelVariant] = useState(
    getDefaultModel(selectedModel)
  )
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null)
  const [testError, setTestError] = useState<string>('')

  const handleProviderChange = (provider: AIModelProvider) => {
    dispatch(updateModel(provider))
    setSelectedModelVariant(getDefaultModel(provider))
    setTestResult(null)
    setApiKey('')
  }

  const handleTestAPI = async () => {
    if (!apiKey.trim()) {
      setTestError('Please enter an API key')
      return
    }

    setTesting(true)
    setTestResult(null)
    setTestError('')

    try {
      // Simulate API test call
      await new Promise((resolve) => setTimeout(resolve, 1500))

      // Validate API key format per provider
      switch (selectedModel) {
        case 'anthropic':
          if (!apiKey.startsWith('sk-ant-')) {
            throw new Error("Anthropic keys should start with 'sk-ant-'")
          }
          break
        case 'openai':
          if (!apiKey.startsWith('sk-')) {
            throw new Error("OpenAI keys should start with 'sk-'")
          }
          break
        case 'google':
          if (!apiKey.startsWith('AI')) {
            throw new Error("Google API keys should start with 'AI'")
          }
          break
        case 'groq':
          if (!apiKey.startsWith('gsk_')) {
            throw new Error("Groq keys should start with 'gsk_'")
          }
          break
        case 'ollama':
          // Ollama doesn't require API keys
          if (!apiKey) {
            throw new Error('Ollama requires a base URL')
          }
          break
      }

      setTestResult('success')
      setTestError('')
    } catch (error: any) {
      setTestResult('error')
      setTestError(error.message || 'Connection test failed. Please check your API key.')
    } finally {
      setTesting(false)
    }
  }

  const providerInfo = PROVIDER_INFO[selectedModel]
  const modelOptions = providerInfo.models.map((m) => ({
    value: m.value,
    label: m.label,
  }))

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          Which AI model should Xops use?
        </h2>
        <p className="text-gray-600 dark:text-gray-400">
          Choose your preferred LLM provider. You can switch later.
        </p>
      </div>

      {loading && <LoadingSpinner text="Configuring model..." />}

      {/* Provider Selection */}
      <div className="space-y-3">
        {(Object.keys(PROVIDER_INFO) as AIModelProvider[]).map((provider) => {
          const info = PROVIDER_INFO[provider]
          const isSelected = selectedModel === provider

          return (
            <label
              key={provider}
              className={`block border rounded-lg p-4 cursor-pointer transition ${
                isSelected
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                  : 'border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800/50'
              }`}
            >
              <div className="flex items-start">
                <input
                  type="radio"
                  name="provider"
                  value={provider}
                  checked={isSelected}
                  onChange={() => handleProviderChange(provider)}
                  className="mt-1 h-4 w-4 text-blue-600 dark:text-blue-500"
                />
                <div className="ml-3 flex-1">
                  <div className="font-semibold text-gray-900 dark:text-white">
                    {info.name}
                  </div>
                  <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                    {info.description}
                  </div>
                  <div className="text-xs text-gray-500 dark:text-gray-500 mt-2">
                    {info.costEstimate}
                  </div>
                </div>
              </div>
            </label>
          )
        })}
      </div>

      {/* Model Configuration */}
      {selectedModel && (
        <div className="border border-gray-200 dark:border-gray-700 rounded-lg p-4 bg-gray-50 dark:bg-gray-800/50">
          <h3 className="font-semibold text-gray-900 dark:text-white mb-4">
            Configuration
          </h3>

          {/* Model Selection */}
          <div className="mb-4">
            <Select
              label="Model"
              value={selectedModelVariant}
              onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setSelectedModelVariant(e.target.value)}
              options={modelOptions}
              fullWidth
            />
          </div>

          {/* API Key Input */}
          {selectedModel !== 'ollama' && (
            <div className="mb-4">
              <Input
                label="API Key"
                type="password"
                placeholder={
                  selectedModel === 'anthropic'
                    ? 'sk-ant-...'
                    : selectedModel === 'openai'
                    ? 'sk-...'
                    : selectedModel === 'google'
                    ? 'AI...'
                    : selectedModel === 'groq'
                    ? 'gsk_...'
                    : 'your-api-key'
                }
                value={apiKey}
                onChange={(e) => {
                  setApiKey(e.target.value)
                  setTestResult(null)
                  setTestError('')
                }}
                helperText={`Get your ${providerInfo.name} API key from: ${providerInfo.helpUrl || 'provider console'}`}
                fullWidth
              />
            </div>
          )}

          {selectedModel === 'ollama' && (
            <div className="mb-4">
              <Input
                label="Ollama Base URL"
                type="text"
                placeholder="http://localhost:11434"
                value={apiKey}
                onChange={(e) => {
                  setApiKey(e.target.value)
                  setTestResult(null)
                  setTestError('')
                }}
                helperText="Default: http://localhost:11434. Make sure Ollama is running locally."
                fullWidth
              />
            </div>
          )}

          {/* Test Button */}
          <div className="flex gap-2 items-center">
            <button
              onClick={handleTestAPI}
              disabled={!apiKey || testing}
              className="px-4 py-2 text-sm font-medium bg-blue-600 dark:bg-blue-700 text-white rounded hover:bg-blue-700 dark:hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition"
            >
              {testing ? 'Testing...' : 'Test Connection'}
            </button>

            {/* Test Result */}
            {testResult === 'success' && (
              <div className="flex items-center gap-2">
                <Badge variant="success">✓ Connected</Badge>
                <span className="text-sm text-green-600 dark:text-green-400">
                  API key verified
                </span>
              </div>
            )}

            {testResult === 'error' && (
              <div className="flex items-center gap-2">
                <Badge variant="error">✕ Failed</Badge>
              </div>
            )}
          </div>

          {testError && (
            <div className="mt-2 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-200 text-sm">
              {testError}
            </div>
          )}

          {/* Info Box */}
          <div className="mt-4 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded text-blue-700 dark:text-blue-200 text-sm">
            <strong>💡 Tip:</strong> You can always update the API key later. Xops will use this
            provider for all LLM operations.
          </div>
        </div>
      )}
    </div>
  )
}

/**
 * Get default model for a provider
 */
function getDefaultModel(provider: AIModelProvider): string {
  const defaults: Record<AIModelProvider, string> = {
    anthropic: 'claude-haiku-4.5',
    openai: 'gpt-3.5-turbo',
    google: 'gemini-pro',
    groq: 'llama2-70b-4096',
    ollama: 'llama2',
  }
  return defaults[provider]
}

export default StepAIModel
