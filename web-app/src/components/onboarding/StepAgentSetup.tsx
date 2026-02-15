import React, { useState } from 'react'
import { useAppDispatch, useAppSelector, updateAgent } from '@/store'
import Input from '@/components/common/Input'
import TextArea from '@/components/common/TextArea'
import Select from '@/components/common/Select'
import Checkbox from '@/components/common/Checkbox'
import Button from '@/components/common/Button'

interface StepAgentSetupProps {
  onBack: () => void
  onNext: () => void
}

const models = [
  { value: 'claude', label: 'Claude 3.5 Sonnet (Anthropic)' },
  { value: 'gpt-4', label: 'GPT-4 (OpenAI)' },
  { value: 'gemini', label: 'Gemini 2.5 Flash (Google)' },
  { value: 'ollama', label: 'Ollama (Self-hosted)' },
]

const agentTypes = [
  { value: 'analyst', label: 'Analyst', description: 'Specialized in data analysis and reporting' },
  { value: 'coordinator', label: 'Coordinator', description: 'Orchestrates workflows and team tasks' },
  { value: 'specialist', label: 'Specialist', description: 'Expert in specific domains' },
]

const allCapabilities = [
  'Shell Commands',
  'HTTP Requests',
  'File Operations',
  'Database Access',
  'Email Sending',
  'Git Operations',
  'Kubernetes Commands',
  'Custom Integrations',
]

export const StepAgentSetup: React.FC<StepAgentSetupProps> = ({ onBack, onNext }) => {
  const dispatch = useAppDispatch()
  const { agent } = useAppSelector((state) => state.onboarding)
  const [errors, setErrors] = useState<Record<string, string>>({})

  const handleChange = (field: string, value: any) => {
    dispatch(updateAgent({ [field]: value }))
    if (errors[field]) {
      setErrors((prev) => {
        const newErrors = { ...prev }
        delete newErrors[field]
        return newErrors
      })
    }
  }

  const toggleCapability = (capability: string) => {
    const newCapabilities = agent.capabilities?.includes(capability)
      ? agent.capabilities.filter((c) => c !== capability)
      : [...(agent.capabilities || []), capability]
    handleChange('capabilities', newCapabilities)
  }

  const handleNext = () => {
    const newErrors: Record<string, string> = {}

    if (!agent.name || agent.name.length < 2) {
      newErrors.name = 'Agent name is required'
    }

    if (!agent.instructions || agent.instructions.length < 10) {
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
        <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">Create your first agent</h2>
        <p className="text-gray-600 dark:text-gray-400">Configure the agent that will perform your tasks.</p>
      </div>

      <div className="space-y-4">
        <Input
          label="Agent Name"
          placeholder="e.g., DevOps Specialist"
          value={agent.name}
          onChange={(e) => handleChange('name', e.target.value)}
          error={errors.name}
          required
          fullWidth
        />

        <Select
          label="LLM Model"
          options={models}
          value={agent.model}
          onChange={(value) => handleChange('model', value)}
          fullWidth
        />

        <div className="space-y-3">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">Agent Type</label>
          <div className="space-y-2">
            {agentTypes.map((type) => (
              <label key={type.value} className="flex items-start gap-3 p-3 border rounded-lg cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700">
                <input
                  type="radio"
                  name="agentType"
                  value={type.value}
                  checked={agent.type === type.value}
                  onChange={(e) => handleChange('type', e.target.value)}
                  className="mt-1"
                />
                <div>
                  <p className="font-medium text-gray-900 dark:text-gray-100">{type.label}</p>
                  <p className="text-sm text-gray-500 dark:text-gray-400">{type.description}</p>
                </div>
              </label>
            ))}
          </div>
        </div>

        <TextArea
          label="Instructions"
          placeholder="Give detailed instructions about what this agent should do..."
          value={agent.instructions}
          onChange={(e) => handleChange('instructions', e.target.value)}
          error={errors.instructions}
          rows={4}
          required
          fullWidth
        />

        <div className="space-y-3">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">Capabilities</label>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {allCapabilities.map((cap) => (
              <Checkbox
                key={cap}
                label={cap}
                checked={agent.capabilities?.includes(cap) || false}
                onChange={() => toggleCapability(cap)}
              />
            ))}
          </div>
        </div>
      </div>

      <div className="flex justify-between gap-3 pt-4">
        <Button variant="secondary" onClick={onBack}>
          Back
        </Button>
        <Button onClick={handleNext}>Next</Button>
      </div>
    </div>
  )
}

export default StepAgentSetup
