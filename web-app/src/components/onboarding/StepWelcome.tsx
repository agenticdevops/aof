import React, { useState } from 'react'
import { useAppDispatch, useAppSelector, updateProject } from '@/store'
import Input from '@/components/common/Input'
import TextArea from '@/components/common/TextArea'
import Button from '@/components/common/Button'

interface StepWelcomeProps {
  onNext: () => void
}

export const StepWelcome: React.FC<StepWelcomeProps> = ({ onNext }) => {
  const dispatch = useAppDispatch()
  const { project } = useAppSelector((state) => state.onboarding)
  const [errors, setErrors] = useState<{ name?: string; description?: string }>({})

  const handleChange = (field: string, value: string) => {
    dispatch(updateProject({ [field]: value } as any))
    if (errors[field as keyof typeof errors]) {
      setErrors({ ...errors, [field]: undefined })
    }
  }

  const handleNext = () => {
    const newErrors: typeof errors = {}
    if (!project.name || project.name.trim().length < 3) {
      newErrors.name = 'Project name must be at least 3 characters'
    }
    if (!project.description || project.description.trim().length < 10) {
      newErrors.description = 'Project description must be at least 10 characters'
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
        <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">Let's set up your project</h2>
        <p className="text-gray-600 dark:text-gray-400">Start by giving your project a name and description</p>
      </div>

      <Input
        label="Project Name"
        placeholder="My Agent Squad"
        value={project.name}
        onChange={(e) => handleChange('name', e.target.value)}
        error={errors.name}
        fullWidth
      />

      <TextArea
        label="Project Description"
        placeholder="What will your agents do? Describe the purpose and goals of this project..."
        value={project.description}
        onChange={(e) => handleChange('description', e.target.value)}
        error={errors.description}
        rows={4}
        fullWidth
      />

      <div className="flex gap-3 justify-end">
        <Button variant="primary" onClick={handleNext} fullWidth>
          Next
        </Button>
      </div>
    </div>
  )
}

export default StepWelcome
