import React, { useState } from 'react'
import { useAppDispatch, useAppSelector, updateOnboardingAgent } from '@/store'
import Input from '@/components/common/Input'
import TextArea from '@/components/common/TextArea'
import Select from '@/components/common/Select'
import Checkbox from '@/components/common/Checkbox'
import Button from '@/components/common/Button'

interface StepAgentSetupProps {
  onNext: () => void
  onBack: () => void
}

const capabilities = [
  'Shell Commands',
  'HTTP Requests',
  'File Operations',
  'Database Queries',
  'API Integrations',
  'Code Execution',
]

export const StepAgentSetup: React.FC<StepAgentSetupProps> = ({ onNext, onBack }) => {
  const dispatch = useAppDispatch()
  const { agent } = useAppSelector((state) => state.onboarding)
  const [errors, setErrors] = useState<{ name?: string; instructions?: string }>({})

  const handleChange = (field: string, value: any) => {
    dispatch(updateOnboardingAgent({ [field]: value } as any))
    if (errors[field as keyof typeof errors]) {
      setErrors({ ...errors, [field]: undefined })
    }
  }

  const handleCapabilityToggle = (capability: string) => {
    const currentCapabilities = agent.capabilities || []
    const updated = currentCapabilities.includes(capability)
      ? currentCapabilities.filter((c) => c !== capability)
      : [...currentCapabilities, capability]
    dispatch(updateOnboardingAgent({ capabilities: updated }))
  }

  const handleNext = () => {
    const newErrors: typeof errors = {}
    if (!agent.name || agent.name.trim().length < 2) {
      newErrors.name = 'Agent name must be at least 2 characters'
    }
    if (!agent.instructions || agent.instructions.trim().length < 10) {
      newErrors.instructions = 'Instructions must be at least 10 characters'
    }

    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors)
      return
    }

    onNext()
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">Create your first agent</h2>
        <p className="text-gray-600 dark:text-gray-400">Configure the AI model, personality, and capabilities</p>
      </div>

      <Input
        label="Agent Name"
        placeholder="e.g., Analyzer, Coordinator, Specialist"
        value={agent.name}
        onChange={(e) => handleChange('name', e.target.value)}
        error={errors.name}
        fullWidth
      />

      <Select
        label="AI Model"
        value={agent.model}
        onChange={(e) => handleChange('model', e.target.value)}
        options={[
          { value: 'claude', label: 'Claude 3.5 Sonnet' },
          { value: 'gpt-4', label: 'GPT-4 Turbo' },
          { value: 'gemini', label: 'Gemini 2.5 Pro' },
          { value: 'ollama', label: 'Ollama (Local)' },
        ]}
        fullWidth
      />

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">Agent Type</label>
        <div className="space-y-2">
          {(['analyst', 'coordinator', 'specialist'] as const).map((type) => (
            <label key={type} className="flex items-center cursor-pointer">
              <input
                type="radio"
                name="agent-type"
                value={type}
                checked={agent.type === type}
                onChange={(e) => handleChange('type', e.target.value)}
                className="w-4 h-4 text-sky-400 dark:text-sky-500"
              />
              <span className="ml-3 text-sm font-medium text-gray-700 dark:text-gray-300 capitalize">{type}</span>
            </label>
          ))}
        </div>
      </div>

      <TextArea
        label="Instructions"
        placeholder="Tell the agent what it should do, its behavior, and any constraints..."
        value={agent.instructions}
        onChange={(e) => handleChange('instructions', e.target.value)}
        error={errors.instructions}
        rows={4}
        fullWidth
      />

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">Agent Capabilities</label>
        <div className="space-y-2">
          {capabilities.map((capability) => (
            <Checkbox
              key={capability}
              label={capability}
              checked={(agent.capabilities || []).includes(capability)}
              onChange={() => handleCapabilityToggle(capability)}
            />
          ))}
        </div>
      </div>

      <div className="flex gap-3 justify-end">
        <Button variant="secondary" onClick={onBack}>
          Back
        </Button>
        <Button variant="primary" onClick={handleNext}>
          Next
        </Button>
      </div>
    </div>
  )
}

export default StepAgentSetup
