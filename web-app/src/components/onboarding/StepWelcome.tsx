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
  const [errors, setErrors] = useState<Record<string, string>>({})

  const handleChange = (field: string, value: string) => {
    dispatch(updateProject({ [field]: value }))
    if (errors[field]) {
      setErrors((prev) => {
        const newErrors = { ...prev }
        delete newErrors[field]
        return newErrors
      })
    }
  }

  const handleNext = () => {
    const newErrors: Record<string, string> = {}

    if (!project.name || project.name.length < 3) {
      newErrors.name = 'Project name must be at least 3 characters'
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
        <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">Let's set up your first AOF project</h2>
        <p className="text-gray-600 dark:text-gray-400">Create your project and we'll configure your first agent in the next step.</p>
      </div>

      <div className="space-y-4">
        <Input
          label="Project Name"
          placeholder="My AOF Project"
          value={project.name}
          onChange={(e) => handleChange('name', e.target.value)}
          error={errors.name}
          required
          fullWidth
        />

        <TextArea
          label="Description"
          placeholder="What will this project do? (optional)"
          value={project.description}
          onChange={(e) => handleChange('description', e.target.value)}
          rows={3}
          fullWidth
        />
      </div>

      <div className="flex justify-end gap-3 pt-4">
        <Button variant="secondary" onClick={() => {}}>
          Skip
        </Button>
        <Button onClick={handleNext}>Next</Button>
      </div>
    </div>
  )
}

export default StepWelcome
